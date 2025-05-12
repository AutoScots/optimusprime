use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{Confirm, theme::ColorfulTheme};
use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SubmissionConfig {
    // Required fields
    api_key: String,
    
    // Optional fields
    #[serde(default)]
    competition_id: Option<String>,
    
    #[serde(default)]
    format: Option<String>,
    
    #[serde(default = "default_server_url")]
    server_url: String,
    
    #[serde(default = "default_compression_level")]
    compression_level: u8,
    
    #[serde(default)]
    exclude: Vec<String>,
    
    #[serde(default)]
    preferences: Preferences,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
struct Preferences {
    #[serde(default)]
    auto_confirm: bool,
    
    #[serde(default = "default_true")]
    save_history: bool,
}

fn default_true() -> bool {
    true
}

fn default_server_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_compression_level() -> u8 {
    6
}

#[derive(Deserialize, Debug)]
struct CheckResponse {
    submission_approved: bool,
    required_format: String,
    remaining_attempts: i32,
    last_submission_by_user: Option<u64>,
    competition_name: Option<String>,
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
        /// Path to the submission.yml config file
        #[arg(long, default_value = "submission.yml")]
        config: String,
        
        /// Competition ID (overrides config file)
        #[arg(long)]
        competition_id: Option<String>,
        
        /// API key for authentication (overrides config file)
        #[arg(long)]
        api_key: Option<String>,
        
        /// Base URL for the server (overrides config file)
        #[arg(long)]
        server: Option<String>,
        
        /// Compression level (0-9, overrides config file)
        #[arg(long)]
        compression: Option<u8>,
        
        /// Skip server check and force a specific format (repo or py) (overrides config file)
        #[arg(long)]
        force_format: Option<String>,
        
        /// Auto-confirm submission without prompting (overrides config file)
        #[arg(long)]
        auto_confirm: bool,
    },
    
    /// Initialize a new submission.yml configuration file
    Init {
        /// Path to create the submission.yml config file
        #[arg(long, default_value = "submission.yml")]
        config: String,
        
        /// API key for authentication
        #[arg(long)]
        api_key: Option<String>,
        
        /// Competition ID
        #[arg(long)]
        competition_id: Option<String>,
    },
}

/// Load the configuration file or create a default one if it doesn't exist
fn load_config(config_path: &str) -> Result<SubmissionConfig> {
    let config_file = PathBuf::from(config_path);
    
    if !config_file.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file '{}' not found. You can create one with `optimus init`.", 
            config_path
        ));
    }
    
    // Read the config file
    let file = File::open(config_file)?;
    let config: SubmissionConfig = serde_yaml::from_reader(file)?;
    
    Ok(config)
}

/// Create a new configuration file
fn create_config_file(config_path: &str, api_key: Option<String>, competition_id: Option<String>) -> Result<()> {
    let config_file = PathBuf::from(config_path);
    
    if config_file.exists() {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Config file '{}' already exists. Overwrite?", config_path))
            .default(false)
            .interact()?;
        
        if !overwrite {
            println!("❌ Config creation aborted.");
            return Ok(());
        }
    }
    
    // Create a default config
    let config = SubmissionConfig {
        api_key: api_key.unwrap_or_else(|| "your-api-key-here".to_string()),
        competition_id,
        format: None,
        server_url: default_server_url(),
        compression_level: default_compression_level(),
        exclude: vec![
            ".git".to_string(),
            ".DS_Store".to_string(), 
            "node_modules".to_string(),
            "target".to_string(),
            ".env".to_string(),
            "venv".to_string(),
        ],
        preferences: Preferences {
            auto_confirm: false,
            save_history: true,
        },
    };
    
    // Write the config to file
    let file = File::create(config_file)?;
    serde_yaml::to_writer(file, &config)?;
    
    println!("✅ Created configuration file: {}", config_path);
    println!("   Please edit it to set your API key and other preferences.");
    
    Ok(())
}

