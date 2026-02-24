# Auto Trader — RPC Reference

The `auto_trader` module exposes these RPC endpoints via `local_rpc`.
All endpoints are at `http://127.0.0.1:9104`.

## Decision

Submit a trading decision after evaluating the market.

```
local_rpc(url="http://127.0.0.1:9104/rpc/decision", method="POST", body={
  "decision": "BUY" | "SELL" | "HOLD",
  "token_address": "0x...",
  "token_symbol": "SYMBOL",
  "reason": "brief explanation of why"
})
```

- **BUY**: Module constructs an unsigned swap tx (WETH → token) via 0x API, stores it, and fires the `auto_trader_sign_tx` hook with tx details.
- **SELL**: Module constructs an unsigned swap tx (token → WETH) via 0x API, same flow.
- **HOLD**: No tx is constructed. Decision is logged for audit.

Response includes `decision_id` and, for BUY/SELL, the unsigned tx fields.

## Sign

After signing a transaction with `sign_raw_tx`, submit the signed hex:

```
local_rpc(url="http://127.0.0.1:9104/rpc/sign", method="POST", body={
  "tx_id": 123,
  "signed_tx": "0x..."
})
```

Module broadcasts via `eth_sendRawTransaction`, polls for receipt, and updates status.

## History

Query recent trade decisions:

```
local_rpc(url="http://127.0.0.1:9104/rpc/history", method="POST", body={
  "limit": 20,
  "status": "executed"
})
```

Optional filters: `limit` (default 20), `status` ("pending", "executed", "failed", "all").

## Stats

Aggregate trading statistics:

```
local_rpc(url="http://127.0.0.1:9104/rpc/stats", method="GET")
```

Returns total decisions, buys, sells, holds, executed count, and failed count.

## Config

View or update trader configuration:

```
local_rpc(url="http://127.0.0.1:9104/rpc/config", method="GET")
local_rpc(url="http://127.0.0.1:9104/rpc/config", method="POST", body={
  "key": "pulse_interval",
  "value": "240"
})
```

Config keys: `pulse_interval` (seconds), `max_trade_usd`, `chain`, `enabled`, `weth_address`.

## Control

Control the trading loop:

```
local_rpc(url="http://127.0.0.1:9104/rpc/control", method="POST", body={
  "action": "start" | "stop" | "trigger"
})
```

- **start**: Enable the background pulse timer.
- **stop**: Disable it.
- **trigger**: Fire a pulse immediately (ignores timer).

## Portfolio

View current token holdings:

```
local_rpc(url="http://127.0.0.1:9104/rpc/portfolio", method="GET")
```

Returns list of held tokens with addresses, symbols, amounts, and acquisition info.
