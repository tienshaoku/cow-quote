#!/bin/bash

# Exit on error
set -e

AWS_REGION="ap-northeast-1" 
FUNCTION_NAME="cow-quote"
IMAGE_TAG="latest"
EC2_INSTANCE_IP="54.238.9.128"
KEY_PAIR_PATH="$HOME/.ssh/cow-quote.pem"

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

echo "Updating Docker on EC2 instance..."
ssh -i ${KEY_PAIR_PATH} ec2-user@${EC2_INSTANCE_IP} << EOF
    # Install Docker if not already installed
    if ! command -v docker &> /dev/null; then
        echo "Installing Docker..."
        sudo yum update -y
        sudo yum install -y docker
        sudo service docker start
        sudo usermod -a -G docker ec2-user
        # Apply group changes without exiting
        newgrp docker
    else
        echo "Docker is already installed"
    fi

    # Setup ECR credential helper only if config doesn't exist
    if [ ! -f ~/.docker/config.json ]; then
        echo "Setting up ECR credential helper..."
        sudo yum install -y amazon-ecr-credential-helper jq
        mkdir -p ~/.docker
        echo "{
            \"credHelpers\": {
                \"*.dkr.ecr.${AWS_REGION}.amazonaws.com\": \"ecr-login\"
            },
            \"credsStore\": \"ecr-login\"
        }" > ~/.docker/config.json
    else
        echo "ECR credential helper already configured"
    fi

    echo "Stopping and removing existing containers..."
    docker ps -q --filter ancestor=${ECR_REPO}:${IMAGE_TAG} | xargs -r docker stop
    docker ps -aq --filter ancestor=${ECR_REPO}:${IMAGE_TAG} | xargs -r docker rm
    
    echo "Pulling Docker image on EC2..."
    docker pull ${ECR_REPO}:${IMAGE_TAG}
    
    # ssh into the instance and create .env file then run this
    echo "Running Docker container..."
    docker run --env-file .env -p 9000:9000 ${ECR_REPO}:${IMAGE_TAG}
EOF

echo "Deployment complete!"