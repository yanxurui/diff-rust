<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Props {
  initialLeftWidth?: number;
  minLeftWidth?: number;
  maxLeftWidth?: number;
}

const props = withDefaults(defineProps<Props>(), {
  initialLeftWidth: 250,
  minLeftWidth: 150,
  maxLeftWidth: 500,
});

const leftWidth = ref(props.initialLeftWidth);
const isDragging = ref(false);
const containerRef = ref<HTMLElement | null>(null);

function startDrag(event: MouseEvent) {
  isDragging.value = true;
  event.preventDefault();
}

function onDrag(event: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return;

  const containerRect = containerRef.value.getBoundingClientRect();
  const newWidth = event.clientX - containerRect.left;

  leftWidth.value = Math.max(
    props.minLeftWidth,
    Math.min(props.maxLeftWidth, newWidth)
  );
}

function stopDrag() {
  isDragging.value = false;
}

onMounted(() => {
  window.addEventListener('mousemove', onDrag);
  window.addEventListener('mouseup', stopDrag);
});

onUnmounted(() => {
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
});
</script>

<template>
  <div
    ref="containerRef"
    class="split-pane flex h-full"
    :class="{ 'select-none': isDragging }"
  >
    <!-- Left Panel -->
    <div
      class="left-panel flex-shrink-0 h-full overflow-hidden"
      :style="{ width: `${leftWidth}px` }"
    >
      <slot name="left"></slot>
    </div>

    <!-- Divider -->
    <div
      class="divider w-1 bg-gray-700 hover:bg-blue-500 cursor-col-resize flex-shrink-0 transition-colors"
      :class="{ 'bg-blue-500': isDragging }"
      @mousedown="startDrag"
    ></div>

    <!-- Right Panel -->
    <div class="right-panel flex-1 h-full overflow-hidden min-w-0">
      <slot name="right"></slot>
    </div>
  </div>
</template>

<style scoped>
.split-pane {
  user-select: none;
}

.split-pane:not(.select-none) {
  user-select: auto;
}

.divider:active {
  background-color: #3b82f6;
}
</style>
