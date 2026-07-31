use clap::{Parser};
use std::path::PathBuf;
use std::io::{Read, Write};
use std::println;
use std::time::Duration;
use serialport;
use log::{debug, info, warn, error};
use env_logger;
use regex::Regex;

mod cmd_iterators;

const LONG_ABOUT: &str = "Command line tool for executing SCPI queries and gathering responses.

Uses Rust env_logger, so debug verbosity can be set with the RUST_LOG environment variable.
";

const EXAMPLE: &str = "Example:
    $ ./scpi-tool /dev/ttyUSB0 115200 \"*IDN?\"
    Highland Technology,P200-A1,eng-02,23E201-1.0
";

#[derive(Parser)]
#[command(version, about, after_long_help=EXAMPLE, long_about=LONG_ABOUT)]
struct Cli {
    /// Serial port
    port: PathBuf,

    /// Baud rate.  Defaults to no-change if possible, or 38400 as a last resort.
    #[arg(short, long)]
    baud: Option<u32>,

    /// Error fetch mode.  After sending command (if present), query SYST:ERR? until the queue is empty.
    #[arg(short, long)]
    errors: bool,

    /// The command or query to send to the device.  Should probably be quoted.
    /// If no command is provided, uses stdin.  This will be an interactive session
    /// if stdin is a tty.
    #[clap(trailing_var_arg=true)]
    command: Vec<String>,
}

/// Transact a command, returning the result if it's a query.
///
/// port is the open connection to the SCPI device.
/// command should not end in a newline.
///
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

/// Make a call to the 'stty' program to determine the existing baud rate of a serial port
fn get_baud_from_stty(port: &str) -> Result<u32, Box<dyn std::error::Error>> {
    debug!("Trying to read baud rate from {port}");
    let result = std::process::Command::new("stty")
        .arg("-F")
        .arg(port)
        .output()?;

    let text = str::from_utf8(&result.stdout)?;
    debug!("{text}");
    let re = Regex::new(r"speed\s+(\d+)\s+baud").unwrap();
    let mo = re.captures(text).ok_or("no baud rate found")?;
    let baud: u32 = mo[1].parse()?;
    if baud == 0 {
        Err("zero baud rate".into())
    } else {
        Ok(baud)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error"))
        .format_timestamp(None)
        .init();

    let args = Cli::parse();

    let path = args.port.to_string_lossy();
    let baud = match args.baud {
        Some(x) => { x },
        None => {
            match get_baud_from_stty(&path) {
                Ok(b) =>    { info!("Keeping baudrate {b}"); b },
                Err(_) =>   { 38400 }
            }
        }
    };

    let mut port = serialport::new(path, baud)
                        .timeout(Duration::from_millis(1000))
                        .open()?;

    info!("{port:?}");

    // If we have a command then we only iterate over that one command.
    // Otherwise, build a readline based iterator to keep going until
    // Ctrl-D.
    //
    let command = String::from(args.command.join(" ").trim());
    let cmd_iter : Box<dyn Iterator<Item = String>> = if command.is_empty() {
        Box::new(cmd_iterators::ReadlineCommands::new())
    } else {
        Box::new(Option::from(command).into_iter())
    };

    // And iterate over all the commands we have until we're
    for c in cmd_iter {
        // Send the command out if one was provided.
        if !(c.is_empty()) {
            match transaction(&mut port, &c) {
                Ok(None) =>     {debug!("No query");},
                Ok(Some(x)) =>  {println!("{x}");}
                Err(e) =>       {error!("{e}"); return Err(e);}
            }
        }

        // If the --errors flag was provided, flush out the 
        // system error queue.
        if args.errors {
            loop {
                match transaction(&mut port, "SYST:ERR?") {
                    Ok(None) =>     {panic!("transaction() didn't see a query")},
                    Ok(Some(x)) =>  {
                        println!("{x}");
                        if x.contains("No error") { break; }
                    }
                    Err(e) =>       {error!("{e}"); return Err(e);}
                }
            }
        }
    }
    Ok(())
}
