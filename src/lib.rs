use std::ops::{Deref, DerefMut};

// The signals documented by SignalKind, expect for SIGINFO (which does not exist on Linux).
pub const ALL_INTERESTING_SIGNALS: &[Signal] = &[
    // By default, these signals terminate the process:
    Signal(libc::SIGALRM), // A real-time timer has expired.
    Signal(libc::SIGHUP),  // The terminal is disconnected.
    Signal(libc::SIGINT),  // Interrupt a program.
    Signal(libc::SIGPIPE), // The process attempts to write to a pipe which has no reader.
    Signal(libc::SIGQUIT), // Issue a shutdown of the process, after which the OS will dump the process core.
    Signal(libc::SIGTERM), // Issue a shutdown of the process.
    Signal(libc::SIGUSR1), // A user defined signal.
    Signal(libc::SIGUSR2), // A user defined signal.
    // By default, these signals are ignored by the process:
    Signal(libc::SIGIO), // When I/O operations are possible on some file descriptor.
    Signal(libc::SIGCHLD), // The status of a child process has changed.
    Signal(libc::SIGWINCH), // The terminal window is resized.
];

#[derive(Debug, Clone, Copy)]
pub struct Signal(libc::c_int);

impl Deref for Signal {
    type Target = libc::c_int;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Signal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Signal(code) = self;
        let name = signal_name(*code);
        write!(f, "{name}")
    }
}

fn signal_name(signal: libc::c_int) -> &'static str {
    match signal {
        libc::SIGALRM => "SIGALRM",
        libc::SIGCHLD => "SIGCHLD",
        libc::SIGHUP => "SIGHUP",
        libc::SIGINT => "SIGINT",
        libc::SIGIO => "SIGIO",
        libc::SIGPIPE => "SIGPIPE",
        libc::SIGQUIT => "SIGQUIT",
        libc::SIGTERM => "SIGTERM",
        libc::SIGUSR1 => "SIGUSR1",
        libc::SIGUSR2 => "SIGUSR2",
        libc::SIGWINCH => "SIGWINCH",
        _ => "UNKNOWN",
    }
}
