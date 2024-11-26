## AWS Commands

### Table

- List tables

```
aws dynamodb list-tables
```

- Check all orders

```
aws dynamodb scan --table-name orders
```

- List tables

```
aws dynamodb list-tables
```

### Lambda

- Start a local lambda watch

```
cargo lambda watch
```

and then post it with `curl`

```
curl -XPOST "http://localhost:9000/lambda-url/cow-quote" -d '{}'
```

- Invoke aws deployment

```
aws lambda invoke --function-name cow-quote output.json
```

### Scripts

- Create a new table `orders`

```
./script/create_table.sh
```

- Delete the table `orders`

```
./script/delete_table.sh
```

- Deploy the lambda function

```
./script/deploy-lambda.sh
```
