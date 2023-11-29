use clap::Parser;
use scheduler::sys;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use tokio::process::{Child, Command};
use tokio::signal::ctrl_c;
use tokio::time::{self, Duration, Instant};
use tokio_util::sync::CancellationToken;

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    command: String,
    #[arg(short, long)]
    working_directory: PathBuf,
    #[arg(short, long, default_value = "60")]
    interval: u64,
    #[arg(short, long, default_value = "30")]
    timeout: u64,
    #[clap(last = true)]
    arguments: Vec<String>,
}

#[tokio::main]
async fn main() {
    let Arguments {
        command,
        working_directory,
        interval,
        timeout,
        arguments,
    } = Arguments::parse();

    let mut command = Command::new(command);
    command.args(arguments);

    let token = CancellationToken::new();
    tokio::spawn(listen_for_shutdown(token.clone()));

    schedule(command, interval, timeout, &token, working_directory).await;
}

async fn schedule(
    mut command: Command,
    interval: u64,
    timeout: u64,
    token: &CancellationToken,
    working_directory: PathBuf,
) {
    let start = Instant::now();
    let mut clock = time::interval_at(start, Duration::from_secs(interval));
    loop {
        let instant = tokio::select! {
            instant = clock.tick() => { instant }
            _ = token.cancelled() => {
                    log("Stopping");
                    return
            }
        };
        let run_directory = name_run_directory(start, instant, &working_directory);
        command = setup(command, &run_directory);
        let outcome = wait_with_output(&mut command, timeout, token).await;
        persist(&outcome, &run_directory);
    }
}

#[derive(Debug)]
struct WithPath {
    #[allow(dead_code)]
    error: io::Error,
    #[allow(dead_code)]
    path: PathBuf,
}

fn log(message: &str) {
    println!("{}", message);
}

fn name_run_directory(start: Instant, instant: Instant, working_directory: &Path) -> PathBuf {
    let name = instant.duration_since(start).as_secs();
    working_directory.join(format!("{name}"))
}

fn persist(outcome: &Outcome, run_directory: &Path) {
    let mut status = create_file(&run_directory.join("status")).unwrap();
    status
        .write_all(format!("{:?}", outcome).as_bytes())
        .unwrap();
}

fn create_file(path: &Path) -> Result<File, WithPath> {
    File::create(path).map_err(|error| WithPath {
        error,
        path: path.into(),
    })
}

fn create_dir(path: &Path) -> Result<(), WithPath> {
    fs::create_dir(path).map_err(|error| WithPath {
        error,
        path: path.into(),
    })
}

fn setup(mut command: Command, run_directory: &Path) -> Command {
    create_dir(&run_directory).unwrap();
    let stdout = create_file(&run_directory.join("stdout")).unwrap();
    let stderr = create_file(&run_directory.join("stderr")).unwrap();
    command.stdout(stdout);
    command.stderr(stderr);
    command
}

async fn listen_for_shutdown(token: CancellationToken) {
    ctrl_c().await.unwrap();
    token.cancel();
}

type Status = io::Result<ExitStatus>;

#[derive(Debug)]
enum Outcome {
    Cancelled(Status),
    TimedOut(Status),
    Complete(Status),
}

async fn wait_with_output(
    command: &mut Command,
    timeout: u64,
    token: &CancellationToken,
) -> Outcome {
    log("Starting");
    let mut child = command.spawn().expect("Failed to execute command");
    tokio::select! {
        output = child.wait() => {
            Outcome::Complete(output)
        }
        _ = time::sleep(Duration::from_secs(timeout)) => {
            Outcome::TimedOut(kill(&mut child).await)
        }
        _ = token.cancelled() => {
            Outcome::Cancelled(kill(&mut child).await)
        }
    }
}

async fn kill(child: &mut Child) -> Status {
    log("KILLING");

    #[cfg(target_os = "linux")]
    sys::linux::interrupt(child);

    #[cfg(target_os = "windows")]
    sys::windows::interrupt();

    child.wait().await
}
