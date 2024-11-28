#!/bin/bash

# Exit on error
set -e

echo "Building local Docker image..."
docker build -t cow-quote-local -f Dockerfile.local .

echo "Starting local Lambda container..."
docker run --env-file .env -p 9000:9000 cow-quote-local

# The container will keep running until exited with Ctrl+C