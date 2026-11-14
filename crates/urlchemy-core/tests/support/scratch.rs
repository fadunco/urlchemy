use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Scratch(PathBuf);
impl Scratch {
    pub fn new() -> Self {
        for _ in 0..100 {
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("urlchemy-test-{}-{time}-{n}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(e) => panic!("Cannot create test directory: {e}"),
            }
        }
        panic!("Cannot allocate test directory");
    }
    pub fn root(&self) -> &Path {
        &self.0
    }
    pub fn file(&self) -> PathBuf {
        self.0.join("config.yaml")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.file());
        let _ = std::fs::remove_dir(&self.0);
    }
}
