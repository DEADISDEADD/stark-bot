//! Wallet Monitor module — tracks ETH wallet activity and flags large trades
//!
//! Delegates to the standalone wallet-monitor-service via RPC.
//! The service must be running separately on WALLET_MONITOR_URL (default: http://127.0.0.1:9100).

use async_trait::async_trait;
use crate::db::Database;
use crate::integrations::wallet_monitor_client::WalletMonitorClient;
use crate::tools::builtin::cryptocurrency::wallet_monitor::{
    WalletActivityTool, WalletMonitorControlTool, WalletWatchlistTool,
};
use crate::tools::registry::Tool;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct WalletMonitorModule;

impl WalletMonitorModule {
    fn make_client() -> Arc<WalletMonitorClient> {
        let url = Self::url_from_env();
        Arc::new(WalletMonitorClient::new(&url))
    }

    fn url_from_env() -> String {
        std::env::var("WALLET_MONITOR_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:9100".to_string())
    }
}

#[async_trait]
impl super::Module for WalletMonitorModule {
    fn name(&self) -> &'static str {
        "wallet_monitor"
    }

    fn description(&self) -> &'static str {
        "Monitor ETH wallets for activity and whale trades (Mainnet + Base)"
    }

    fn version(&self) -> &'static str {
        "1.1.0"
    }

    fn default_port(&self) -> u16 {
        9100
    }

    fn service_url(&self) -> String {
        Self::url_from_env()
    }

    fn has_tools(&self) -> bool {
        true
    }

    fn has_dashboard(&self) -> bool {
        true
    }

    fn create_tools(&self) -> Vec<Arc<dyn Tool>> {
        let client = Self::make_client();
        vec![
            Arc::new(WalletWatchlistTool::new(client.clone())),
            Arc::new(WalletActivityTool::new(client.clone())),
            Arc::new(WalletMonitorControlTool::new(client)),
        ]
    }

    fn skill_content(&self) -> Option<&'static str> {
        Some(include_str!("wallet_monitor.md"))
    }

    async fn dashboard_data(&self, _db: &Database) -> Option<Value> {
        let client = Self::make_client();
        let watchlist = client.list_watchlist().await.ok()?;
        let stats = client.get_activity_stats().await.ok()?;
        let filter = wallet_monitor_types::ActivityFilter {
            limit: Some(10),
            ..Default::default()
        };
        let recent = client.query_activity(&filter).await.ok()?;

        let watchlist_json: Vec<Value> = watchlist.iter().map(|w| {
            json!({
                "id": w.id,
                "address": w.address,
                "label": w.label,
                "chain": w.chain,
                "monitor_enabled": w.monitor_enabled,
                "large_trade_threshold_usd": w.large_trade_threshold_usd,
                "last_checked_at": w.last_checked_at,
            })
        }).collect();

        let recent_activity_json: Vec<Value> = recent.iter().map(|a| {
            json!({
                "chain": a.chain,
                "tx_hash": a.tx_hash,
                "activity_type": a.activity_type,
                "usd_value": a.usd_value,
                "asset_symbol": a.asset_symbol,
                "amount_formatted": a.amount_formatted,
                "is_large_trade": a.is_large_trade,
                "created_at": a.created_at,
            })
        }).collect();

        Some(json!({
            "watched_wallets": stats.watched_wallets,
            "active_wallets": stats.active_wallets,
            "total_transactions": stats.total_transactions,
            "large_trades": stats.large_trades,
            "watchlist": watchlist_json,
            "recent_activity": recent_activity_json,
        }))
    }
}
