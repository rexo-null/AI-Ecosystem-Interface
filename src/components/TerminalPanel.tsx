import { useEffect, useRef, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { i18n } from '../i18n';
import '@xterm/xterm/css/xterm.css';

interface TerminalTab {
  id: string;
  name: string;
  terminal: Terminal;
  fitAddon: FitAddon;
  isAlive: boolean;
}

interface TerminalOutput {
  terminal_id: string;
  data: string;
}

let tabCounter = 0;

export default function TerminalPanel() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [tabs, setTabs] = useState<TerminalTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const tabsRef = useRef<TerminalTab[]>([]);

  // Keep ref in sync with state
  useEffect(() => {
    tabsRef.current = tabs;
  }, [tabs]);

  // Listen for terminal output events
  useEffect(() => {
    const unlistenOutput = listen<TerminalOutput>('terminal-output', (event) => {
      const { terminal_id, data } = event.payload;
      const tab = tabsRef.current.find(t => t.id === terminal_id);
      if (tab) {
        tab.terminal.write(data);
      }
    });

    const unlistenExit = listen<string>('terminal-exit', (event) => {
      const terminalId = event.payload;
      setTabs(prev => prev.map(t =>
        t.id === terminalId ? { ...t, isAlive: false } : t
      ));
    });

    return () => {
      unlistenOutput.then(fn => fn());
      unlistenExit.then(fn => fn());
    };
  }, []);

  // Auto-create first terminal on mount
  useEffect(() => {
    if (tabs.length === 0) {
      createNewTab();
    }
  }, []);

  // Fit terminal when active tab changes or container resizes
  useEffect(() => {
    const activeTab = tabs.find(t => t.id === activeTabId);
    if (activeTab && containerRef.current) {
      // Hide all terminals, show active
      tabs.forEach(t => {
        if (t.terminal.element) {
          t.terminal.element.style.display = t.id === activeTabId ? 'block' : 'none';
        }
      });
      activeTab.fitAddon.fit();
      activeTab.terminal.focus();
    }
  }, [activeTabId, tabs]);

  // Observe container resize
  useEffect(() => {
    if (!containerRef.current) return;
    const observer = new ResizeObserver(() => {
      const activeTab = tabsRef.current.find(t => t.id === activeTabId);
      if (activeTab) {
        try { activeTab.fitAddon.fit(); } catch {}
      }
    });
    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, [activeTabId]);

  const createNewTab = useCallback(async () => {
    if (!containerRef.current) return;

    tabCounter++;
    const terminalId = `term-${Date.now()}-${tabCounter}`;
    const name = `Terminal ${tabCounter}`;

    const terminal = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: 'Consolas, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#cccccc',
        cursor: '#ffffff',
        selectionBackground: '#264f78',
      },
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    terminal.open(containerRef.current);
    fitAddon.fit();

    // Send user input to PTY backend
    terminal.onData((data: string) => {
      invoke('terminal_write', { terminalId, data }).catch(err => {
        console.error('terminal_write error:', err);
      });
    });

    // Handle resize — notify backend
    terminal.onResize(({ cols, rows }) => {
      invoke('terminal_resize', { terminalId, cols, rows }).catch(() => {});
    });

    const newTab: TerminalTab = {
      id: terminalId,
      name,
      terminal,
      fitAddon,
      isAlive: true,
    };

    // Register tab BEFORE creating PTY so event handler can find it
    tabsRef.current = [...tabsRef.current, newTab];
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(terminalId);

    // Create PTY session on backend (output events can now be handled)
    try {
      await invoke('terminal_create', {
        terminalId,
        cols: terminal.cols,
        rows: terminal.rows,
      });
    } catch (err) {
      terminal.writeln(`\r\n\x1b[31mFailed to create terminal: ${err}\x1b[0m`);
    }
  }, []);

  const closeTab = useCallback(async (tabId: string) => {
    const tab = tabsRef.current.find(t => t.id === tabId);
    if (tab) {
      tab.terminal.dispose();
      try {
        await invoke('terminal_close', { terminalId: tabId });
      } catch {}
    }

    setTabs(prev => prev.filter(t => t.id !== tabId));
    setActiveTabId(prevActive => {
      if (prevActive !== tabId) return prevActive;
      const remaining = tabsRef.current.filter(t => t.id !== tabId);
      return remaining.length > 0 ? remaining[remaining.length - 1].id : null;
    });
  }, []);

  // Write command to active terminal (used by ChatPanel integration)
  const writeToActiveTerminal = useCallback(async (command: string) => {
    if (!activeTabId) return;
    try {
      await invoke('terminal_write', { terminalId: activeTabId, data: command + '\n' });
    } catch (err) {
      console.error('Failed to write to terminal:', err);
    }
  }, [activeTabId]);

  // Expose for external use via window
  useEffect(() => {
    (window as unknown as Record<string, unknown>).__iskin_terminal_write = writeToActiveTerminal;
    return () => {
      delete (window as unknown as Record<string, unknown>).__iskin_terminal_write;
    };
  }, [writeToActiveTerminal]);

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Tab bar */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '2px',
        padding: '2px 4px',
        borderBottom: '1px solid var(--border)',
        minHeight: '28px',
        flexShrink: 0,
      }}>
        {tabs.map(tab => (
          <div
            key={tab.id}
            onClick={() => setActiveTabId(tab.id)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
              padding: '2px 8px',
              fontSize: '11px',
              cursor: 'pointer',
              borderRadius: '3px',
              background: tab.id === activeTabId ? 'var(--bg-secondary)' : 'transparent',
              color: tab.isAlive ? 'var(--text-primary)' : 'var(--text-secondary)',
            }}
          >
            <span style={{
              width: '6px',
              height: '6px',
              borderRadius: '50%',
              background: tab.isAlive ? '#4caf50' : '#f44336',
              flexShrink: 0,
            }} />
            <span>{tab.name}</span>
            <span
              onClick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
              style={{
                marginLeft: '4px',
                cursor: 'pointer',
                opacity: 0.5,
                fontSize: '13px',
                lineHeight: 1,
              }}
            >
              x
            </span>
          </div>
        ))}
        <button
          onClick={createNewTab}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-secondary)',
            cursor: 'pointer',
            fontSize: '14px',
            padding: '0 6px',
            lineHeight: 1,
          }}
          title={i18n.t('terminal.title')}
        >
          +
        </button>
        <div style={{ flex: 1 }} />
        <button
          onClick={() => {
            const activeTab = tabs.find(t => t.id === activeTabId);
            if (activeTab) activeTab.terminal.clear();
          }}
          className="btn"
          style={{ padding: '1px 6px', fontSize: '10px' }}
        >
          {i18n.t('terminal.clear')}
        </button>
      </div>

      {/* Terminal container */}
      <div
        ref={containerRef}
        style={{
          flex: 1,
          overflow: 'hidden',
          background: '#1e1e1e',
        }}
      />
    </div>
  );
}
