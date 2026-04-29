import { create } from 'zustand';

// ============================================================
// Types
// ============================================================

interface FileContent {
  path: string;
  content: string;
  language?: string;
}

type MemoryType = 'Constitution' | 'Protocol' | 'Pattern' | 'UserRule' | 'ToolDefinition' | 'ProjectContext';

interface MemoryEntry {
  id: string;
  title: string;
  content: string;
  memory_type: MemoryType;
  tags: string[];
  priority: number;
  access_count: number;
  created_at: number;
  updated_at: number;
}

interface CodeSymbol {
  name: string;
  kind: string;
  start_line: number;
  end_line: number;
  start_col: number;
  end_col: number;
  signature: string | null;
  doc_comment: string | null;
}

interface SearchCodeResult {
  file_path: string;
  language: string;
  symbols: CodeSymbol[];
  line_count: number;
  preview: string;
}

interface IndexStats {
  total_files: number;
  total_symbols: number;
  total_lines: number;
  total_bytes: number;
  languages: Record<string, number>;
}

interface RuleEntry {
  id: string;
  name: string;
  description: string;
  priority: string;
  is_active: boolean;
  tags: string[];
  created_at: number;
  updated_at: number;
}

interface KnowledgeBaseStats {
  total_entries: number;
  active_entries: number;
  type_counts: Record<string, number>;
  total_access_count: number;
}

// ============================================================
// Sandbox Types
// ============================================================

interface DockerStatus {
  connected: boolean;
  version: string | null;
  api_version: string | null;
  containers_running: number;
  images_count: number;
}

interface HealingStats {
  total_checks: number;
  total_recoveries: number;
  successful_recoveries: number;
  failed_recoveries: number;
  active_monitors: number;
  error_patterns_count: number;
  uptime_percentage: number;
}

interface SandboxStatusData {
  docker: DockerStatus;
  containers_count: number;
  vnc_sessions_count: number;
  browser_status: string;
  healing_stats: HealingStats;
}

interface ContainerInfo {
  id: string;
  docker_id: string | null;
  image: string;
  name: string | null;
  status: string;
  created_at: number;
  health_check_failures: number;
}

// ============================================================
// Agent Types
// ============================================================

type AgentPhase = 
  | 'ReceiveTask'
  | 'Decompose'
  | 'ImpactAssessment'
  | 'DryRun'
  | 'Execute'
  | 'Verify'
  | 'ArtifactSync'
  | 'Commit'
  | 'QueueNext';

interface ImpactReport {
  affected_files: string[];
  doc_sync_needed: boolean;
  tests_to_run: string[];
  rollback_plan: RollbackPlan;
  risk_level: RiskLevel;
}

interface RollbackPlan {
  steps: string[];
  estimated_time: number;
}

type RiskLevel = 'low' | 'medium' | 'high' | 'critical';

interface AgentAction {
  id: string;
  phase: AgentPhase;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  timestamp: number;
  duration_ms?: number;
  error?: string;
}

interface AgentTask {
  id: string;
  description: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  subtasks: string[];
  created_at: number;
}

// ============================================================
// Tauri invoke wrapper (falls back to mock for browser dev)
// ============================================================

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    // @ts-ignore — Tauri injects __TAURI__ at runtime
    if (window.__TAURI__) {
      // @ts-ignore
      return await window.__TAURI__.core.invoke(cmd, args);
    }
  } catch (err) {
    console.warn(`Tauri invoke "${cmd}" failed:`, err);
  }
  throw new Error(`Backend unavailable for command: ${cmd}`);
}

// ============================================================
// Store
// ============================================================

interface EditorStore {
  // Files
  activeFile: FileContent | null;
  openTabs: FileContent[];
  setActiveFile: (file: FileContent) => void;
  openFile: (file: FileContent) => void;
  closeTab: (path: string) => void;

  // Agent State
  agentPhase: AgentPhase;
  agentActive: boolean;
  agentTasks: AgentTask[];
  currentTaskId: string | null;
  agentActions: AgentAction[];
  impactReport: ImpactReport | null;
  setAgentPhase: (phase: AgentPhase) => void;
  setAgentActive: (active: boolean) => void;
  addAgentTask: (task: Omit<AgentTask, 'id' | 'created_at'>) => void;
  updateTaskStatus: (taskId: string, status: AgentTask['status']) => void;
  addAgentAction: (action: Omit<AgentAction, 'id' | 'timestamp'>) => void;
  setImpactReport: (report: ImpactReport | null) => void;

