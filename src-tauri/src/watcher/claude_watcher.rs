use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum ClaudeWatcherEvent {
    SessionCreated(PathBuf),
    SessionModified(PathBuf),
    SessionDeleted(PathBuf),
}

pub struct ClaudeWatcher {
    _watcher: RecommendedWatcher,
    _debounce_sender: mpsc::Sender<(ClaudeWatcherEvent, std::time::Instant)>,
}

impl ClaudeWatcher {
    pub fn new(
        debounce_duration: Duration,
        event_sender: mpsc::Sender<ClaudeWatcherEvent>,
    ) -> Result<Self> {
        // Create debouncing channel
        let (debounce_sender, mut debounce_receiver) = mpsc::channel::<(ClaudeWatcherEvent, std::time::Instant)>(100);

        // Start the debouncer in a background task
        tokio::spawn(async move {
            let mut last_events: std::collections::HashMap<PathBuf, std::time::Instant> = std::collections::HashMap::new();

            while let Some((event, timestamp)) = debounce_receiver.recv().await {
                let path = match &event {
                    ClaudeWatcherEvent::SessionCreated(path) => path,
                    ClaudeWatcherEvent::SessionModified(path) => path,
                    ClaudeWatcherEvent::SessionDeleted(path) => path,
                }.clone();

                let should_emit = if let Some(last_time) = last_events.get(&path) {
                    timestamp.duration_since(*last_time) >= debounce_duration
                } else {
                    true // First event for this path, always emit
                };

                if should_emit {
                    if let Err(e) = event_sender.send(event).await {
                        log::error!("Failed to send debounced event: {}", e);
                        break;
                    }
                    last_events.insert(path, timestamp);
                } else {
                    // Update timestamp but don't emit event (debounced)
                    last_events.insert(path, timestamp);
                }
            }
        });

        let event_tx = debounce_sender.clone();
        let watcher_config = Config::default()
            .with_poll_interval(Duration::from_millis(500))
            .with_compare_contents(false);

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    if let Some(path) = event.paths.first() {
                        let event_type = match event.kind {
                            notify::EventKind::Create(_) => ClaudeWatcherEvent::SessionCreated(path.clone()),
                            notify::EventKind::Modify(_) => ClaudeWatcherEvent::SessionModified(path.clone()),
                            notify::EventKind::Remove(_) => ClaudeWatcherEvent::SessionDeleted(path.clone()),
                            _ => return, // Ignore other event types
                        };

                        // Send event to debouncer (blocking send in sync context)
                        if let Err(e) = event_tx.blocking_send((event_type, std::time::Instant::now())) {
                            log::error!("Failed to send event to debouncer: {}", e);
                        }
                    }
                }
            },
            watcher_config,
        )?;

        Ok(ClaudeWatcher {
            _watcher: watcher,
            _debounce_sender: debounce_sender,
        })
    }

    pub fn watch_sessions_directory(&mut self, sessions_dir: &PathBuf) -> Result<()> {
        self._watcher.watch(sessions_dir, RecursiveMode::Recursive)
    }
}