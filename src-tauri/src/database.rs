use rusqlite::{Connection, Result as SqliteResult};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Serialize, Clone)]
pub struct Recipe {
    pub id: i64,
    pub mod_name: String,
    pub path: String,
    pub recipe_type: String,
    pub result_item: Option<String>,
    pub result_count: Option<i32>,
    pub ingredients: Vec<String>,
    pub raw_json: String,
}

#[derive(Serialize)]
pub struct ExtractionResult {
    pub mods_processed: usize,
    pub recipes_extracted: usize,
    pub errors: Vec<String>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS mods (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                scanned_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS recipes (
                id INTEGER PRIMARY KEY,
                mod_id INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
                path TEXT NOT NULL,
                recipe_type TEXT NOT NULL,
                result_item TEXT,
                result_count INTEGER,
                raw_json TEXT NOT NULL,
                UNIQUE(mod_id, path)
            );

            CREATE TABLE IF NOT EXISTS recipe_ingredients (
                id INTEGER PRIMARY KEY,
                recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
                item TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_recipes_result ON recipes(result_item);
            CREATE INDEX IF NOT EXISTS idx_recipes_mod ON recipes(mod_id);
            CREATE INDEX IF NOT EXISTS idx_ingredients_item ON recipe_ingredients(item);
            CREATE INDEX IF NOT EXISTS idx_ingredients_recipe ON recipe_ingredients(recipe_id);
            "
        )?;

        Ok(())
    }

    pub fn clear_all(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            DELETE FROM recipe_ingredients;
            DELETE FROM recipes;
            DELETE FROM mods;
            "
        )?;
        Ok(())
    }

    pub fn insert_mod(&self, name: &str, path: &str) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono_lite_now();

        conn.execute(
            "INSERT OR REPLACE INTO mods (name, path, scanned_at) VALUES (?1, ?2, ?3)",
            [name, path, &now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn insert_recipe(
        &self,
        mod_id: i64,
        path: &str,
        recipe_type: &str,
        result_item: Option<&str>,
        result_count: Option<i32>,
        raw_json: &str,
        ingredients: &[String],
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO recipes (mod_id, path, recipe_type, result_item, result_count, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![mod_id, path, recipe_type, result_item, result_count, raw_json],
        )?;

        let recipe_id = conn.last_insert_rowid();

        // Clear existing ingredients for this recipe (in case of replace)
        conn.execute(
            "DELETE FROM recipe_ingredients WHERE recipe_id = ?1",
            [recipe_id],
        )?;

        // Insert ingredients
        for item in ingredients {
            conn.execute(
                "INSERT INTO recipe_ingredients (recipe_id, item) VALUES (?1, ?2)",
                rusqlite::params![recipe_id, item],
            )?;
        }

        Ok(recipe_id)
    }

    pub fn search_by_output(&self, item: &str) -> SqliteResult<Vec<Recipe>> {
        let conn = self.conn.lock().unwrap();
        let search_term = format!("%{}%", item);
        let mut stmt = conn.prepare(
            "SELECT r.id, m.name, r.path, r.recipe_type, r.result_item, r.result_count, r.raw_json
             FROM recipes r
             JOIN mods m ON r.mod_id = m.id
             WHERE r.result_item LIKE ?1
             ORDER BY r.result_item, m.name"
        )?;
        self.collect_recipes(&conn, &mut stmt, &[&search_term])
    }

    pub fn search_by_ingredient(&self, item: &str) -> SqliteResult<Vec<Recipe>> {
        let conn = self.conn.lock().unwrap();
        let search_term = format!("%{}%", item);
        let mut stmt = conn.prepare(
            "SELECT DISTINCT r.id, m.name, r.path, r.recipe_type, r.result_item, r.result_count, r.raw_json
             FROM recipes r
             JOIN mods m ON r.mod_id = m.id
             JOIN recipe_ingredients ri ON r.id = ri.recipe_id
             WHERE ri.item LIKE ?1
             ORDER BY r.result_item, m.name"
        )?;
        self.collect_recipes(&conn, &mut stmt, &[&search_term])
    }

    pub fn list_recipes(&self, offset: i64, limit: i64) -> SqliteResult<Vec<Recipe>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT r.id, m.name, r.path, r.recipe_type, r.result_item, r.result_count, r.raw_json
             FROM recipes r
             JOIN mods m ON r.mod_id = m.id
             ORDER BY m.name, r.path
             LIMIT ?1 OFFSET ?2"
        )?;
        self.collect_recipes(&conn, &mut stmt, &[&limit, &offset])
    }

    fn collect_recipes(
        &self,
        conn: &Connection,
        stmt: &mut rusqlite::Statement,
        params: &[&dyn rusqlite::ToSql],
    ) -> SqliteResult<Vec<Recipe>> {
        let recipe_rows = stmt.query_map(params, |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<i32>>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut recipes = Vec::new();
        for row in recipe_rows {
            let (id, mod_name, path, recipe_type, result_item, result_count, raw_json) = row?;
            let ingredients = self.get_ingredients_for_recipe(conn, id)?;
            recipes.push(Recipe {
                id,
                mod_name,
                path,
                recipe_type,
                result_item,
                result_count,
                ingredients,
                raw_json,
            });
        }
        Ok(recipes)
    }

    pub fn get_recipe_count(&self) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM recipes", [], |row| row.get(0))
    }

    fn get_ingredients_for_recipe(&self, conn: &Connection, recipe_id: i64) -> SqliteResult<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT item FROM recipe_ingredients WHERE recipe_id = ?1 ORDER BY item"
        )?;

        let items = stmt.query_map([recipe_id], |row| row.get(0))?;
        items.collect()
    }
}

// Simple timestamp without external dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
