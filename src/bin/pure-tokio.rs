use scheduler::{Signal, ALL_INTERESTING_SIGNALS};
use tokio::signal::unix;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pid = std::process::id();
    println!("{pid}");
    let mut printers: JoinSet<_> = JoinSet::new();
    for &signal in ALL_INTERESTING_SIGNALS.iter() {
        printers.spawn(print_signal_arrival(signal));
    }
    loop {
        printers.join_next().await;
    }
}

async fn print_signal_arrival(signal: Signal) {
    let mut stream = unix::signal(unix::SignalKind::from_raw(*signal))
        .expect("Failed to register {signal_name}");
    loop {
        stream.recv().await;
        println!("{signal}");
    }
}
