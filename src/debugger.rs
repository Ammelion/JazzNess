use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Breakpoint {
    pub on_read: bool,
    pub on_write: bool,
}

impl Breakpoint {
    // ... (no changes to methods) ...
    pub fn on_read() -> Self {
        Self {
            on_read: true,
            on_write: false,
        }
    }
    pub fn on_write() -> Self {
        Self {
            on_read: false,
            on_write: true,
        }
    }
    pub fn on_rw() -> Self {
        Self {
            on_read: true,
            on_write: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DebuggerState {
    breakpoints: HashMap<u16, Breakpoint>,
    paused: bool,
}

#[derive(Debug)]
pub struct Debugger {
    breakpoints: HashMap<u16, Breakpoint>,
    pub paused: Arc<AtomicBool>,
}

impl Debugger {
    pub fn new() -> Self {
        Debugger {
            breakpoints: HashMap::new(),
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add_breakpoint(&mut self, addr: u16, bp: Breakpoint) {
        println!("[DEBUG] Breakpoint added at {:#06X} (Read: {}, Write: {})", addr, bp.on_read, bp.on_write);
        self.breakpoints.insert(addr, bp);
    }

    pub fn remove_breakpoint(&mut self, addr: u16) -> Option<Breakpoint> {
        println!("[DEBUG] Breakpoint removed from {:#06X}", addr);
        self.breakpoints.remove(&addr)
    }
    
    pub fn get_breakpoints(&self) -> Vec<u16> {
        self.breakpoints.keys().cloned().collect()
    }

    pub fn check_read(&self, addr: u16) {
        if let Some(bp) = self.breakpoints.get(&addr) {
            if bp.on_read {
                println!("[DEBUG] Read Breakpoint HIT at {:#06X}", addr);
                self.paused.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn check_write(&self, addr: u16, value: u8) {
        if let Some(bp) = self.breakpoints.get(&addr) {
            if bp.on_write {
                println!("[DEBUG] Write Breakpoint HIT at {:#06X} (Value: {:#04X})", addr, value);
                self.paused.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn save_state(&self) -> DebuggerState {
        DebuggerState {
            breakpoints: self.breakpoints.clone(),
            paused: self.paused.load(Ordering::SeqCst),
        }
    }

    pub fn load_state(&mut self, state: &DebuggerState) {
        self.breakpoints = state.breakpoints.clone();
        self.paused.store(state.paused, Ordering::SeqCst);
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}