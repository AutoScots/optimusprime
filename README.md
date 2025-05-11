# Optimus Prime

A CLI tool for zipping directories and sending them to a server, particularly designed for competitions and submissions.

## Features

- Config-driven workflow with YAML configuration file
- Recursively zip the current directory and all subdirectories
- Send the zip file to a server via HTTPS
- Authentication with API key
- Competition submission tracking
- Interactive confirmation based on remaining attempts
- Support for different packaging formats
- Easy to install and use

## Installation

### Recommended Installation Method

The standard installation script automatically fixes compatibility issues and installs Optimus Prime:

```bash
# Download and run the install script
curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/main/install.sh | bash
```

### Alternative Installation Methods

#### Clone and Install Locally

If you prefer to clone the repository first:

```bash 
git clone --depth 1 https://github.com/AutoScots/optimusprime.git
cd optimusprime
./install.sh
```

#### Legacy Installation Methods

These methods may encounter compatibility issues and are not recommended:

```bash
# Legacy GitHub installation (may require fixes)
curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/main/install-github-legacy.sh | bash

# Legacy local installation (may require fixes)
curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/main/install-legacy.sh | bash
```

## Configuration

Optimus Prime uses a YAML configuration file (`submission.yml`) to manage submissions. This is the recommended way to provide API keys, competition IDs, and other settings.

### Creating a Configuration File

You can create a default configuration file:

```bash
optimus init
```

This creates a `submission.yml` file in the current directory with default values. You can specify an API key and competition ID:

```bash
optimus init --api-key "your-api-key" --competition-id "comp-123"
```

### Example Configuration File

```yaml
# API key for authentication (required)
api_key: "your-api-key-here"

# Competition ID (optional, can be overridden via command line)
competition_id: "competition-123"

# Submission format: 'repo' or 'py' (optional, will check with server if not specified)
format: "repo"

# Server URL (optional, defaults to http://localhost:3000)
server_url: "http://localhost:3000"

# Compression level (0-9, optional, default is 6)
compression_level: 6

# Excluded directories/files (optional, adds to default exclusions)
exclude:
  - ".git"
  - ".DS_Store"
  - "node_modules"
  - "target"
  - ".env"
  - "venv"

# User preferences (optional)
preferences:
  # Whether to auto-confirm submissions (optional, default is false)
  auto_confirm: false
  
  # Whether to save submission history (optional, default is true)
  save_history: true
```

## Usage

### Basic Usage

To submit the current directory using settings from `submission.yml`:

```bash
optimus send
```

The tool will:
1. Contact the server to determine the required format and remaining attempts
2. Ask for confirmation to proceed (unless auto-confirm is enabled)
3. Create a zip file according to the required format
4. Send the zip file to the server
5. Display the response from the server

### Command Line Options

All configuration options can be overridden via command line:

```bash
optimus send --help
```

Available options:

- `--config <PATH>`: Path to the configuration file (default: `submission.yml`)
- `--api-key <KEY>`: API key for authentication (overrides config file)
- `--competition-id <ID>`: Competition ID (overrides config file)
- `--server <URL>`: Base URL for the server (overrides config file)
- `--compression <LEVEL>`: Compression level (0-9, overrides config file)
- `--force-format <FORMAT>`: Skip server check and force a specific format (repo or py) 
- `--auto-confirm`: Auto-confirm submission without prompting

### Submission Formats

The tool supports different packaging formats:

- `repo`: Full repository zipping (includes all files except exclusions like .git)
- `py`: Python-focused zipping (only includes Python files and Python project files)

By default, the tool contacts the server's `/check` endpoint to determine which format to use.

## Examples

### Initialize Configuration

```bash
# Create default configuration file
optimus init

# Create configuration with API key and competition ID
optimus init --api-key "your-api-key" --competition-id "comp-123"
```

### Submit Directory

```bash
# Submit using configuration file
optimus send

# Override server URL
optimus send --server "https://api.example.com"

# Force Python format and auto-confirm
optimus send --force-format py --auto-confirm

# Submit to a specific competition
optimus send --competition-id "special-competition-456"
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