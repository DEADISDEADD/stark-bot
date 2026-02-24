# /// script
# requires-python = ">=3.12"
# dependencies = ["flask", "requests", "starkbot-sdk", "web3"]
#
# [tool.uv.sources]
# starkbot-sdk = { path = "../starkbot_sdk" }
# ///
"""
Auto Trader module — autonomous DeFi trader on Base.

Scans DexScreener for trending/new tokens, makes trade decisions via
the auto_trader agent persona, constructs swap transactions using the
0x Swap API, and broadcasts signed transactions on-chain.

RPC protocol endpoints:
  GET  /rpc/status         -> service health
  POST /rpc/decision       -> agent submits BUY/SELL/HOLD decision
  POST /rpc/sign           -> agent submits signed tx hex for broadcast
  POST /rpc/history        -> query trade decision history
  GET  /rpc/stats          -> aggregate trading statistics
  GET  /rpc/config         -> view trader config
  POST /rpc/config         -> update trader config
  POST /rpc/control        -> start/stop/trigger trading loop
  GET  /rpc/portfolio      -> current token holdings
  POST /rpc/backup/export  -> export data for backup
  POST /rpc/backup/restore -> restore data from backup
  GET  /                   -> HTML dashboard

Launch with:  uv run service.py
"""

from flask import request, Response
from starkbot_sdk import create_app, success, error
import sqlite3
import os
import json
import time
import logging
import threading
import requests as http_requests
from datetime import datetime, timezone

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

DB_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "auto_trader.db")
BACKEND_URL = os.environ.get("STARKBOT_BACKEND_URL", "http://127.0.0.1:8080")
INTERNAL_TOKEN = os.environ.get("STARKBOT_INTERNAL_TOKEN", "")
ALCHEMY_API_KEY = os.environ.get("ALCHEMY_API_KEY", "")
ZEROX_API_KEY = os.environ.get("ZEROX_API_KEY", "")

BASE_RPC_URL = f"https://base-mainnet.g.alchemy.com/v2/{ALCHEMY_API_KEY}" if ALCHEMY_API_KEY else ""
BASE_CHAIN_ID = 8453
WETH_BASE = "0x4200000000000000000000000000000000000006"

# 0x Swap API v2 (Permit2)
ZEROX_SWAP_URL = "https://api.0x.org/swap/permit2/quote"

# Defaults
DEFAULT_PULSE_INTERVAL = 240  # 4 minutes
DEFAULT_MAX_TRADE_USD = "20"

# Module state
_start_time = time.time()
_worker_running = False
_worker_lock = threading.Lock()
_last_pulse_at = None

# ---------------------------------------------------------------------------
# Database
# ---------------------------------------------------------------------------

def get_db():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")
    return conn


