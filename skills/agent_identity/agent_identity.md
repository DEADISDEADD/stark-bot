---
name: agent_identity
description: "Create, import, and register your EIP-8004 agent identity"
version: 3.0.1
author: starkbot
homepage: https://eips.ethereum.org/EIPS/eip-8004
tags: [crypto, identity, eip8004, registration, agent, discovery, nft]
requires_tools: [import_identity, register_new_identity, unregister_identity, identity_post_register, x402_rpc, web3_preset_function_call, broadcast_web3_tx, verify_tx_broadcast, read_file, define_tasks]
arguments:
  agent_name:
    description: "Name for the agent identity"
    required: false
  agent_description:
    description: "Description of the agent"
    required: false
  image_url:
    description: "URL to agent avatar/image"
    required: false
---

# EIP-8004 Agent Identity Management

Manage your on-chain agent identity using the EIP-8004 standard.

**Contract:** `0xa23a42D266653846e05d8f356a52298844537472` (Base mainnet, UUPS proxy)
**Payment token:** STARKBOT (`0x587Cd533F418825521f3A1daa7CCd1E7339A1B07`)
**Registration fee:** 1000 STARKBOT (burned on registration, mints an ERC-721 NFT)

---

## ROUTING: Read the correct flow file FIRST

Determine user intent, then `read_file` the matching flow document **before doing anything else**.

| User Intent | Flow File |
|-------------|-----------|
| "create a new identity" / "register new agent" / "set up my identity from scratch" | `read_file` → `{baseDir}/flows/create_and_register.md` |
| "what is my identity?" / "show my agent" / "import identity" / "import agent #N" | `read_file` → `{baseDir}/flows/import_identity.md` |
| "update my URI" / "set metadata" / "change my agent URL" | `read_file` → `{baseDir}/flows/update_identity.md` |
| "how many agents?" / "check fee" / "who owns agent #5?" / "get URI" / "get metadata" | `read_file` → `{baseDir}/flows/query_registry.md` |
| "unregister" / "remove identity" / "clear identity" | `read_file` → `{baseDir}/flows/unregister.md` |

**Example:** User says "create a new identity for my agent":

```json
{"tool": "read_file", "path": "{baseDir}/flows/create_and_register.md"}
```

Then follow the instructions in that flow file exactly.

---

## IMPORTANT: Import vs Create

- **NEVER** use `register_new_identity` when the user asks to import an existing NFT
- **"what is my identity?"** → import flow (read-only, returns existing DB identity)
- **"create from scratch"** → create_and_register flow (multi-step on-chain process)

---

## Identity File Format

The IDENTITY.json file follows the EIP-8004 registration file schema:

```json
{
  "type": "https://eips.ethereum.org/EIPS/eip-8004#registration-v1",
  "name": "Agent Name",
  "description": "What this agent does",
  "image": "https://example.com/avatar.png",
  "services": [
    {
      "name": "x402",
      "endpoint": "https://agent.example.com/x402",
      "version": "1.0"
    }
  ],
  "x402Support": true,
  "active": true,
  "supportedTrust": ["reputation", "x402-payments"]
}
```

## Available Presets

| Preset | Description |
|--------|-------------|
| `identity_approve_registry` | Approve 1000 STARKBOT for registration |
| `identity_allowance_registry` | Check STARKBOT allowance for registry |
| `identity_register` | Register with URI (requires approval) |
| `identity_register_no_uri` | Register without URI |
| `identity_set_uri` | Update agent URI |
| `identity_get_uri` | Get agent URI |
| `identity_registration_fee` | Get current fee |
| `identity_total_agents` | Get total registered agents |
| `identity_balance` | Get agent NFT count for wallet |
| `identity_owner_of` | Get owner of agent ID |
| `identity_token_of_owner` | Get first agent ID for wallet |
| `identity_set_metadata` | Set on-chain metadata |
| `identity_get_metadata` | Get on-chain metadata |
