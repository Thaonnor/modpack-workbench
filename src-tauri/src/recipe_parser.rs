use serde_json::Value;

pub struct ParsedRecipe {
    pub recipe_type: String,
    pub result_item: Option<String>,
    pub result_count: Option<i32>,
    pub ingredients: Vec<String>,
}

pub fn parse_recipe(json_str: &str) -> Result<ParsedRecipe, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let recipe_type = value.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut ingredients = Vec::new();
    let mut result_item = None;
    let mut result_count = None;

    // Extract result based on recipe type
    if let Some(result) = value.get("result") {
        let (item, count) = extract_item_and_count(result);
        result_item = item;
        result_count = count;
    }

    // Extract ingredients based on recipe type
    match recipe_type.as_str() {
        // Shaped crafting: has pattern and key
        "minecraft:crafting_shaped" | "crafting_shaped" => {
            if let Some(key) = value.get("key").and_then(|k| k.as_object()) {
                for (_symbol, ingredient) in key {
                    extract_ingredients_from_value(ingredient, &mut ingredients);
                }
            }
        }

        // Shapeless crafting: has ingredients array
        "minecraft:crafting_shapeless" | "crafting_shapeless" => {
            if let Some(ing_array) = value.get("ingredients").and_then(|i| i.as_array()) {
                for ing in ing_array {
                    extract_ingredients_from_value(ing, &mut ingredients);
                }
            }
        }

        // Smelting/cooking recipes: single ingredient
        "minecraft:smelting" | "minecraft:blasting" | "minecraft:smoking" | "minecraft:campfire_cooking"
        | "smelting" | "blasting" | "smoking" | "campfire_cooking" => {
            if let Some(ingredient) = value.get("ingredient") {
                extract_ingredients_from_value(ingredient, &mut ingredients);
            }
        }

        // Stonecutting: single ingredient, result is just item string
        "minecraft:stonecutting" | "stonecutting" => {
            if let Some(ingredient) = value.get("ingredient") {
                extract_ingredients_from_value(ingredient, &mut ingredients);
            }
            // Stonecutting result is sometimes just the item string
            if result_item.is_none() {
                if let Some(result_str) = value.get("result").and_then(|r| r.as_str()) {
                    result_item = Some(result_str.to_string());
                }
            }
        }

        // Smithing recipes (1.20+): template + base + addition
        "minecraft:smithing_transform" | "minecraft:smithing_trim" | "smithing_transform" | "smithing_trim" => {
            if let Some(template) = value.get("template") {
                extract_ingredients_from_value(template, &mut ingredients);
            }
            if let Some(base) = value.get("base") {
                extract_ingredients_from_value(base, &mut ingredients);
            }
            if let Some(addition) = value.get("addition") {
                extract_ingredients_from_value(addition, &mut ingredients);
            }
        }

        // Legacy smithing (pre-1.20)
        "minecraft:smithing" | "smithing" => {
            if let Some(base) = value.get("base") {
                extract_ingredients_from_value(base, &mut ingredients);
            }
            if let Some(addition) = value.get("addition") {
                extract_ingredients_from_value(addition, &mut ingredients);
            }
        }

        // Special recipes (usually no ingredients/result to extract)
        _ if recipe_type.contains("special") => {
            // These are hardcoded recipes like firework_rocket, map_cloning, etc.
        }

        // Modded recipe types - try common patterns
        _ => {
            // Try to find ingredients in common locations
            if let Some(ingredients_val) = value.get("ingredients").or(value.get("ingredient")) {
                if let Some(arr) = ingredients_val.as_array() {
                    for ing in arr {
                        extract_ingredients_from_value(ing, &mut ingredients);
                    }
                } else {
                    extract_ingredients_from_value(ingredients_val, &mut ingredients);
                }
            }

            // Try key-based ingredients
            if let Some(key) = value.get("key").and_then(|k| k.as_object()) {
                for (_symbol, ingredient) in key {
                    extract_ingredients_from_value(ingredient, &mut ingredients);
                }
            }

            // Try input/inputs for modded recipes
            if let Some(input) = value.get("input").or(value.get("inputs")) {
                if let Some(arr) = input.as_array() {
                    for ing in arr {
                        extract_ingredients_from_value(ing, &mut ingredients);
                    }
                } else {
                    extract_ingredients_from_value(input, &mut ingredients);
                }
            }
        }
    }

    // Deduplicate ingredients
    ingredients.sort();
    ingredients.dedup();

    Ok(ParsedRecipe {
        recipe_type,
        result_item,
        result_count,
        ingredients,
    })
}

