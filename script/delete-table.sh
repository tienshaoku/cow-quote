#!/bin/bash

TABLE_NAME="orders"

# Delete the table
aws dynamodb delete-table --table-name ${TABLE_NAME}

# Wait for table to be deleted
echo "Waiting for table ${TABLE_NAME} to be deleted..."
aws dynamodb wait table-not-exists --table-name ${TABLE_NAME}

echo "Table ${TABLE_NAME} has been deleted!"
