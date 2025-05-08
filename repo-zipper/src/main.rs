use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::blocking::{Client, multipart};
use std::env;
use std::fs::File;
use std::io::{Read, Write, copy};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

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
        
        /// Endpoint to send the zip file to
        #[arg(long, default_value = "http://localhost:3000/submit")]
        endpoint: String,
        
        /// Compression level (0-9, where 0 is no compression and 9 is maximum compression)
        #[arg(long, default_value_t = 6)]
        compression: u8,
    },
}

/// Create a zip archive of the current directory
fn create_zip_archive(compression: u8) -> Result<PathBuf> {
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
    
    // Excluded directories and files
    let excluded = vec![
        ".git", 
        ".DS_Store", 
        "target",
        "node_modules",
        ".zip",
        ".gitignore"
    ];
    
    // Walk through the directory tree and add files to the zip
    for entry in WalkDir::new(&current_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Skip excluded directories and files
        if excluded.iter().any(|e| path.to_string_lossy().contains(e)) {
            continue;
        }
        
        // Skip if the path is the same as the current directory
        if path == current_dir {
            continue;
        }
        
        let name = path.strip_prefix(&current_dir)?;
        
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
fn send_zip_to_endpoint(zip_path: &Path, api_key: &str, endpoint: &str) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut zip_content = Vec::new();
    
    // Read the zip file into memory
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut zip_content)?;
    
    println!("📦 Sending zip file to server: {}", endpoint);
    
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
    let response = client.post(endpoint)
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
        Commands::Send { api_key, endpoint, compression } => {
            println!("🔄 Creating zip archive of the current directory...");
            let zip_path = create_zip_archive(*compression)?;
            println!("✅ Created zip archive at: {}", zip_path.display());
            
            send_zip_to_endpoint(&zip_path, api_key, endpoint)?;
        }
    }
    
    Ok(())
}
