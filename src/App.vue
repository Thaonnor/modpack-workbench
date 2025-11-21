<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

interface FileInfo {
  name: string;
  path: string;
}

interface JarEntry {
  name: string;
  is_dir: boolean;
}

interface Recipe {
  id: number;
  mod_name: string;
  path: string;
  recipe_type: string;
  result_item: string | null;
  result_count: number | null;
  ingredients: string[];
}

interface ExtractionResult {
  mods_processed: number;
  recipes_extracted: number;
  errors: string[];
}

interface ExtractionProgress {
  current: number;
  total: number;
  current_mod: string;
}

const selectedFolder = ref("");
const scanResults = ref<FileInfo[]>([]);
const scanError = ref("");
const selectedJar = ref<FileInfo | null>(null);
const jarContents = ref<JarEntry[]>([]);
const jarError = ref("");

// Recipe extraction state
const isExtracting = ref(false);
const extractionResult = ref<ExtractionResult | null>(null);
const extractionError = ref("");
const extractionProgress = ref<ExtractionProgress | null>(null);

// Event listener cleanup
let unlistenProgress: UnlistenFn | null = null;

onMounted(async () => {
  unlistenProgress = await listen<ExtractionProgress>("extraction-progress", (event) => {
    extractionProgress.value = event.payload;
  });
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
});

// Recipe browser state
const recipes = ref<Recipe[]>([]);
const recipeCount = ref(0);
const searchQuery = ref("");
const searchType = ref<"output" | "ingredient">("output");
const currentPage = ref(0);
const pageSize = 50;
const isSearching = ref(false);

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Select Mods Folder"
  });

  if (selected) {
    selectedFolder.value = selected as string;
  }
}

async function scanMods() {
  if (!selectedFolder.value) {
    scanError.value = "Please select a mods folder first";
    return;
  }

  try {
    scanError.value = "";
    selectedJar.value = null;
    jarContents.value = [];
    jarError.value = "";
    scanResults.value = await invoke<FileInfo[]>("scan_folder", { path: selectedFolder.value });
  } catch (e) {
    scanError.value = String(e);
    scanResults.value = [];
  }
}

async function viewJarContents(file: FileInfo) {
  try {
    jarError.value = "";
    selectedJar.value = file;
    jarContents.value = await invoke<JarEntry[]>("get_jar_contents", { path: file.path });
  } catch (e) {
    jarError.value = String(e);
    jarContents.value = [];
  }
}

function closeJarViewer() {
  selectedJar.value = null;
  jarContents.value = [];
  jarError.value = "";
}

async function extractAllRecipes() {
  if (scanResults.value.length === 0) {
    extractionError.value = "No jars to extract from. Scan a mods folder first.";
    return;
  }

  try {
    isExtracting.value = true;
    extractionError.value = "";
    extractionResult.value = null;
    extractionProgress.value = null;

    const paths = scanResults.value.map(f => f.path);
    extractionResult.value = await invoke<ExtractionResult>("extract_all_recipes", { paths });

    // Load recipe count after extraction
    await loadRecipeCount();
    // Load first page of recipes
    await loadRecipes();
  } catch (e) {
    extractionError.value = String(e);
  } finally {
    isExtracting.value = false;
    extractionProgress.value = null;
  }
}

async function loadRecipeCount() {
  try {
    recipeCount.value = await invoke<number>("get_recipe_count");
  } catch (e) {
    console.error("Failed to load recipe count:", e);
  }
}

async function loadRecipes() {
  try {
    isSearching.value = true;
    const offset = currentPage.value * pageSize;
    recipes.value = await invoke<Recipe[]>("list_recipes", { offset, limit: pageSize });
  } catch (e) {
    console.error("Failed to load recipes:", e);
  } finally {
    isSearching.value = false;
  }
}

async function searchRecipes() {
  if (!searchQuery.value.trim()) {
    currentPage.value = 0;
    await loadRecipes();
    return;
  }

  try {
    isSearching.value = true;
    if (searchType.value === "output") {
      recipes.value = await invoke<Recipe[]>("search_recipes_by_output", { item: searchQuery.value });
    } else {
      recipes.value = await invoke<Recipe[]>("search_recipes_by_ingredient", { item: searchQuery.value });
    }
  } catch (e) {
    console.error("Search failed:", e);
  } finally {
    isSearching.value = false;
  }
}

async function nextPage() {
  if ((currentPage.value + 1) * pageSize < recipeCount.value) {
    currentPage.value++;
    await loadRecipes();
  }
}

