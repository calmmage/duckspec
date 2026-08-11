//! Shared filesystem fixtures for duckcore (and UI) tests.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FS_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique temp directory, removed on drop.
pub struct FsTmp(PathBuf);

impl Default for FsTmp {
    fn default() -> Self {
        Self::new()
    }
}

impl FsTmp {
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let counter = FS_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!("duckcore-test-{nanos}-{counter}"));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for FsTmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The single lock guarding `HOME` mutation across all test modules.
static HOME_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` with `HOME` set to `home` so `paths::data_dir` resolves under it,
/// restoring the previous value afterward. Serialised through `HOME_LOCK`.
pub fn with_home<R>(home: &Path, f: impl FnOnce() -> R) -> R {
    let _guard = HOME_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let prev = std::env::var_os("HOME");
    // SAFETY: tests serialise through HOME_LOCK so concurrent set_var is impossible.
    unsafe { std::env::set_var("HOME", home) };
    let out = f();
    // SAFETY: same lock guarantees no concurrent reader observing the mutation race.
    unsafe {
        match prev {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
    out
}
