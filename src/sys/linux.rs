#![cfg(target_os = "linux")]
use tokio::process::Child;

pub fn interrupt(child: &mut Child) {
    if let Some(pid) = child.id() {
        unsafe {
            let gid = libc::getpgid(pid as i32);
            libc::kill(gid, libc::SIGINT);
        }
    }
}
