<script setup lang="ts">
import { ref, computed } from 'vue';
import type { FileTreeNode } from '../stores/app';

const props = defineProps<{
  node: FileTreeNode;
  depth: number;
  selectedPath?: string;
}>();

const emit = defineEmits<{
  select: [node: FileTreeNode];
}>();

const isExpanded = ref(true);
const showContextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);

const statusColors: Record<string, string> = {
  Added: 'text-green-400',
  Deleted: 'text-red-400',
  Modified: 'text-yellow-400',
  Renamed: 'text-blue-400',
};

const statusLabels: Record<string, string> = {
  Added: 'A',
  Deleted: 'D',
  Modified: 'M',
  Renamed: 'R',
};

function toggle() {
  if (props.node.is_dir) {
    isExpanded.value = !isExpanded.value;
  }
}

function handleClick() {
  if (!props.node.is_dir) {
    emit('select', props.node);
  } else {
    toggle();
  }
}

function handleChildSelect(node: FileTreeNode) {
  emit('select', node);
}

function handleContextMenu(e: MouseEvent) {
  if (props.node.is_dir) return;
  e.preventDefault();
  contextMenuX.value = e.clientX;
  contextMenuY.value = e.clientY;
  showContextMenu.value = true;

  // Close menu when clicking elsewhere
  const closeMenu = () => {
    showContextMenu.value = false;
    document.removeEventListener('click', closeMenu);
  };
  setTimeout(() => document.addEventListener('click', closeMenu), 0);
}

async function copyPath(type: 'left' | 'right') {
  const path = type === 'left' ? props.node.left_path : props.node.right_path;
  if (path) {
    await navigator.clipboard.writeText(path);
  }
  showContextMenu.value = false;
}

const isSelected = computed(() => props.selectedPath === props.node.path);
const paddingLeft = computed(() => `${props.depth * 12 + 8}px`);
</script>

<template>
  <div>
    <div
      class="flex items-center py-1 px-2 cursor-pointer hover:bg-gray-800 transition-colors"
      :class="{ 'bg-blue-900/30': isSelected }"
      :style="{ paddingLeft }"
      @click="handleClick"
      @contextmenu="handleContextMenu"
      tabindex="0"
      role="treeitem"
    >
      <span class="mr-1.5 text-gray-500 flex-shrink-0 w-4">
        <template v-if="node.is_dir">
          <svg v-if="isExpanded" class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
          <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
          </svg>
        </template>
        <template v-else>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </template>
      </span>

      <span
        class="truncate text-sm flex-1"
        :class="node.is_dir ? 'text-gray-300 font-medium' : 'text-gray-400'"
      >
        {{ node.name }}
      </span>

      <span
        v-if="node.status"
        class="ml-2 text-xs font-mono flex-shrink-0"
        :class="statusColors[node.status]"
      >
        {{ statusLabels[node.status] }}
      </span>
    </div>

    <div v-if="node.is_dir && isExpanded && node.children && node.children.length > 0">
      <TreeItem
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :selected-path="selectedPath"
        @select="handleChildSelect"
      />
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="showContextMenu"
        class="context-menu"
        :style="{ left: contextMenuX + 'px', top: contextMenuY + 'px' }"
      >
        <div
          v-if="node.left_path"
          class="context-menu-item"
          @click="copyPath('left')"
        >
          Copy left path
        </div>
        <div
          v-if="node.right_path"
          class="context-menu-item"
          @click="copyPath('right')"
        >
          Copy right path
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style>
.context-menu {
  position: fixed;
  background: #1f2937;
  border: 1px solid #374151;
  border-radius: 6px;
  padding: 4px 0;
  min-width: 160px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.context-menu-item {
  padding: 8px 12px;
  font-size: 13px;
  color: #d1d5db;
  cursor: pointer;
  transition: background-color 0.1s;
}

.context-menu-item:hover {
  background: #374151;
}
</style>
