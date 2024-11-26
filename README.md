## AWS Commands

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

### Scripts

- Create a new table `orders`

```
./script/create_table.sh
```

- Delete the table `orders`

```
./script/delete_table.sh
```
