# Optimus Prime

A CLI tool for zipping directories and sending them to a server.

## Features

- Recursively zip the current directory and all subdirectories
- Send the zip file to a server via HTTPS
- Authentication with API key
- Easy to install and use

## Installation

### Using curl (recommended)

This method will automatically install Rust and Cargo if they're not already installed:

```bash
# For main branch
curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/main/direct-install.sh | bash

# If the above command doesn't work, try:
curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/master/direct-install.sh | bash
```

#### Alternative Install

If you encounter any issues with the direct installation, try this command instead:

```bash 
cargo install --git https://github.com/AutoScots/optimusprime.git --branch main --path repo-zipper
```

### Using cargo

If you already have Rust and Cargo installed, you can install Optimus directly:

```bash
cargo install --git https://github.com/AutoScots/optimusprime.git --path repo-zipper
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
- `--server <URL>`: Base URL for the server (default: http://localhost:3000)
- `--compression <LEVEL>`: Compression level (0-9, default: 6)
- `--force-format <FORMAT>`: Skip server check and force a specific format (repo or py)

### Formats

The tool supports different packaging formats:

- `repo`: Full repository zipping (includes all files except exclusions like .git)
- `py`: Python-focused zipping (only includes Python files and Python project files)

By default, the tool contacts the server's `/check` endpoint to determine which format to use.

## Example

```bash
# Set your API key
export OPTIMUS_API_KEY="your-api-key"

# Zip and send the current directory to a custom server with maximum compression
optimus send --server https://api.example.com --compression 9

# Force Python format without checking with the server
optimus send --force-format py
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
