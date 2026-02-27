# Import / Read Identity

Read your existing identity from the database, or import one from on-chain.

---

## When to use

- **"what is my identity?"** / **"show my agent info"** → call with no params (reads from DB)
- **"import agent #N"** → call with `agent_id: N` (forces on-chain lookup)
- **Auto-discover** (no identity in DB yet) → call with no params (scans wallet via `balanceOf + tokenOfOwnerByIndex`)

---

## Read existing identity (no params)

```json
{"tool": "import_identity"}
```

If identity exists in the DB, returns it immediately without going on-chain.

---

## Import specific agent by ID

```json
{"tool": "import_identity", "agent_id": <number>}
```

Forces an on-chain lookup: verifies ownership, fetches the agent URI, persists the agent_id locally, and sets the `agent_id` register so you can immediately use on-chain presets.

---

## After import

Report the result to the user:
- Agent ID, name, description
- Whether it was loaded from DB or imported from on-chain
- The agent is ready for queries/updates using the on-chain presets
