# CowSwap Settlement Price Comparison

This is a WIP side project, aiming to build a dashboard to showcase the generous surplus of CowSwap settlements, which is difficult to be estimated pre-swap.

## Motivation

CowSwap's settlement price, which includes extra surplus compared to pre-swap quote, is often better than price offers from many liquidity aggregators and DEXs.

However, the final settlement price and surplus cannot be predicted perfectly pre-swap, for solvers on CowSwap compete to offer the best price. This makes it difficult for users to be aware of the great prices on CowSwap.

Thus, this project aims to build a dashboard to visualise comparison between CowSwap and other liquidity aggregators and DEXs.

## Goals and Progress

- [x] Get order info from CowSwap API
- [x] Comparison with 0x API
- [x] Comparison with Uni V3 using Foundry Anvil
  - By forking one block before a settlement and impersonating the user to perfectly simulate "what if the trade was executed on Uni V3" swap
- [x] Setup AWS EC2
- [x] Setup DynamoDB
- [ ] Setup API Gateway
- [ ] Setup Frontend

## Commands

```
cargo build
cargo run
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

### AWS

#### Table

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

### Docker

- Test locally

```
docker run --env-file .env -p 9000:9000 cow-quote-local
```

- Go into the local image

```
docker run -it --rm cow-quote-local /bin/sh
```

## Built with

- Rust
- Foundry
- AWS EC2
- AWS DynamoDB
- Docker
- Alchemy, 0x & CowSwap APIs
