# Optimus Server Example

This is a simple Node.js server example that can receive zip files sent by the Optimus CLI tool. It includes an endpoint to check required format for zipping files.

## Installation

1. Install Node.js and npm if you don't have them already.

2. Install the dependencies:
```bash
npm install
```

3. Set your API key (must match the one used by clients):
```bash
export API_KEY="your-secret-api-key"
```

4. Optionally set the default format (repo or py):
```bash
export DEFAULT_FORMAT="repo"
```

5. Start the server:
```bash
npm start
```

## Systemd Service Setup

For production use, you can set up the server as a systemd service:

1. Copy the service file to systemd directory:
```bash
sudo cp ../systemd/optimus-server.service /etc/systemd/system/
```

2. Edit the service file to set your API key:
```bash
sudo nano /etc/systemd/system/optimus-server.service
```

3. Enable and start the service:
```bash
sudo systemctl enable optimus-server
sudo systemctl start optimus-server
```

4. Check the service status:
```bash
sudo systemctl status optimus-server
```

## API

### GET /check

Checks with the server to determine the required format for zipping files.

Headers:
- `Authorization: Bearer <API_KEY>`

Returns:
```json
{
  "required_format": "repo", // or "py"
  "last_submission_by_user": 1620000000 // timestamp in seconds, null if no previous submission
}
```

- 200 OK with the response above
- 401 Unauthorized if the Authorization header is missing
- 403 Forbidden if the API key is invalid

### POST /submit

Submits a zip file to the server.

Expects a multipart form with a file field containing the zip archive.

Headers:
- `Authorization: Bearer <API_KEY>`

Returns:
```json
{
  "message": "File received successfully",
  "filename": "1620000000-archive.zip",
  "size": 1024,
  "timestamp": 1620000000000
}
```

- 200 OK with details about the received file
- 400 Bad Request if no file was uploaded
- 401 Unauthorized if the Authorization header is missing
- 403 Forbidden if the API key is invalid

## Format Types

The server provides different zip packaging requirements:

- `repo`: Full repository zipping (includes all files except exclusions like .git)
- `py`: Python-focused zipping (only includes Python files and Python project files)