use clap::Parser;
use std::io::{self, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    command: String,
    #[arg(short, long, default_value = "60")]
    interval: u64,
    #[clap(last = true)]
    arguments: Vec<String>,
}

fn main() {
    let arguments = Arguments::parse();

    let mut command = Command::new(arguments.command);
    command.args(arguments.arguments);

    loop {
        let output = command.output().expect("Failed to execute command");
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        println!("{:?}", output.status.code());
        sleep(Duration::from_secs(arguments.interval));
    }
}