  // Knowledge Base
  memoryEntries: MemoryEntry[];
  allMemoryEntries: MemoryEntry[];
  knowledgeLoading: boolean;
  knowledgeError: string | null;
  loadKnowledgeEntries: (memoryType?: string, tags?: string[]) => Promise<void>;
  addMemoryEntry: (entry: { title: string; content?: string; memory_type: string; tags: string[]; priority: number }) => Promise<void>;
  deleteMemoryEntry: (id: string) => Promise<void>;
  searchKnowledge: (query: string, memoryType?: string, limit?: number) => Promise<void>;
  getKnowledgeStats: () => Promise<KnowledgeBaseStats | null>;

  // Code Search (Tree-sitter)
  searchResults: SearchCodeResult[];
  indexStats: IndexStats | null;
  codeSearchLoading: boolean;
  indexProject: (path: string) => Promise<void>;
  searchCode: (query: string, language?: string, limit?: number) => Promise<void>;
  getFileSymbols: (filePath: string) => Promise<CodeSymbol[]>;
  refreshIndexStats: () => Promise<void>;

  // Rules Engine
  rules: RuleEntry[];
  rulesLoading: boolean;
  loadRules: () => Promise<void>;
  addRule: (params: {
    name: string;
    description: string;
    condition_type: string;
    condition_value: string;
    action_type: string;
    action_value?: string;
    priority?: string;
    tags: string[];
  }) => Promise<void>;
  deleteRule: (id: string) => Promise<void>;

  // Sandbox
  sandboxStatus: SandboxStatusData | null;
  containers: ContainerInfo[];
  sandboxLoading: boolean;
  loadSandboxStatus: () => Promise<void>;
  loadContainers: () => Promise<void>;
  createContainer: (params: { image: string; name?: string; memory_limit_mb?: number }) => Promise<void>;
  startContainer: (id: string) => Promise<void>;
  stopContainer: (id: string) => Promise<void>;
  removeContainer: (id: string) => Promise<void>;
  launchBrowser: () => Promise<void>;
  closeBrowser: () => Promise<void>;

  // Backward compatibility: localStorage fallback
  loadMemory: () => void;
  saveMemory: () => void;
}

const MEMORY_STORAGE_KEY = 'iskin_memory_entries';

