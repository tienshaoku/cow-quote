#!/bin/bash

# Ensure we're in the correct directory
cd "$(dirname "$0")/.."

# Build the Lambda function
cargo lambda build --release

# Deploy
cargo lambda deploy cow-quote