async function prevPage() {
  if (currentPage.value > 0) {
    currentPage.value--;
    await loadRecipes();
  }
}

function clearSearch() {
  searchQuery.value = "";
  currentPage.value = 0;
  loadRecipes();
}
</script>

<template>
  <main class="container">
    <h1>Modpack Workbench</h1>
    <p>Minecraft recipe scanner and analyzer</p>

    <div class="scanner-section">
      <h2>Select Mods Folder</h2>

      <div class="folder-picker">
        <div class="folder-display">
          <span v-if="selectedFolder" class="selected-path">{{ selectedFolder }}</span>
          <span v-else class="placeholder">No folder selected</span>
        </div>
        <button @click="selectFolder" class="folder-btn">Browse Folder</button>
      </div>

      <div class="scan-section">
        <button
          @click="scanMods"
          :disabled="!selectedFolder"
          class="scan-btn"
          :class="{ disabled: !selectedFolder }"
        >
          Scan Mods
        </button>
      </div>

      <div v-if="scanError" class="error">
        <p>{{ scanError }}</p>
      </div>

      <div v-if="scanResults.length > 0" class="panel results">
        <div class="results-header">
          <h3>Scan Results ({{ scanResults.length }} jar files found)</h3>
          <button
            @click="extractAllRecipes"
            :disabled="isExtracting"
            class="extract-btn"
          >
            {{ isExtracting ? "Extracting..." : "Extract All Recipes" }}
          </button>
        </div>

        <div v-if="extractionProgress" class="progress-section">
          <div class="progress-bar">
            <div
              class="progress-fill"
              :style="{ width: `${(extractionProgress.current / extractionProgress.total) * 100}%` }"
            ></div>
          </div>
          <p class="progress-text">
            {{ extractionProgress.current }} / {{ extractionProgress.total }}: {{ extractionProgress.current_mod }}
          </p>
        </div>

        <ul class="file-list">
          <li
            v-for="file in scanResults"
            :key="file.path"
            class="file-item clickable"
            :class="{ selected: selectedJar?.path === file.path }"
            :title="file.path"
            @click="viewJarContents(file)"
          >
            <span class="file-name">{{ file.name }}</span>
          </li>
        </ul>
      </div>

      <div v-if="extractionError" class="error">
        <p>{{ extractionError }}</p>
      </div>

      <div v-if="extractionResult" class="panel extraction-result">
        <h3>Extraction Complete</h3>
        <p>Processed {{ extractionResult.mods_processed }} mods, extracted {{ extractionResult.recipes_extracted }} recipes</p>
        <div v-if="extractionResult.errors.length > 0" class="extraction-errors">
          <p>{{ extractionResult.errors.length }} errors occurred:</p>
          <ul>
            <li v-for="(err, i) in extractionResult.errors.slice(0, 10)" :key="i">{{ err }}</li>
            <li v-if="extractionResult.errors.length > 10">... and {{ extractionResult.errors.length - 10 }} more</li>
          </ul>
        </div>
      </div>

      <div v-if="recipeCount > 0" class="panel recipe-browser">
        <h3>Recipe Browser ({{ recipeCount }} total)</h3>

        <div class="search-controls">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search recipes..."
            @keyup.enter="searchRecipes"
          />
          <select v-model="searchType">
            <option value="output">By Output</option>
            <option value="ingredient">By Ingredient</option>
          </select>
          <button @click="searchRecipes" :disabled="isSearching">Search</button>
          <button @click="clearSearch" v-if="searchQuery">Clear</button>
        </div>

        <div v-if="recipes.length > 0" class="recipe-list">
          <div v-for="recipe in recipes" :key="recipe.id" class="recipe-item">
            <div class="recipe-row">
              <div class="recipe-output">
                <span class="label">Output:</span>
                <span class="value">{{ recipe.result_item || "Unknown" }}</span>
                <span v-if="recipe.result_count && recipe.result_count > 1" class="count">x{{ recipe.result_count }}</span>
              </div>
              <div class="recipe-meta">
                <span class="recipe-type">{{ recipe.recipe_type }}</span>
              </div>
            </div>
            <div v-if="recipe.ingredients.length > 0" class="recipe-inputs">
              <span class="label">Inputs:</span>
              <span class="value">{{ recipe.ingredients.join(", ") }}</span>
            </div>
            <div class="recipe-source">
              <span class="label">Source:</span>
              <span class="value">{{ recipe.mod_name }}</span>
            </div>
          </div>
        </div>

        <div v-else-if="!isSearching" class="empty-state">
          <p>No recipes found</p>
        </div>

        <div v-if="!searchQuery && recipes.length > 0" class="pagination">
          <button @click="prevPage" :disabled="currentPage === 0">Previous</button>
          <span>Page {{ currentPage + 1 }}</span>
          <button @click="nextPage" :disabled="(currentPage + 1) * pageSize >= recipeCount">Next</button>
        </div>
      </div>

      <div v-if="selectedJar" class="panel jar-contents">
        <div class="jar-header">
          <h3>Contents of {{ selectedJar.name }}</h3>
          <button @click="closeJarViewer" class="close-btn">Close</button>
        </div>

        <div v-if="jarError" class="error">
          <p>{{ jarError }}</p>
        </div>

        <div v-else-if="jarContents.length > 0" class="contents-info">
          <p>{{ jarContents.length }} recipe entries found</p>
          <ul class="entry-list">
            <li v-for="entry in jarContents" :key="entry.name" class="entry-item">
              <span class="entry-name" :class="{ 'is-dir': entry.is_dir }">
                {{ entry.name }}
              </span>
            </li>
          </ul>
        </div>

        <div v-else class="empty-state">
          <p>No recipe entries found in this jar</p>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.scanner-section {
  max-width: 600px;
  margin: 2em auto;
  text-align: left;
}

