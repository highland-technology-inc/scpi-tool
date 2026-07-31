use clap::{Parser};
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use timeout_readwrite::TimeoutReader;
use std::time::Duration;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Serial port
    port: PathBuf,

    #[clap(trailing_var_arg=true)]
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut port = File::options().write(true).create(true).open(args.port)?;

    // Send the command out.
    let command = args.command.join(" ");
    port.write(command.as_bytes())?;
    port.write(b"\n")?;

    if command.find("?").is_some() {
        // That was a query.
        let mut response = String::new();
        let mut reader = TimeoutReader::new(port, Duration::from_millis(1000));
        reader.read_to_string(&mut response)?;
        print!("{response}");
    }
    Ok(())
}
