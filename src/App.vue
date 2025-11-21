<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-opener";

const selectedFolder = ref("");
const scanResults = ref("");

async function selectFolder() {
  // This will be implemented later with proper folder picker
  // For now, just show a placeholder
  selectedFolder.value = "C:\\path\\to\\mods\\folder";
}

async function scanMods() {
  if (!selectedFolder.value) {
    scanResults.value = "Please select a mods folder first";
    return;
  }

  // Call the scanner test for now
  scanResults.value = await invoke("scanner_test");
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

      <div v-if="scanResults" class="results">
        <h3>Scan Results</h3>
        <p>{{ scanResults }}</p>
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
  border: none;
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
  border: none;
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

.results {
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

  .results {
    background: #2a2a2a;
    border-color: #555;
  }

  .results h3 {
    color: #f6f6f6;
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

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
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

#greet-input {
  margin-right: 5px;
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