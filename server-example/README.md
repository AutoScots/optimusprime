# Optimus Server Example

This is a Node.js server example that can receive zip files sent by the Optimus CLI tool. It includes competition tracking, attempt limits, and format specifications.

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

Checks with the server to determine the required format for zipping files and get remaining attempts.

Headers:
- `Authorization: Bearer <API_KEY>`

Query Parameters:
- `competition` - (Optional) The competition ID to check

Returns:
```json
{
  "required_format": "repo", 
  "remaining_attempts": 3,
  "last_submission_by_user": 1620000000,
  "competition_name": "Demo Competition"
}
```

- 200 OK with the response above
- 401 Unauthorized if the Authorization header is missing
- 403 Forbidden if the API key is invalid

### POST /submit

Submits a zip file to the server for a competition.

Expects a multipart form with:
- `file` - The zip archive to submit
- `competition` - (Optional) The competition ID for this submission

Headers:
- `Authorization: Bearer <API_KEY>`

Returns:
```json
{
  "message": "File received successfully",
  "filename": "1620000000-user-archive.zip",
  "size": 1024,
  "timestamp": 1620000000000,
  "competition": "competition-123",
  "attempts_remaining": 2
}
```

- 200 OK with details about the received file
- 400 Bad Request if no file was uploaded
- 401 Unauthorized if the Authorization header is missing
- 403 Forbidden if the API key is invalid or no attempts remaining

### GET /competitions

Lists available competitions.

Headers:
- `Authorization: Bearer <API_KEY>`

Returns:
```json
{
  "competitions": [
    {
      "id": "competition-123",
      "name": "Demo Competition",
      "max_attempts": 3
    },
    {
      "id": "competition-456",
      "name": "Advanced Competition",
      "max_attempts": 5
    }
  ]
}
```

## Competition System

The server tracks user submissions per competition:

1. Each competition has a maximum number of submission attempts
2. Users must specify the competition ID when submitting files
3. The server tracks remaining attempts per user per competition
4. Submissions are rejected when a user has no attempts remaining

## Format Types

The server provides different zip packaging requirements:

- `repo`: Full repository zipping (includes all files except exclusions like .git)
- `py`: Python-focused zipping (only includes Python files and Python project files)

Different competitions may require different formats. The format required is specified by the server's response to the /check endpoint.