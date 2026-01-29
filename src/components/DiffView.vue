<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted, computed } from 'vue';
import { useAppStore } from '../stores/app';
import { useDiff } from '../composables/useDiff';

const store = useAppStore();
const { loadDiff } = useDiff();

const diffContainer = ref<HTMLElement | null>(null);
const leftPanel = ref<HTMLElement | null>(null);
const rightPanel = ref<HTMLElement | null>(null);
const currentHunkIndex = ref(0);
const hunkElements = ref<NodeListOf<Element> | null>(null);

// Side-by-side divider dragging
const dividerPosition = ref(50); // percentage
const isDragging = ref(false);

// Check if we're in side-by-side mode with separate panels
const isSideBySide = computed(() => {
  return store.currentDiff?.left_html && store.currentDiff?.right_html;
});

// Watch for selected file changes and load diff
watch(() => store.selectedFile, async (file) => {
  if (file) {
    await loadDiff(file);
    await nextTick();
    findHunks();
    currentHunkIndex.value = 0;
  }
});

// Watch for diff content changes
watch(() => store.currentDiff?.html, async () => {
  await nextTick();
  findHunks();
});

function findHunks() {
  if (diffContainer.value) {
    // Look for hunk separator lines (delta outputs lines with ─── pattern)
    // or find lines with changed content (colored backgrounds)
    const allLines = diffContainer.value.querySelectorAll('.diff-line');
    const hunks: Element[] = [];
    let inChange = false;

    allLines.forEach((line) => {
      const html = line.innerHTML;
      // Detect hunk separators (lines with ─ characters) or changed lines (colored spans)
      const isHunkSep = html.includes('───') || html.includes('────');
      const isChangedLine = html.includes('background') || html.includes('bg-');

      if (isHunkSep) {
        hunks.push(line);
        inChange = false;
      } else if (isChangedLine && !inChange) {
        // First changed line after context - this is a hunk start
        hunks.push(line);
        inChange = true;
      } else if (!isChangedLine) {
        inChange = false;
      }
    });

    hunkElements.value = hunks.length > 0 ? hunks as unknown as NodeListOf<Element> : null;
  }
}

function scrollToHunk(index: number) {
  if (hunkElements.value && index >= 0 && index < hunkElements.value.length) {
    hunkElements.value[index].scrollIntoView({ behavior: 'smooth', block: 'start' });
    currentHunkIndex.value = index;
  }
}

function nextHunk() {
  if (hunkElements.value && currentHunkIndex.value < hunkElements.value.length - 1) {
    scrollToHunk(currentHunkIndex.value + 1);
  }
}

function prevHunk() {
  if (hunkElements.value && currentHunkIndex.value > 0) {
    scrollToHunk(currentHunkIndex.value - 1);
  }
}

// Divider dragging
function startDrag(event: MouseEvent) {
  isDragging.value = true;
  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
  event.preventDefault();
}

function onDrag(event: MouseEvent) {
  if (!isDragging.value || !diffContainer.value) return;

  const rect = diffContainer.value.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = (x / rect.width) * 100;

  // Clamp between 20% and 80%
  dividerPosition.value = Math.max(20, Math.min(80, percentage));
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
}

// Synchronized scrolling for side-by-side
const isSyncing = ref(false);

function syncScroll(source: 'left' | 'right') {
  if (!leftPanel.value || !rightPanel.value || isSyncing.value) return;

  isSyncing.value = true;

  const sourceEl = source === 'left' ? leftPanel.value : rightPanel.value;
  const targetEl = source === 'left' ? rightPanel.value : leftPanel.value;

  // Sync vertical scroll (by percentage)
  const scrollTop = sourceEl.scrollTop;
  const scrollHeight = sourceEl.scrollHeight - sourceEl.clientHeight;
  const scrollPercentY = scrollHeight > 0 ? scrollTop / scrollHeight : 0;
  const targetScrollHeight = targetEl.scrollHeight - targetEl.clientHeight;
  targetEl.scrollTop = scrollPercentY * targetScrollHeight;

  // Sync horizontal scroll (absolute position - both panels should align)
  targetEl.scrollLeft = sourceEl.scrollLeft;

  // Prevent feedback loop
  requestAnimationFrame(() => {
    isSyncing.value = false;
  });
}

