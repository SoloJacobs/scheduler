#![cfg(target_os = "linux")]
use libc;

pub fn interrupt(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGINT);
    }
}

pub fn kill_gracefully(child_id: i32) {
    unsafe {
        libc::kill(child_id, libc::SIGTERM);
    }
}
