use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{DebouncedEvent, RecursiveMode, watcher, Watcher};
use notify::DebouncedEvent::Write;

use crate::content_compiler;

// https://docs.rs/notify/4.0.15/notify/
pub fn start_monitor() {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

    watcher.watch("./content", RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(path)) => {
                content_compiler::load_site_content();
            }
            Err(e) => println!("watch error: {:?}", e),
            _ => {}
        }
    }
}