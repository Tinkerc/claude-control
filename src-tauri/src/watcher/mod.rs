//! Claude Session Watcher Module
//!
//! Monitors Claude Code session files for changes and notifies other components

mod claude_watcher;

pub use claude_watcher::{ClaudeWatcher, ClaudeWatcherEvent};