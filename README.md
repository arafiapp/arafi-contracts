# Arafi Contracts

Soroban smart contracts for the [Arafi](https://arafi.app) payment platform, built on the [Stellar](https://stellar.org) blockchain.

## Contracts

| Contract | Description |
|---|---|
| [`arafi-payment`](./contracts/arafi-payment) | One-time payment creation and settlement |
| [`arafi-subscription`](./contracts/arafi-subscription) | Recurring billing / subscription management |

---

### `arafi-payment`

Records payments on-chain and holds funds in escrow until the Arafi backend confirms the off-chain event and triggers settlement.

**Key functions**

| Function | Auth | Description |
|---|---|---|
| `init(admin)` | — | Deploy-time initialisation |
| `create_payment(sender, recipient, token, amount, reference)` | sender | Creates a payment and pulls funds into escrow |
| `settle(payment_id)` | admin | Releases escrowed funds to recipient |
| `get_payment(id)` | — | Read a payment record |
| `payment_count()` | — | Total payment count |

---

### `arafi-subscription`

Enables merchants to set up recurring billing plans. Arafi's backend (admin) calls `charge` on each billing cycle.

**Key functions**

| Function | Auth | Description |
|---|---|---|
| `init(admin)` | — | Deploy-time initialisation |
| `subscribe(subscriber, merchant, token, amount, interval)` | subscriber | Creates subscription and charges first cycle |
| `charge(sub_id)` | admin | Charges the subscriber for the current cycle |
| `cancel(caller, sub_id)` | subscriber or admin | Cancels a subscription |
| `get_subscription(id)` | — | Read a subscription record |

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- `wasm32v1-none` target: `rustup target add wasm32v1-none`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli): `cargo install --locked stellar-cli`

## Development

```bash
# Run all tests
cargo test

# Build all contracts (WASM)
cargo build --release --target wasm32v1-none

# Build a specific contract
cd contracts/arafi-payment && make build

# Run tests for a specific contract
cd contracts/arafi-payment && cargo test
```

## Deployment (Testnet)

```bash
# Configure testnet network
stellar network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Fund a deploy account
stellar keys generate --global deployer --network testnet

# Deploy arafi-payment
stellar contract deploy \
  --wasm target/wasm32v1-none/release/arafi_payment.wasm \
  --source deployer \
  --network testnet

# Deploy arafi-subscription
stellar contract deploy \
  --wasm target/wasm32v1-none/release/arafi_subscription.wasm \
  --source deployer \
  --network testnet
```

## Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts/overview)
- [Soroban Examples](https://github.com/stellar/soroban-examples)
- [SEP-41 Token Standard](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
