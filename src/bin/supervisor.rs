use clap::Parser;
use std::io::{self, Write};
use tokio::process::Command;
use tokio::signal::ctrl_c;
use tokio::time;
use tokio_util::sync::CancellationToken;
use scheduler::sys::linux::kill_gracefully;

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    command: String,
    #[arg(short, long, default_value = "60")]
    interval: u64,
    #[clap(last = true)]
    arguments: Vec<String>,
}

#[tokio::main]
async fn main() {
    let pid = std::process::id();
    println!("{pid}");
    let arguments = Arguments::parse();

    let mut command = Command::new(arguments.command);
    command.args(arguments.arguments);

    let interval = time::interval(time::Duration::from_secs(arguments.interval));

    let token = CancellationToken::new();
    let k = tokio::spawn(run(command, interval, token.clone()));
    let _ = ctrl_c().await;
    token.cancel();
    let _ = k.await;
}

async fn run(mut command: Command, mut interval: time::Interval, token: CancellationToken) {
    loop {
        tokio::select! {
            _ = interval.tick() => {}
            _ = token.cancelled() => { return }
        }
        println!("Starting");
        let child = command.spawn().expect("Failed to execute command");
        if let Some(child_id) = child.id() {
            tokio::select! {
                output = child.wait_with_output() => {
                    let outcome = output.expect("Failed to execute command");
                    io::stdout().write_all(&outcome.stdout).unwrap();
                    io::stderr().write_all(&outcome.stderr).unwrap();
                    println!("{:?}", outcome.status.code());
                }
                _ = token.cancelled() => {
                    println!("KILLING");
                    kill_gracefully(child_id as i32);
                    return
                }
            }
        }
    }
}
