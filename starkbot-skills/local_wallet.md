---
name: local_wallet
description: "Access the local burner wallet for on-chain queries and signing."
version: 1.0.0
author: starkbot
metadata: {"clawdbot":{"emoji":"wallet","requires":{"tools":["local_burner_wallet"]}}}
tags: [wallet, crypto, local, burner, address, base, ethereum]
---

# Local Wallet Access

Access the local burner wallet configured via `BURNER_WALLET_BOT_PRIVATE_KEY` environment variable.

## Default Behavior

When a user asks about their local wallet, **always include**:
1. Wallet address
2. ETH balance on Base
3. USDC balance on Base (contract: `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`)

## Using the local_burner_wallet Tool

### Get Wallet Address

```json
{
  "action": "address"
}
```

Returns the public address derived from the private key.

### Check ETH Balance

```json
{
  "action": "balance",
  "network": "base"
}
```

Networks: `base` (default), `mainnet`

### Check Token Balance (ERC20)

```json
{
  "action": "token_balance",
  "network": "base",
  "token": "0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b"
}
```

**Common tokens on Base:**
- BNKR: `0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b`
- USDC: `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`
- WETH: `0x4200000000000000000000000000000000000006`

### Sign a Message

```json
{
  "action": "sign",
  "message": "Hello, world!"
}
```

Returns the wallet address and signature.

## Example: Check BNKR Balance

```json
{
  "action": "token_balance",
  "network": "base",
  "token": "0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b"
}
```

Response:
```
Wallet: 0x57bf3C9d7e9ec12d02B63D645da1714e2eb1D989
Token: 0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b (BNKR)
Balance: 1000.0 (base)
```

## Important Notes

- This is a **burner wallet** - only use for testing/small amounts
- The private key is used for signing - use with caution
- Balance checks use public RPC endpoints (no cost)
- For paid RPC calls, use the `x402_rpc` tool instead
