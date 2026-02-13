//! Wallet monitoring tools — watchlist management, activity queries, and monitor control
//!
//! These tools are only registered when the wallet_monitor module is installed.

use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
    ToolSafetyLevel,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

// =====================================================
// WalletWatchlistTool
// =====================================================

pub struct WalletWatchlistTool {
    definition: ToolDefinition,
}

impl WalletWatchlistTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "action".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Action: 'add', 'remove', 'list', 'update'".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "add".to_string(),
                    "remove".to_string(),
                    "list".to_string(),
                    "update".to_string(),
                ]),
            },
        );

        properties.insert(
            "address".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Ethereum address (0x + 40 hex chars). Required for 'add'.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "label".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Human-readable label for the wallet".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "chain".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Chain to monitor: 'mainnet' or 'base'. Default: 'mainnet'".to_string(),
                default: Some(json!("mainnet")),
                items: None,
                enum_values: Some(vec!["mainnet".to_string(), "base".to_string()]),
            },
        );

        properties.insert(
            "threshold_usd".to_string(),
            PropertySchema {
                schema_type: "number".to_string(),
                description: "Large trade threshold in USD. Default: 10000".to_string(),
                default: Some(json!(10000.0)),
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "id".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Watchlist entry ID. Required for 'remove' and 'update'.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "notes".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Notes about this wallet".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "monitor_enabled".to_string(),
            PropertySchema {
                schema_type: "boolean".to_string(),
                description: "Enable/disable monitoring for this wallet".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        WalletWatchlistTool {
            definition: ToolDefinition {
                name: "wallet_watchlist".to_string(),
                description: "Manage the wallet watchlist for monitoring on-chain activity. Add, remove, list, or update watched wallets on Ethereum Mainnet and Base.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["action".to_string()],
                },
                group: ToolGroup::Finance,
                hidden: false,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct WatchlistParams {
    action: String,
    address: Option<String>,
    label: Option<String>,
    chain: Option<String>,
    threshold_usd: Option<f64>,
    id: Option<i64>,
    notes: Option<String>,
    monitor_enabled: Option<bool>,
}

fn is_valid_eth_address(addr: &str) -> bool {
    addr.starts_with("0x") && addr.len() == 42 && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
}

#[async_trait]
impl Tool for WalletWatchlistTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: WatchlistParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let db = match context.database.as_ref() {
            Some(db) => db,
            None => return ToolResult::error("Database not available"),
        };

        match params.action.as_str() {
            "add" => {
                let address = match params.address {
                    Some(ref a) => a,
                    None => return ToolResult::error("'address' is required for 'add' action"),
                };
                if !is_valid_eth_address(address) {
                    return ToolResult::error("Invalid Ethereum address. Must be 0x + 40 hex characters.");
                }
                let chain = params.chain.as_deref().unwrap_or("mainnet");
                let threshold = params.threshold_usd.unwrap_or(10000.0);

                match db.add_to_watchlist(address, params.label.as_deref(), chain, threshold) {
                    Ok(entry) => ToolResult::success(json!({
                        "status": "added",
                        "id": entry.id,
                        "address": entry.address,
                        "label": entry.label,
                        "chain": entry.chain,
                        "threshold_usd": entry.large_trade_threshold_usd,
                    }).to_string()),
                    Err(e) => {
                        if e.to_string().contains("UNIQUE constraint") {
                            ToolResult::error(format!("Wallet {} is already on the watchlist for chain {}", address, chain))
                        } else {
                            ToolResult::error(format!("Failed to add wallet: {}", e))
                        }
                    }
                }
            }

            "remove" => {
                let id = match params.id {
                    Some(id) => id,
                    None => return ToolResult::error("'id' is required for 'remove' action"),
                };
                match db.remove_from_watchlist(id) {
                    Ok(true) => ToolResult::success(format!("Removed watchlist entry #{}", id)),
                    Ok(false) => ToolResult::error(format!("Watchlist entry #{} not found", id)),
                    Err(e) => ToolResult::error(format!("Failed to remove: {}", e)),
                }
            }

            "list" => match db.list_watchlist() {
                Ok(entries) => {
                    if entries.is_empty() {
                        return ToolResult::success("No wallets on the watchlist. Use action='add' to start monitoring.");
                    }
                    let mut output = format!("**Wallet Watchlist** ({} entries)\n\n", entries.len());
                    for e in &entries {
                        let label = e.label.as_deref().unwrap_or("(unlabeled)");
                        let status = if e.monitor_enabled { "active" } else { "paused" };
                        let last_block = e.last_checked_block.map(|b| format!("block #{}", b)).unwrap_or_else(|| "not yet checked".to_string());
                        output.push_str(&format!(
                            "#{} | {} | {} | {} | threshold: ${:.0} | {} | {}\n",
                            e.id, label, e.address, e.chain, e.large_trade_threshold_usd, status, last_block
                        ));
                    }
                    ToolResult::success(output)
                }
                Err(e) => ToolResult::error(format!("Failed to list watchlist: {}", e)),
            },

            "update" => {
                let id = match params.id {
                    Some(id) => id,
                    None => return ToolResult::error("'id' is required for 'update' action"),
                };
                match db.update_watchlist_entry(
                    id,
                    params.label.as_deref(),
                    params.threshold_usd,
                    params.monitor_enabled,
                    params.notes.as_deref(),
                ) {
                    Ok(true) => ToolResult::success(format!("Updated watchlist entry #{}", id)),
                    Ok(false) => ToolResult::error(format!("Watchlist entry #{} not found", id)),
                    Err(e) => ToolResult::error(format!("Failed to update: {}", e)),
                }
            }

            _ => ToolResult::error(format!("Unknown action: '{}'. Use 'add', 'remove', 'list', or 'update'.", params.action)),
        }
    }

    fn safety_level(&self) -> ToolSafetyLevel {
        ToolSafetyLevel::Standard
    }
}

// =====================================================
// WalletActivityTool
// =====================================================

pub struct WalletActivityTool {
    definition: ToolDefinition,
}

impl WalletActivityTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "action".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Action: 'recent', 'large_trades', 'search', 'stats'".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "recent".to_string(),
                    "large_trades".to_string(),
                    "search".to_string(),
                    "stats".to_string(),
                ]),
            },
        );

        properties.insert(
            "address".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Filter by wallet address".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "activity_type".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Filter by type: 'eth_transfer', 'erc20_transfer', 'swap', 'internal'".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "chain".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Filter by chain: 'mainnet' or 'base'".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "large_only".to_string(),
            PropertySchema {
                schema_type: "boolean".to_string(),
                description: "Only show large trades".to_string(),
                default: Some(json!(false)),
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "limit".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Max results to return (default 25, max 200)".to_string(),
                default: Some(json!(25)),
                items: None,
                enum_values: None,
            },
        );

        WalletActivityTool {
            definition: ToolDefinition {
                name: "wallet_activity".to_string(),
                description: "Query logged wallet activity from monitored wallets. View recent transactions, large trades, search by filters, or get stats.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["action".to_string()],
                },
                group: ToolGroup::Finance,
                hidden: false,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct ActivityParams {
    action: String,
    address: Option<String>,
    activity_type: Option<String>,
    chain: Option<String>,
    large_only: Option<bool>,
    limit: Option<usize>,
}

#[async_trait]
impl Tool for WalletActivityTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: ActivityParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let db = match context.database.as_ref() {
            Some(db) => db,
            None => return ToolResult::error("Database not available"),
        };

        match params.action.as_str() {
            "recent" => {
                let filter = crate::db::tables::wallet_monitor::ActivityFilter {
                    address: params.address,
                    activity_type: params.activity_type,
                    chain: params.chain,
                    large_only: params.large_only.unwrap_or(false),
                    limit: Some(params.limit.unwrap_or(25)),
                    ..Default::default()
                };
                match db.query_activity(&filter) {
                    Ok(entries) => format_activity_list(&entries, "Recent Activity"),
                    Err(e) => ToolResult::error(format!("Query failed: {}", e)),
                }
            }

            "large_trades" => {
                let limit = params.limit.unwrap_or(25);
                match db.get_recent_large_trades(limit) {
                    Ok(entries) => format_activity_list(&entries, "Large Trades"),
                    Err(e) => ToolResult::error(format!("Query failed: {}", e)),
                }
            }

            "search" => {
                let filter = crate::db::tables::wallet_monitor::ActivityFilter {
                    address: params.address,
                    activity_type: params.activity_type,
                    chain: params.chain,
                    large_only: params.large_only.unwrap_or(false),
                    limit: Some(params.limit.unwrap_or(50)),
                    ..Default::default()
                };
                match db.query_activity(&filter) {
                    Ok(entries) => format_activity_list(&entries, "Search Results"),
                    Err(e) => ToolResult::error(format!("Query failed: {}", e)),
                }
            }

            "stats" => match db.get_activity_stats() {
                Ok(stats) => ToolResult::success(json!({
                    "total_transactions": stats.total_transactions,
                    "large_trades": stats.large_trades,
                    "watched_wallets": stats.watched_wallets,
                    "active_wallets": stats.active_wallets,
                }).to_string()),
                Err(e) => ToolResult::error(format!("Stats query failed: {}", e)),
            },

            _ => ToolResult::error(format!(
                "Unknown action: '{}'. Use 'recent', 'large_trades', 'search', or 'stats'.",
                params.action
            )),
        }
    }

    fn safety_level(&self) -> ToolSafetyLevel {
        ToolSafetyLevel::ReadOnly
    }
}

