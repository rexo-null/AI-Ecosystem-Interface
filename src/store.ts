import { create } from 'zustand';

interface FileContent {
  path: string;
  content: string;
  language?: string;
}

interface MemoryEntry {
  id: string;
  title: string;
  type: 'Constitution' | 'Protocol' | 'Pattern' | 'UserRule' | 'Tool';
  content?: string;
  tags: string[];
  priority: number;
  createdAt: number;
  updatedAt: number;
}

interface EditorStore {
  // Файлы
  activeFile: FileContent | null;
  openTabs: FileContent[];
  setActiveFile: (file: FileContent) => void;
  openFile: (file: FileContent) => void;
  closeTab: (path: string) => void;
  
  // Память
  memoryEntries: MemoryEntry[];
  addMemoryEntry: (entry: Omit<MemoryEntry, 'id' | 'createdAt' | 'updatedAt'>) => void;
  updateMemoryEntry: (id: string, entry: Partial<MemoryEntry>) => void;
  deleteMemoryEntry: (id: string) => void;
  loadMemory: () => void;
  saveMemory: () => void;
}

const MEMORY_STORAGE_KEY = 'iskin_memory_entries';

export const useEditorStore = create<EditorStore>((set, get) => ({
  activeFile: null,
  openTabs: [],
  memoryEntries: [],

  setActiveFile: (file: FileContent) => {
    set({ activeFile: file });
  },

  openFile: (file: FileContent) => {
    set((state) => {
      const exists = state.openTabs.some(tab => tab.path === file.path);
      const newTabs = exists ? state.openTabs : [...state.openTabs, file];
      return {
        openTabs: newTabs,
        activeFile: file,
      };
    });
  },

  closeTab: (path: string) => {
    set((state) => {
      const newTabs = state.openTabs.filter(tab => tab.path !== path);
      const newActive = state.activeFile?.path === path 
        ? (newTabs.length > 0 ? newTabs[newTabs.length - 1] : null)
        : state.activeFile;
      
      return {
        openTabs: newTabs,
        activeFile: newActive,
      };
    });
  },

  addMemoryEntry: (entry) => {
    set((state) => {
      const newEntry: MemoryEntry = {
        ...entry,
        id: Date.now().toString(),
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      return {
        memoryEntries: [...state.memoryEntries, newEntry],
      };
    });
    get().saveMemory();
  },

  updateMemoryEntry: (id: string, entry: Partial<MemoryEntry>) => {
    set((state) => {
      return {
        memoryEntries: state.memoryEntries.map(e =>
          e.id === id
            ? {
                ...e,
                ...entry,
                updatedAt: Date.now(),
              }
            : e
        ),
      };
    });
    get().saveMemory();
  },

  deleteMemoryEntry: (id: string) => {
    set((state) => {
      return {
        memoryEntries: state.memoryEntries.filter(e => e.id !== id),
      };
    });
    get().saveMemory();
  },

  loadMemory: () => {
    try {
      const stored = localStorage.getItem(MEMORY_STORAGE_KEY);
      if (stored) {
        const entries = JSON.parse(stored);
        set({ memoryEntries: entries });
      }
    } catch (error) {
      console.error('Failed to load memory:', error);
    }
  },

  saveMemory: () => {
    try {
      const entries = get().memoryEntries;
      localStorage.setItem(MEMORY_STORAGE_KEY, JSON.stringify(entries));
    } catch (error) {
      console.error('Failed to save memory:', error);
    }
  },
}));

// Инициализация памяти при загрузке приложения
useEditorStore.getState().loadMemory();
