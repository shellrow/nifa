[crates-badge]: https://img.shields.io/crates/v/nifa.svg
[crates-url]: https://crates.io/crates/nifa
[license-badge]: https://img.shields.io/crates/l/nifa.svg

# nifa [![Crates.io][crates-badge]][crates-url] ![License][license-badge]
Cross-platform network inspection tool - a modern, read-only alternative to classic network commands.

## Features
- List all network interfaces with detailed information
- Inspect a specific interface with full metadata (addresses, DNS, gateway, speeds, flags, stats)
- View routing tables (IPv4/IPv6)
- View neighbor table (ARP/NDP) with optional vendor (OUI) lookup
- Inspect open TCP/UDP sockets with process association
- Monitor live per-interface traffic statistics in a TUI
- Fetch your public IPv4/IPv6
- Display OS, kernel, proxy, permission capabilities, and default interface info
- Export view as JSON/YAML for automation

## Supported Platforms
- **Linux**
- **macOS**
- **Windows**

## Installation

### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/shellrow/nifa/releases/latest/download/nifa-installer.sh | sh
```

### Install prebuilt binaries via powershell script

```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/shellrow/nifa/releases/latest/download/nifa-installer.ps1 | iex"
```

### From Releases
You can download precompiled binaries from the [releases](https://github.com/shellrow/nifa/releases) 

### Using Cargo

```sh
cargo install nifa
```

## Usage
```
Usage: nifa [OPTIONS] [COMMAND]

Commands:
  ifaces   Show all interfaces
  iface    Show details for specified interface
  monitor  Monitor traffic statistics for interfaces in TUI
  route    Show routing tables (IPv4/IPv6)
  neigh    Show neighbor table (ARP/NDP)
  socket   Show open TCP/UDP sockets and associated processes
  public   Show public IP information
  system   Show OS / kernel / proxy / default interface
  help     Print this message or the help of the given subcommand(s)

Options:
  -l, --log-level <LOG_LEVEL>  Set log level [default: error] [possible values: error, warn, info, debug, trace]
  -h, --help                   Print help
  -V, --version                Print version
```

See `nifa <sub-command> -h` for more detail.

## Note for Developers
If you are looking for a Rust library for network interface,
consider using [netdev](https://github.com/shellrow/netdev).