fn extract_item_and_count(value: &Value) -> (Option<String>, Option<i32>) {
    match value {
        // Simple string: "minecraft:iron_ingot"
        Value::String(s) => (Some(s.clone()), Some(1)),

        // Object with item/id and optional count
        Value::Object(obj) => {
            let item = obj.get("item")
                .or(obj.get("id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let count = obj.get("count")
                .and_then(|c| c.as_i64())
                .map(|c| c as i32)
                .or(Some(1));

            (item, count)
        }

        _ => (None, None),
    }
}

fn extract_ingredients_from_value(value: &Value, ingredients: &mut Vec<String>) {
    match value {
        // Simple string: "minecraft:iron_ingot"
        Value::String(s) => {
            ingredients.push(s.clone());
        }

        // Object with item or tag
        Value::Object(obj) => {
            if let Some(item) = obj.get("item").and_then(|v| v.as_str()) {
                ingredients.push(item.to_string());
            } else if let Some(tag) = obj.get("tag").and_then(|v| v.as_str()) {
                // Store tags with a prefix so we can identify them
                ingredients.push(format!("#{}", tag));
            }
        }

        // Array of alternatives (any of these items work)
        Value::Array(arr) => {
            for item in arr {
                extract_ingredients_from_value(item, ingredients);
            }
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaped_recipe() {
        let json = r####"{
            "type": "minecraft:crafting_shaped",
            "pattern": ["###", " | ", " | "],
            "key": {
                "#": {"item": "minecraft:iron_ingot"},
                "|": {"item": "minecraft:stick"}
            },
            "result": {"item": "minecraft:iron_pickaxe", "count": 1}
        }"####;

        let parsed = parse_recipe(json).unwrap();
        assert_eq!(parsed.recipe_type, "minecraft:crafting_shaped");
        assert_eq!(parsed.result_item, Some("minecraft:iron_pickaxe".to_string()));
        assert_eq!(parsed.result_count, Some(1));
        assert!(parsed.ingredients.contains(&"minecraft:iron_ingot".to_string()));
        assert!(parsed.ingredients.contains(&"minecraft:stick".to_string()));
    }

    #[test]
    fn test_shapeless_recipe() {
        let json = r#"{
            "type": "minecraft:crafting_shapeless",
            "ingredients": [
                {"item": "minecraft:red_dye"},
                {"item": "minecraft:white_dye"}
            ],
            "result": {"item": "minecraft:pink_dye", "count": 2}
        }"#;

        let parsed = parse_recipe(json).unwrap();
        assert_eq!(parsed.recipe_type, "minecraft:crafting_shapeless");
        assert_eq!(parsed.result_item, Some("minecraft:pink_dye".to_string()));
        assert_eq!(parsed.result_count, Some(2));
    }

    #[test]
    fn test_smelting_recipe() {
        let json = r#"{
            "type": "minecraft:smelting",
            "ingredient": {"item": "minecraft:iron_ore"},
            "result": "minecraft:iron_ingot",
            "experience": 0.7,
            "cookingtime": 200
        }"#;

        let parsed = parse_recipe(json).unwrap();
        assert_eq!(parsed.recipe_type, "minecraft:smelting");
        assert!(parsed.ingredients.contains(&"minecraft:iron_ore".to_string()));
    }

    #[test]
    fn test_tag_ingredient() {
        let json = r####"{
            "type": "minecraft:crafting_shaped",
            "pattern": ["###"],
            "key": {
                "#": {"tag": "forge:ingots/iron"}
            },
            "result": {"item": "minecraft:iron_block"}
        }"####;

        let parsed = parse_recipe(json).unwrap();
        assert!(parsed.ingredients.contains(&"#forge:ingots/iron".to_string()));
    }
}
