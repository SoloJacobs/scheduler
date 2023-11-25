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
    #[clap(last = true)]
    arguments: Vec<String>,
}

#[tokio::main]
async fn main() {
    let Arguments {
        command,
        working_directory,
        interval,
        arguments,
    } = Arguments::parse();

    let mut command = Command::new(command);
    command.args(arguments);

    let token = CancellationToken::new();
    tokio::spawn(listen_for_shutdown(token.clone()));

    run(command, interval, token, working_directory).await;
}

async fn run(
    mut command: Command,
    interval: u64,
    token: CancellationToken,
    working_directory: PathBuf,
) {
    let start = Instant::now();
    let mut clock = time::interval_at(start, Duration::from_secs(interval));
    loop {
        let instant = tokio::select! {
            instant = clock.tick() => { instant }
            _ = token.cancelled() => {
                    println!("Stopping");
                    return
            }
        };
        let name = instant.duration_since(start).as_secs();
        let run_directory = working_directory.join(format!("{name}"));
        command = setup(command, &run_directory);
        let mut status = create_file(&run_directory.join("status")).unwrap();
        println!("Starting");
        let mut child = command.spawn().expect("Failed to execute command");
        let outcome = wait_with_output(&mut child, &token).await;
        status
            .write_all(format!("{:?}", outcome).as_bytes())
            .unwrap();
    }
}

#[derive(Debug)]
struct WithPath {
    #[allow(dead_code)]
    error: io::Error,
    #[allow(dead_code)]
    path: PathBuf,
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

#[derive(Debug)]
enum Outcome {
    Cancelled,
    Complete(ExitStatus),
}

async fn wait_with_output(child: &mut Child, token: &CancellationToken) -> Outcome {
    tokio::select! {
        output = child.wait() => {
            Outcome::Complete(output.unwrap())
        }
        _ = token.cancelled() => {
            if let Some(id) = child.id() {
                println!("KILLING");
                #[cfg(target_os="linux")]
                sys::linux::kill_gracefully(id as i32);

                #[cfg(target_os="windows")]
                sys::windows::kill_gracefully();
            }
            Outcome::Cancelled
        }
    }
}