// Keyboard navigation
function handleKeydown(event: KeyboardEvent) {
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
    return;
  }

  switch (event.key) {
    case 'j':
      event.preventDefault();
      store.selectNextFile();
      break;
    case 'k':
      event.preventDefault();
      store.selectPrevFile();
      break;
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

defineExpose({ nextHunk, prevHunk });
</script>

<template>
  <div class="diff-view-container h-full flex flex-col bg-gray-900">
    <!-- File header -->
    <div v-if="store.selectedFile" class="file-header px-4 py-2 bg-gray-800 border-b border-gray-700">
      <div class="flex items-center gap-2">
        <span
          class="text-xs font-mono px-1.5 py-0.5 rounded"
          :class="{
            'bg-green-900 text-green-300': store.selectedFile.status === 'Added',
            'bg-red-900 text-red-300': store.selectedFile.status === 'Deleted',
            'bg-yellow-900 text-yellow-300': store.selectedFile.status === 'Modified',
            'bg-blue-900 text-blue-300': store.selectedFile.status === 'Renamed',
          }"
        >
          {{ store.selectedFile.status }}
        </span>
        <span class="text-sm text-gray-300 font-mono">{{ store.selectedFile.path }}</span>
      </div>
    </div>

    <!-- Diff content -->
    <div class="flex-1 overflow-auto" ref="diffContainer">
      <!-- Loading -->
      <div v-if="store.isLoadingDiff" class="diff-loading">
        <div class="spinner"></div>
        <span>Loading diff...</span>
      </div>

      <!-- Error -->
      <div v-else-if="store.diffError" class="diff-error">
        <div class="error-icon">!</div>
        <div>{{ store.diffError }}</div>
      </div>

      <!-- No selection -->
      <div v-else-if="!store.selectedFile" class="flex items-center justify-center h-full text-gray-500">
        <div class="text-center">
          <svg class="w-16 h-16 mx-auto mb-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-lg">Select a file to view diff</p>
          <p class="text-sm mt-2">Use the file tree on the left</p>
        </div>
      </div>

      <!-- Side-by-side view with draggable divider -->
      <div
        v-else-if="store.currentDiff && isSideBySide"
        class="sbs-container"
      >
        <!-- Left panel (old) -->
        <div
          ref="leftPanel"
          class="sbs-left"
          :style="{ flexBasis: `calc(${dividerPosition}% - 4px)` }"
          @scroll="syncScroll('left')"
          v-html="store.currentDiff.left_html"
        ></div>

        <!-- Draggable divider -->
        <div
          class="sbs-divider"
          :class="{ 'sbs-divider-dragging': isDragging }"
          @mousedown="startDrag"
        >
          <div class="sbs-divider-line"></div>
        </div>

        <!-- Right panel (new) -->
        <div
          ref="rightPanel"
          class="sbs-right"
          :style="{ flexBasis: `calc(${100 - dividerPosition}% - 4px)` }"
          @scroll="syncScroll('right')"
          v-html="store.currentDiff.right_html"
        ></div>
      </div>

      <!-- Inline delta output -->
      <div
        v-else-if="store.currentDiff"
        class="diff-view"
        v-html="store.currentDiff.html"
      ></div>
    </div>
  </div>
</template>

<style>
@import '../styles/delta.css';
</style>

<style scoped>
.diff-view-container {
  min-width: 0;
}

.file-header {
  flex-shrink: 0;
}

.diff-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #6b7280;
  gap: 8px;
}

.diff-loading .spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #374151;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.diff-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #ef4444;
  padding: 20px;
}

.diff-error .error-icon {
  width: 48px;
  height: 48px;
  border: 2px solid currentColor;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: bold;
  margin-bottom: 12px;
}

/* Side-by-side container */
.sbs-container {
  display: flex;
  height: 100%;
  overflow: hidden;
  width: 100%;
}

.sbs-left,
.sbs-right {
  overflow: auto;
  height: 100%;
  min-width: 0;
  flex-shrink: 1;
  flex-grow: 0;
}

.sbs-left :deep(pre),
.sbs-right :deep(pre) {
  min-width: fit-content;
}

.sbs-left {
  border-right: none;
}

.sbs-right {
  border-left: none;
}

/* Draggable divider */
.sbs-divider {
  width: 8px;
  flex-shrink: 0;
  background: #1f2937;
  cursor: col-resize;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s;
}

.sbs-divider:hover,
.sbs-divider-dragging {
  background: #374151;
}

.sbs-divider-line {
  width: 2px;
  height: 40px;
  background: #4b5563;
  border-radius: 1px;
}

.sbs-divider:hover .sbs-divider-line,
.sbs-divider-dragging .sbs-divider-line {
  background: #6b7280;
}
</style>
