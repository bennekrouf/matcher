#!/bin/bash

# Configuration
HOST="0.0.0.0:50030"

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to test a single query
test_query() {
    local query="$1"
    local language="$2"
    local description="$3"

    echo -e "${BLUE}Testing: $description${NC}"
    echo "Query: $query"
    echo "Language: $language"
    echo "-----------------"

    REQUEST_PAYLOAD=$(cat <<EOF
{
    "query": "$query",
    "language": "$language",
    "debug": false,
    "show_all_matches": false
}
EOF
    )

    echo "Request payload:"
    echo "$REQUEST_PAYLOAD"
    echo "-----------------"

    # Execute the request and capture the response
    response=$(grpcurl -plaintext \
        -d "$REQUEST_PAYLOAD" \
        $HOST \
        matcher.Matcher/MatchQuery 2>&1)
    
    # Check if the command was successful
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Success:${NC}"
        echo "$response"
    else
        echo -e "${RED}Error:${NC}"
        echo "$response"
    fi
    echo "-----------------"
    echo
}


echo "Starting matcher service tests..."
echo "Using server at $HOST"
echo "=========================="
echo

# Test 1: Basic email sending
test_query \
    "envoie le document par email à fawzan@gmail.com" \
    "fr" \
    "Basic email sending test"

# Test 2: Another test case (you can add more)
test_query \
    "send the document to john@example.com" \
    "en" \
    "English email sending test"

# Test 3: Example with different parameters
test_query \
    "envoi le rapport à team@company.com" \
    "fr" \
    "Report sending test"

# List available services (for verification)
echo "Checking available services:"
echo "-----------------"
grpcurl -plaintext $HOST list

echo
echo "All tests completed."
