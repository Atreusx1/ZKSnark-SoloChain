#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "Setting up ZK-SNARK development environment..."

# Check for rustup
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}rustup not found. Installing...${NC}"
    curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
    source $HOME/.cargo/env
fi

# Check for Node.js (version 10 or higher)
if ! command -v node &> /dev/null; then
    echo -e "${RED}Node.js not found. Please install Node.js version 10 or higher${NC}"
    exit 1
fi

NODE_VERSION=$(node -v | cut -d. -f1 | tr -d 'v')
if [ "$NODE_VERSION" -lt 10 ]; then
    echo -e "${RED}Node.js version must be 10 or higher${NC}"
    exit 1
fi

# Install circom from source
if ! command -v circom &> /dev/null; then
    echo "Installing circom from source..."
    git clone https://github.com/iden3/circom.git
    cd circom
    cargo build --release
    cargo install --path circom
    cd ..
    rm -rf circom
fi

# Install snarkjs
if ! command -v snarkjs &> /dev/null; then
    echo "Installing snarkjs..."
    npm install -g snarkjs
fi

# Create directory structure
mkdir -p pallets/zksnark/src/circuits
mkdir -p pallets/zksnark/build

# Verify installations
echo -e "${GREEN}Checking installations:${NC}"
echo "circom version: $(circom --version)"
echo "snarkjs version: $(snarkjs --version)"
echo "Node.js version: $(node --version)"
echo "npm version: $(npm --version)"

echo -e "${GREEN}Setup completed successfully!${NC}"