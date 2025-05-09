use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{Read, Write, copy};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

#[derive(Deserialize, Debug)]
struct CheckResponse {
    required_format: String,
    last_submission_by_user: Option<u64>,
}

#[derive(Parser, Debug)]
#[clap(name = "optimus", about = "CLI tool to zip directories and submit them", author, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Zip the current directory and send it to the server
    Send {
        /// API key for authentication
        #[arg(long, env = "OPTIMUS_API_KEY")]
        api_key: String,
        
        /// Base URL for the server (without trailing slash)
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
        
        /// Compression level (0-9, where 0 is no compression and 9 is maximum compression)
        #[arg(long, default_value_t = 6)]
        compression: u8,
        
        /// Skip server check and force a specific format (repo or py)
        #[arg(long)]
        force_format: Option<String>,
    },
}

/// Check with the server for format requirements
fn check_with_server(server_url: &str, api_key: &str) -> Result<CheckResponse> {
    let check_url = format!("{}/check", server_url);
    println!("🔍 Checking with server: {}", check_url);
    
    let client = Client::new();
    let response = client.get(&check_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(Duration::from_secs(10))
        .send()?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to check with server. Status: {}, Body: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }
    
    let check_response: CheckResponse = response.json()?;
    
    // Print information about the server response
    println!("✅ Server requires format: {}", check_response.required_format);
    if let Some(last_submission) = check_response.last_submission_by_user {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH + Duration::from_secs(last_submission))
            .unwrap_or(Duration::from_secs(0));
        
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        println!("📊 Last submission was {} hours and {} minutes ago", hours, minutes);
    } else {
        println!("📊 No previous submissions found");
    }
    
    Ok(check_response)
}

/// Create a zip archive based on the specified format
fn create_zip_archive(compression: u8, format: &str) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let dir_name = current_dir.file_name()
        .context("Failed to get directory name")?
        .to_string_lossy();
    
    let temp_dir = env::temp_dir();
    let zip_path = temp_dir.join(format!("{}.zip", dir_name));
    
    // Delete the zip file if it already exists
    if zip_path.exists() {
        std::fs::remove_file(&zip_path)?;
    }
    
    // Create a new zip file
    let file = File::create(&zip_path)?;
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755)
        .compression_level(Some(compression));
    
    let mut zip = ZipWriter::new(file);
    
    // Common excluded directories and files
    let mut excluded = vec![
        ".git", 
        ".DS_Store", 
        "target",
        "node_modules",
        ".zip"
    ];
    
    // Build include pattern based on format
    let include_patterns: Vec<&str> = match format {
        "py" => {
            // Only include Python files and Python project files
            println!("🐍 Using Python format: Only including Python files and project configuration");
            vec![".py", "requirements.txt", "pyproject.toml", "setup.py", "setup.cfg", "Pipfile", "Pipfile.lock", "poetry.lock"]
        },
        _ => {
            // Default "repo" format: include everything except excluded files
            println!("📦 Using Repository format: Including all files except excluded ones");
            vec![]
        }
    };
    
    println!("🔄 Creating zip archive...");
    
    // Walk through the directory tree and add files to the zip
    for entry in WalkDir::new(&current_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let path_str = path.to_string_lossy();
        
        // Skip excluded directories and files
        if excluded.iter().any(|e| path_str.contains(e)) {
            continue;
        }
        
        // Skip if the path is the same as the current directory
        if path == current_dir {
            continue;
        }
        
        let name = path.strip_prefix(&current_dir)?;
        
        // For Python format, only include specific file types
        if format == "py" && path.is_file() {
            let should_include = include_patterns.iter()
                .any(|pattern| path_str.ends_with(pattern));
            
            if !should_include {
                continue;
            }
        }
        
        // If the path is a file, add it to the zip
        if path.is_file() {
            let mut file = File::open(path)?;
            zip.start_file(name.to_string_lossy(), options)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if path.is_dir() && !name.as_os_str().is_empty() {
            // If the path is a directory, add it as a directory entry to the zip
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    
    // Finish writing the zip file
    zip.finish()?;
    
    Ok(zip_path)
}

/// Send the zip file to the endpoint
fn send_zip_to_endpoint(zip_path: &Path, api_key: &str, submit_url: &str) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut zip_content = Vec::new();
    
    // Read the zip file into memory
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut zip_content)?;
    
    println!("📦 Sending zip file to server: {}", submit_url);
    
    // Create a multipart form with the zip file
    let file_name = zip_path.file_name()
        .context("Failed to get zip file name")?
        .to_string_lossy();
    
    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(zip_content)
            .file_name(file_name.to_string())
            .mime_str("application/zip")?);
    
    // Send the POST request with the API key in the header
    let client = Client::new();
    let response = client.post(submit_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()?;
    
    // Check if the request was successful
    if response.status().is_success() {
        println!("✅ Successfully sent the zip file to the server!");
        println!("   Response: {}", response.text()?);
    } else {
        return Err(anyhow::anyhow!(
            "Failed to send zip file to endpoint. Status: {}, Body: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }
    
    // Clean up the temporary zip file
    std::fs::remove_file(zip_path)?;
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Send { api_key, server, compression, force_format } => {
            // Determine the format to use - either from force_format or server check
            let format = if let Some(forced) = force_format {
                println!("⚠️ Bypassing server check, using forced format: {}", forced);
                forced.to_string()
            } else {
                // Contact the server to check the required format
                let check_response = check_with_server(server, api_key)?;
                check_response.required_format
            };
            
            // Validate format is either "repo" or "py"
            if format != "repo" && format != "py" {
                return Err(anyhow::anyhow!("Unsupported format from server: {}. Expected 'repo' or 'py'", format));
            }
            
            // Create zip archive based on the required format
            let zip_path = create_zip_archive(*compression, &format)?;
            println!("✅ Created zip archive at: {}", zip_path.display());
            
            // Send the zip file to the submit endpoint
            let submit_url = format!("{}/submit", server);
            send_zip_to_endpoint(&zip_path, api_key, &submit_url)?;
        }
    }
    
    Ok(())
}
