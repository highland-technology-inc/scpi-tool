use clap::{Parser};
use std::path::PathBuf;
use std::io::{Read, Write};
use std::println;
use std::time::Duration;
use serialport;
use log::{debug, info, warn, error};
use env_logger;

const LONG_ABOUT: &str = "Command line tool for executing SCPI queries and gathering responses.

Uses Rust env_logger, so debug verbosity can be set with the RUST_LOG environment variable.
";


#[derive(Parser)]
#[command(version, about, long_about=LONG_ABOUT)]
struct Cli {
    /// Serial port
    port: PathBuf,

    /// Baud rate
    #[arg(short, long, default_value_t=38400)]
    baud: u32,

    /// Error fetch mode
    #[arg(short, long)]
    errors: bool,

    #[clap(trailing_var_arg=true)]
    command: Vec<String>,
}

/// Transact a command against the port.
///
/// command should not end in a newline
fn transaction<T: Read+Write>(port: &mut T, command: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    debug!("X> {command}");
    port.write(command.as_bytes())?;
    port.write(b"\n")?;

    if command.contains("?") {
        let mut buffer = [0; 1024];
        let mut pos = 0;

        loop {
            match port.read(&mut buffer[pos..]) {
                Ok(x) => {
                    if buffer[pos..pos+x].contains(&b'\n') {
                        let s = std::str::from_utf8(&buffer[..pos+x])?;
                        return Ok(Some(String::from(s.trim())));
                    } else {
                        pos += x;
                    }
                }
                Err(e) => {
                    if (pos > 0) && let Ok(s) = std::str::from_utf8(&buffer[..pos]) {
                        warn!("Recv without newline: {s}");
                    }
                    return Err(std::boxed::Box::new(e));
                }
            }
        }
    }
    Ok(None)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error"))
        .format_timestamp(None)
        .init();

    let args = Cli::parse();

    let path = args.port.to_string_lossy();
    let mut port = serialport::new(path, args.baud)
                        .timeout(Duration::from_millis(1000))
                        .open()
                        .expect("Error opening serial port");

    info!("{port:?}");

    // Send the command out.
    let command = String::from(args.command.join(" ").trim());
    if !(command.is_empty()) {
        match transaction(&mut port, &command) {
            Ok(None) => {debug!("No query");},
            Ok(Some(x)) => {println!("{x}");}
            Err(e) => {
                error!("{e}");
            }
        }
    }

    if args.errors {
        loop {
            match transaction(&mut port, "SYST:ERR?") {
                Ok(None) => {panic!("transaction() didn't see a query")},
                Ok(Some(x)) => {
                    println!("{x}");
                    if x.contains("No error") { break; }
                }
                Err(e) => {
                    error!("{e}");
                    break;
                }
            }
        }
    }

    Ok(())
}
