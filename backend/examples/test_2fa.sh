#!/bin/bash

# Test script for 2FA endpoints
# Usage: ./test_2fa.sh

BASE_URL="http://localhost:8080"
USER_ID="550e8400-e29b-41d4-a716-446655440000"

echo "🔐 Testing InheritX 2FA Implementation"
echo "======================================"
echo ""

# Test 1: Send OTP
echo "📤 Test 1: Sending OTP..."
SEND_RESPONSE=$(curl -s -X POST "$BASE_URL/user/send-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$USER_ID\"}")

echo "Response: $SEND_RESPONSE"
echo ""

# Extract OTP from logs (in development, check server logs)
echo "⚠️  Check server logs for the OTP code"
echo ""

# Test 2: Verify OTP (you'll need to input the OTP from logs)
read -p "Enter the OTP from server logs: " OTP

echo "✅ Test 2: Verifying OTP..."
VERIFY_RESPONSE=$(curl -s -X POST "$BASE_URL/user/verify-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$USER_ID\", \"otp\": \"$OTP\"}")

echo "Response: $VERIFY_RESPONSE"
echo ""

# Test 3: Try invalid OTP
echo "❌ Test 3: Testing invalid OTP..."
INVALID_RESPONSE=$(curl -s -X POST "$BASE_URL/user/verify-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$USER_ID\", \"otp\": \"000000\"}")

echo "Response: $INVALID_RESPONSE"
echo ""

# Test 4: Try expired OTP (send new OTP and wait 5+ minutes)
echo "⏰ Test 4: Testing expired OTP..."
echo "Sending new OTP..."
curl -s -X POST "$BASE_URL/user/send-2fa" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$USER_ID\"}" > /dev/null

echo "Wait 5+ minutes and try to verify the OTP to test expiration"
echo ""

echo "✨ Testing complete!"
