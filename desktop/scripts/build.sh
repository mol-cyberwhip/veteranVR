#!/bin/bash

# Build script for Veteran Desktop
# This script runs all tests, regenerates bindings, and builds the application

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Veteran Desktop Build Script${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

# Check if we're in the right directory
if [ ! -f "src-tauri/Cargo.toml" ]; then
    echo -e "${RED}Error: src-tauri/Cargo.toml not found${NC}"
    echo "Please run this script from the desktop directory"
    exit 1
fi

echo -e "${YELLOW}Step 1: Running Rust tests...${NC}"
cd src-tauri
cargo test --lib
if [ $? -ne 0 ]; then
    echo -e "${RED}Rust tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Rust tests passed${NC}"
echo ""

echo -e "${YELLOW}Step 2: Regenerating TypeScript bindings...${NC}"
cargo test --lib generate_bindings
if [ $? -ne 0 ]; then
    echo -e "${RED}Binding generation failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Bindings regenerated${NC}"
echo ""

echo -e "${YELLOW}Step 3: Copying bindings to frontend...${NC}"
cp ../src/bindings.ts ../frontend/src/bindings.ts
echo -e "${GREEN}✓ Bindings copied${NC}"
echo ""

echo -e "${YELLOW}Step 4: Building frontend...${NC}"
cd ../frontend
npm run build
if [ $? -ne 0 ]; then
    echo -e "${RED}Frontend build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Frontend built${NC}"
echo ""

echo -e "${YELLOW}Step 5: Building Tauri application...${NC}"
cd ../src-tauri
# Use --no-before-build-command since we already built the frontend in Step 4
cargo tauri build --no-before-build-command
if [ $? -ne 0 ]; then
    echo -e "${RED}Tauri build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Tauri application built${NC}"
echo ""

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Build completed successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Output locations:"
echo "  - Frontend dist: frontend/dist/"
echo "  - Tauri bundles: src-tauri/target/release/bundle/"
echo ""
