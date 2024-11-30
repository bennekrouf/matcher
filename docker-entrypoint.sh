#!/bin/bash
set -e

# Initialize the database first
echo "Initializing database..."
./matcher --reload

# Start the server with logging
echo "Starting server..."
exec ./matcher --server 2>&1 | tee -a /app/logs/matcher.log
