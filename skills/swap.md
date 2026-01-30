---
name: swap
description: "Swap ERC20 tokens on Base using 0x DEX aggregator via quoter.defirelay.com"
version: 5.1.0
author: starkbot
homepage: https://0x.org
metadata: {"requires_auth": false, "clawdbot":{"emoji":"🔄"}}
tags: [crypto, defi, swap, dex, base, trading, 0x]
---

# Token Swap Integration (0x via DeFi Relay)

## IMPORTANT: ETH Must Be Wrapped First!

When selling ETH, you MUST wrap it to WETH first. The swap always uses WETH, not native ETH.

---

## Workflow A: Swapping ETH → Token

Use this when the user wants to swap ETH for another token.

### 1. Lookup WETH as sell token
```tool:token_lookup
symbol: WETH
cache_as: sell_token
```

### 2. Check ETH and WETH balances

Check WETH balance:
```tool:web3_function_call
preset: weth_balance
network: base
call_only: true
```

Check ETH balance:
```tool:x402_rpc
preset: get_balance
network: base
```

**Important:** Report both balances to the user. If they have enough WETH already, skip steps 3-4 and go directly to step 5.

### 3. Set wrap amount (only if WETH balance is insufficient)
```tool:register_set
key: wrap_amount
value: "100000000000000"
```

### 4. Wrap ETH to WETH (only if WETH balance is insufficient)
```tool:web3_function_call
preset: weth_deposit
network: base
```

### 5. Lookup buy token
```tool:token_lookup
symbol: USDC
cache_as: buy_token
```

### 6. Set sell amount
```tool:register_set
key: sell_amount
value: "100000000000000"
```

### 7. Get swap quote
```tool:x402_fetch
preset: swap_quote
network: base
cache_as: swap_quote
```

### 8. Get gas price
```tool:x402_rpc
preset: gas_price
network: base
```

### 9. Execute swap
```tool:web3_tx
from_register: swap_quote
max_fee_per_gas: "<GAS_PRICE>"
network: base
```

---

## Workflow B: Swapping Token → Token or Eth

Use this when selling any token OTHER than ETH (e.g., USDC → WETH).

### 1. Lookup sell token and check balance

Lookup the sell token:
```tool:token_lookup
symbol: USDC
cache_as: sell_token
```

Check the sell token balance (use the erc20_balance preset after setting token_address):
```tool:register_set
key: token_address
value: "<sell_token_address from lookup>"
```

```tool:web3_function_call
preset: erc20_balance
network: base
call_only: true
```

**Important:** Report the balance to the user. If insufficient, stop and inform them.

### 2. Lookup buy token
```tool:token_lookup
symbol: WETH
cache_as: buy_token
```

### 3. Set sell amount
```tool:register_set
key: sell_amount
value: "1000000"
```

### 4. Get swap quote
```tool:x402_fetch
preset: swap_quote
network: base
cache_as: swap_quote
```

### 5. Get gas price
```tool:x402_rpc
preset: gas_price
network: base
```

### 6. Execute swap
```tool:web3_tx
from_register: swap_quote
max_fee_per_gas: "<GAS_PRICE>"
network: base
```

---

## Quick Reference: Which Workflow?

| Selling | Workflow | Key Difference |
|---------|----------|----------------|
| ETH | Workflow A | Wrap ETH → WETH first, then swap WETH |
| WETH | Workflow B | No wrapping needed |
| USDC, other tokens | Workflow B | No wrapping needed |

---

## Amount Reference (Wei Values)

For ETH/WETH (18 decimals):
- 0.0001 ETH = `100000000000000`
- 0.001 ETH = `1000000000000000`
- 0.01 ETH = `10000000000000000`
- 0.1 ETH = `100000000000000000`
- 1 ETH = `1000000000000000000`

For USDC (6 decimals):
- 1 USDC = `1000000`
- 10 USDC = `10000000`
- 100 USDC = `100000000`

---

## CRITICAL RULES

### You CANNOT use register_set for these registers:
- `sell_token` - use `token_lookup` with `cache_as: "sell_token"`
- `buy_token` - use `token_lookup` with `cache_as: "buy_token"`

### Always wrap ETH before swapping!
If user says "swap ETH for X", you MUST:
1. Wrap ETH to WETH first (using `weth_deposit` preset)
2. Then swap WETH for X

---

## Supported Tokens

Use the `token_lookup` tool to check if a token is supported. The tool will return available tokens if the requested one isn't found.

---

## Error Handling

| Error | Fix |
|-------|-----|
| "Cannot set 'sell_token' via register_set" | Use `token_lookup` with `cache_as: "sell_token"` |
| "Cannot set 'buy_token' via register_set" | Use `token_lookup` with `cache_as: "buy_token"` |
| "Preset requires register 'X'" | Run the tool that sets register X first |
| "Insufficient balance" | Check balance before swapping |
| Swap fails with ETH | Make sure you wrapped ETH to WETH first! |
| **402 Payment Required / Settlement error** | **Wait 30 seconds and retry the same `x402_fetch` call. This is a temporary payment relay issue that usually resolves on retry. Retry up to 3 times before giving up.** |
