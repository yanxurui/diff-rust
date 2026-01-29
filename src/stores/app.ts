import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface FileEntry {
  path: string;
  name: string;
  status: 'Added' | 'Deleted' | 'Modified' | 'Renamed' | 'Unchanged';
  is_dir: boolean;
  left_path: string | null;
  right_path: string | null;
}

export interface FileTreeNode {
  name: string;
  path: string;
  status: 'Added' | 'Deleted' | 'Modified' | 'Renamed' | null;
  is_dir: boolean;
  children: FileTreeNode[];
  left_path: string | null;
  right_path: string | null;
}

export interface DiffOptions {
  side_by_side: boolean;
  line_numbers: boolean;
  collapsed: boolean;
  show_whitespace: boolean;
}

export interface DiffResult {
  html: string;
  has_changes: boolean;
  hunk_count: number;
  left_html: string | null;
  right_html: string | null;
}

export const useAppStore = defineStore('app', () => {
  // Directory paths
  const leftDir = ref<string | null>(null);
  const rightDir = ref<string | null>(null);

  // File tree
  const fileTree = ref<FileTreeNode[]>([]);
  const files = ref<FileEntry[]>([]);
  const totalChanges = ref(0);
  const addedCount = ref(0);
  const deletedCount = ref(0);
  const modifiedCount = ref(0);

  // Selected file
  const selectedFile = ref<FileEntry | null>(null);

  // Diff result
  const currentDiff = ref<DiffResult | null>(null);
  const isLoadingDiff = ref(false);
  const diffError = ref<string | null>(null);

  // View options
  const viewOptions = ref<DiffOptions>({
    side_by_side: true,
    line_numbers: true,
    collapsed: true,
    show_whitespace: false,
  });

  // Delta availability
  const deltaInstalled = ref(true);

  // Loading state
  const isLoadingTree = ref(false);
  const treeError = ref<string | null>(null);

  // Computed
  const hasSelection = computed(() => selectedFile.value !== null);
  const hasChanges = computed(() => totalChanges.value > 0);

  // Flatten file tree in display order (directories first, then alphabetically)
  function flattenTree(nodes: FileTreeNode[]): FileEntry[] {
    const result: FileEntry[] = [];
    for (const node of nodes) {
      if (node.is_dir) {
        // Recurse into directories
        result.push(...flattenTree(node.children));
      } else if (node.status) {
        // Add file entry
        result.push({
          path: node.path,
          name: node.name,
          status: node.status,
          is_dir: false,
          left_path: node.left_path,
          right_path: node.right_path,
        });
      }
    }
    return result;
  }

  // Get all changed files in tree display order (for navigation)
  const changedFiles = computed(() => {
    return flattenTree(fileTree.value);
  });

  // Current file index in the list
  const currentFileIndex = computed(() => {
    if (!selectedFile.value) return -1;
    return changedFiles.value.findIndex(f => f.path === selectedFile.value?.path);
  });

  // Navigation
  const canGoPrev = computed(() => currentFileIndex.value > 0);
  const canGoNext = computed(() => currentFileIndex.value < changedFiles.value.length - 1);

  // Actions
  function setDirectories(left: string, right: string) {
    leftDir.value = left;
    rightDir.value = right;
  }

  function setFileTree(
    tree: FileTreeNode[],
    fileList: FileEntry[],
    total: number,
    added: number,
    deleted: number,
    modified: number
  ) {
    fileTree.value = tree;
    files.value = fileList;
    totalChanges.value = total;
    addedCount.value = added;
    deletedCount.value = deleted;
    modifiedCount.value = modified;
    treeError.value = null;
    isLoadingTree.value = false;
  }

  function selectFile(file: FileEntry | null) {
    selectedFile.value = file;
    currentDiff.value = null;
    diffError.value = null;
  }

  function setDiff(result: DiffResult) {
    currentDiff.value = result;
    isLoadingDiff.value = false;
    diffError.value = null;
  }

  function setDiffLoading(loading: boolean) {
    isLoadingDiff.value = loading;
  }

  function setDiffError(error: string) {
    diffError.value = error;
    isLoadingDiff.value = false;
    currentDiff.value = null;
  }

  function setTreeLoading(loading: boolean) {
    isLoadingTree.value = loading;
  }

  function setTreeError(error: string) {
    treeError.value = error;
    isLoadingTree.value = false;
  }

  function setViewOption<K extends keyof DiffOptions>(key: K, value: DiffOptions[K]) {
    viewOptions.value[key] = value;
  }

  function toggleViewOption(key: keyof DiffOptions) {
    if (typeof viewOptions.value[key] === 'boolean') {
      (viewOptions.value[key] as boolean) = !(viewOptions.value[key] as boolean);
    }
  }

  function setDeltaInstalled(installed: boolean) {
    deltaInstalled.value = installed;
  }

  function selectPrevFile() {
    if (canGoPrev.value) {
      selectFile(changedFiles.value[currentFileIndex.value - 1]);
    }
  }

  function selectNextFile() {
    if (canGoNext.value) {
      selectFile(changedFiles.value[currentFileIndex.value + 1]);
    }
  }

  function selectFirstFile() {
    if (changedFiles.value.length > 0) {
      selectFile(changedFiles.value[0]);
    }
  }

  return {
    // State
    leftDir,
    rightDir,
    fileTree,
    files,
    totalChanges,
    addedCount,
    deletedCount,
    modifiedCount,
    selectedFile,
    currentDiff,
    isLoadingDiff,
    diffError,
    viewOptions,
    deltaInstalled,
    isLoadingTree,
    treeError,

    // Computed
    hasSelection,
    hasChanges,
    changedFiles,
    currentFileIndex,
    canGoPrev,
    canGoNext,

    // Actions
    setDirectories,
    setFileTree,
    selectFile,
    setDiff,
    setDiffLoading,
    setDiffError,
    setTreeLoading,
    setTreeError,
    setViewOption,
    toggleViewOption,
    setDeltaInstalled,
    selectPrevFile,
    selectNextFile,
    selectFirstFile,
  };
});
