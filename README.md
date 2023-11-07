# pulso
A simple metrics collector for TCP/IP. Counts new connection events by source IP and port.

## Design Goals
* small resource footprint
* minimal interface
* clean and tested

## Features
* IP addresses are hashed (disable with `privacy` feature flag)
* Supports IPV6

## Dependencies
* [libpcap](https://www.tcpdump.org/)

## Why
I wanted to get my hands dirty with Rust. The same functionality can probably be implemented
with `tcpdump` and `awk`/`ag`/`lnav`. Not a mighty tool, just a foundation for one.

## Installation
Clone the repository and `cargo install` (assuming `libpcap` is installed already).

For Linux, there are [MUSL](https://wiki.musl-libc.org/supported-platforms.html)
binaries in the releases directory.

You'll need to give permission to read sockets
`sudo setcap cap_net_raw=eip /path/to/pulso`

## Usage

```
TCP connection counter

Usage: pulso [OPTIONS] --device <DEVICE>

Options:
  -d, --device <DEVICE>                      device name
  -c, --connection-limit <CONNECTION_LIMIT>  max connections
  -t, --time-limit <TIME_LIMIT>              max seconds
  -h, --help                                 Print help
  -V, --version                              Print version

Environment Variables:
  PULSO_SECRET    (required) encryption key used for sensitive information
```

Logs are produced to the standard error stream by setting the RUST_LOG environment variable.

### Examples

Produce a digest after 10 connections
```
tar xvfz releases/x86_64-unknown-linux-musl/pulso_0.1.0.tar.gz
sudo setcap cap_net_raw=eip ./pulso
PULSO_SECRET=foo ./pulso -d lo -c 10
# 2da25a664b49c9b5:10 9306:9 9056:1
```

Show all logs and produce a digest after 1 minute
```
RUST_LOG=info PULSO_SECRET=test pulso -d eth0 -t 60
```

For a more complete picture of the intended functionality, refer to the integration tests.

## Development

This project is almost feature complete. I haven't deployed it yet. I doubt I will devote
more energy to this but there might be a new release after I've had a chance to test it
in the wild.

`cargo make test`
