#!/bin/bash

# Reserve Health API Testing Script
# This script tests all reserve health endpoints

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
ADMIN_TOKEN="${ADMIN_TOKEN:-your_admin_token_here}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $2"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $2"
        ((TESTS_FAILED++))
    fi
}

# Helper function to make API calls
api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ -z "$data" ]; then
        curl -s -X "$method" \
            -H "Authorization: Bearer $ADMIN_TOKEN" \
            -H "Content-Type: application/json" \
            "$BASE_URL$endpoint"
    else
        curl -s -X "$method" \
            -H "Authorization: Bearer $ADMIN_TOKEN" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$BASE_URL$endpoint"
    fi
}

echo "=========================================="
echo "Reserve Health API Testing"
echo "=========================================="
echo "Base URL: $BASE_URL"
echo ""

# Test 1: Health Check
echo "Test 1: Health Check"
response=$(curl -s "$BASE_URL/health")
if echo "$response" | grep -q "ok"; then
    print_result 0 "Server is healthy"
else
    print_result 1 "Server health check failed"
    echo "Response: $response"
fi
echo ""

# Test 2: Get All Reserve Health
echo "Test 2: GET /api/admin/reserve-health"
response=$(api_call GET "/api/admin/reserve-health")
if echo "$response" | grep -q "success"; then
    print_result 0 "Get all reserve health"
    echo "Response preview:"
    echo "$response" | jq '.data[0]' 2>/dev/null || echo "$response"
else
    print_result 1 "Get all reserve health"
    echo "Response: $response"
fi
echo ""

# Test 3: Get Specific Asset Health (USDC)
echo "Test 3: GET /api/admin/reserve-health/USDC"
response=$(api_call GET "/api/admin/reserve-health/USDC")
if echo "$response" | grep -q "success"; then
    print_result 0 "Get USDC reserve health"
    echo "Response:"
    echo "$response" | jq '.data' 2>/dev/null || echo "$response"
else
    print_result 1 "Get USDC reserve health"
    echo "Response: $response"
fi
echo ""

# Test 4: Get Specific Asset Health (XLM)
echo "Test 4: GET /api/admin/reserve-health/XLM"
response=$(api_call GET "/api/admin/reserve-health/XLM")
if echo "$response" | grep -q "success"; then
    print_result 0 "Get XLM reserve health"
    echo "Response:"
    echo "$response" | jq '.data' 2>/dev/null || echo "$response"
else
    print_result 1 "Get XLM reserve health"
    echo "Response: $response"
fi
echo ""

# Test 5: Sync Reserve Health
echo "Test 5: POST /api/admin/reserve-health/sync"
response=$(api_call POST "/api/admin/reserve-health/sync")
if echo "$response" | grep -q "success"; then
    print_result 0 "Sync reserve health"
    echo "Response:"
    echo "$response" | jq '.message' 2>/dev/null || echo "$response"
else
    print_result 1 "Sync reserve health"
    echo "Response: $response"
fi
echo ""

# Test 6: Analytics Endpoint
echo "Test 6: GET /api/admin/analytics/reserve-health"
response=$(api_call GET "/api/admin/analytics/reserve-health")
if echo "$response" | grep -q "success"; then
    print_result 0 "Get reserve health analytics"
    echo "Number of pools:"
    echo "$response" | jq '.data | length' 2>/dev/null || echo "N/A"
else
    print_result 1 "Get reserve health analytics"
    echo "Response: $response"
fi
echo ""

# Test 7: Verify Metrics Structure
echo "Test 7: Verify metrics structure"
response=$(api_call GET "/api/admin/reserve-health")
has_coverage=$(echo "$response" | jq '.data[0].coverage_ratio' 2>/dev/null)
has_utilization=$(echo "$response" | jq '.data[0].utilization_rate' 2>/dev/null)
has_status=$(echo "$response" | jq '.data[0].health_status' 2>/dev/null)

if [ "$has_coverage" != "null" ] && [ "$has_utilization" != "null" ] && [ "$has_status" != "null" ]; then
    print_result 0 "Metrics structure is correct"
    echo "Sample metrics:"
    echo "$response" | jq '.data[0] | {asset_code, coverage_ratio, utilization_rate, health_status}' 2>/dev/null
else
    print_result 1 "Metrics structure is incorrect"
fi
echo ""

# Test 8: Test Invalid Asset
echo "Test 8: GET /api/admin/reserve-health/INVALID"
response=$(api_call GET "/api/admin/reserve-health/INVALID")
if echo "$response" | grep -q "not found\|NotFound"; then
    print_result 0 "Invalid asset returns proper error"
else
    print_result 1 "Invalid asset should return error"
    echo "Response: $response"
fi
echo ""

# Test 9: Test Without Authentication
echo "Test 9: Test without authentication"
response=$(curl -s -X GET "$BASE_URL/api/admin/reserve-health")
if echo "$response" | grep -q "Unauthorized\|unauthorized\|401"; then
    print_result 0 "Unauthorized access properly rejected"
else
    print_result 1 "Should reject unauthorized access"
    echo "Response: $response"
fi
echo ""

# Test 10: Check Health Status Values
echo "Test 10: Verify health status values"
response=$(api_call GET "/api/admin/reserve-health")
statuses=$(echo "$response" | jq -r '.data[].health_status' 2>/dev/null)
valid_statuses=("healthy" "warning" "critical" "high_utilization" "moderate")
all_valid=true

for status in $statuses; do
    if [[ ! " ${valid_statuses[@]} " =~ " ${status} " ]]; then
        all_valid=false
        break
    fi
done

if [ "$all_valid" = true ]; then
    print_result 0 "All health statuses are valid"
    echo "Found statuses: $statuses"
else
    print_result 1 "Invalid health status found"
    echo "Statuses: $statuses"
fi
echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
