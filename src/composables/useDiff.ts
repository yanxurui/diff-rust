import { invoke } from '@tauri-apps/api/core';
import { useAppStore, type FileEntry, type FileTreeNode } from '../stores/app';

interface FileTreeResult {
  tree: FileTreeNode[];
  files: FileEntry[];
  total_changes: number;
  added: number;
  deleted: number;
  modified: number;
}

interface DiffResult {
  html: string;
  has_changes: boolean;
  hunk_count: number;
  left_html: string | null;
  right_html: string | null;
}

export function useDiff() {
  const store = useAppStore();

  async function loadFileTree(leftDir: string, rightDir: string): Promise<void> {
    console.log('loadFileTree called with:', leftDir, rightDir);
    store.setTreeLoading(true);
    store.setDirectories(leftDir, rightDir);

    try {
      console.log('Invoking get_file_tree...');
      const result = await invoke<FileTreeResult>('get_file_tree', {
        leftDir,
        rightDir,
      });
      console.log('get_file_tree result:', result);

      store.setFileTree(
        result.tree,
        result.files,
        result.total_changes,
        result.added,
        result.deleted,
        result.modified
      );
      console.log('Tree set, files:', result.files.length);

      // Auto-select first file if there are changes
      if (result.files.length > 0) {
        store.selectFirstFile();
      }
    } catch (error) {
      console.error('loadFileTree error:', error);
      store.setTreeError(String(error));
    }
  }

  async function loadDiff(file: FileEntry): Promise<void> {
    store.setDiffLoading(true);
    store.selectFile(file);

    try {
      const result = await invoke<DiffResult>('get_diff', {
        leftPath: file.left_path,
        rightPath: file.right_path,
        options: store.viewOptions,
      });

      store.setDiff(result);
    } catch (error) {
      store.setDiffError(String(error));
    }
  }

  async function refreshDiff(): Promise<void> {
    if (!store.selectedFile) return;
    await loadDiff(store.selectedFile);
  }

  async function checkDeltaInstalled(): Promise<boolean> {
    try {
      const installed = await invoke<boolean>('check_delta');
      store.setDeltaInstalled(installed);
      return installed;
    } catch {
      store.setDeltaInstalled(false);
      return false;
    }
  }

  async function getAppArgs(): Promise<string[]> {
    try {
      const args = await invoke<string[]>('get_app_args');
      console.log('App args:', args);
      return args;
    } catch (error) {
      console.error('getAppArgs error:', error);
      return [];
    }
  }

  async function readFileContent(path: string): Promise<string> {
    return await invoke<string>('read_file_content', { path });
  }

  function toggleSideBySide() {
    store.toggleViewOption('side_by_side');
    refreshDiff();
  }

  function toggleLineNumbers() {
    store.toggleViewOption('line_numbers');
    refreshDiff();
  }

  function toggleCollapsed() {
    store.toggleViewOption('collapsed');
    refreshDiff();
  }

  function toggleWhitespace() {
    store.toggleViewOption('show_whitespace');
    refreshDiff();
  }

  return {
    loadFileTree,
    loadDiff,
    refreshDiff,
    checkDeltaInstalled,
    getAppArgs,
    readFileContent,
    toggleSideBySide,
    toggleLineNumbers,
    toggleCollapsed,
    toggleWhitespace,
  };
}
