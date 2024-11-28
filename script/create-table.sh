#!/bin/bash

# Table name
TABLE_NAME="orders"

# Create the table with GSIs
aws dynamodb create-table \
    --table-name ${TABLE_NAME} \
    --attribute-definitions \
        AttributeName=uid,AttributeType=S \
        AttributeName=buy_token,AttributeType=S \
        AttributeName=sell_token,AttributeType=S \
        AttributeName=owner,AttributeType=S \
    --key-schema AttributeName=uid,KeyType=HASH \
    --global-secondary-indexes \
        "[
          {
            \"IndexName\": \"BuyTokenIndex\",
            \"KeySchema\": [{\"AttributeName\":\"buy_token\",\"KeyType\":\"HASH\"}],
            \"Projection\": {\"ProjectionType\":\"ALL\"}
          },
          {
            \"IndexName\": \"SellTokenIndex\",
            \"KeySchema\": [{\"AttributeName\":\"sell_token\",\"KeyType\":\"HASH\"}],
            \"Projection\": {\"ProjectionType\":\"ALL\"}
          },
          {
            \"IndexName\": \"OwnerIndex\",
            \"KeySchema\": [{\"AttributeName\":\"owner\",\"KeyType\":\"HASH\"}],
            \"Projection\": {\"ProjectionType\":\"ALL\"}
          }
        ]" \
    --billing-mode PAY_PER_REQUEST

# Wait for table to be created and become active
echo "Waiting for table ${TABLE_NAME} to become active..."
aws dynamodb wait table-exists --table-name ${TABLE_NAME}

# Verify table status
TABLE_STATUS=$(aws dynamodb describe-table --table-name ${TABLE_NAME} --query 'Table.TableStatus' --output text)
if [ "$TABLE_STATUS" = "ACTIVE" ]; then
    echo "Table ${TABLE_NAME} created successfully and is now active!"
else
    echo "Table creation might have issues. Current status: ${TABLE_STATUS}"
fi
