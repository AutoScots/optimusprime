#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Installing Optimus Prime...${NC}"

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo -e "${YELLOW}Git is not installed. Installing it now...${NC}"
    
    # Check the OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux - Try apt-get first (Debian/Ubuntu)
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y git
        # Try yum (Red Hat/CentOS)
        elif command -v yum &> /dev/null; then
            sudo yum install -y git
        # Try pacman (Arch)
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm git
        # Try zypper (openSUSE)
        elif command -v zypper &> /dev/null; then
            sudo zypper install -y git
        else
            echo -e "${RED}Error: Could not install Git. Unknown package manager.${NC}"
            echo -e "${BLUE}Please install Git manually and try again.${NC}"
            exit 1
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS - Check if brew is installed
        if command -v brew &> /dev/null; then
            brew install git
        else
            echo -e "${RED}Error: Homebrew is not installed on macOS.${NC}"
            echo -e "${BLUE}Please install Git manually and try again.${NC}"
            exit 1
        fi
    else
        echo -e "${RED}Error: Unknown OS. Could not install Git.${NC}"
        echo -e "${BLUE}Please install Git manually and try again.${NC}"
        exit 1
    fi
    
    # Verify installation
    if ! command -v git &> /dev/null; then
        echo -e "${RED}Failed to install Git.${NC}"
        echo -e "${BLUE}Please install Git manually and try again.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Git has been successfully installed!${NC}"
fi

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

# Create a temporary directory
TMP_DIR=$(mktemp -d)
echo -e "${BLUE}Created temporary directory: ${TMP_DIR}${NC}"

# Clone the repository
echo -e "${BLUE}Cloning Optimus repository...${NC}"
git clone --depth 1 https://github.com/AutoScots/optimusprime.git "$TMP_DIR/optimus"
cd "$TMP_DIR/optimus/repo-zipper"

# Fix the code issues
echo -e "${BLUE}Fixing code issues...${NC}"

# Use the updated Cargo.toml with the new dependencies
# Update Cargo.toml if needed
if ! grep -q "serde_yaml" Cargo.toml; then
    echo -e "${BLUE}Adding required dependencies...${NC}"
    cargo add serde_yaml --quiet
    cargo add dialoguer --quiet 
    cargo add dirs --quiet
fi

# Build and install
echo -e "${BLUE}Building and installing Optimus...${NC}"
cargo install --path .

# Clean up
echo -e "${BLUE}Cleaning up...${NC}"
cd -
rm -rf "$TMP_DIR"

# Verify installation
if command -v optimus &> /dev/null; then
    echo -e "${GREEN}✅ Optimus has been successfully installed!${NC}"
    echo -e "${BLUE}You can now use 'optimus' to zip and send your directories.${NC}"
    echo -e "${BLUE}To get started, create a configuration file:${NC}"
    echo -e "${GREEN}optimus init${NC}"
    echo -e ""
    echo -e "${BLUE}Then edit it to set your API key and preferences:${NC}"
    echo -e "${GREEN}nano submission.yml${NC}"
    echo -e ""
    echo -e "${BLUE}Finally, send your directory:${NC}"
    echo -e "${GREEN}optimus send${NC}"
else
    echo -e "${RED}❌ Installation failed. Please try installing manually with 'cargo install --git https://github.com/AutoScots/optimusprime.git --path repo-zipper'${NC}"
    exit 1
fi