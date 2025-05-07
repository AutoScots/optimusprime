use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::{Client, multipart};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(name = "repo-zipper", about = "Zips a git repository and sends it to an endpoint")]
struct Cli {
    #[clap(long, default_value = "API_KEY")]
    api_key_env: String,
    
    #[clap(long, default_value = "http://localhost:3000/submit")]
    endpoint: String,
    
    #[clap(long, default_value = "fastest")]
    compression: String,
}

fn find_repo_root() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;
    
    loop {
        if current_dir.join(".git").is_dir() {
            return Ok(current_dir);
        }
        
        if !current_dir.pop() {
            return Err(anyhow::anyhow!("Not inside a git repository"));
        }
    }
}

fn create_zip_from_repo(repo_root: &Path, compression: &str) -> Result<PathBuf> {
    let repo_name = repo_root.file_name()
        .context("Failed to get repository name")?
        .to_string_lossy();
    
    let zip_path = repo_root.join(format!("{}.zip", repo_name));
    
    // Check if zip_path already exists, and remove it if it does
    if zip_path.exists() {
        std::fs::remove_file(&zip_path)?;
    }
    
    // Set compression level based on user preference
    let compression_flag = match compression {
        "best" => "-9",    // Best compression
        "normal" => "-6",  // Default compression
        "fastest" => "-1", // Fastest compression
        "store" => "-0",   // No compression, just store
        _ => "-1",         // Default to fastest
    };
    
    // Create a command to use the system zip utility
    let status = Command::new("zip")
        .arg(compression_flag)
        .arg("-r")   // Recursive
        .arg(&zip_path)
        .arg(".")  // Current directory
        .current_dir(repo_root)
        .args(&[
            "-x", "*.git/*", 
            "-x", "*.DS_Store", 
            "-x", "target/*",
            "-x", "node_modules/*",
            "-x", "*.zip"
        ])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create zip file, zip command exited with non-zero status"));
    }
    
    Ok(zip_path)
}

fn send_zip_to_endpoint(zip_path: &Path, api_key: &str, endpoint: &str) -> Result<()> {
    let mut file = File::open(zip_path)?;
    let mut zip_content = Vec::new();
    file.read_to_end(&mut zip_content)?;
    
    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(zip_content)
            .file_name(zip_path.file_name().unwrap().to_string_lossy().to_string())
            .mime_str("application/zip")?);
    
    let client = Client::new();
    let response = client.post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to send zip file to endpoint. Status: {}, Body: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    
    let repo_root = find_repo_root()?;
    let zip_path = create_zip_from_repo(&repo_root, &args.compression)?;
    let api_key = env::var(&args.api_key_env)?;
    
    send_zip_to_endpoint(&zip_path, &api_key, &args.endpoint)?;
    
    Ok(())
}
