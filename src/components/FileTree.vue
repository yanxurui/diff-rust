<script setup lang="ts">
import { useAppStore, type FileTreeNode, type FileEntry } from '../stores/app';
import { useDiff } from '../composables/useDiff';
import TreeItem from './TreeItem.vue';

const store = useAppStore();
const { loadDiff } = useDiff();

function handleFileClick(node: FileTreeNode) {
  if (!node.is_dir) {
    const file: FileEntry = {
      path: node.path,
      name: node.name,
      status: node.status || 'Unchanged',
      is_dir: false,
      left_path: node.left_path,
      right_path: node.right_path,
    };
    loadDiff(file);
  }
}
</script>

<template>
  <div class="file-tree h-full overflow-y-auto bg-gray-900 border-r border-gray-700">
    <!-- Header -->
    <div class="sticky top-0 bg-gray-900 border-b border-gray-700 px-3 py-2 z-10">
      <h2 class="text-sm font-semibold text-gray-300">Changed Files</h2>
      <div class="text-xs text-gray-500 mt-1">
        <span class="text-green-400">+{{ store.addedCount }}</span>
        <span class="mx-1">/</span>
        <span class="text-red-400">-{{ store.deletedCount }}</span>
        <span class="mx-1">/</span>
        <span class="text-yellow-400">~{{ store.modifiedCount }}</span>
      </div>
    </div>

    <!-- Tree -->
    <div class="py-1">
      <template v-if="store.isLoadingTree">
        <div class="px-3 py-8 text-center text-gray-500">
          <div class="animate-spin w-6 h-6 border-2 border-gray-600 border-t-blue-500 rounded-full mx-auto mb-2"></div>
          Loading files...
        </div>
      </template>
      <template v-else-if="store.treeError">
        <div class="px-3 py-4 text-red-400 text-sm">
          {{ store.treeError }}
        </div>
      </template>
      <template v-else-if="store.fileTree.length === 0">
        <div class="px-3 py-8 text-center text-gray-500 text-sm">
          No changes found
        </div>
      </template>
      <template v-else>
        <TreeItem
          v-for="node in store.fileTree"
          :key="node.path"
          :node="node"
          :depth="0"
          :selected-path="store.selectedFile?.path"
          @select="handleFileClick"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.file-tree {
  min-width: 200px;
}
</style>
