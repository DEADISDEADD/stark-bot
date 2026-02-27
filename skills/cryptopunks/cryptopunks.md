---
name: cryptopunks
description: "Query, transfer, buy, sell, and bid on CryptoPunks on Ethereum mainnet"
version: 1.0.0
author: starkbot
metadata: {"clawdbot":{"emoji":"👾"}}
tags: [crypto, nft, cryptopunks, collectible, mainnet, marketplace]
abis: [cryptopunks]
presets_file: web3_presets.ron
requires_tools: [set_nft_token_id, set_address, to_raw_amount, web3_preset_function_call, list_queued_web3_tx, broadcast_web3_tx, verify_tx_broadcast, select_web3_network, define_tasks]
---

# CryptoPunks Skill

Interact with the original CryptoPunks contract on Ethereum mainnet.

**Contract:** `0xb47e3cd837dDF8e4c57F05d70Ab865de6e193BBB` (Ethereum mainnet)

**Important:** CryptoPunks are NOT ERC721. They use a custom marketplace contract with their own transfer, offer, bid, and buy functions.

## CRITICAL RULES

1. **ONE TASK AT A TIME.** Only do the work described in the CURRENT task. Do NOT work ahead.
2. **Do NOT call `say_to_user` with `finished_task: true` until the current task is truly done.**
3. **Use `say_to_user` WITHOUT `finished_task`** for progress updates. Only set `finished_task: true` OR call `task_fully_completed` when ALL steps in the current task are done.
4. **Sequential tool calls only.** Never call two tools in parallel when the second depends on the first.
5. **Register pattern prevents hallucination.** Never pass raw addresses/token IDs directly — always use registers set by the tools.
6. **Always select mainnet.** CryptoPunks only exist on Ethereum mainnet.

---

## Transfer Punk — Full 4-Task Workflow

### Step 1: Define the four tasks

```json
{"tool": "define_tasks", "tasks": [
  "TASK 1 — Select mainnet, set punk index, check ownership. See cryptopunks skill 'Task 1'.",
  "TASK 2 — Set recipient address. See cryptopunks skill 'Task 2'.",
  "TASK 3 — Execute transferPunk and broadcast. See cryptopunks skill 'Task 3'.",
  "TASK 4 — Verify the transfer and report to user. See cryptopunks skill 'Task 4'."
]}
```

### Task 1: Prepare — select network, check ownership

#### 1a. Select mainnet

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

#### 1b. Set token ID

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```

#### 1c. Check ownership

```json
{"tool": "web3_preset_function_call", "preset": "punk_owner", "network": "mainnet", "call_only": true}
```

Verify the owner matches the wallet address. If not, tell the user they do not own this punk.

#### 1d. Report findings

```json
{"tool": "say_to_user", "message": "CryptoPunk #<INDEX> owner: 0x...\nYou own this punk. Ready to transfer.", "finished_task": true}
```

### Task 2: Set recipient address

```json
{"tool": "set_address", "register": "nft_recipient_address", "address": "<RECIPIENT_ADDRESS>"}
```

```json
{"tool": "task_fully_completed", "summary": "Recipient set. Ready to execute transfer."}
```

### Task 3: Execute the transfer

#### 3a. Create the transfer transaction

```json
{"tool": "web3_preset_function_call", "preset": "punk_transfer", "network": "mainnet"}
```

Wait for the result. Extract the `uuid` from the response.

#### 3b. Broadcast it

```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid_from_3a>"}
```

### Task 4: Verify the transfer

```json
{"tool": "verify_tx_broadcast"}
```

Report success/failure to the user. Call `task_fully_completed` when verified.

---

## Buy a Punk — Full Workflow

```json
{"tool": "define_tasks", "tasks": [
  "TASK 1 — Select mainnet, set punk index, check if punk is for sale. See cryptopunks skill.",
  "TASK 2 — Set buy price and execute buyPunk. See cryptopunks skill.",
  "TASK 3 — Verify purchase and report. See cryptopunks skill."
]}
```

### Task 1: Check if for sale

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_check_offer", "network": "mainnet", "call_only": true}
```