.scanner-section h2 {
  margin-bottom: 1em;
  color: #333;
}

.folder-picker {
  display: flex;
  gap: 1em;
  margin-bottom: 2em;
  align-items: center;
}

.folder-display {
  flex: 1;
  padding: 0.8em;
  border: 2px dashed #ccc;
  border-radius: 8px;
  min-height: 1.5em;
  display: flex;
  align-items: center;
}

.selected-path {
  font-family: monospace;
  color: #333;
}

.placeholder {
  color: #999;
  font-style: italic;
}

.folder-btn {
  background: #646cff;
  color: white;
  white-space: nowrap;
}

.folder-btn:hover {
  background: #535bf2;
  border-color: #535bf2;
}

.scan-section {
  text-align: center;
  margin-bottom: 2em;
}

.scan-btn {
  background: #4caf50;
  color: white;
  font-size: 1.1em;
  padding: 0.8em 2em;
}

.scan-btn:hover:not(.disabled) {
  background: #45a049;
  border-color: #45a049;
}

.scan-btn.disabled {
  background: #ccc;
  color: #666;
  cursor: not-allowed;
}

.panel {
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  padding: 1.5em;
  margin-top: 2em;
}

.results h3 {
  margin-top: 0;
  color: #333;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1em;
}

.results-header h3 {
  margin: 0;
}

.extract-btn {
  background: #ff9800;
  color: white;
  font-size: 0.9em;
}

.extract-btn:hover:not(:disabled) {
  background: #f57c00;
  border-color: #f57c00;
}

.extract-btn:disabled {
  background: #ccc;
  color: #666;
  cursor: not-allowed;
}

.progress-section {
  margin: 1em 0;
}

.progress-bar {
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #4caf50;
  transition: width 0.2s ease;
}