def init_db():
    conn = get_db()
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS trade_decisions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            decision        TEXT    NOT NULL,
            token_address   TEXT,
            token_symbol    TEXT,
            reason          TEXT,
            status          TEXT    NOT NULL DEFAULT 'pending',
            created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS trade_executions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            decision_id     INTEGER NOT NULL REFERENCES trade_decisions(id),
            raw_tx_to       TEXT,
            raw_tx_data     TEXT,
            raw_tx_value    TEXT,
            raw_tx_gas      TEXT,
            signed_tx       TEXT,
            tx_hash         TEXT,
            status          TEXT    NOT NULL DEFAULT 'unsigned',
            error_msg       TEXT,
            created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS trader_config (
            key     TEXT PRIMARY KEY,
            value   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS portfolio (
            token_address   TEXT PRIMARY KEY,
            token_symbol    TEXT,
            amount_raw      TEXT    NOT NULL DEFAULT '0',
            avg_buy_price   REAL,
            last_tx_hash    TEXT,
            updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );
    """)
    # Seed defaults if not present
    for k, v in [
        ("pulse_interval", str(DEFAULT_PULSE_INTERVAL)),
        ("max_trade_usd", DEFAULT_MAX_TRADE_USD),
        ("chain", "base"),
        ("enabled", "true"),
        ("weth_address", WETH_BASE),
    ]:
        conn.execute(
            "INSERT OR IGNORE INTO trader_config (key, value) VALUES (?, ?)", (k, v)
        )
    conn.commit()
    conn.close()


def get_config_value(key: str, default: str = "") -> str:
    conn = get_db()
    row = conn.execute("SELECT value FROM trader_config WHERE key = ?", (key,)).fetchone()
    conn.close()
    return row["value"] if row else default


def set_config_value(key: str, value: str):
    conn = get_db()
    conn.execute(
        "INSERT INTO trader_config (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (key, value),
    )
    conn.commit()
    conn.close()


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# 0x Swap API helpers
# ---------------------------------------------------------------------------

def get_swap_quote(sell_token: str, buy_token: str, sell_amount: str) -> dict | None:
    """Get a swap quote from 0x API for Base chain."""
    if not ZEROX_API_KEY:
        return None
    headers = {"0x-api-key": ZEROX_API_KEY, "0x-chain-id": str(BASE_CHAIN_ID)}
    params = {
        "chainId": BASE_CHAIN_ID,
        "sellToken": sell_token,
        "buyToken": buy_token,
        "sellAmount": sell_amount,
    }
    try:
        resp = http_requests.get(ZEROX_SWAP_URL, params=params, headers=headers, timeout=15)
        if resp.status_code == 200:
            return resp.json()
        logging.warning(f"[AUTO_TRADER] 0x quote failed ({resp.status_code}): {resp.text[:200]}")
    except Exception as e:
        logging.error(f"[AUTO_TRADER] 0x quote error: {e}")
    return None


def construct_swap_tx(decision: str, token_address: str, trade_amount_wei: str) -> dict | None:
    """Construct an unsigned swap tx via 0x API.

    BUY:  WETH -> token  (sell WETH, buy token)
    SELL: token -> WETH   (sell token, buy WETH)
    """
    if decision == "BUY":
        quote = get_swap_quote(WETH_BASE, token_address, trade_amount_wei)
    elif decision == "SELL":
        quote = get_swap_quote(token_address, WETH_BASE, trade_amount_wei)
    else:
        return None

    if not quote:
        return None

    tx = quote.get("transaction") or quote.get("tx")
    if not tx:
        return None

    return {
        "to": tx.get("to", ""),
        "data": tx.get("data", "0x"),
        "value": tx.get("value", "0"),
        "gas": tx.get("gas") or tx.get("gasLimit") or "350000",
    }


# ---------------------------------------------------------------------------
# Broadcast helper
# ---------------------------------------------------------------------------

def broadcast_tx(signed_tx_hex: str) -> str | None:
    """Broadcast a signed tx to Base via Alchemy and return the tx hash."""
    if not BASE_RPC_URL:
        return None
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_sendRawTransaction",
        "params": [signed_tx_hex],
    }
    try:
        resp = http_requests.post(BASE_RPC_URL, json=payload, timeout=30)
        data = resp.json()
        if "result" in data:
            return data["result"]
        err = data.get("error", {})
        logging.error(f"[AUTO_TRADER] Broadcast error: {err}")
    except Exception as e:
        logging.error(f"[AUTO_TRADER] Broadcast exception: {e}")
    return None


def poll_receipt(tx_hash: str, attempts: int = 12, delay: float = 5.0) -> dict | None:
    """Poll for a tx receipt on Base."""
    if not BASE_RPC_URL:
        return None
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getTransactionReceipt",
        "params": [tx_hash],
    }
    for _ in range(attempts):
        try:
            resp = http_requests.post(BASE_RPC_URL, json=payload, timeout=10)
            data = resp.json()
            receipt = data.get("result")
            if receipt:
                return receipt
        except Exception:
            pass
        time.sleep(delay)
    return None


# ---------------------------------------------------------------------------
# Hook firing
# ---------------------------------------------------------------------------

def fire_hook(event: str, data: dict | None = None):
    """Fire a custom persona hook via the backend internal API."""
    if not INTERNAL_TOKEN:
        logging.warning("[AUTO_TRADER] No STARKBOT_INTERNAL_TOKEN — cannot fire hooks")
        return
    try:
        http_requests.post(
            f"{BACKEND_URL}/api/internal/hooks/fire",
            json={"event": event, "data": data or {}},
            headers={"X-Internal-Token": INTERNAL_TOKEN},
            timeout=10,
        )
    except Exception as e:
        logging.error(f"[AUTO_TRADER] Hook fire error: {e}")


# ---------------------------------------------------------------------------
# Background pulse worker
# ---------------------------------------------------------------------------

def pulse_worker():
    global _last_pulse_at, _worker_running
    logger = logging.getLogger("auto_trader.worker")
    logger.info("[AUTO_TRADER] Pulse worker started")
    # Short initial delay
    time.sleep(10)
    while _worker_running:
        interval = int(get_config_value("pulse_interval", str(DEFAULT_PULSE_INTERVAL)))
        enabled = get_config_value("enabled", "true").lower() == "true"
        if enabled:
            logger.info("[AUTO_TRADER] Firing auto_trader_pulse hook")
            fire_hook("auto_trader_pulse")
            _last_pulse_at = now_iso()
        time.sleep(interval)


def start_worker():
    global _worker_running
    with _worker_lock:
        if _worker_running:
            return
        _worker_running = True
        t = threading.Thread(target=pulse_worker, daemon=True)
        t.start()


def stop_worker():
    global _worker_running
    with _worker_lock:
        _worker_running = False


# ---------------------------------------------------------------------------
# Flask app
# ---------------------------------------------------------------------------

def extra_status():
    return {
        "worker_running": _worker_running,
        "last_pulse_at": _last_pulse_at,
        "enabled": get_config_value("enabled", "true"),
    }


app = create_app("auto_trader", status_extra_fn=extra_status)


# ----- /rpc/decision -----

@app.route("/rpc/decision", methods=["POST"])
def rpc_decision():
    body = request.get_json(silent=True) or {}
    decision = (body.get("decision") or "").upper()
    if decision not in ("BUY", "SELL", "HOLD"):
        return error("decision must be BUY, SELL, or HOLD")

    token_address = body.get("token_address", "")
    token_symbol = body.get("token_symbol", "")
    reason = body.get("reason", "")

    conn = get_db()
    cur = conn.execute(
        "INSERT INTO trade_decisions (decision, token_address, token_symbol, reason, status) VALUES (?, ?, ?, ?, ?)",
        (decision, token_address, token_symbol, reason, "logged" if decision == "HOLD" else "pending"),
    )
    decision_id = cur.lastrowid
    conn.commit()

    result = {"decision_id": decision_id, "decision": decision, "token_symbol": token_symbol}

    if decision in ("BUY", "SELL"):
        # Construct swap tx via 0x API
        max_trade_usd = float(get_config_value("max_trade_usd", DEFAULT_MAX_TRADE_USD))
        # Approximate: $20 ≈ 0.006 ETH ≈ 6e15 wei at ~$3300/ETH (rough default)
        trade_amount_wei = str(int(max_trade_usd / 3300 * 1e18))

        tx = construct_swap_tx(decision, token_address, trade_amount_wei)
        if tx:
            conn2 = get_db()
            cur2 = conn2.execute(
                "INSERT INTO trade_executions (decision_id, raw_tx_to, raw_tx_data, raw_tx_value, raw_tx_gas, status) VALUES (?, ?, ?, ?, ?, 'unsigned')",
                (decision_id, tx["to"], tx["data"], tx["value"], tx["gas"]),
            )
            tx_id = cur2.lastrowid
            conn2.execute(
                "UPDATE trade_decisions SET status = 'tx_constructed' WHERE id = ?",
                (decision_id,),
            )
            conn2.commit()
            conn2.close()

            result["tx_id"] = tx_id
            result["tx"] = tx

            # Fire sign hook so the agent signs the tx
            fire_hook("auto_trader_sign_tx", {
                "tx_id": tx_id,
                "decision_id": decision_id,
                "decision": decision,
                "token_symbol": token_symbol,
                "to": tx["to"],
                "data": tx["data"],
                "value": tx["value"],
                "gas": tx["gas"],
                "chain_id": BASE_CHAIN_ID,
            })
        else:
            conn3 = get_db()
            conn3.execute(
                "UPDATE trade_decisions SET status = 'quote_failed' WHERE id = ?",
                (decision_id,),
            )
            conn3.commit()
            conn3.close()
            result["warning"] = "Failed to get swap quote from 0x API"

    conn.close()
    return success(result)


# ----- /rpc/sign -----

@app.route("/rpc/sign", methods=["POST"])
def rpc_sign():
    body = request.get_json(silent=True) or {}
    tx_id = body.get("tx_id")
    signed_tx = body.get("signed_tx", "")

    if not tx_id:
        return error("tx_id is required")
    if not signed_tx or not signed_tx.startswith("0x"):
        return error("signed_tx must be a 0x-prefixed hex string")

    conn = get_db()
    row = conn.execute("SELECT * FROM trade_executions WHERE id = ?", (tx_id,)).fetchone()
    if not row:
        conn.close()
        return error(f"No execution found with tx_id={tx_id}", 404)

    # Store signed tx
    conn.execute(
        "UPDATE trade_executions SET signed_tx = ?, status = 'signed', updated_at = ? WHERE id = ?",
        (signed_tx, now_iso(), tx_id),
    )
    conn.commit()
    conn.close()

    # Broadcast in background
    def do_broadcast():
        tx_hash = broadcast_tx(signed_tx)
        c = get_db()
        if tx_hash:
            c.execute(
                "UPDATE trade_executions SET tx_hash = ?, status = 'broadcasted', updated_at = ? WHERE id = ?",
                (tx_hash, now_iso(), tx_id),
            )
            c.execute(
                "UPDATE trade_decisions SET status = 'broadcasted', updated_at = ? WHERE id = (SELECT decision_id FROM trade_executions WHERE id = ?)",
                (now_iso(), tx_id),
            )
            c.commit()
            logging.info(f"[AUTO_TRADER] Broadcasted tx_id={tx_id} hash={tx_hash}")

            # Poll for receipt
            receipt = poll_receipt(tx_hash)
            if receipt:
                status_int = int(receipt.get("status", "0x0"), 16)
                final_status = "executed" if status_int == 1 else "reverted"
                c2 = get_db()
                c2.execute(
                    "UPDATE trade_executions SET status = ?, updated_at = ? WHERE id = ?",
                    (final_status, now_iso(), tx_id),
                )
                c2.execute(
                    "UPDATE trade_decisions SET status = ?, updated_at = ? WHERE id = (SELECT decision_id FROM trade_executions WHERE id = ?)",
                    (final_status, now_iso(), tx_id),
                )
                c2.commit()
                c2.close()

                # Update portfolio on successful BUY
                if final_status == "executed":
                    _update_portfolio_after_trade(tx_id, tx_hash)
            else:
                logging.warning(f"[AUTO_TRADER] Receipt timeout for tx_id={tx_id}")
        else:
            c.execute(
                "UPDATE trade_executions SET status = 'broadcast_failed', error_msg = 'RPC error', updated_at = ? WHERE id = ?",
                (now_iso(), tx_id),
            )
            c.execute(
                "UPDATE trade_decisions SET status = 'failed', updated_at = ? WHERE id = (SELECT decision_id FROM trade_executions WHERE id = ?)",
                (now_iso(), tx_id),
            )
            c.commit()
        c.close()

    threading.Thread(target=do_broadcast, daemon=True).start()
    return success({"tx_id": tx_id, "status": "broadcasting"})


def _update_portfolio_after_trade(tx_id: int, tx_hash: str):
    """Update portfolio table after a confirmed trade."""
    conn = get_db()
    row = conn.execute(
        "SELECT d.decision, d.token_address, d.token_symbol FROM trade_decisions d JOIN trade_executions e ON e.decision_id = d.id WHERE e.id = ?",
        (tx_id,),
    ).fetchone()
    if not row:
        conn.close()
        return
    decision = row["decision"]
    token_address = row["token_address"]
    token_symbol = row["token_symbol"]

    if decision == "BUY":
        conn.execute(
            """INSERT INTO portfolio (token_address, token_symbol, amount_raw, last_tx_hash, updated_at)
               VALUES (?, ?, '1', ?, ?)
               ON CONFLICT(token_address) DO UPDATE SET
                 amount_raw = CAST(CAST(amount_raw AS INTEGER) + 1 AS TEXT),
                 last_tx_hash = excluded.last_tx_hash,
                 updated_at = excluded.updated_at""",
            (token_address, token_symbol, tx_hash, now_iso()),
        )
    elif decision == "SELL":
        conn.execute(
            "DELETE FROM portfolio WHERE token_address = ?",
            (token_address,),
        )
    conn.commit()
    conn.close()


# ----- /rpc/history -----

@app.route("/rpc/history", methods=["GET", "POST"])
def rpc_history():
    body = request.get_json(silent=True) or {}
    limit = int(body.get("limit", 20))
    status_filter = body.get("status", "all")

    conn = get_db()
    if status_filter == "all":
        rows = conn.execute(
            "SELECT * FROM trade_decisions ORDER BY created_at DESC LIMIT ?", (limit,)
        ).fetchall()
    else:
        rows = conn.execute(
            "SELECT * FROM trade_decisions WHERE status = ? ORDER BY created_at DESC LIMIT ?",
            (status_filter, limit),
        ).fetchall()
    conn.close()
    return success([dict(r) for r in rows])


# ----- /rpc/stats -----

@app.route("/rpc/stats", methods=["GET"])
def rpc_stats():
    conn = get_db()
    total = conn.execute("SELECT COUNT(*) as c FROM trade_decisions").fetchone()["c"]
    buys = conn.execute("SELECT COUNT(*) as c FROM trade_decisions WHERE decision='BUY'").fetchone()["c"]
    sells = conn.execute("SELECT COUNT(*) as c FROM trade_decisions WHERE decision='SELL'").fetchone()["c"]
    holds = conn.execute("SELECT COUNT(*) as c FROM trade_decisions WHERE decision='HOLD'").fetchone()["c"]
    executed = conn.execute("SELECT COUNT(*) as c FROM trade_decisions WHERE status='executed'").fetchone()["c"]
    failed = conn.execute("SELECT COUNT(*) as c FROM trade_decisions WHERE status IN ('failed','reverted','broadcast_failed','quote_failed')").fetchone()["c"]
    conn.close()
    return success({
        "total_decisions": total,
        "buys": buys,
        "sells": sells,
        "holds": holds,
        "executed": executed,
        "failed": failed,
    })


# ----- /rpc/config -----

@app.route("/rpc/config", methods=["GET", "POST"])
def rpc_config():
    if request.method == "GET":
        conn = get_db()
        rows = conn.execute("SELECT * FROM trader_config").fetchall()
        conn.close()
        return success({r["key"]: r["value"] for r in rows})

    body = request.get_json(silent=True) or {}
    key = body.get("key")
    value = body.get("value")
    if not key or value is None:
        return error("key and value are required")
    allowed_keys = {"pulse_interval", "max_trade_usd", "chain", "enabled", "weth_address"}
    if key not in allowed_keys:
        return error(f"Unknown config key: {key}. Allowed: {', '.join(sorted(allowed_keys))}")
    set_config_value(key, str(value))
    return success({"key": key, "value": str(value)})


# ----- /rpc/control -----

@app.route("/rpc/control", methods=["POST"])
def rpc_control():
    body = request.get_json(silent=True) or {}
    action = body.get("action", "")

    if action == "start":
        start_worker()
        return success({"action": "start", "worker_running": True})
    elif action == "stop":
        stop_worker()
        return success({"action": "stop", "worker_running": False})
    elif action == "trigger":
        fire_hook("auto_trader_pulse")
        return success({"action": "trigger", "fired": True})
    else:
        return error("action must be 'start', 'stop', or 'trigger'")


# ----- /rpc/portfolio -----

@app.route("/rpc/portfolio", methods=["GET"])
def rpc_portfolio():
    conn = get_db()
    rows = conn.execute("SELECT * FROM portfolio ORDER BY updated_at DESC").fetchall()
    conn.close()
    return success([dict(r) for r in rows])


# ----- /rpc/backup -----

@app.route("/rpc/backup/export", methods=["POST"])
def rpc_backup_export():
    conn = get_db()
    decisions = conn.execute("SELECT * FROM trade_decisions ORDER BY id").fetchall()
    executions = conn.execute("SELECT * FROM trade_executions ORDER BY id").fetchall()
    config = conn.execute("SELECT * FROM trader_config").fetchall()
    portfolio = conn.execute("SELECT * FROM portfolio").fetchall()
    conn.close()
    return success({
        "decisions": [dict(r) for r in decisions],
        "executions": [dict(r) for r in executions],
        "config": {r["key"]: r["value"] for r in config},
        "portfolio": [dict(r) for r in portfolio],
    })


@app.route("/rpc/backup/restore", methods=["POST"])
def rpc_backup_restore():
    body = request.get_json(silent=True) or {}
    data = body.get("data") or body
    conn = get_db()
    restored = 0

    for d in data.get("decisions", []):
        try:
            conn.execute(
                "INSERT OR REPLACE INTO trade_decisions (id, decision, token_address, token_symbol, reason, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (d["id"], d["decision"], d.get("token_address"), d.get("token_symbol"), d.get("reason"), d.get("status", "pending"), d.get("created_at"), d.get("updated_at")),
            )
            restored += 1
        except Exception:
            pass

    for e in data.get("executions", []):
        try:
            conn.execute(
                "INSERT OR REPLACE INTO trade_executions (id, decision_id, raw_tx_to, raw_tx_data, raw_tx_value, raw_tx_gas, signed_tx, tx_hash, status, error_msg, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (e["id"], e["decision_id"], e.get("raw_tx_to"), e.get("raw_tx_data"), e.get("raw_tx_value"), e.get("raw_tx_gas"), e.get("signed_tx"), e.get("tx_hash"), e.get("status", "unsigned"), e.get("error_msg"), e.get("created_at"), e.get("updated_at")),
            )
            restored += 1
        except Exception:
            pass

    for k, v in data.get("config", {}).items():
        set_config_value(k, v)

    for p in data.get("portfolio", []):
        try:
            conn.execute(
                "INSERT OR REPLACE INTO portfolio (token_address, token_symbol, amount_raw, avg_buy_price, last_tx_hash, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                (p["token_address"], p.get("token_symbol"), p.get("amount_raw", "0"), p.get("avg_buy_price"), p.get("last_tx_hash"), p.get("updated_at")),
            )
            restored += 1
        except Exception:
            pass

    conn.commit()
    conn.close()
    return success({"restored": restored})


# ----- Dashboard -----

DASHBOARD_HTML = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Auto Trader</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;
     background:#0d1117;color:#c9d1d9;padding:24px}
h1{font-size:1.4rem;margin-bottom:16px;color:#58a6ff}
h2{font-size:1.1rem;margin:20px 0 10px;color:#79c0ff}
.stats{display:flex;gap:12px;flex-wrap:wrap;margin-bottom:20px}
.stat{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:12px 18px;min-width:100px}
.stat .val{font-size:1.6rem;font-weight:bold;color:#58a6ff}
.stat .lbl{font-size:.8rem;color:#8b949e;margin-top:2px}
.stat.buy .val{color:#3fb950}
.stat.sell .val{color:#f85149}
.toolbar{display:flex;gap:8px;margin-bottom:12px;align-items:center}
.btn{background:#238636;color:#fff;border:none;padding:6px 14px;border-radius:6px;
     cursor:pointer;font-size:.85rem;font-weight:500}
.btn:hover{background:#2ea043}
.btn-danger{background:#da3633}.btn-danger:hover{background:#f85149}
.btn-secondary{background:#30363d;color:#c9d1d9}.btn-secondary:hover{background:#484f58}
.badge{display:inline-block;padding:2px 8px;border-radius:10px;font-size:.75rem;font-weight:600}
.badge-buy{background:#238636;color:#fff}
.badge-sell{background:#da3633;color:#fff}
.badge-hold{background:#30363d;color:#8b949e}
.badge-ok{background:#238636;color:#fff}
.badge-fail{background:#da3633;color:#fff}
.badge-pending{background:#d29922;color:#000}
table{width:100%;border-collapse:collapse;margin-top:8px}
th,td{text-align:left;padding:8px 10px;border-bottom:1px solid #21262d;font-size:.85rem}
th{color:#8b949e;font-weight:600;text-transform:uppercase;font-size:.75rem}
td{font-family:"SF Mono",Consolas,monospace}
.empty{color:#484f58;padding:20px;text-align:center}
.worker-status{font-size:.85rem;color:#8b949e;flex:1}
.worker-status .dot{display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:4px}
.dot-on{background:#3fb950}.dot-off{background:#f85149}
.toast{position:fixed;bottom:20px;right:20px;padding:10px 16px;border-radius:6px;
       opacity:0;transition:opacity .3s;pointer-events:none;z-index:99;color:#fff}
.toast.show{opacity:1}.toast.ok{background:#238636}.toast.err{background:#da3633}
</style>
</head>
<body>
<h1>Auto Trader</h1>

<div class="stats" id="stats"><div class="stat"><div class="val">...</div><div class="lbl">Loading</div></div></div>

<div class="toolbar">
  <div class="worker-status" id="worker-status">...</div>
  <button class="btn" onclick="ctrl('trigger')">Trigger Pulse</button>
  <button class="btn btn-secondary" onclick="ctrl('start')">Start Worker</button>
  <button class="btn btn-danger" onclick="ctrl('stop')">Stop Worker</button>
</div>

<h2>Recent Decisions</h2>
<table>
<thead><tr><th>ID</th><th>Decision</th><th>Token</th><th>Reason</th><th>Status</th><th>Time</th></tr></thead>
<tbody id="decisions"><tr><td colspan="6" class="empty">Loading...</td></tr></tbody>
</table>

<h2>Portfolio</h2>
<table>
<thead><tr><th>Token</th><th>Address</th><th>Amount</th><th>Last TX</th><th>Updated</th></tr></thead>
<tbody id="portfolio"><tr><td colspan="5" class="empty">Loading...</td></tr></tbody>
</table>

<div class="toast" id="toast"></div>

<script>
function api(path,opts){return fetch(path,opts).then(r=>r.json())}

function toast(msg,ok){
  const t=document.getElementById('toast');
  t.textContent=msg;t.className='toast show '+(ok?'ok':'err');
  setTimeout(()=>t.className='toast',2000);
}

function badge(type,text){
  const cls={'BUY':'buy','SELL':'sell','HOLD':'hold',
    'executed':'ok','broadcasted':'ok','signed':'pending',
    'tx_constructed':'pending','pending':'pending','logged':'hold',
    'failed':'fail','reverted':'fail','broadcast_failed':'fail','quote_failed':'fail'};
  return '<span class="badge badge-'+(cls[type]||'hold')+'">'+text+'</span>';
}

function loadStats(){
  api('rpc/stats').then(d=>{
    const s=d.data||{};
    document.getElementById('stats').innerHTML=
      '<div class="stat"><div class="val">'+s.total_decisions+'</div><div class="lbl">Decisions</div></div>'+
      '<div class="stat buy"><div class="val">'+s.buys+'</div><div class="lbl">Buys</div></div>'+
      '<div class="stat sell"><div class="val">'+s.sells+'</div><div class="lbl">Sells</div></div>'+
      '<div class="stat"><div class="val">'+s.holds+'</div><div class="lbl">Holds</div></div>'+
      '<div class="stat buy"><div class="val">'+s.executed+'</div><div class="lbl">Executed</div></div>'+
      '<div class="stat sell"><div class="val">'+s.failed+'</div><div class="lbl">Failed</div></div>';
  });
}

function loadDecisions(){
  api('rpc/history',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({limit:30})}).then(d=>{
    const rows=d.data||[];
    const tb=document.getElementById('decisions');
    if(!rows.length){tb.innerHTML='<tr><td colspan="6" class="empty">No decisions yet</td></tr>';return}
    tb.innerHTML=rows.map(r=>'<tr>'+
      '<td>'+r.id+'</td>'+
      '<td>'+badge(r.decision,r.decision)+'</td>'+
      '<td>'+(r.token_symbol||'—')+'</td>'+
      '<td style="max-width:300px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">'+(r.reason||'—')+'</td>'+
      '<td>'+badge(r.status,r.status)+'</td>'+
      '<td>'+(r.created_at||'')+'</td>'+
    '</tr>').join('');
  });
}

function loadPortfolio(){
  api('rpc/portfolio').then(d=>{
    const rows=d.data||[];
    const tb=document.getElementById('portfolio');
    if(!rows.length){tb.innerHTML='<tr><td colspan="5" class="empty">No positions</td></tr>';return}
    tb.innerHTML=rows.map(r=>'<tr>'+
      '<td>'+(r.token_symbol||'?')+'</td>'+
      '<td title="'+r.token_address+'">'+(r.token_address?r.token_address.slice(0,6)+'...'+r.token_address.slice(-4):'—')+'</td>'+
      '<td>'+r.amount_raw+'</td>'+
      '<td title="'+(r.last_tx_hash||'')+'">'+(r.last_tx_hash?r.last_tx_hash.slice(0,10)+'...':'—')+'</td>'+
      '<td>'+(r.updated_at||'')+'</td>'+
    '</tr>').join('');
  });
}

function loadWorker(){
  api('rpc/status').then(d=>{
    const s=d.data||{};
    const running=s.worker_running;
    const el=document.getElementById('worker-status');
    el.innerHTML='<span class="dot '+(running?'dot-on':'dot-off')+'"></span>Worker '+(running?'running':'stopped')+
      (s.last_pulse_at?' &middot; Last pulse: '+s.last_pulse_at:'');
  });
}

function ctrl(action){
  api('rpc/control',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({action:action})}).then(d=>{
    toast(action+' OK',d.success!==false);
    loadWorker();
    if(action==='trigger')setTimeout(()=>{loadDecisions();loadStats()},3000);
  });
}

loadStats();loadDecisions();loadPortfolio();loadWorker();
setInterval(()=>{loadStats();loadDecisions();loadPortfolio();loadWorker()},15000);
</script>
</body>
</html>"""


@app.route("/")
def dashboard():
    return Response(DASHBOARD_HTML, content_type="text/html")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
    logging.getLogger("werkzeug").setLevel(logging.ERROR)
    init_db()
    port = int(os.environ.get("MODULE_PORT", os.environ.get("AUTO_TRADER_PORT", "9104")))
    # Start pulse worker if enabled
    if get_config_value("enabled", "true").lower() == "true":
        start_worker()
    app.run(host="127.0.0.1", port=port)