/// Check with the server for submission approval and format requirements
fn check_with_server(server_url: &str, api_key: &str, competition_id: Option<&str>) -> Result<CheckResponse> {
    let mut check_url = format!("{}/check", server_url);

    // Add competition_id query parameter if available
    if let Some(comp_id) = competition_id {
        check_url = format!("{}?competition={}", check_url, comp_id);
    }

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

    if check_response.submission_approved {
        println!("✅ Submission approved. Remaining attempts: {}", check_response.remaining_attempts);
    } else {
        println!("❌ Submission not approved. No remaining attempts.");
    }

    if let Some(competition_name) = &check_response.competition_name {
        println!("🏆 Competition: {}", competition_name);
    }

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

/// Create a zip archive based on the specified format and exclusions
fn create_zip_archive(compression: u8, format: &str, custom_exclusions: &[String]) -> Result<PathBuf> {
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
        .compression_level(Some(compression.into()));
    
    let mut zip = ZipWriter::new(file);
    
    // Common excluded directories and files
    let mut excluded = vec![
        ".git".to_string(), 
        ".DS_Store".to_string(), 
        "target".to_string(),
        "node_modules".to_string(),
        ".zip".to_string()
    ];
    
    // Add custom exclusions
    excluded.extend(custom_exclusions.iter().cloned());
    
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
        
        // Skip submission.yml
        if path_str.ends_with("submission.yml") {
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
fn send_zip_to_endpoint(zip_path: &Path, api_key: &str, submit_url: &str, competition_id: Option<&str>) -> Result<()> {
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
    
    let mut form = multipart::Form::new()
        .part("file", multipart::Part::bytes(zip_content)
            .file_name(file_name.to_string())
            .mime_str("application/zip")?);
    
    // Add competition_id if available
    if let Some(comp_id) = competition_id {
        form = form.text("competition", comp_id.to_string());
    }
    
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
        Commands::Init { config, api_key, competition_id } => {
            create_config_file(config, api_key.clone(), competition_id.clone())?;
        },
        
        Commands::Send { 
            config, 
            competition_id, 
            api_key, 
            server,
            compression, 
            force_format,
            auto_confirm
        } => {
            // Load the configuration
            let mut config_data = load_config(config)?;
            
            // Override config with command line arguments if provided
            if let Some(api) = api_key {
                config_data.api_key = api.clone();
            }
            
            let comp_id = competition_id.as_deref().or(config_data.competition_id.as_deref());
            
            let server_url = match server {
                Some(s) => s.clone(),
                None => config_data.server_url.clone(),
            };
            
            let comp_level = compression.unwrap_or(config_data.compression_level);
            
            let auto_confirm_submission = *auto_confirm || config_data.preferences.auto_confirm;
            
            // Determine the format to use - either from force_format, config, or server check
            let format = if let Some(forced) = force_format {
                println!("⚠️ Bypassing server check, using forced format: {}", forced);
                forced.clone()
            } else if let Some(config_format) = &config_data.format {
                println!("⚠️ Using format from config file: {}", config_format);
                config_format.clone()
            } else {
                // Contact the server to check for submission approval and format
                let check_response = check_with_server(&server_url, &config_data.api_key, comp_id)?;

                // Check if submission is approved
                if !check_response.submission_approved {
                    println!("❌ Submission not allowed. No remaining attempts.");
                    return Ok(());
                }

                // Prompt the user for confirmation
                if !auto_confirm_submission {
                    let confirm_msg = format!(
                        "Proceed with submission? You have {} attempts remaining.",
                        check_response.remaining_attempts
                    );

                    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(confirm_msg)
                        .default(true)
                        .interact()?;

                    if !confirmed {
                        println!("❌ Submission cancelled.");
                        return Ok(());
                    }
                }

                check_response.required_format
            };
            
            // Validate format is either "repo" or "py"
            if format != "repo" && format != "py" {
                return Err(anyhow::anyhow!(
                    "Unsupported format: {}. Expected 'repo' or 'py'", 
                    format
                ));
            }
            
            // Create zip archive based on the required format
            let zip_path = create_zip_archive(comp_level, &format, &config_data.exclude)?;
            println!("✅ Created zip archive at: {}", zip_path.display());
            
            // Send the zip file to the submit endpoint
            let submit_url = format!("{}/submit", server_url);
            send_zip_to_endpoint(&zip_path, &config_data.api_key, &submit_url, comp_id)?;
        }
    }
    
    Ok(())
}