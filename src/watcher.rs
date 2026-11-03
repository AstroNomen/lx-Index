use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

pub struct VaultWatcher {
    _watcher: RecommendedWatcher,
    _handle: thread::JoinHandle<()>,
}

impl VaultWatcher {
    /// Starts watching the specified vault path recursively.
    /// Whenever a relevant change occurs (PDF or MD added, removed, or modified),
    /// the `on_change` callback is triggered with debounce.
    pub fn start<F>(vault_path: PathBuf, on_change: F) -> Result<Self, notify::Error>
    where
        F: Fn() + Send + 'static,
    {
        let (tx, rx): (std::sync::mpsc::Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) = channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(&vault_path, RecursiveMode::Recursive)?;

        let handle = thread::spawn(move || {
            let mut last_trigger = Instant::now() - Duration::from_secs(10);
            let debounce_interval = Duration::from_millis(500);

            while let Ok(res) = rx.recv() {
                if let Ok(event) = res {
                    // Check if event paths involve .pdf or .md files, ignoring hidden files (.git, .lxindex, etc.)
                    let is_relevant = event.paths.iter().any(|path| {
                        let path_str = path.to_string_lossy();
                        if path_str.contains("/.") || path_str.contains("\\.") {
                            return false;
                        }

                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            ext.eq_ignore_ascii_case("pdf") || ext.eq_ignore_ascii_case("md")
                        } else {
                            false
                        }
                    });

                    if is_relevant {
                        let now = Instant::now();
                        if now.duration_since(last_trigger) >= debounce_interval {
                            last_trigger = now;
                            // Wait a short delay for file writing to finish
                            thread::sleep(Duration::from_millis(150));
                            on_change();
                        }
                    }
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            _handle: handle,
        })
    }
}

pub type SharedWatcher = Arc<Mutex<Option<VaultWatcher>>>;

pub fn create_shared_watcher() -> SharedWatcher {
    Arc::new(Mutex::new(None))
}

pub fn restart_watcher<F>(shared: &SharedWatcher, vault_path: &Path, on_change: F)
where
    F: Fn() + Send + 'static,
{
    let mut lock = shared.lock().unwrap();
    // Dropping previous watcher stops it
    *lock = None;

    if vault_path.exists() {
        if let Ok(w) = VaultWatcher::start(vault_path.to_path_buf(), on_change) {
            *lock = Some(w);
        }
    }
}
