use std::time::Duration;

use futures::future::select_all;
use futures::FutureExt;
use tokio::signal::unix;

type WonkyResult<T> = Result<T, &'static str>;

// The signals documented by SignalKind, expect for SIGINFO (which does not exist on Linux).
const ALL_SIGNALS: &[libc::c_int] = &[
    // By default, these signals terminate the process:
    libc::SIGALRM, // A real-time timer has expired.
    libc::SIGHUP,  // The terminal is disconnected.
    libc::SIGINT,  // Interrupt a program.
    libc::SIGPIPE, // The process attempts to write to a pipe which has no reader.
    libc::SIGQUIT, // Issue a shutdown of the process, after which the OS will dump the process core.
    libc::SIGTERM, // Issue a shutdown of the process.
    libc::SIGUSR1, // A user defined signal.
    libc::SIGUSR2, // A user defined signal.
    // By default, these signals are ignored by the process:
    libc::SIGIO,    // When I/O operations are possible on some file descriptor.
    libc::SIGCHLD,  // The status of a child process has changed.
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
async fn main() -> WonkyResult<()> {
    let pid = std::process::id();
    println!("{pid}");

    // prepare signal handler futures
    let handle_signals = ALL_SIGNALS
        .iter()
        .map(|&signal| print_signal_arrival(signal).boxed());

    // prepare real work future
    let do_work = std::iter::once(my_real_task().boxed());

    // chain all futures in a big iterator
    let app_tasks = handle_signals.chain(do_work);

    // select returns the first future that returns, so
    //
    // - in case of a signal, it returns an error
    // - in case of no signal, work will go on (and eventually finish in real world scenarios)
    select_all(app_tasks).map(|(res, _, _)| res).await
}

async fn my_real_task() -> WonkyResult<()> {
    loop {
        println!("Work work work 🤖");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn print_signal_arrival(signal: libc::c_int) -> WonkyResult<()> {
    let signal_name = show(signal);
    let mut stream =
        unix::signal(unix::SignalKind::from_raw(signal)).expect("Failed to register {signal_name}");
    loop {
        if stream.recv().await.is_some() {
            return Err(signal_name);
        };
    }
}
