#!/bin/bash

# 2FA Testing Script
# This script helps test the 2FA endpoints

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
USER_ID="${USER_ID:-00000000-0000-0000-0000-000000000001}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================="
echo "2FA Testing Script"
echo "=================================="
echo "Base URL: $BASE_URL"
echo "User ID: $USER_ID"
echo ""

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Test 1: Health Check
echo "Test 1: Health Check"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/health")
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "200" ]; then
    print_success "Health check passed"
    echo "Response: $body"
else
    print_error "Health check failed (HTTP $http_code)"
    echo "Response: $body"
fi
echo ""

# Test 2: Send OTP
echo "Test 2: Send OTP"
response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/user/send-2fa" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$USER_ID\"}")
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "200" ]; then
    print_success "OTP sent successfully"
    echo "Response: $body"
    print_info "Check server logs for the OTP code"
else
    print_error "Failed to send OTP (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi
echo ""

# Prompt for OTP
echo "=================================="
print_info "Please enter the OTP from the server logs:"
read -p "OTP: " otp

if [ -z "$otp" ]; then
    print_error "No OTP provided"
    exit 1
fi

if [ ${#otp} -ne 6 ]; then
    print_error "OTP must be 6 digits"
    exit 1
fi

echo ""

# Test 3: Verify OTP
echo "Test 3: Verify OTP"
response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/user/verify-2fa" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$USER_ID\", \"otp\": \"$otp\"}")
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "200" ]; then
    print_success "OTP verified successfully"
    echo "Response: $body"
else
    print_error "Failed to verify OTP (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi
echo ""

# Test 4: Try to verify again (should fail - OTP already used)
echo "Test 4: Try to verify again (should fail)"
response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/user/verify-2fa" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$USER_ID\", \"otp\": \"$otp\"}")
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "400" ]; then
    print_success "Correctly rejected used OTP"
    echo "Response: $body"
else
    print_error "Should have rejected used OTP (HTTP $http_code)"
    echo "Response: $body"
fi
echo ""

# Test 5: Test invalid OTP format
echo "Test 5: Test invalid OTP format"
response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/user/verify-2fa" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$USER_ID\", \"otp\": \"12345\"}")
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "400" ]; then
    print_success "Correctly rejected invalid format"
    echo "Response: $body"
else
    print_error "Should have rejected invalid format (HTTP $http_code)"
    echo "Response: $body"
fi
echo ""

# Test 6: Test non-existent user
echo "Test 6: Test non-existent user"
response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/user/send-2fa" \
    -H "Content-Type: application/json" \
    -d '{"user_id": "00000000-0000-0000-0000-999999999999"}')
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "404" ]; then
    print_success "Correctly rejected non-existent user"
    echo "Response: $body"
else
    print_error "Should have rejected non-existent user (HTTP $http_code)"
    echo "Response: $body"
fi
echo ""

echo "=================================="
print_success "All tests completed!"
echo "=================================="