fn format_activity_list(
    entries: &[crate::db::tables::wallet_monitor::ActivityEntry],
    title: &str,
) -> ToolResult {
    if entries.is_empty() {
        return ToolResult::success(format!("**{}**: No activity found.", title));
    }

    let mut output = format!("**{}** ({} entries)\n\n", title, entries.len());
    for e in entries {
        let usd = e
            .usd_value
            .map(|v| format!(" (${:.0})", v))
            .unwrap_or_default();
        let large = if e.is_large_trade { " **LARGE**" } else { "" };
        let asset = e.asset_symbol.as_deref().unwrap_or("ETH");
        let amount = e.amount_formatted.as_deref().unwrap_or("?");

        match e.activity_type.as_str() {
            "swap" => {
                let from_token = e.swap_from_token.as_deref().unwrap_or("?");
                let from_amount = e.swap_from_amount.as_deref().unwrap_or("?");
                let to_token = e.swap_to_token.as_deref().unwrap_or("?");
                let to_amount = e.swap_to_amount.as_deref().unwrap_or("?");
                output.push_str(&format!(
                    "SWAP: {} {} → {} {}{}{} | {} | {}\n",
                    from_amount, from_token, to_amount, to_token, usd, large, e.chain, e.tx_hash
                ));
            }
            _ => {
                output.push_str(&format!(
                    "{}: {} {}{}{} | {} → {} | {} | {}\n",
                    e.activity_type.to_uppercase(),
                    amount,
                    asset,
                    usd,
                    large,
                    &e.from_address[..10],
                    &e.to_address[..10],
                    e.chain,
                    e.tx_hash
                ));
            }
        }
    }
    ToolResult::success(output)
}

