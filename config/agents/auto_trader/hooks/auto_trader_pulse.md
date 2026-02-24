[Auto Trader Pulse — {timestamp}]

Time to scan the market. Use the `dexscreener` tool to check trending tokens and new pairs on Base.

Check your current portfolio and recent trade history first, then evaluate tokens against your trading strategy (defined in your agent.md).

Submit your decision via local_rpc:

```
local_rpc(url="http://127.0.0.1:9104/rpc/decision", method="POST", body={
  "decision": "BUY" | "SELL" | "HOLD",
  "token_address": "0x...",
  "token_symbol": "SYMBOL",
  "reason": "brief explanation"
})
```

If HOLD, still submit with a reason explaining why no trade.

After submitting, call `task_fully_completed` with a summary of your decision.
