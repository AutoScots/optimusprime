// Simple Express server to receive and handle zip file uploads
const express = require('express');
const multer = require('multer');
const path = require('path');
const fs = require('fs');
const app = express();

// Configuration
const PORT = process.env.PORT || 3000;
const API_KEY = process.env.API_KEY || 'your-api-key-goes-here';
const UPLOAD_DIR = path.join(__dirname, 'uploads');
const DEFAULT_FORMAT = process.env.DEFAULT_FORMAT || 'repo'; // 'repo' or 'py'

// Create uploads directory if it doesn't exist
if (!fs.existsSync(UPLOAD_DIR)) {
  fs.mkdirSync(UPLOAD_DIR, { recursive: true });
}

// In-memory storage for user submissions
// In a real app, this would be a database
const userSubmissions = new Map();

// Set up multer for file uploads
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    cb(null, UPLOAD_DIR)
  },
  filename: function (req, file, cb) {
    // Use original filename with timestamp to avoid conflicts
    const timestamp = Date.now();
    cb(null, `${timestamp}-${file.originalname}`)
  }
});

const upload = multer({ 
  storage: storage,
  limits: { fileSize: 50 * 1024 * 1024 } // 50MB limit
});

// Authentication middleware
function authenticate(req, res, next) {
  const authHeader = req.headers.authorization;
  
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ error: 'Missing authorization header' });
  }
  
  const token = authHeader.split(' ')[1];
  
  if (token !== API_KEY) {
    return res.status(403).json({ error: 'Invalid API key' });
  }
  
  // Add the token to the request for later use
  req.apiKey = token;
  next();
}

// Extract user ID from the API key (simplified for demo)
function getUserIdFromToken(token) {
  // In a real implementation, you would decode/validate the token
  // For this example, we'll just use the token as the ID
  return token;
}

// Check endpoint to determine which format to use and get last submission time
app.get('/check', authenticate, (req, res) => {
  const userId = getUserIdFromToken(req.apiKey);
  
  // Get the last submission time for this user
  const lastSubmission = userSubmissions.get(userId);
  
  // Determine which format to use (could be based on user, project, time, etc.)
  // This is a simple example that alternates between 'repo' and 'py'
  const now = Date.now();
  const dayOfYear = Math.floor((now - new Date(now).setHours(0,0,0,0)) / 86400000);
  const formatToUse = (dayOfYear % 2 === 0) ? 'repo' : 'py';
  
  console.log(`Check request from user ${userId}, requesting format: ${formatToUse}`);
  
  return res.status(200).json({
    required_format: formatToUse,
    last_submission_by_user: lastSubmission ? Math.floor(lastSubmission / 1000) : null
  });
});

// Handle zip file uploads
app.post('/submit', authenticate, upload.single('file'), (req, res) => {
  if (!req.file) {
    return res.status(400).json({ error: 'No file uploaded' });
  }
  
  const userId = getUserIdFromToken(req.apiKey);
  const now = Date.now();
  
  // Store the submission time for the user
  userSubmissions.set(userId, now);
  
  console.log(`Received file from user ${userId}: ${req.file.originalname} (${req.file.size} bytes)`);
  
  // Process the file as needed
  // ...
  
  return res.status(200).json({ 
    message: 'File received successfully',
    filename: req.file.filename,
    size: req.file.size,
    timestamp: now
  });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Optimus server running on port ${PORT}`);
  console.log(`Check endpoint: http://localhost:${PORT}/check`);
  console.log(`Submit endpoint: http://localhost:${PORT}/submit`);
});