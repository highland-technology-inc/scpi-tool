# scpi-tool
A command-line SCPI tool, written in Rust.

`scpi-tool` provides a command-line mechanism to send SCPI commands and queries
over a serial port to a SCPI device.  This can be done interactively or
noninteractively.

## Interactive Use

```
$ scpi-tool /dev/ttyUSB0 --baud 115200 --errors=quiet
> *IDN?
Highland Technology,P200-1A,ENG02,23E200-1-0.0.1
> *IDM?
[ERROR scpi_tool] Operation timed out
-113,"Undefined header"
> *CLS
> OUTP:STATE?
0
> OUTP:STATE 1
> OUTP:STATE?
1
> 
```

`scpi-tool` uses readyline (a readline equivalent) to provide line-editing and
history (up/down arrow) for an interactive terminal whenever a command is not
provided on the command line and stdin is a tty.  Queries (with a question mark)
expect responses, commands do not.  Press Ctrl-D to end session.

## Non-Interactive Use

```
$ scpi-tool /dev/ttyUSB0 --baud 115200 "*IDN?"
Highland Technology,P200-1A,ENG02,23E200-1-0.0.1
$ echo -e "*IDN?\n*OPC\nSYST:STATE?\n" | scpi-tool /dev/ttyUSB0 --baud 115200 
Highland Technology,P200-1A,ENG02,23E200-1-0.0.1
DISCHARGING
$
```

In non-interactive mode, scpi-tool sends commands either from the command line or
from a non-tty stdin.  In the former case only the command is sent; in the latter
commands are sent until reaching EOF.

## Command Line Syntax

*Usage:* scpi-tool [OPTIONS] <PORT> [COMMAND]...

*Arguments:*
  <PORT>
          Serial port

  [COMMAND]...
          The command or query to send to the device.  Should probably be quoted. If no command is provided, uses stdin.  This will be an interactive session if stdin is a tty

*Options:*
  -b, --baud <BAUD>
          Baud rate.  Defaults to no-change if possible, or 38400 as a last resort

  -e, --errors <ERRORS>
          Error fetch mode.  After sending command (if present), query SYST:ERR? until the queue is empty

          Possible values:
          - none:  No error checking
          - quiet: Report all errors, ignore "No error"
          - all:   Report everything from the error queue
          
          [default: none]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

## Author

Rob Gaddi, Highland Technology, Inc. 2026