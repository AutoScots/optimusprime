#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Fixing Optimus Prime code issues...${NC}"

# Create a temporary directory
TMP_DIR=$(mktemp -d)
echo -e "${BLUE}Created temporary directory: ${TMP_DIR}${NC}"

# Clone the repository
echo -e "${BLUE}Cloning Optimus repository...${NC}"
git clone --depth 1 https://github.com/AutoScots/optimusprime.git "$TMP_DIR/optimus"

# Fix the code issues
echo -e "${BLUE}Fixing code issues...${NC}"

# 1. Fix the env attribute in clap
sed -i.bak 's/#\[arg(long, env = "OPTIMUS_API_KEY")]/#\[arg(long)]/' "$TMP_DIR/optimus/repo-zipper/src/main.rs"

# 2. Fix the type mismatch in compression_level
sed -i.bak 's/.compression_level(Some(compression));/.compression_level(Some(compression.into()));/' "$TMP_DIR/optimus/repo-zipper/src/main.rs"

# 3. Remove unused imports
sed -i.bak 's/use serde::{Deserialize, Serialize};/use serde::Deserialize;/' "$TMP_DIR/optimus/repo-zipper/src/main.rs"
sed -i.bak 's/use std::io::{Read, Write, copy};/use std::io::{Read, Write};/' "$TMP_DIR/optimus/repo-zipper/src/main.rs"

# Cleanup backups
rm "$TMP_DIR/optimus/repo-zipper/src/main.rs.bak"

# Build and install with the fixed code
echo -e "${BLUE}Building and installing Optimus with fixed code...${NC}"
cd "$TMP_DIR/optimus/repo-zipper"
cargo install --path .

# Cleanup
echo -e "${BLUE}Cleaning up...${NC}"
cd -
rm -rf "$TMP_DIR"

# Verify installation
if command -v optimus &> /dev/null; then
    echo -e "${GREEN}✅ Optimus has been successfully installed!${NC}"
    echo -e "${BLUE}You can now use 'optimus send' to zip and send your directories.${NC}"
    echo -e "${YELLOW}Note: This is a legacy version of Optimus. ${NC}"
    echo -e "${BLUE}For the latest version with configuration support, please use:${NC}"
    echo -e "${GREEN}curl -sSL https://raw.githubusercontent.com/AutoScots/optimusprime/main/install.sh | bash${NC}"
else
    echo -e "${RED}❌ Installation failed. ${NC}"
    exit 1
fi