#!/bin/bash

# CI Check Script - Validates all CI requirements locally
# This script runs the same checks as the GitHub Actions CI workflow

set -e  # Exit on any error

echo "🔍 Running CI checks for InheritX Backend..."
echo "=============================================="
echo ""

# Check 1: Formatting
echo "📝 Check 1: Code Formatting"
echo "Running: cargo fmt --all -- --check"
if cargo fmt --all -- --check; then
    echo "✅ Formatting check passed"
else
    echo "❌ Formatting check failed"
    echo "Run 'cargo fmt --all' to fix formatting issues"
    exit 1
fi
echo ""

# Check 2: Clippy (linting)
echo "🔍 Check 2: Clippy Linting"
echo "Running: cargo clippy --all-targets --all-features -- -D warnings"
if cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
    echo "✅ Clippy check passed"
else
    echo "❌ Clippy check failed"
    echo "Fix the warnings above"
    exit 1
fi
echo ""

# Check 3: Tests
echo "🧪 Check 3: Running Tests"
echo "Running: cargo test"
if cargo test 2>&1; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi
echo ""

# Check 4: Build
echo "🔨 Check 4: Release Build"
echo "Running: cargo build --release"
if cargo build --release 2>&1; then
    echo "✅ Build passed"
else
    echo "❌ Build failed"
    exit 1
fi
echo ""

echo "=============================================="
echo "🎉 All CI checks passed!"
echo "Your code is ready to be pushed."
echo ""