.progress-text {
  margin: 0.5em 0 0 0;
  font-size: 0.85em;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.extraction-result h3 {
  margin-top: 0;
  color: #4caf50;
}

.extraction-errors {
  margin-top: 1em;
  padding: 0.5em;
  background: #fff3e0;
  border-radius: 4px;
}

.extraction-errors ul {
  margin: 0.5em 0 0 0;
  padding-left: 1.5em;
  font-size: 0.85em;
}

.recipe-browser h3 {
  margin-top: 0;
  color: #333;
}

.search-controls {
  display: flex;
  gap: 0.5em;
  margin-bottom: 1em;
  flex-wrap: wrap;
}

.search-controls input {
  flex: 1;
  min-width: 150px;
}

.search-controls select {
  padding: 0.6em;
  border-radius: 8px;
  border: 1px solid transparent;
  background: #fff;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

.recipe-list {
  max-height: 500px;
  overflow-y: auto;
}

.recipe-item {
  padding: 0.75em;
  border-bottom: 1px solid #e0e0e0;
}

.recipe-item:last-child {
  border-bottom: none;
}

.recipe-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1em;
}

.recipe-output,
.recipe-inputs,
.recipe-source {
  margin-bottom: 0.25em;
  font-size: 0.9em;
  line-height: 1.4;
}

.recipe-output .label,
.recipe-inputs .label,
.recipe-source .label {
  color: #666;
  margin-right: 0.5em;
}

.recipe-output .value {
  font-weight: 500;
  color: #1976d2;
}

.recipe-output .count {
  color: #666;
  margin-left: 0.25em;
}

.recipe-inputs .value {
  font-family: monospace;
  font-size: 0.85em;
  word-break: break-word;
}

.recipe-source .value {
  font-size: 0.85em;
  color: #888;
}

.recipe-meta {
  flex-shrink: 0;
}

.recipe-type {
  font-family: monospace;
  font-size: 0.75em;
  color: #666;
  background: #f0f0f0;
  padding: 0.2em 0.4em;
  border-radius: 3px;
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1em;
  margin-top: 1em;
  padding-top: 1em;
  border-top: 1px solid #e0e0e0;
}

.pagination button {
  padding: 0.4em 0.8em;
  font-size: 0.9em;
}

.pagination button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error {
  background: #ffebee;
  border: 1px solid #f44336;
  border-radius: 8px;
  padding: 1em;
  color: #c62828;
}

.file-list {
  list-style: none;
  padding: 0;
  margin: 0;
  max-height: 400px;
  overflow-y: auto;
}

.file-item {
  padding: 0.5em;
  border-bottom: 1px solid #e0e0e0;
}

.file-item:last-child {
  border-bottom: none;
}

.file-name {
  font-family: monospace;
  font-size: 0.9em;
}

.file-item.clickable {
  cursor: pointer;
}

.file-item.clickable:hover {
  background: #e3f2fd;
}

.file-item.selected {
  background: #bbdefb;
}


.jar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1em;
}

.jar-header h3 {
  margin: 0;
  color: #333;
  word-break: break-all;
}

.close-btn {
  background: #f44336;
  color: white;
  padding: 0.4em 0.8em;
  font-size: 0.9em;
}

.close-btn:hover {
  background: #d32f2f;
  border-color: #d32f2f;
}

.contents-info p {
  margin: 0 0 1em 0;
  color: #666;
}

.entry-list {
  list-style: none;
  padding: 0;
  margin: 0;
  max-height: 300px;
  overflow-y: auto;
}

.entry-item {
  padding: 0.3em 0.5em;
  border-bottom: 1px solid #e0e0e0;
}

.entry-item:last-child {
  border-bottom: none;
}

.entry-name {
  font-family: monospace;
  font-size: 0.85em;
  word-break: break-all;
}

.entry-name.is-dir {
  color: #1976d2;
}

.empty-state {
  text-align: center;
  color: #666;
  font-style: italic;
}

.empty-state p {
  margin: 0;
}

@media (prefers-color-scheme: dark) {
  .scanner-section h2 {
    color: #f6f6f6;
  }

  .folder-display {
    border-color: #555;
    background: #2a2a2a;
  }

  .selected-path {
    color: #f6f6f6;
  }

  .panel {
    background: #2a2a2a;
    border-color: #555;
  }

  .results h3,
  .jar-header h3 {
    color: #f6f6f6;
  }

  .error {
    background: #3e2723;
    border-color: #d32f2f;
    color: #ef9a9a;
  }

  .file-item {
    border-bottom-color: #555;
  }

  .file-item.clickable:hover {
    background: #3a3a3a;
  }

  .file-item.selected {
    background: #1565c0;
  }

  .contents-info p,
  .empty-state {
    color: #aaa;
  }

  .entry-item {
    border-bottom-color: #555;
  }

  .entry-name.is-dir {
    color: #64b5f6;
  }

  .extraction-result h3 {
    color: #81c784;
  }

  .extraction-errors {
    background: #3e2723;
  }

  .recipe-browser h3 {
    color: #f6f6f6;
  }

  .search-controls select {
    background: #0f0f0f98;
    color: #fff;
  }

  .recipe-item {
    border-bottom-color: #555;
  }

  .recipe-output .value {
    color: #64b5f6;
  }

  .recipe-output .label,
  .recipe-inputs .label,
  .recipe-source .label {
    color: #aaa;
  }

  .recipe-output .count {
    color: #aaa;
  }

  .recipe-source .value {
    color: #888;
  }

  .recipe-type {
    background: #444;
    color: #ccc;
  }

  .progress-bar {
    background: #444;
  }

  .progress-text {
    color: #aaa;
  }

  .pagination {
    border-top-color: #555;
  }
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>