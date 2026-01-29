---
name: bankr
description: "Interact with Bankr - check token info, wallet balances, and use the Agent API to execute prompts and transactions."
version: 1.0.0
author: starkbot
homepage: https://bankr.bot
metadata: {"requires_auth": true, "clawdbot":{"emoji":"🏦","requires":{"bins":["curl"]}}}
requires_binaries: [curl]
tags: [crypto, defi, bankr, bnkr, base, wallet, yield, token, agent]
---

# Bankr Integration

Bankr is an AI-powered crypto banking agent. This skill provides two levels of access:

1. **Public APIs** - Read-only token info, prices, balances (no API key needed)
2. **Agent API** - Execute prompts and transactions (requires API key with Agent access)

---

# Agent API (Requires API Key)

**You'll need an API key with Agent API access enabled.** Sign in, generate one, and enable agent access at: https://bankr.bot/api

## What You Can Do

With the Bankr Agent API, you can:
- Submit prompts to the Bankr AI agent for your wallet
- Check the status of submitted jobs
- Cancel pending or processing jobs

## Important Security Notes

The Agent API is powerful - it controls your Bankr wallet via API. **With great power comes great responsibility.**

**Recommended Setup:**
1. Sign up for a new Bankr account via email
2. Generate a new Bankr API key and enable agent access
3. Fund the account with limited assets
4. Ensure your API key is not publicly shared *anywhere* or with *anyone*
5. Explore the API and understand it well before increasing assets

**WARNING:** Do not share your Bankr API key with anyone or any untrusted app. If you share your API key with agent access enabled, you risk losing all your assets in that Bankr account.

If you leak your API key, visit https://bankr.bot/api and revoke it immediately.

## Authentication

All Agent API endpoints require authentication via API key:

```
X-API-Key: your_api_key_here
```

**Base URL:** `https://api.bankr.bot`

## Agent API Operations

### Submit a Prompt

Send a prompt to the Bankr AI agent for processing.

```bash
curl -X POST "https://api.bankr.bot/agent/prompt" \
  -H "X-API-Key: $BANKR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "What is the current price of ETH?"}'
```

**Response (202 Accepted):**
```json
{
  "success": true,
  "jobId": "job_abc123",
  "status": "pending",
  "message": "Job submitted successfully"
}
```

**Prompt Limits:** Max 10,000 characters

### Check Job Status

Get the current status of a submitted job.

```bash
curl "https://api.bankr.bot/agent/job/JOB_ID" \
  -H "X-API-Key: $BANKR_API_KEY"
```

**Job Statuses:**
- `pending` - Job is queued for processing
- `processing` - Job is currently being processed
- `completed` - Job finished successfully
- `failed` - Job encountered an error
- `cancelled` - Job was cancelled by user

**Completed Job Response:**
```json
{
  "success": true,
  "jobId": "job_abc123",
  "status": "completed",
  "prompt": "What is the current price of ETH?",
  "response": "The current price of ETH is $3,245.67",
  "richData": [...],
  "createdAt": "2024-01-15T10:30:00Z",
  "completedAt": "2024-01-15T10:30:05Z",
  "processingTime": 5000
}
```

### Cancel a Job

Cancel a pending or processing job.

```bash
curl -X POST "https://api.bankr.bot/agent/job/JOB_ID/cancel" \
  -H "X-API-Key: $BANKR_API_KEY"
```

**Response:**
```json
{
  "success": true,
  "jobId": "job_abc123",
  "status": "cancelled",
  "cancelledAt": "2024-01-15T10:30:02Z"
}
```

## Workflow: Submit and Poll for Result

```bash
# 1. Submit prompt
JOB_ID=$(curl -s -X POST "https://api.bankr.bot/agent/prompt" \
  -H "X-API-Key: $BANKR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Check my wallet balance"}' | jq -r '.jobId')

echo "Job ID: $JOB_ID"

# 2. Poll for completion
while true; do
  STATUS=$(curl -s "https://api.bankr.bot/agent/job/$JOB_ID" \
    -H "X-API-Key: $BANKR_API_KEY")

  JOB_STATUS=$(echo $STATUS | jq -r '.status')
  echo "Status: $JOB_STATUS"

  if [ "$JOB_STATUS" = "completed" ]; then
    echo "Response: $(echo $STATUS | jq -r '.response')"
    break
  elif [ "$JOB_STATUS" = "failed" ]; then
    echo "Error: $(echo $STATUS | jq -r '.error')"
    break
  fi

  sleep 2
done
```

