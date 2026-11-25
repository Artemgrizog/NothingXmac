//! Minimal file log: ~/Library/Logs/NothingX.log (truncated at start-up).

use std::io::Write;
use std::sync::Mutex;

static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init() {
    if let Ok(home) = std::env::var("HOME") {
        let dir = std::path::PathBuf::from(home).join("Library/Logs");
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(f) = std::fs::File::create(dir.join("NothingX.log")) {
            *FILE.lock().unwrap() = Some(f);
        }
    }
}

pub fn write(msg: &str) {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    if let Some(f) = FILE.lock().unwrap().as_mut() {
        let _ = writeln!(f, "{:.3} {}", t % 100000.0, msg);
    }
}

#[macro_export]
macro_rules! log {
    ($($a:tt)*) => { $crate::log::write(&format!($($a)*)) };
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")
}
