use clap::Parser;
use scheduler::sys;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use tokio::process::Command;
use tokio::signal::ctrl_c;
use tokio::time;
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
    let arguments = Arguments::parse();

    let mut command = Command::new(arguments.command);
    command.args(arguments.arguments);

    let interval = time::interval(time::Duration::from_secs(arguments.interval));

    let token = CancellationToken::new();
    let runner = tokio::spawn(run(
        command,
        interval,
        token.clone(),
        arguments.working_directory.clone(),
    ));
    let _ = ctrl_c().await;
    token.cancel();
    let _ = runner.await;
}

async fn run(
    mut command: Command,
    mut interval: time::Interval,
    token: CancellationToken,
    working_directory: std::path::PathBuf,
) {
    let mut i = 0;
    loop {
        tokio::select! {
            _instant = interval.tick() => { }
            _ = token.cancelled() => { return }
        }
        println!("Starting");

        let run_directory = working_directory.join(format!("{i}"));
        command = setup(command, &run_directory);
        let mut status = create_file(&run_directory.join("status")).unwrap();
        let child = command.spawn().expect("Failed to execute command");
        if let Some(child_id) = child.id() {
            tokio::select! {
                output = child.wait_with_output() => {
                    let outcome = output.expect("Failed to execute command");
                    status.write_all(format!("{:?}", outcome.status).as_bytes()).unwrap();
                }
                _ = token.cancelled() => {
                    println!("KILLING");
                    #[cfg(target_os="linux")]
                    sys::linux::kill_gracefully(child_id as i32);

                    #[cfg(target_os="windows")]
                    sys::windows::kill_gracefully();
                    return
                }
            }
        }
        i += 1;
    }
}

#[derive(Debug)]
struct WithPath{
    error: io::Error,
    path: PathBuf,
}

fn create_file(path: &Path) -> Result<File, WithPath> {
    File::create(path).map_err(|error| WithPath{error, path: path.into()})
}

fn create_dir(path: &Path) -> Result<(), WithPath> {
    fs::create_dir(path).map_err(|error| WithPath{error, path: path.into()})
}

fn setup(mut command: Command, run_directory: &Path) -> Command {
    create_dir(&run_directory).unwrap();
    let stdout = create_file(&run_directory.join("stdout")).unwrap();
    let stderr = create_file(&run_directory.join("stderr")).unwrap();
    command.stdout(stdout);
    command.stderr(stderr);
    command
}