If `isForSale` is false, tell the user this punk is not for sale and stop. Otherwise report the min price and seller.

### Task 2: Buy

```json
{"tool": "to_raw_amount", "amount": "<ETH_AMOUNT>", "decimals": 18, "cache_as": "punk_buy_price"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_buy", "network": "mainnet"}
```

Broadcast:
```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid>"}
```

### Task 3: Verify

```json
{"tool": "verify_tx_broadcast"}
```

---

## Offer Punk for Sale

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```

```json
{"tool": "to_raw_amount", "amount": "<MIN_PRICE_ETH>", "decimals": 18, "cache_as": "punk_sale_price"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_offer_for_sale", "network": "mainnet"}
```

Broadcast and verify.

---

## Delist Punk from Sale

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_no_longer_for_sale", "network": "mainnet"}
```

Broadcast and verify.

---

## Place a Bid

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```

```json
{"tool": "to_raw_amount", "amount": "<BID_ETH>", "decimals": 18, "cache_as": "punk_bid_amount"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_enter_bid", "network": "mainnet"}
```

Broadcast and verify.

---

## Accept a Bid

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```

```json
{"tool": "to_raw_amount", "amount": "<MIN_PRICE_ETH>", "decimals": 18, "cache_as": "punk_min_bid_price"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_accept_bid", "network": "mainnet"}
```

Broadcast and verify.

---

## Withdraw Earnings

```json
{"tool": "select_web3_network", "network": "mainnet"}
```

```json
{"tool": "web3_preset_function_call", "preset": "punk_withdraw_earnings", "network": "mainnet"}
```

Broadcast and verify.

---

## Query-Only Flows (No Transaction)

### Check who owns a punk

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "punk_owner", "network": "mainnet", "call_only": true}
```

### Check how many punks an address owns

```json
{"tool": "web3_preset_function_call", "preset": "punk_balance", "network": "mainnet", "call_only": true}
```

### Check if a punk is for sale

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "punk_check_offer", "network": "mainnet", "call_only": true}
```

### Check current bid on a punk

```json
{"tool": "set_nft_token_id", "token_id": "<PUNK_INDEX>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "punk_check_bid", "network": "mainnet", "call_only": true}
```

### Check pending withdrawals

```json
{"tool": "web3_preset_function_call", "preset": "punk_pending_withdrawals", "network": "mainnet", "call_only": true}
```

---

## Available Presets

| Preset | Description | Required Registers |
|--------|-------------|-------------------|
| `punk_owner` | Get owner of a punk | `nft_token_id` |
| `punk_balance` | Count punks owned | `wallet_address` (intrinsic) |
| `punk_transfer` | Transfer a punk | `nft_recipient_address`, `nft_token_id` |
| `punk_offer_for_sale` | List for sale | `nft_token_id`, `punk_sale_price` |
| `punk_offer_for_sale_to_address` | List for sale to specific buyer | `nft_token_id`, `punk_sale_price`, `nft_recipient_address` |
| `punk_buy` | Buy a punk (sends ETH) | `nft_token_id`, `punk_buy_price` |
| `punk_no_longer_for_sale` | Delist from sale | `nft_token_id` |
| `punk_enter_bid` | Place a bid (sends ETH) | `nft_token_id`, `punk_bid_amount` |
| `punk_withdraw_bid` | Withdraw a bid | `nft_token_id` |
| `punk_accept_bid` | Accept highest bid | `nft_token_id`, `punk_min_bid_price` |
| `punk_withdraw_earnings` | Withdraw sale ETH | (none) |
| `punk_check_offer` | Check sale listing | `nft_token_id` |
| `punk_check_bid` | Check current bid | `nft_token_id` |
| `punk_pending_withdrawals` | Check pending ETH | `wallet_address` (intrinsic) |
