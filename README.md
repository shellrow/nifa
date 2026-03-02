[crates-badge]: https://img.shields.io/crates/v/nifa.svg
[crates-url]: https://crates.io/crates/nifa
[license-badge]: https://img.shields.io/crates/l/nifa.svg

# nifa [![Crates.io][crates-badge]][crates-url] ![License][license-badge]

Cross-platform network inspection tool.

It is designed for Linux, macOS, and Windows with a consistent command surface:

```bash
nifa <sub-command> [options]
```

With no sub-command (`nifa`), it shows the primary network interface summary.

## Principles

- Read-only only: no mutation commands
- Tree-first output with terminal-width-aware auto format
- Structured output for automation (`json` / `yaml`)

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

## Commands

```text
if      Network interfaces
addr    IP addresses
link    Layer 2 information
route   Routing table
neigh   ARP / NDP entries
sock    Sockets (ss/netstat equivalent)
sys     Network/system summary
mon     TUI monitor
```

## Output Options

All non-TUI commands support:

```text
--format auto|tree|table|json|yaml
--wide
--no-color
--no-truncate
```

### Format behavior

- Default: `auto`
- `auto` selects:
  - narrow terminal: `tree`
  - wide terminal: `table`

## Examples

```bash
# Interfaces (summary)
nifa if

# Interface details
nifa if show en0

# Addresses
nifa addr
nifa addr --iface en0 --ipv6

# Link layer
nifa link --up

# Routes
nifa route --default
nifa route --ipv4 --detail

# Neighbors
nifa neigh --ipv4 --vendor

# Sockets
nifa sock --proto tcp --listen
nifa sock --proto tcp --established --pid

# System summary
nifa sys
nifa sys --dns
nifa sys --proxy

# TUI monitor
nifa mon --iface en0 --interval 1 --sort total
```
