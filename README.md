[crates-badge]: https://img.shields.io/crates/v/nifa.svg
[crates-url]: https://crates.io/crates/nifa
[license-badge]: https://img.shields.io/crates/l/nifa.svg

# nifa [![Crates.io][crates-badge]][crates-url] ![License][license-badge]
Cross-platform CLI tool for network information

## Features
- List all network interfaces with detailed information
- Show complete details of a specific interface
- Monitor live traffic statistics in TUI
- Export snapshot in JSON or YAML for automation
- Fetch your public IPv4/IPv6
- Display system information along with default interface

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
You can download archives of precompiled binaries from the [releases](https://github.com/shellrow/nifa/releases) 

### Using Cargo

```sh
cargo install nifa
```

## Usage
```
Usage: nifa [OPTIONS] [COMMAND]

Commands:
  list     Show all interfaces
  show     Show details for specified interface
  monitor  Monitor traffic statistics for all interfaces
  os       Show OS/network stack/permission information
  export   Export snapshot as JSON/YAML
  public   Show public IP information
  help     Print this message or the help of the given subcommand(s)

Options:
  -d, --default          Show only default interface
  -f, --format <FORMAT>  Output format [default: tree] [possible values: tree, json, yaml]
      --with-vendor      With vendor info (OUI lookup)
  -h, --help             Print help
  -V, --version          Print version
```

See `nifa <sub-command> -h` for more detail.

## Note for Developers
If you are looking for a Rust library for network interface,
please check out [netdev](https://github.com/shellrow/netdev).