export const useEditorStore = create<EditorStore>((set, get) => ({
  // Files
  activeFile: null,
  openTabs: [],
  
  // Agent State
  agentPhase: 'ReceiveTask',
  agentActive: false,
  agentTasks: [],
  currentTaskId: null,
  agentActions: [],
  impactReport: null,
  
  setAgentPhase: (phase) => set({ agentPhase: phase }),
  setAgentActive: (active) => set({ agentActive: active }),
  
  addAgentTask: (task) => {
    const newTask: AgentTask = {
      ...task,
      id: crypto.randomUUID(),
      created_at: Date.now(),
    };
    set((state) => ({ 
      agentTasks: [...state.agentTasks, newTask],
      currentTaskId: newTask.id,
    }));
  },
  
  updateTaskStatus: (taskId, status) => {
    set((state) => ({
      agentTasks: state.agentTasks.map(t => 
        t.id === taskId ? { ...t, status } : t
      ),
    }));
  },
  
  addAgentAction: (action) => {
    const newAction: AgentAction = {
      ...action,
      id: crypto.randomUUID(),
      timestamp: Date.now(),
    };
    set((state) => ({ 
      agentActions: [...state.agentActions, newAction],
    }));
  },
  
  setImpactReport: (report) => set({ impactReport: report }),
  
  // Memory
  memoryEntries: [],
  allMemoryEntries: [],
  knowledgeLoading: false,
  knowledgeError: null,
  searchResults: [],
  indexStats: null,
  codeSearchLoading: false,
  rules: [],
  rulesLoading: false,
  sandboxStatus: null,
  containers: [],
  sandboxLoading: false,

  setActiveFile: (file: FileContent) => {
    set({ activeFile: file });
  },

  openFile: (file: FileContent) => {
    set((state) => {
      const exists = state.openTabs.some(tab => tab.path === file.path);
      const newTabs = exists ? state.openTabs : [...state.openTabs, file];
      return { openTabs: newTabs, activeFile: file };
    });
  },

  closeTab: (path: string) => {
    set((state) => {
      const newTabs = state.openTabs.filter(tab => tab.path !== path);
      const newActive = state.activeFile?.path === path 
        ? (newTabs.length > 0 ? newTabs[newTabs.length - 1] : null)
        : state.activeFile;
      return { openTabs: newTabs, activeFile: newActive };
    });
  },

  // ============================================================
  // Knowledge Base
  // ============================================================

  loadKnowledgeEntries: async (memoryType?: string, tags?: string[]) => {
    set({ knowledgeLoading: true, knowledgeError: null });
    try {
      const params: Record<string, unknown> = {};
      if (memoryType || tags) {
        params.params = { memory_type: memoryType, tags };
      }
      const entries = await tauriInvoke<MemoryEntry[]>('get_knowledge_entries', params);
      set({ memoryEntries: entries, allMemoryEntries: entries, knowledgeLoading: false });
    } catch {
      // Fallback to localStorage
      get().loadMemory();
      const entries = get().memoryEntries;
      set({ allMemoryEntries: entries, knowledgeLoading: false });
    }
  },

  addMemoryEntry: async (entry) => {
    set({ knowledgeLoading: true });
    try {
      await tauriInvoke<string>('add_knowledge_entry', { params: entry });
      await get().loadKnowledgeEntries();
    } catch {
      // Fallback to localStorage
      const now = Date.now();
      const newEntry: MemoryEntry = {
        id: now.toString(),
        title: entry.title,
        content: entry.content || '',
        memory_type: entry.memory_type as MemoryType,
        tags: entry.tags,
        priority: entry.priority,
        access_count: 0,
        created_at: now,
        updated_at: now,
      };
      set((state) => ({
        memoryEntries: [...state.memoryEntries, newEntry],
        allMemoryEntries: [...state.allMemoryEntries, newEntry],
        knowledgeLoading: false,
      }));
      get().saveMemory();
    }
  },

  deleteMemoryEntry: async (id: string) => {
    try {
      await tauriInvoke<void>('delete_knowledge_entry', { id });
      await get().loadKnowledgeEntries();
    } catch {
      // Fallback
      set((state) => ({
        memoryEntries: state.memoryEntries.filter(e => e.id !== id),
        allMemoryEntries: state.allMemoryEntries.filter(e => e.id !== id),
      }));
      get().saveMemory();
    }
  },

  searchKnowledge: async (query: string, memoryType?: string, limit?: number) => {
    set({ knowledgeLoading: true, knowledgeError: null });
    try {
      const entries = await tauriInvoke<MemoryEntry[]>('search_knowledge', {
        params: { query, memory_type: memoryType, limit: limit || 50 },
      });
      set({ memoryEntries: entries, knowledgeLoading: false });
    } catch {
      // Fallback: local search from full dataset (not filtered)
      const { allMemoryEntries } = get();
      const q = query.toLowerCase();
      const filtered = allMemoryEntries.filter(e =>
        e.title.toLowerCase().includes(q) ||
        e.content.toLowerCase().includes(q) ||
        e.tags.some(t => t.toLowerCase().includes(q))
      );
      set({ memoryEntries: filtered, knowledgeLoading: false });
    }
  },

  getKnowledgeStats: async () => {
    try {
      return await tauriInvoke<KnowledgeBaseStats>('get_knowledge_stats');
    } catch {
      return null;
    }
  },

  // ============================================================
  // Code Search (Tree-sitter)
  // ============================================================

  indexProject: async (path: string) => {
    set({ codeSearchLoading: true });
    try {
      await tauriInvoke<{ files_indexed: number; stats: IndexStats }>('index_project', { path });
      await get().refreshIndexStats();
    } catch (err) {
      console.error('Index project failed:', err);
    } finally {
      set({ codeSearchLoading: false });
    }
  },

  searchCode: async (query: string, language?: string, limit?: number) => {
    set({ codeSearchLoading: true });
    try {
      const results = await tauriInvoke<SearchCodeResult[]>('search_code', {
        params: { query, language, limit: limit || 20 },
      });
      set({ searchResults: results, codeSearchLoading: false });
    } catch (err) {
      console.error('Search code failed:', err);
      set({ searchResults: [], codeSearchLoading: false });
    }
  },

  getFileSymbols: async (filePath: string) => {
    try {
      return await tauriInvoke<CodeSymbol[]>('get_file_symbols', { file_path: filePath });
    } catch {
      return [];
    }
  },

  refreshIndexStats: async () => {
    try {
      const stats = await tauriInvoke<IndexStats>('get_index_stats');
      set({ indexStats: stats });
    } catch {
      set({ indexStats: null });
    }
  },

  // ============================================================
  // Rules Engine
  // ============================================================

  loadRules: async () => {
    set({ rulesLoading: true });
    try {
      const rules = await tauriInvoke<RuleEntry[]>('list_rules');
      set({ rules, rulesLoading: false });
    } catch {
      set({ rulesLoading: false });
    }
  },

  addRule: async (params) => {
    try {
      await tauriInvoke<string>('add_rule', { params });
      await get().loadRules();
    } catch (err) {
      console.error('Add rule failed:', err);
    }
  },

  deleteRule: async (id: string) => {
    try {
      await tauriInvoke<void>('delete_rule', { id });
      await get().loadRules();
    } catch (err) {
      console.error('Delete rule failed:', err);
    }
  },

  // ============================================================
  // Sandbox
  // ============================================================

  loadSandboxStatus: async () => {
    set({ sandboxLoading: true });
    try {
      const status = await tauriInvoke<SandboxStatusData>('get_sandbox_status');
      set({ sandboxStatus: status, sandboxLoading: false });
    } catch {
      set({
        sandboxStatus: {
          docker: { connected: false, version: null, api_version: null, containers_running: 0, images_count: 0 },
          containers_count: 0,
          vnc_sessions_count: 0,
          browser_status: 'idle',
          healing_stats: {
            total_checks: 0, total_recoveries: 0, successful_recoveries: 0,
            failed_recoveries: 0, active_monitors: 0, error_patterns_count: 0, uptime_percentage: 100,
          },
        },
        sandboxLoading: false,
      });
    }
  },

  loadContainers: async () => {
    try {
      const containers = await tauriInvoke<ContainerInfo[]>('list_containers');
      set({ containers });
    } catch {
      set({ containers: [] });
    }
  },

  createContainer: async (params) => {
    try {
      await tauriInvoke<string>('create_container', { params });
      await get().loadContainers();
      await get().loadSandboxStatus();
    } catch (err) {
      console.error('Create container failed:', err);
    }
  },

  startContainer: async (id: string) => {
    try {
      await tauriInvoke<void>('start_container', { container_id: id });
      await get().loadContainers();
      await get().loadSandboxStatus();
    } catch (err) {
      console.error('Start container failed:', err);
    }
  },

  stopContainer: async (id: string) => {
    try {
      await tauriInvoke<void>('stop_container', { container_id: id });
      await get().loadContainers();
      await get().loadSandboxStatus();
    } catch (err) {
      console.error('Stop container failed:', err);
    }
  },

  removeContainer: async (id: string) => {
    try {
      await tauriInvoke<void>('remove_container', { container_id: id });
      await get().loadContainers();
      await get().loadSandboxStatus();
    } catch (err) {
      console.error('Remove container failed:', err);
    }
  },

  launchBrowser: async () => {
    try {
      await tauriInvoke<string>('launch_browser');
      await get().loadSandboxStatus();
    } catch (err) {
      console.error('Launch browser failed:', err);
    }
  },

  closeBrowser: async () => {
    try {
      await tauriInvoke<void>('close_browser');
      await get().loadSandboxStatus();
    } catch (err) {
      console.error('Close browser failed:', err);
    }
  },

  // ============================================================
  // localStorage fallback
  // ============================================================

  loadMemory: () => {
    try {
      const stored = localStorage.getItem(MEMORY_STORAGE_KEY);
      if (stored) {
        const entries = JSON.parse(stored);
        set({ memoryEntries: entries, allMemoryEntries: entries });
      }
    } catch (error) {
      console.error('Failed to load memory:', error);
    }
  },

  saveMemory: () => {
    try {
      const entries = get().allMemoryEntries;
      localStorage.setItem(MEMORY_STORAGE_KEY, JSON.stringify(entries));
    } catch (error) {
      console.error('Failed to save memory:', error);
    }
  },
}));

// Initialize: try loading from backend, fallback to localStorage
(async () => {
  try {
    await useEditorStore.getState().loadKnowledgeEntries();
  } catch {
    useEditorStore.getState().loadMemory();
  }
})();
