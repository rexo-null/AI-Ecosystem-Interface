use anyhow::{Context, Result};
use log::{info, warn, error};
use portable_pty::{native_pty_system, CommandBuilder, PtySize, PtyPair, MasterPty, Child};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::Emitter;

// ============================================================
// Types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub id: String,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub is_alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub terminal_id: String,
    pub data: String,
}

// ============================================================
// Terminal Session
// ============================================================

struct TerminalSession {
    id: String,
    shell: String,
    cols: u16,
    rows: u16,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    is_alive: Arc<AtomicBool>,
    child: Box<dyn Child + Send + Sync>,
}

// ============================================================
// Terminal Manager
// ============================================================

pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new terminal session with a PTY
    pub fn create(
        &self,
        terminal_id: String,
        cols: u16,
        rows: u16,
        app_handle: tauri::AppHandle,
    ) -> Result<TerminalInfo> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair: PtyPair = pty_system
            .openpty(size)
            .context("Failed to open PTY")?;

        // Determine shell
        let shell = if cfg!(target_os = "windows") {
            "powershell.exe".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        };

        let mut cmd = CommandBuilder::new(&shell);
        if !cfg!(target_os = "windows") {
            cmd.arg("--login");
        }

        // Spawn the shell process
        let child = pair.slave
            .spawn_command(cmd)
            .context("Failed to spawn shell")?;

        // Get writer for input
        let writer = pair.master
            .take_writer()
            .context("Failed to get PTY writer")?;

        // Get reader for output
        let mut reader = pair.master
            .try_clone_reader()
            .context("Failed to get PTY reader")?;

        // Store master for resize support
        let master = pair.master;

        let is_alive = Arc::new(AtomicBool::new(true));
        let is_alive_clone = is_alive.clone();
        let tid = terminal_id.clone();

        // Spawn output reader thread
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        is_alive_clone.store(false, Ordering::SeqCst);
                        let _ = app_handle.emit("terminal-exit", &tid);
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app_handle.emit("terminal-output", TerminalOutput {
                            terminal_id: tid.clone(),
                            data,
                        });
                    }
                    Err(e) => {
                        error!("PTY read error for {}: {}", tid, e);
                        is_alive_clone.store(false, Ordering::SeqCst);
                        let _ = app_handle.emit("terminal-exit", &tid);
                        break;
                    }
                }
            }
        });

        let info = TerminalInfo {
            id: terminal_id.clone(),
            shell: shell.clone(),
            cols,
            rows,
            is_alive: true,
        };

        let session = TerminalSession {
            id: terminal_id.clone(),
            shell,
            cols,
            rows,
            writer,
            master,
            is_alive,
            child,
        };

        self.sessions.lock().unwrap().insert(terminal_id.clone(), session);
        info!("Terminal created: {}", terminal_id);

        Ok(info)
    }

    /// Write data to a terminal (user input)
    pub fn write(&self, terminal_id: &str, data: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(terminal_id)
            .context(format!("Terminal not found: {}", terminal_id))?;

        if !session.is_alive.load(Ordering::SeqCst) {
            anyhow::bail!("Terminal {} is no longer alive", terminal_id);
        }

        session.writer
            .write_all(data.as_bytes())
            .context("Failed to write to PTY")?;
        session.writer.flush().context("Failed to flush PTY")?;

        Ok(())
    }

    /// Resize a terminal
    pub fn resize(&self, terminal_id: &str, cols: u16, rows: u16) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(terminal_id)
            .context(format!("Terminal not found: {}", terminal_id))?;

        session.cols = cols;
        session.rows = rows;

        session.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }).context("Failed to resize PTY")?;

        info!("Terminal {} resized to {}x{}", terminal_id, cols, rows);
        Ok(())
    }

    /// Close a terminal session
    pub fn close(&self, terminal_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(terminal_id) {
            session.is_alive.store(false, Ordering::SeqCst);
            // Kill the child process
            let _ = session.child.kill();
            info!("Terminal closed: {}", terminal_id);
        } else {
            warn!("Terminal not found for close: {}", terminal_id);
        }
        Ok(())
    }

    /// List all terminal sessions
    pub fn list(&self) -> Vec<TerminalInfo> {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().map(|s| TerminalInfo {
            id: s.id.clone(),
            shell: s.shell.clone(),
            cols: s.cols,
            rows: s.rows,
            is_alive: s.is_alive.load(Ordering::SeqCst),
        }).collect()
    }
}
