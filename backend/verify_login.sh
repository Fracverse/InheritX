#!/bin/bash

# Configuration
API_URL="http://localhost:8080"
EMAIL="admin@example.com"
PASSWORD="securePassword"

# 1. Login with correct credentials
echo "1. Testing Login Success..."
curl -v -X POST "$API_URL/admin/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}"
echo -e "\n"

# 2. Login with incorrect password
echo "2. Testing Login Failure (Bad Password)..."
curl -v -X POST "$API_URL/admin/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$EMAIL\", \"password\": \"wrongpassword\"}"
echo -e "\n"

# 3. Test Rate Limiting (send 10 requests quickly)
echo "3. Testing Rate Limiting..."
for i in {1..10}; do
    curl -s -o /dev/null -w "%{http_code}\n" -X POST "$API_URL/admin/login" \
      -H "Content-Type: application/json" \
      -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}" &
done
wait
echo -e "\nDone."
