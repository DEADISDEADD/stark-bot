# Create & Register New Agent Identity

Full lifecycle: create identity in DB → approve STARKBOT → register on-chain → finalize → verify.

## CRITICAL RULES

1. **ONE TASK AT A TIME.** Only do the work described in the CURRENT task. Do NOT work ahead.
2. **Do NOT call `say_to_user` with `finished_task: true` until the current task is truly done.** Using `finished_task: true` advances the task queue — if you use it prematurely, tasks get skipped.
3. **Use `say_to_user` WITHOUT `finished_task`** for progress updates. Only set `finished_task: true` OR call `task_fully_completed` when ALL steps in the current task are done.
4. **Sequential tool calls only.** Never call two tools in parallel when the second depends on the first.
5. **Register pattern prevents hallucination.** Never pass raw addresses/amounts directly — always use registers set by the tools.

---

## Step 1: Define the five tasks

Call `define_tasks` with all 5 tasks in order:

```json
{"tool": "define_tasks", "tasks": [
  "TASK 1 — Create identity: call register_new_identity with name, description, and optional image. See create_and_register flow 'Task 1'.",
  "TASK 2 — Approve STARKBOT: call identity_approve_registry preset, broadcast, wait for confirmation. See create_and_register flow 'Task 2'.",
  "TASK 3 — Register on-chain: call identity_register (or identity_register_no_uri) preset, broadcast, wait for confirmation. See create_and_register flow 'Task 3'.",
  "TASK 4 — Finalize: call identity_post_register to decode event and save agent_id to DB. See create_and_register flow 'Task 4'.",
  "TASK 5 — Verify registration and report success to the user. See create_and_register flow 'Task 5'."
]}
```

---

## Task 1: Create identity in DB

### 1a. Ask for a name

If the user did NOT already provide a name (via the `agent_name` argument or in their message), you MUST ask them before proceeding:

```json
{"tool": "say_to_user", "message": "What would you like to name your agent?"}
```

Wait for their response. Do NOT proceed until you have a name.

### 1b. Register the identity

Call `register_new_identity` with the user's chosen name, description, and optional image URL:

```json
{"tool": "register_new_identity", "name": "<agent_name>", "description": "<agent_description>", "image": "<optional_image_url>"}
```

This creates the local IDENTITY.json with:
- EIP-8004 registration type URL
- x402 support enabled by default
- Active status set to true
- Default trust types: reputation, x402-payments

**If the tool returns a hosted `agent_uri`**, remember it — you'll need it in Task 3.

After success:

```json
{"tool": "task_fully_completed", "summary": "Identity created in DB. Ready for STARKBOT approval."}
```

---

## Task 2: Approve STARKBOT spending

Approve the StarkLicense contract to spend 1000 STARKBOT (burned on registration).

### 2a. Create the approval transaction

```json
{"tool": "web3_preset_function_call", "preset": "identity_approve_registry", "network": "base"}
```

Wait for the result. Extract the `uuid` from the response.

### 2b. Broadcast the approval

```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid_from_2a>"}
```

Wait for confirmation (the tool polls automatically).

After the approval is confirmed:

```json
{"tool": "task_fully_completed", "summary": "STARKBOT approved for registry contract. Ready to register on-chain."}
```

---

## Task 3: Register on-chain

This mints an ERC-721 NFT and burns 1000 STARKBOT.

### Choose the right preset

- **If Task 1 returned an `agent_uri`** → use `identity_register` (the `agent_uri` register is already set)
- **If no URI available** → use `identity_register_no_uri` (you can set the URI later)

### 3a. Create the registration transaction

```json
{"tool": "web3_preset_function_call", "preset": "identity_register", "network": "base"}
```

Or without URI:

```json
{"tool": "web3_preset_function_call", "preset": "identity_register_no_uri", "network": "base"}
```

Wait for the result. Extract the `uuid` from the response.

### 3b. Broadcast the registration

```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid_from_3a>"}
```

Wait for confirmation. The `Registered` event is emitted with your agentId, URI, and owner address.

After the registration is confirmed:

```json
{"tool": "task_fully_completed", "summary": "Registration transaction confirmed on-chain. Ready to finalize."}
```

---

## Task 4: Finalize — decode event and save agent_id

Call `identity_post_register` to decode the `Registered` event from the transaction receipt, extract the `agentId`, and save it to the local database:

```json
{"tool": "identity_post_register"}
```

This tool:
1. Reads the most recent broadcast tx receipt
2. Decodes the `Registered(uint256 agentId, string uri, address owner)` event
3. Saves the `agent_id` to the DB and sets the `agent_id` register

After success:

```json
{"tool": "task_fully_completed", "summary": "Agent ID extracted and saved to DB. Registration complete."}
```

---

## Task 5: Verify and report success

Report the final result to the user. Include:
- Agent ID (from the `agent_id` register / Task 4 output)
- Agent name and description
- Transaction hash and Base explorer link
- The agent is now discoverable on-chain via EIP-8004

```json
{"tool": "task_fully_completed", "summary": "Identity fully created and registered on-chain. Agent ID: #<id>."}
```
