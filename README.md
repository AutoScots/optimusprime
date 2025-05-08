# Optimus Prime

A CLI tool for zipping directories and sending them to a server.

## Features

- Recursively zip the current directory and all subdirectories
- Send the zip file to a server via HTTPS
- Authentication with API key
- Easy to install and use

## Installation

### Using curl (recommended)

This method will automatically install Rust, Cargo, and Git if they're not already installed:

```bash
curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/main/install.sh | bash
```

### Using cargo

If you already have Rust and Cargo installed, you can install Optimus directly:

```bash
cargo install --git https://github.com/AutoScots/optimusprime.git
```

## Usage

Set your API key as an environment variable:

```bash
export OPTIMUS_API_KEY="your-api-key"
```

Then use the `send` command to zip and send the current directory:

```bash
optimus send
```

### Options

```
optimus send --help
```

- `--api-key <KEY>`: API key for authentication (can also be set via OPTIMUS_API_KEY env var)
- `--endpoint <URL>`: Endpoint to send the zip file to (default: http://localhost:3000/submit)
- `--compression <LEVEL>`: Compression level (0-9, default: 6)

## Example

```bash
# Set your API key
export OPTIMUS_API_KEY="your-api-key"

# Zip and send the current directory to a custom endpoint with maximum compression
optimus send --endpoint https://api.example.com/upload --compression 9
```

## Server Example

This repository includes a simple Node.js server that can receive the zip files sent by the Optimus CLI:

```
server-example/
├── server.js          # Example Express server implementation
├── package.json       # Node.js dependencies
└── README.md          # Server setup instructions
```

See the [server example README](server-example/README.md) for setup instructions.

## Systemd Service

A systemd service file is provided for running the server as a system service:

```
systemd/
└── optimus-server.service  # Systemd service file for the server
```
