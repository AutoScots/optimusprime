# Optimus Server Example

This is a simple Node.js server example that can receive zip files sent by the Optimus CLI tool.

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

4. Start the server:
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

### POST /submit

Expects a multipart form with a file field containing the zip archive.

Headers:
- `Authorization: Bearer <API_KEY>`

Returns:
- 200 OK with details about the received file
- 400 Bad Request if no file was uploaded
- 401 Unauthorized if the Authorization header is missing
- 403 Forbidden if the API key is invalid