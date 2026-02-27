# Update Agent Identity

Update your on-chain agent URI or metadata. Requires an existing registered identity (agent_id must be set).

## CRITICAL RULES

1. **ONE TASK AT A TIME.** Only do the work described in the CURRENT task. Do NOT work ahead.
2. **Do NOT call `say_to_user` with `finished_task: true` until the current task is truly done.** Using `finished_task: true` advances the task queue — if you use it prematurely, tasks get skipped.
3. **Use `say_to_user` WITHOUT `finished_task`** for progress updates. Only set `finished_task: true` OR call `task_fully_completed` when ALL steps in the current task are done.
4. **Sequential tool calls only.** Never call two tools in parallel when the second depends on the first.

---

## Update Agent URI

### Step 1: Define the tasks

```json
{"tool": "define_tasks", "tasks": [
  "TASK 1 — Prepare: ensure agent_id register is set (import identity if needed), set agent_uri register to the new URI. See update flow 'Task 1'.",
  "TASK 2 — Execute: call identity_set_uri preset, broadcast, wait for confirmation. See update flow 'Task 2'.",
  "TASK 3 — Verify the update and report success. See update flow 'Task 3'."
]}
```

### Task 1: Prepare registers

If the agent doesn't have an identity loaded yet, import it first:

```json
{"tool": "import_identity"}
```

The `agent_id` register should now be set. The `agent_uri` register must be set to the new URI value. If the user provided a new URI, it may already be set by `register_new_identity` — otherwise you may need to create/upload a new IDENTITY.json first.

```json
{"tool": "task_fully_completed", "summary": "Registers set: agent_id and agent_uri ready."}
```

### Task 2: Execute the URI update

#### 2a. Create the transaction

```json
{"tool": "web3_preset_function_call", "preset": "identity_set_uri", "network": "base"}
```

Wait for the result. Extract the `uuid`.

#### 2b. Broadcast

```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid_from_2a>"}
```

After confirmation:

```json
{"tool": "task_fully_completed", "summary": "URI updated on-chain."}
```

### Task 3: Verify and report

Report the updated URI and tx hash to the user.

```json
{"tool": "task_fully_completed", "summary": "Agent URI updated successfully."}
```

---

## Set On-Chain Metadata

For storing arbitrary key-value metadata on-chain.

### Step 1: Define the tasks

```json
{"tool": "define_tasks", "tasks": [
  "TASK 1 — Prepare: ensure agent_id register is set, set metadata_key and metadata_value registers. See update flow 'Metadata Task 1'.",
  "TASK 2 — Execute: call identity_set_metadata preset, broadcast, wait for confirmation. See update flow 'Metadata Task 2'.",
  "TASK 3 — Verify the update and report success. See update flow 'Metadata Task 3'."
]}
```

### Metadata Task 1: Prepare registers

Import identity if needed, then set registers:
- `agent_id` — already set from import
- `metadata_key` — the key string
- `metadata_value` — hex-encoded bytes value

```json
{"tool": "task_fully_completed", "summary": "Registers set: agent_id, metadata_key, metadata_value ready."}
```

### Metadata Task 2: Execute the metadata update

#### 2a. Create the transaction

```json
{"tool": "web3_preset_function_call", "preset": "identity_set_metadata", "network": "base"}
```

Wait for the result. Extract the `uuid`.

#### 2b. Broadcast

```json
{"tool": "broadcast_web3_tx", "uuid": "<uuid_from_2a>"}
```

After confirmation:

```json
{"tool": "task_fully_completed", "summary": "Metadata set on-chain."}
```

### Metadata Task 3: Verify and report

Report the metadata key, value, and tx hash to the user.

```json
{"tool": "task_fully_completed", "summary": "On-chain metadata updated successfully."}
```