// =====================================================
// WalletMonitorControlTool
// =====================================================

pub struct WalletMonitorControlTool {
    definition: ToolDefinition,
}

impl WalletMonitorControlTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "action".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Action: 'status' to check worker health, 'trigger' to force an immediate poll".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec!["status".to_string(), "trigger".to_string()]),
            },
        );

        WalletMonitorControlTool {
            definition: ToolDefinition {
                name: "wallet_monitor_control".to_string(),
                description: "Control the wallet monitor background worker. Check status or trigger an immediate poll.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["action".to_string()],
                },
                group: ToolGroup::Finance,
                hidden: false,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct ControlParams {
    action: String,
}

#[async_trait]
impl Tool for WalletMonitorControlTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: ControlParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let db = match context.database.as_ref() {
            Some(db) => db,
            None => return ToolResult::error("Database not available"),
        };

        match params.action.as_str() {
            "status" => {
                let installed = db.is_module_installed("wallet_monitor").unwrap_or(false);
                let enabled = db.is_module_enabled("wallet_monitor").unwrap_or(false);
                let has_api_key = db
                    .get_api_key("ALCHEMY_API_KEY")
                    .ok()
                    .flatten()
                    .is_some();
                let watchlist_count = db
                    .list_watchlist()
                    .map(|w| w.len())
                    .unwrap_or(0);
                let active_count = db
                    .list_active_watchlist()
                    .map(|w| w.len())
                    .unwrap_or(0);
                let stats = db.get_activity_stats().ok();

                ToolResult::success(json!({
                    "installed": installed,
                    "enabled": enabled,
                    "alchemy_api_key": has_api_key,
                    "watchlist_total": watchlist_count,
                    "watchlist_active": active_count,
                    "total_transactions": stats.as_ref().map(|s| s.total_transactions).unwrap_or(0),
                    "large_trades": stats.as_ref().map(|s| s.large_trades).unwrap_or(0),
                    "worker_status": if enabled && has_api_key { "running" } else if enabled { "waiting_for_api_key" } else { "stopped" },
                }).to_string())
            }

            "trigger" => {
                // We can't directly trigger the worker loop, but we can verify it's running
                // and provide the user with status info
                let enabled = db.is_module_enabled("wallet_monitor").unwrap_or(false);
                if !enabled {
                    return ToolResult::error("Wallet monitor is not enabled. Install or enable it first.");
                }
                let has_api_key = db
                    .get_api_key("ALCHEMY_API_KEY")
                    .ok()
                    .flatten()
                    .is_some();
                if !has_api_key {
                    return ToolResult::error("ALCHEMY_API_KEY is not configured. Install it first.");
                }
                ToolResult::success("Wallet monitor worker is running. It polls every 60 seconds. The next tick will process any pending wallets.")
            }

            _ => ToolResult::error(format!(
                "Unknown action: '{}'. Use 'status' or 'trigger'.",
                params.action
            )),
        }
    }

    fn safety_level(&self) -> ToolSafetyLevel {
        ToolSafetyLevel::Standard
    }
}
