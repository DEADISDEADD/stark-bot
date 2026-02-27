# Unregister Identity

Wipes the agent identity from the local database. The on-chain NFT is **not** burned or affected — you can re-import it later with `import_identity`.

---

## Unregister (keep IDENTITY.json file)

```json
{"tool": "unregister_identity", "confirm": true}
```

---

## Unregister and delete IDENTITY.json file

```json
{"tool": "unregister_identity", "confirm": true, "delete_identity_file": true}
```

---

## After unregistering

- The agent will behave as if it has no identity until you run `import_identity` again
- The on-chain NFT remains — you can re-import it anytime
- Tell the user their local identity has been cleared and how to re-import if needed
