---
name: nft_token
description: "Transfer, query, and manage ERC721 NFTs on Base/Ethereum"
version: 1.0.0
author: starkbot
metadata: {"requires_auth": false, "clawdbot":{"emoji":"🖼️"}}
tags: [crypto, nft, erc721, transfer, token, collectible, base, wallet]
abis: [erc721]
presets_file: web3_presets.ron
requires_tools: [set_address, set_nft_token_id, web3_preset_function_call, list_queued_web3_tx, broadcast_web3_tx, verify_tx_broadcast, select_web3_network, define_tasks]
---

# ERC721 NFT Token Skill

## CRITICAL RULES

1. **ONE TASK AT A TIME.** Only do the work described in the CURRENT task. Do NOT work ahead.
2. **Do NOT call `say_to_user` with `finished_task: true` until the current task is truly done.** Using `finished_task: true` advances the task queue — if you use it prematurely, tasks get skipped.
3. **Use `say_to_user` WITHOUT `finished_task`** for progress updates. Only set `finished_task: true` OR call `task_fully_completed` when ALL steps in the current task are done.
4. **Sequential tool calls only.** Never call two tools in parallel when the second depends on the first.
5. **Register pattern prevents hallucination.** Never pass raw addresses/token IDs directly — always use registers set by the tools.

## NFT Transfer — Full 4-Task Workflow

### Step 1: Define the four tasks

Call `define_tasks` with all 4 tasks in order:

```json
{"tool": "define_tasks", "tasks": [
  "TASK 1 — Prepare: select network (if specified), set NFT contract address, get collection info (name/symbol), check ownership of token ID. See nft_token skill 'Task 1'.",
  "TASK 2 — Set up: set recipient address, set token ID. See nft_token skill 'Task 2'.",
  "TASK 3 — Execute: call nft_safe_transfer_from preset, then broadcast_web3_tx. See nft_token skill 'Task 3'.",
  "TASK 4 — Verify the transfer result and report to the user. See nft_token skill 'Task 4'."
]}
```

---

### Task 1: Prepare — set contract, get collection info, check ownership

#### 1a. Select network (if user specified one)

```json
{"tool": "select_web3_network", "network": "<network>"}
```

If no network specified, skip this step (default is base).

#### 1b. Set the NFT contract address

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<NFT_CONTRACT_ADDRESS>"}
```

#### 1c. Get collection info

```json
{"tool": "web3_preset_function_call", "preset": "nft_name", "network": "<network>", "call_only": true}
```

```json
{"tool": "web3_preset_function_call", "preset": "nft_symbol", "network": "<network>", "call_only": true}
```

#### 1d. Set token ID and check ownership

```json
{"tool": "set_nft_token_id", "token_id": "<TOKEN_ID>"}
```

```json
{"tool": "web3_preset_function_call", "preset": "nft_owner_of", "network": "<network>", "call_only": true}
```

Verify the owner matches the wallet address. If not, tell the user they do not own this token.

#### 1e. Report findings and complete

```json
{"tool": "say_to_user", "message": "Found NFT: <NAME> (<SYMBOL>) at 0x...\nToken #<ID> owner: 0x...\nYou own this token. Ready to transfer.", "finished_task": true}
```

**Do NOT proceed to setting recipient address in this task. Just report findings.**

---

### Task 2: Set recipient address and token ID

#### 2a. Set recipient address

```json
{"tool": "set_address", "register": "nft_recipient_address", "address": "<RECIPIENT_ADDRESS>"}
```

#### 2b. Set token ID (if not already set in Task 1)

```json
{"tool": "set_nft_token_id", "token_id": "<TOKEN_ID>"}
```

After both succeed:
```json
{"tool": "task_fully_completed", "summary": "Recipient set and token ID confirmed. Ready to execute transfer."}
```

---

### Task 3: Execute the transfer

**Exactly 2 tool calls, SEQUENTIALLY (one at a time, NOT in parallel):**

#### 3a. Create the transfer transaction (FIRST call)

```json
{"tool": "web3_preset_function_call", "preset": "nft_safe_transfer_from", "network": "<network>"}
```

The `nft_safe_transfer_from` preset reads `wallet_address`, `nft_recipient_address`, and `nft_token_id` from registers automatically.

Wait for the result. Extract the `uuid` from the response.

#### 3b. Broadcast it (SECOND call — after 3a succeeds)

```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid_from_3a>"}
```

The task auto-completes when `broadcast_web3_tx` succeeds.

---

### Task 4: Verify the transfer

Call `verify_tx_broadcast` to poll for the receipt and confirm the result:

```json
{"tool": "verify_tx_broadcast"}
```

Read the output:

- **"TRANSACTION VERIFIED"** → The transfer succeeded AND the AI confirmed it matches the user's intent. Report success with tx hash and explorer link.
- **"TRANSACTION CONFIRMED — INTENT MISMATCH"** → Confirmed on-chain but AI flagged a concern. Tell the user to check the explorer.
- **"TRANSACTION REVERTED"** → The transfer failed. Tell the user.
- **"CONFIRMATION TIMEOUT"** → Tell the user to check the explorer link.

Call `task_fully_completed` when verify_tx_broadcast returned VERIFIED or CONFIRMED.

---

## Query-Only Flows (No Transfer)

### Check who owns a specific token

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "set_nft_token_id", "token_id": "<TOKEN_ID>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_owner_of", "network": "<network>", "call_only": true}
```

### Check how many NFTs an address owns

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_balance_of", "network": "<network>", "call_only": true}
```

### Get token metadata URI

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "set_nft_token_id", "token_id": "<TOKEN_ID>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_token_uri", "network": "<network>", "call_only": true}
```

### Get collection name and symbol

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_name", "network": "<network>", "call_only": true}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_symbol", "network": "<network>", "call_only": true}
```

---

## Approval Flows

### Approve a specific token for an operator

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "set_address", "register": "nft_operator_address", "address": "<OPERATOR>"}
```
```json
{"tool": "set_nft_token_id", "token_id": "<TOKEN_ID>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_approve", "network": "<network>"}
```

### Set approval for all tokens

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "set_address", "register": "nft_operator_address", "address": "<OPERATOR>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_set_approval_for_all", "network": "<network>"}
```

### Check approval status

```json
{"tool": "set_address", "register": "nft_contract_address", "address": "<CONTRACT>"}
```
```json
{"tool": "set_nft_token_id", "token_id": "<TOKEN_ID>"}
```
```json
{"tool": "web3_preset_function_call", "preset": "nft_get_approved", "network": "<network>", "call_only": true}
```
