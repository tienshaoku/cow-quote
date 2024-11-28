#!/bin/bash

# Exit on error
set -e

echo "Building local Docker image..."
docker build -t cow-quote-local -f Dockerfile.local .

echo "Starting local Lambda container..."
docker run -p 9000:9000 \
    -e AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID} \
    -e AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY} \
    -e AWS_DEFAULT_REGION=${AWS_DEFAULT_REGION} \
    cow-quote-local

# The container will keep running until exited with Ctrl+C