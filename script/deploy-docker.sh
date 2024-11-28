#!/bin/bash

# Exit on error
set -e

AWS_REGION="ap-northeast-1" 
FUNCTION_NAME="cow-quote"
IMAGE_TAG="latest"

echo "Getting AWS account ID..."
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_REPO="${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${FUNCTION_NAME}"

echo "Building Docker image for AWS..."
docker build -t ${FUNCTION_NAME}:${IMAGE_TAG} -f Dockerfile.aws .

echo "Creating ECR repository if it doesn't exist..."
aws ecr create-repository --repository-name ${FUNCTION_NAME} || true

echo "Logging into ECR..."
aws ecr get-login-password --region ${AWS_REGION} | \
    docker login --username AWS --password-stdin ${ECR_REPO}

echo "Tagging image..."
docker tag ${FUNCTION_NAME}:${IMAGE_TAG} ${ECR_REPO}:${IMAGE_TAG}

echo "Pushing image to ECR..."
docker push ${ECR_REPO}:${IMAGE_TAG}

echo "Updating Lambda function..."
aws lambda update-function-code \
    --function-name ${FUNCTION_NAME} \
    --image-uri ${ECR_REPO}:${IMAGE_TAG} \
    --region ${AWS_REGION}

echo "Deployment complete!"