use std::time::Duration;

use futures::future::select_all;
use futures::FutureExt;
use scheduler::{Signal, ALL_INTERESTING_SIGNALS};
use tokio::signal::unix;

type WonkyResult<T> = Result<T, String>;

#[tokio::main]
async fn main() -> WonkyResult<()> {
    let pid = std::process::id();
    println!("{pid}");

    // prepare signal handler futures
    let handle_signals = ALL_INTERESTING_SIGNALS
        .iter()
        .map(check_signal_occurence)
        .map(FutureExt::boxed);

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

async fn check_signal_occurence(signal: &Signal) -> WonkyResult<()> {
    let mut stream = unix::signal(unix::SignalKind::from_raw(**signal))
        .expect("Failed to register {signal_name}");
    loop {
        if stream.recv().await.is_some() {
            return Err(signal.to_string());
        };
    }
}
