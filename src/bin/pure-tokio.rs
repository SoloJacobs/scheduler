use libc;
use tokio::signal::unix;
use tokio::task::JoinSet;

// The signals documented by SignalKind, expect for SIGINFO (which does not exist on Linux).
const ALL_SIGNALS: &'static [libc::c_int] = &[
    // By default, these signals terminate the process:
    libc::SIGALRM, // A real-time timer has expired.
    libc::SIGHUP,  // The terminal is disconnected.
    libc::SIGINT, // Interrupt a program.
    libc::SIGPIPE, // The process attempts to write to a pipe which has no reader.
    libc::SIGQUIT, // Issue a shutdown of the process, after which the OS will dump the process core.
    libc::SIGTERM, // Issue a shutdown of the process.
    libc::SIGUSR1, // A user defined signal.
    libc::SIGUSR2, // A user defined signal.
    // By default, these signals are ignored by the process:
    libc::SIGIO, // When I/O operations are possible on some file descriptor.
    libc::SIGCHLD, // The status of a child process has changed.
    libc::SIGWINCH, // The terminal window is resized.
];

fn show(signal: libc::c_int) -> &'static str {
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pid = std::process::id();
    println!("{pid}");
    let mut printers: JoinSet<_> = JoinSet::new();
    for &signal in ALL_SIGNALS.iter() {
        printers.spawn(print_signal_arrival(signal));
    }
    loop {
        printers.join_next().await;
    }
}

async fn print_signal_arrival(signal: libc::c_int) {
    let signal_name = show(signal);
    let mut stream = unix::signal(unix::SignalKind::from_raw(signal))
        .expect("Failed to register {signal_name}");
    loop {
        stream.recv().await;
        println!("{signal_name}");
    }
}
