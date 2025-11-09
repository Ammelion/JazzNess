// src/debugger.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Serialize, Deserialize}; // Import

/// Defines the conditions for a breakpoint.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)] // Add Serialize/Deserialize
pub struct Breakpoint {
    pub on_read: bool,
    pub on_write: bool,
    // We could add on_execute here, but that requires CPU integration.
    // For memory/bus, read/write is what we need.
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

// --- ADD THIS STRUCT ---
#[derive(Serialize, Deserialize)]
pub struct DebuggerState {
    breakpoints: HashMap<u16, Breakpoint>,
    paused: bool,
}
// --- END STRUCT ---


/// The main Debugger struct.
/// It holds the breakpoints and a shared flag to signal the emulator to pause.
#[derive(Debug)]
pub struct Debugger {
    breakpoints: HashMap<u16, Breakpoint>,
    /// A shared, thread-safe flag.
    /// The debugger sets this to `true` when a breakpoint is hit.
    /// The main emulator loop should check this and pause.
    pub paused: Arc<AtomicBool>,
}

impl Debugger {
    /// Creates a new Debugger, starting in a non-paused state.
    pub fn new() -> Self {
        Debugger {
            breakpoints: HashMap::new(),
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Adds a new breakpoint at a specific address.
    pub fn add_breakpoint(&mut self, addr: u16, bp: Breakpoint) {
        println!("[DEBUG] Breakpoint added at {:#06X} (Read: {}, Write: {})", addr, bp.on_read, bp.on_write);
        self.breakpoints.insert(addr, bp);
    }

    /// Removes a breakpoint from an address.
    pub fn remove_breakpoint(&mut self, addr: u16) -> Option<Breakpoint> {
        println!("[DEBUG] Breakpoint removed from {:#06X}", addr);
        self.breakpoints.remove(&addr)
    }
    
    /// Gets a list of all active breakpoint addresses.
    pub fn get_breakpoints(&self) -> Vec<u16> {
        self.breakpoints.keys().cloned().collect()
    }

    /// Checks if a memory read at `addr` should trigger a breakpoint.
    /// This should be called by `bus_read` *before* the read happens.
    pub fn check_read(&self, addr: u16) {
        if let Some(bp) = self.breakpoints.get(&addr) {
            if bp.on_read {
                println!("[DEBUG] Read Breakpoint HIT at {:#06X}", addr);
                self.paused.store(true, Ordering::SeqCst);
            }
        }
    }

    /// Checks if a memory write at `addr` should trigger a breakpoint.
    /// This should be called by `bus_write` *before* the write happens.
    pub fn check_write(&self, addr: u16, value: u8) {
        if let Some(bp) = self.breakpoints.get(&addr) {
            if bp.on_write {
                println!("[DEBUG] Write Breakpoint HIT at {:#06X} (Value: {:#04X})", addr, value);
                self.paused.store(true, Ordering::SeqCst);
            }
        }
    }

    // --- ADD THESE METHODS ---
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
    // --- END METHODS ---
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}