#!/bin/bash

# Script to check and fix formatting before committing
# This prevents CI formatting failures

echo "🔍 Checking Rust code formatting..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed!"
    echo "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check formatting
echo "Running cargo fmt --check..."
if cargo fmt --check; then
    echo "✅ Code is properly formatted!"
    exit 0
else
    echo ""
    echo "❌ Code formatting issues found!"
    echo ""
    echo "Would you like to auto-fix? (y/n)"
    read -r response
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        echo "🔧 Fixing formatting..."
        cargo fmt
        echo "✅ Formatting fixed! Please review changes and commit."
        exit 0
    else
        echo "Please run: cargo fmt"
        exit 1
    fi
fi
