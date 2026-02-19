# StarkBot Roadmap

## Historical Roadmap (Jan 25 – Feb 18, 2026)

**435 commits in 25 days.** From `init repo` to a multi-agent system with plugins, trading, social gateways, and a knowledge graph.

| # | Date | Milestone | What Shipped |
|---|------|-----------|--------------|
| 1 | Jan 25 | **Project Genesis** | Repo init, basic agent loop, dashboard, gateway system |
| 2 | Jan 25 | **Skills & Tools Framework** | Session management, pluggable tools, skill loading from markdown |
| 3 | Jan 25 | **Scheduler & Heartbeat** | Cron system, periodic heartbeat for autonomous behavior |
| 4 | Jan 28 | **SIWE Auth & x402 Payments** | Sign-In with Ethereum, x402 protocol for paid API calls |
| 5 | Jan 28 | **Multi-Agent Architecture** | Agent-to-agent delegation, role separation |
| 6 | Jan 29 | **Intelligence Upgrade & Safety Rails** | Major prompt engineering pass, tool safety guardrails |
| 7 | Jan 30 | **GitHub & Code Skills** | GitHub integrations, code reading/writing/review skills |
| 8 | Jan 31 | **Task System & x402 Tool** | Persistent task management, x402 as a callable agent tool |
| 9 | Feb 1 | **Encrypted Keyring** | Encrypted API key storage, first steps toward rogue agent mode |
| 10 | Feb 2 | **Discord Gateway & Tipping** | Full Discord bot integration, on-chain tipping between users |
| 11 | Feb 3 | **Polymarket Trading** | CLOB client SDK, market trading tool, position management |
| 12 | Feb 3 | **Mind Map & Memory v1** | Knowledge graph prototype, memory compaction system |
| 13 | Feb 4 | **Flash Mode & WalletProvider** | Privy wallet abstraction, WalletProvider trait, Flash control plane |
| 14 | Feb 4 | **Cloud Backup & Keystore** | Encrypted cloud backup/restore, channel settings backup, keystore API |
| 15 | Feb 6 | **X (Twitter) & Telegram Gateways** | Twitter posting/reading, Telegram webhook integration |
| 16 | Feb 11 | **CodeEngineer Boost & Tool Safety** | 6 coding performance improvements, ToolSafetyLevel enum for permission tiers |
| 17 | Feb 12 | **StarkHub Marketplace** | SIWA-authenticated skill uploads, browsable marketplace, one-click install |
| 18 | Feb 14 | **Async Module System & Plugins** | Async Module trait, installable plugin modules, wallet monitor, Slack gateway |
| 19 | Feb 15 | **Subagents & Tool Loop Speedup** | Recursive subagents, director pattern, massive tool loop speed optimization |
| 20 | Feb 18 | **Memory System v2 & Spacebot Merge** | Vector embeddings, hybrid search (RRF), association graph, memory decay, D3.js memory graph UI, impulse map, unified inference router |

---

## Future Roadmap

### Near-Term — Stabilize & Deepen

| Feature | Status | Description |
|---------|--------|-------------|
| **Whisper Voice Integration** | Implemented, polish needed | Frontend audio recording to backend transcription via self-hosted ONNX whisper server. Next: streaming transcription, wake-word detection, voice-to-action pipeline |
| **Memory Graph v2** | Implemented, hardening | D3.js force-directed graph, 7 association types, background auto-discovery loop (every 5 min). Next: scale testing, graph pruning strategies, community-level shared memory |
| **Impulse Evolver** | Implemented | Knowledge graph of goals/ideas/projects with heartbeat-driven autonomous reflection. Next: priority scoring, goal decomposition into tasks, cross-session planning persistence |
| **Embeddings at Scale** | In progress | Remote ONNX embedding server, cosine similarity search. Next: ANN indexing (HNSW), batch backfill optimization, model hot-swap |

### Mid-Term — Agent Autonomy

| Feature | Description |
|---------|-------------|
| **Branch Mode** | Lightweight context forks — agent explores multiple solution paths in parallel, merges the best one back |
| **Message Coalescing** | Debounce + max-wait timers to batch rapid-fire messages into coherent chunks before processing |
| **Three-Tier Context Compaction** | Background / aggressive / emergency compaction levels to handle long-running sessions without context loss |
| **Worker Checkpoints** | 25-turn segments with overflow recovery — enables resumable long tasks across crashes |
| **Cortex Bulletin System** | Cross-channel awareness — agent knowledge discovered in Discord surfaces in Telegram and vice versa |
| **Model Failover Chains** | Multi-model routing with automatic failover, auth profile rotation, cost-optimized model selection |

### Long-Term — Platform Evolution

| Feature | Description |
|---------|-------------|
| **Plugin SDK with Lifecycle Hooks** | Full SDK for third-party module developers — init, shutdown, config, and event hooks |
| **Structured Observability** | Prometheus metrics, distributed tracing, `starkbot doctor` CLI for self-diagnosis |
| **Progressive Streaming UX** | Block-chunked streaming with partial render — tools, code, and text stream independently |
| **SSE Real-Time Dashboard** | Server-sent events replacing polling — live agent status, memory updates, task progress |
| **Unified YAML Configuration** | Single config file with hot-reload, replacing scattered env vars and DB settings |
| **Agent-to-Agent Marketplace** | Agents discover and hire other agents via StarkHub for specialized tasks (trading, research, coding) |

---

### The Arc

**Weeks 1-2** were about building the skeleton: agent loop, tools, skills, multi-agent, gateways.

**Weeks 2-3** were about making it real: wallets, trading, social platforms, marketplace, plugins.

**Week 4** (now) is about making it *think*: memory graphs, embeddings, hybrid search, impulse evolution, autonomous reflection.

**What's next** is making it *persistent and reliable*: branch mode, checkpoints, compaction, failover — so the agent can run indefinitely, across channels, without losing its mind.
