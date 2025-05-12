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

// In-memory storage for user submissions and competitions
// In a real app, this would be a database
const userSubmissions = new Map();
const competitions = new Map([
  ['competition-123', { maxAttempts: 3, name: 'Demo Competition' }],
  ['competition-456', { maxAttempts: 5, name: 'Advanced Competition' }],
]);

// Track attempts per user per competition
const submissionAttempts = new Map(); // Map<userId_competitionId, count>

// Set up multer for file uploads
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    // Create a directory for the competition if specified
    let targetDir = UPLOAD_DIR;
    if (req.body.competition) {
      targetDir = path.join(UPLOAD_DIR, req.body.competition);
      if (!fs.existsSync(targetDir)) {
        fs.mkdirSync(targetDir, { recursive: true });
      }
    }
    cb(null, targetDir);
  },
  filename: function (req, file, cb) {
    // Use original filename with timestamp and user ID to avoid conflicts
    const timestamp = Date.now();
    const userId = getUserIdFromToken(req.apiKey);
    cb(null, `${timestamp}-${userId}-${file.originalname}`);
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

// Get competition ID from query parameter or use default
function getCompetitionId(req) {
  return req.query.competition || 'default';
}

// Get attempts remaining for a user in a competition
function getRemainingAttempts(userId, competitionId) {
  // Get competition info
  const competition = competitions.get(competitionId);
  const maxAttempts = competition ? competition.maxAttempts : 3; // Default to 3 if no competition found
  
  // Get attempts used
  const key = `${userId}_${competitionId}`;
  const attemptsUsed = submissionAttempts.get(key) || 0;
  
  return Math.max(0, maxAttempts - attemptsUsed);
}

// Check endpoint to determine which format to use and get remaining attempts
app.get('/check', authenticate, (req, res) => {
  const userId = getUserIdFromToken(req.apiKey);
  const competitionId = getCompetitionId(req);
  
  // Get the last submission time for this user
  const lastSubmissionKey = `${userId}_${competitionId}`;
  const lastSubmission = userSubmissions.get(lastSubmissionKey);
  
  // Get remaining attempts
  const remainingAttempts = getRemainingAttempts(userId, competitionId);
  
  // Determine which format to use 
  // This could be based on user, competition, time, etc.
  let formatToUse = DEFAULT_FORMAT;
  
  // For demo purposes, use different formats for different competitions
  if (competitionId === 'competition-456') {
    formatToUse = 'py';
  }
  
  console.log(`Check request from user ${userId} for competition ${competitionId}`);
  console.log(`Format: ${formatToUse}, Remaining attempts: ${remainingAttempts}`);
  
  return res.status(200).json({
    required_format: formatToUse,
    remaining_attempts: remainingAttempts,
    last_submission_by_user: lastSubmission ? Math.floor(lastSubmission / 1000) : null,
    competition_name: competitions.get(competitionId)?.name || 'Unknown Competition'
  });
});

// Handle zip file uploads
app.post('/submit', authenticate, upload.single('file'), (req, res) => {
  if (!req.file) {
    return res.status(400).json({ error: 'No file uploaded' });
  }
  
  const userId = getUserIdFromToken(req.apiKey);
  const competitionId = req.body.competition || 'default';
  const now = Date.now();
  
  // Check if the user has attempts remaining
  const remainingAttempts = getRemainingAttempts(userId, competitionId);
  
  if (remainingAttempts <= 0) {
    return res.status(403).json({ 
      error: 'No submission attempts remaining for this competition'
    });
  }
  
  // Increment the attempt counter
  const attemptsKey = `${userId}_${competitionId}`;
  const currentAttempts = submissionAttempts.get(attemptsKey) || 0;
  submissionAttempts.set(attemptsKey, currentAttempts + 1);
  
  // Store the submission time for the user
  const submissionKey = `${userId}_${competitionId}`;
  userSubmissions.set(submissionKey, now);
  
  console.log(`Received file from user ${userId} for competition ${competitionId}`);
  console.log(`File: ${req.file.originalname} (${req.file.size} bytes)`);
  console.log(`Attempts used: ${currentAttempts + 1}, remaining: ${remainingAttempts - 1}`);
  
  // Process the file as needed
  // ...
  
  return res.status(200).json({ 
    message: 'File received successfully',
    filename: req.file.filename,
    size: req.file.size,
    timestamp: now,
    competition: competitionId,
    attempts_remaining: remainingAttempts - 1
  });
});

// Get competition info
app.get('/competitions', authenticate, (req, res) => {
  const compList = Array.from(competitions.entries()).map(([id, info]) => ({
    id,
    name: info.name,
    max_attempts: info.maxAttempts
  }));
  
  return res.status(200).json({ competitions: compList });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Optimus server running on port ${PORT}`);
  console.log(`Check endpoint: http://localhost:${PORT}/check`);
  console.log(`Submit endpoint: http://localhost:${PORT}/submit`);
  console.log(`Competitions endpoint: http://localhost:${PORT}/competitions`);
});