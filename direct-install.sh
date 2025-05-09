#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Installing Optimus Prime directly...${NC}"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust and Cargo are not installed. Installing them now...${NC}"
    
    # Check for curl
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}Error: curl is required to install Rust.${NC}"
        echo -e "${BLUE}Please install curl and try again.${NC}"
        exit 1
    fi
    
    # Install Rust using rustup
    echo -e "${BLUE}Installing Rust using rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source cargo environment
    source "$HOME/.cargo/env"
    
    # Verify installation
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Failed to install Rust and Cargo.${NC}"
        echo -e "${BLUE}Please install Rust manually from https://rustup.rs/ and try again.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Rust and Cargo have been successfully installed!${NC}"
fi

# Install directly from the git repository
echo -e "${BLUE}Installing Optimus from GitHub...${NC}"
cargo install --git https://github.com/AutoScots/optimusprime.git --branch main --path repo-zipper

# Verify installation
if command -v optimus &> /dev/null; then
    echo -e "${GREEN}✅ Optimus has been successfully installed!${NC}"
    echo -e "${BLUE}You can now use 'optimus send' to zip and send your directories.${NC}"
    echo -e "${BLUE}Don't forget to set your API key:${NC}"
    echo -e "${GREEN}export OPTIMUS_API_KEY=\"your-api-key\"${NC}"
else
    echo -e "${RED}❌ Installation failed. Please try installing manually with:${NC}"
    echo -e "${GREEN}cargo install --git https://github.com/AutoScots/optimusprime.git --branch main --path repo-zipper${NC}"
    exit 1
fi