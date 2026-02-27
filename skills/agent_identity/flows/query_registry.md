# Query Agent Registry

Read-only queries against the StarkLicense registry contract. All use `call_only: true` (no transaction, no gas).

---

## Check Registration Fee

```json
{"tool": "web3_preset_function_call", "preset": "identity_registration_fee", "network": "base", "call_only": true}
```

Returns the current fee in STARKBOT (raw units — divide by 10^18 for human-readable).

---

## Total Registered Agents

```json
{"tool": "web3_preset_function_call", "preset": "identity_total_agents", "network": "base", "call_only": true}
```

---

## How Many Agents Does a Wallet Own?

Set `wallet_address` register first (to the address you want to check):

```json
{"tool": "web3_preset_function_call", "preset": "identity_balance", "network": "base", "call_only": true}
```

---

## Get Agent ID for a Wallet

Set `wallet_address` register first:

```json
{"tool": "web3_preset_function_call", "preset": "identity_token_of_owner", "network": "base", "call_only": true}
```

Returns the first agent ID owned by that wallet.

---

## Who Owns an Agent?

Set `agent_id` register first:

```json
{"tool": "web3_preset_function_call", "preset": "identity_owner_of", "network": "base", "call_only": true}
```

---

## Get Agent URI

Set `agent_id` register first:

```json
{"tool": "web3_preset_function_call", "preset": "identity_get_uri", "network": "base", "call_only": true}
```

---

## Get On-Chain Metadata

Set `agent_id` and `metadata_key` registers first:

```json
{"tool": "web3_preset_function_call", "preset": "identity_get_metadata", "network": "base", "call_only": true}
```

---

## Notes

- All queries are free (no gas, no signing)
- Results are returned directly from the contract call
- For queries that need registers, the agent should set them before calling the preset
