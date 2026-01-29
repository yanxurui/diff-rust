<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useAppStore } from './stores/app';
import { useDiff } from './composables/useDiff';
import { getCurrentWindow } from '@tauri-apps/api/window';
import SplitPane from './components/SplitPane.vue';
import FileTree from './components/FileTree.vue';
import Toolbar from './components/Toolbar.vue';
import DiffView from './components/DiffView.vue';

const store = useAppStore();
const { loadFileTree, checkDeltaInstalled, getAppArgs } = useDiff();


const leftDirInput = ref('');
const rightDirInput = ref('');
const showDirectoryPicker = ref(true);

async function updateWindowTitle(leftDir: string, rightDir: string) {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.setTitle(`${leftDir} vs ${rightDir}`);
  } catch (e) {
    console.error('Failed to set window title:', e);
  }
}

async function initialize() {
  // Check if delta is installed
  await checkDeltaInstalled();

  // Check command-line arguments
  const args = await getAppArgs();

  // Args format: [executable, left_dir, right_dir]
  if (args.length >= 3) {
    const leftDir = args[1];
    const rightDir = args[2];

    if (leftDir && rightDir) {
      leftDirInput.value = leftDir;
      rightDirInput.value = rightDir;
      showDirectoryPicker.value = false;
      await loadFileTree(leftDir, rightDir);
      await updateWindowTitle(leftDir, rightDir);
    }
  }
}

async function handleLoadDirs() {
  if (leftDirInput.value && rightDirInput.value) {
    showDirectoryPicker.value = false;
    await loadFileTree(leftDirInput.value, rightDirInput.value);
    await updateWindowTitle(leftDirInput.value, rightDirInput.value);
  }
}

function handleShowPicker() {
  showDirectoryPicker.value = true;
}

onMounted(() => {
  initialize();
});
</script>

<template>
  <div class="app h-screen w-screen flex flex-col bg-gray-900 text-gray-100">
    <!-- Directory Picker Modal -->
    <div
      v-if="showDirectoryPicker"
      class="directory-picker fixed inset-0 bg-gray-900 flex items-center justify-center z-50"
    >
      <div class="bg-gray-800 rounded-lg p-6 w-full max-w-lg mx-4 shadow-xl border border-gray-700">
        <h1 class="text-xl font-semibold mb-4 text-gray-100">diffr - Diff Viewer</h1>
        <p class="text-gray-400 text-sm mb-6">
          Enter the paths to two directories to compare their contents.
        </p>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Left Directory (Old)</label>
            <input
              v-model="leftDirInput"
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500"
              placeholder="/path/to/old/directory"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Right Directory (New)</label>
            <input
              v-model="rightDirInput"
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500"
              placeholder="/path/to/new/directory"
            />
          </div>

          <button
            @click="handleLoadDirs"
            :disabled="!leftDirInput || !rightDirInput"
            class="w-full py-2 px-4 bg-blue-600 text-white rounded-md font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Compare Directories
          </button>
        </div>

        <div class="mt-6 pt-4 border-t border-gray-700">
          <h3 class="text-sm font-medium text-gray-400 mb-2">Git Integration</h3>
          <div class="text-xs text-gray-500 font-mono bg-gray-900 p-3 rounded">
            <p>git config --global diff.tool diffr</p>
            <p>git config --global difftool.diffr.cmd 'diffr "$LOCAL" "$REMOTE"'</p>
            <p class="mt-2 text-gray-400"># Then use: git difftool --dir-diff</p>
          </div>
        </div>

      </div>
    </div>

    <!-- Main App -->
    <template v-else>
      <!-- Header with directory info -->
      <div class="header flex items-center gap-4 px-4 py-2 bg-gray-800 border-b border-gray-700 text-sm">
        <button
          @click="handleShowPicker"
          class="text-gray-400 hover:text-gray-200 transition-colors"
          title="Change directories"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </button>
        <div class="flex-1 flex items-center gap-2 text-gray-400 overflow-hidden">
          <span class="truncate" :title="store.leftDir || ''">{{ store.leftDir }}</span>
          <span class="text-gray-600">vs</span>
          <span class="truncate" :title="store.rightDir || ''">{{ store.rightDir }}</span>
        </div>
      </div>

      <!-- Toolbar -->
      <Toolbar />

      <!-- Main Content -->
      <div class="flex-1 overflow-hidden">
        <SplitPane :initial-left-width="280" :min-left-width="200" :max-left-width="500">
          <template #left>
            <FileTree />
          </template>
          <template #right>
            <DiffView />
          </template>
        </SplitPane>
      </div>
    </template>
  </div>
</template>

<style>
@import './styles/main.css';
</style>
