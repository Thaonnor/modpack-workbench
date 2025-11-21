<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface FileInfo {
  name: string;
  path: string;
}

interface JarEntry {
  name: string;
  is_dir: boolean;
}

const selectedFolder = ref("");
const scanResults = ref<FileInfo[]>([]);
const scanError = ref("");
const selectedJar = ref<FileInfo | null>(null);
const jarContents = ref<JarEntry[]>([]);
const jarError = ref("");

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
        <h3>Scan Results ({{ scanResults.length }} jar files found)</h3>
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