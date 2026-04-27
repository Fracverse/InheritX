#!/bin/bash

# Storage Directory Setup Script for Legacy Content Uploads

echo "Setting up storage directories for InheritX Legacy Content..."

# Create base storage directory
mkdir -p storage/legacy_content

# Set permissions (adjust as needed for your environment)
chmod 755 storage
chmod 755 storage/legacy_content

echo "✅ Storage directories created successfully!"
echo ""
echo "Directory structure:"
echo "  storage/"
echo "  └── legacy_content/"
echo ""
echo "Files will be organized as:"
echo "  storage/legacy_content/{user_id}/{year}/{month}/{day}/{filename}"
echo ""
echo "Next steps:"
echo "  1. Run migrations: cd backend && sqlx migrate run"
echo "  2. Run tests: cargo test legacy_content_tests"
echo "  3. Start backend: cargo run"