## Example Prompts

- `"What is my wallet balance?"`
- `"What is the current price of ETH?"`
- `"Show me trending tokens"`
- `"Swap 0.1 ETH for USDC"`
- `"What tokens do I hold?"`

## Error Handling

| Status | Error | Meaning |
|--------|-------|---------|
| 400 | Invalid request | Bad request format |
| 400 | Prompt too long | Exceeds 10,000 chars |
| 401 | Authentication required | Missing or invalid API key |
| 403 | Agent API access not enabled | API key doesn't have agent access |
| 404 | Job not found | Invalid job ID or not your job |

---

# Public APIs (No API Key Required)

For read-only data, you can use public APIs without authentication.

## Key Info

- **BNKR Token**: `0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b` (Base)
- **Chain**: Base (chainId 8453)
- **Website**: https://bankr.bot
- **Swap**: https://swap.bankr.bot

## Token Info & Price

Get BNKR token details and current price:

```bash
# Get price from DexScreener
curl -s "https://api.dexscreener.com/latest/dex/tokens/0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b" | jq '.pairs[0] | {price: .priceUsd, priceChange24h: .priceChange.h24, volume24h: .volume.h24, liquidity: .liquidity.usd, dex: .dexId}'
```

## Check Wallet Balance

Check BNKR and ETH balance for any address on Base:

```bash
ADDRESS="0x..."

# Get ETH balance on Base
curl -s "https://api.basescan.org/api?module=account&action=balance&address=$ADDRESS&tag=latest" | jq '.result | tonumber / 1e18 | "ETH: \(.)"'

# Get BNKR token balance
curl -s "https://api.basescan.org/api?module=account&action=tokenbalance&contractaddress=0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b&address=$ADDRESS&tag=latest" | jq '.result | tonumber / 1e18 | "BNKR: \(.)"'
```

## Explore Pools & Liquidity

Find BNKR liquidity pools:

```bash
curl -s "https://api.dexscreener.com/latest/dex/tokens/0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b" | jq '.pairs[] | {pair: .pairAddress, dex: .dexId, baseToken: .baseToken.symbol, quoteToken: .quoteToken.symbol, price: .priceUsd, liquidity: .liquidity.usd}'
```

---

# About Bankr

Bankr is an AI-powered crypto banker that works on X (Twitter) and Farcaster. Key features:

- **Trading**: Swap tokens, trade perps, prediction markets
- **Advanced Orders**: Limit, stop loss, trailing stop, TWAP, DCA
- **Bankr Earn**: Auto-optimizes USDC yield across chains
- **NFTs**: Mint and manage NFTs via natural language

### Tokenomics
- 90% of platform revenue goes to BNKR stakers and LP providers
- Fixed 100B supply, ownership-renounced contract
- Available on Aerodrome, Uniswap (Base), and CEXs (MEXC, BingX, Gate.io)

### Supported Chains
- Base (primary)
- Ethereum
- Polygon
- Solana

---

# Resources

- **API Dashboard:** https://bankr.bot/api
- **Example Apps:** https://github.com/BankrBot/bankr-api-examples
- **Swap UI:** https://swap.bankr.bot
- **Twitter:** https://x.com/bankrbot
- **Token:** https://basescan.org/token/0x22aF33FE49fD1Fa80c7149773dDe5890D3c76F3b

---

# Best Practices

1. **Start with limited funds** - Test with small amounts first
2. **Never share your API key** - Treat it like a password
3. **Poll responsibly** - Use 2-second intervals, don't spam
4. **Handle all statuses** - Check for failed/cancelled, not just completed
5. **Check richData** - Contains valuable structured information
6. **Set timeouts** - Don't poll forever, implement max attempts
7. **Revoke compromised keys immediately** - If leaked, revoke at https://bankr.bot/api
