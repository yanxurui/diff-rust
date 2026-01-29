<script setup lang="ts">
import { useAppStore } from '../stores/app';
import { useDiff } from '../composables/useDiff';

const store = useAppStore();
const { toggleSideBySide, toggleWhitespace, toggleCollapsed } = useDiff();
</script>

<template>
  <div class="toolbar flex items-center gap-3 px-4 py-2 bg-gray-800 border-b border-gray-700">
    <!-- View Mode Toggle (adjacent buttons) -->
    <div class="flex items-center">
      <button
        class="px-3 py-1.5 text-xs font-medium rounded-l border border-gray-600 transition-colors"
        :class="!store.viewOptions.side_by_side
          ? 'bg-blue-600 text-white border-blue-600'
          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'"
        @click="toggleSideBySide"
        title="Inline view"
      >
        Inline
      </button>
      <button
        class="px-3 py-1.5 text-xs font-medium rounded-r border border-l-0 border-gray-600 transition-colors"
        :class="store.viewOptions.side_by_side
          ? 'bg-blue-600 text-white border-blue-600'
          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'"
        @click="toggleSideBySide"
        title="Side-by-side view"
      >
        Side-by-side
      </button>
    </div>

    <!-- Divider -->
    <div class="w-px h-6 bg-gray-600"></div>

    <!-- Collapsed/Full Toggle -->
    <button
      class="px-2 py-1.5 text-xs font-medium rounded transition-colors flex items-center gap-1"
      :class="store.viewOptions.collapsed
        ? 'bg-gray-700 text-gray-300 hover:bg-gray-600'
        : 'bg-blue-600 text-white'"
      @click="toggleCollapsed"
      :title="store.viewOptions.collapsed ? 'Show full file' : 'Show only changes'"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path v-if="store.viewOptions.collapsed" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
        <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25" />
      </svg>
      <span>{{ store.viewOptions.collapsed ? 'Changes' : 'Full' }}</span>
    </button>

    <!-- Divider -->
    <div class="w-px h-6 bg-gray-600"></div>

    <!-- Whitespace Toggle -->
    <button
      class="px-2 py-1.5 text-xs font-medium rounded transition-colors flex items-center gap-1"
      :class="store.viewOptions.show_whitespace
        ? 'bg-blue-600 text-white'
        : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
      @click="toggleWhitespace"
      :title="store.viewOptions.show_whitespace ? 'Hide whitespace changes' : 'Show whitespace changes'"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h8M8 12h8m-8 5h8" />
      </svg>
      <span>Whitespace</span>
    </button>

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- File indicator -->
    <span v-if="store.selectedFile" class="text-xs text-gray-400">
      {{ store.currentFileIndex + 1 }} / {{ store.changedFiles.length }} files
    </span>
  </div>
</template>

<style scoped>
.toolbar {
  min-height: 44px;
}
</style>
