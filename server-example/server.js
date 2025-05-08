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

// Create uploads directory if it doesn't exist
if (!fs.existsSync(UPLOAD_DIR)) {
  fs.mkdirSync(UPLOAD_DIR, { recursive: true });
}

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
  
  next();
}

// Handle zip file uploads
app.post('/submit', authenticate, upload.single('file'), (req, res) => {
  if (!req.file) {
    return res.status(400).json({ error: 'No file uploaded' });
  }
  
  console.log(`Received file: ${req.file.originalname} (${req.file.size} bytes)`);
  
  // Process the file as needed
  // ...
  
  return res.status(200).json({ 
    message: 'File received successfully',
    filename: req.file.filename,
    size: req.file.size
  });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Optimus server running on port ${PORT}`);
  console.log(`Upload endpoint: http://localhost:${PORT}/submit`);
});