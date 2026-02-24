//! sign_raw_tx — sign an EIP-1559 transaction without broadcasting.
//!
//! Takes raw transaction fields (to, data, value, gas, etc.), signs with the
//! configured wallet provider, and returns the signed tx hex + tx hash.
//!
//! This tool has **no tool group** — it is only available to agent personas
//! that explicitly list it in `additional_tools`.

use crate::tools::registry::Tool;
use crate::tools::rpc_config::resolve_rpc_from_context;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
    ToolSafetyLevel,
};
use crate::x402::X402EvmRpc;
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers::types::transaction::eip2718::TypedTransaction;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
pub struct SignRawTxTool {
    definition: ToolDefinition,
}

impl SignRawTxTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "to".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Destination address (0x-prefixed)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "data".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Calldata hex (0x-prefixed)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "value".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Wei value to send, default \"0\"".to_string(),
                default: Some(json!("0")),
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "chain_id".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Chain ID, default 8453 (Base)".to_string(),
                default: Some(json!(8453)),
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "gas".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Gas limit. If omitted, estimated from RPC.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "max_fee_per_gas".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Max fee per gas (wei). If omitted, estimated from RPC.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "max_priority_fee_per_gas".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Max priority fee per gas (wei). If omitted, estimated from RPC."
                    .to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "nonce".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Nonce. If omitted, fetched from RPC.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        SignRawTxTool {
            definition: ToolDefinition {
                name: "sign_raw_tx".to_string(),
                description: "Sign an EIP-1559 transaction without broadcasting. Returns the signed transaction hex and tx hash. Only signs — does NOT send to the network.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["to".to_string(), "data".to_string()],
                },
                group: ToolGroup::Finance,
                hidden: true,
            },
        }
    }

    fn network_for_chain_id(chain_id: u64) -> &'static str {
        match chain_id {
            1 => "mainnet",
            137 => "polygon",
            42161 => "arbitrum",
            10 => "optimism",
            _ => "base",
        }
    }
}

impl Default for SignRawTxTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SignRawTxParams {
    to: String,
    data: String,
    #[serde(default = "default_value")]
    value: String,
    #[serde(default = "default_chain_id")]
    chain_id: u64,
    gas: Option<String>,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
    nonce: Option<u64>,
}

fn default_value() -> String {
    "0".to_string()
}
fn default_chain_id() -> u64 {
    8453
}

#[async_trait]
impl Tool for SignRawTxTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    fn safety_level(&self) -> ToolSafetyLevel {
        ToolSafetyLevel::Standard
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let p: SignRawTxParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Invalid parameters: {}", e)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        // Require wallet provider
        let wallet_provider = match &context.wallet_provider {
            Some(wp) => wp.clone(),
            None => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some("No wallet provider configured".to_string()),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        let network = Self::network_for_chain_id(p.chain_id);
        let rpc_config = resolve_rpc_from_context(&context.extra, network);

        // Create RPC client for nonce/gas estimation
        let rpc = match X402EvmRpc::new_with_wallet_provider(
            wallet_provider.clone(),
            network,
            Some(rpc_config.url.clone()),
            rpc_config.use_x402,
        ) {
            Ok(r) => r,
            Err(e) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Failed to create RPC client: {}", e)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        // Parse addresses
        let from_str = wallet_provider.get_address();
        let from_address: Address = match from_str.parse() {
            Ok(a) => a,
            Err(_) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Invalid wallet address: {}", from_str)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };
        let to_address: Address = match p.to.parse() {
            Ok(a) => a,
            Err(_) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Invalid 'to' address: {}", p.to)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        // Parse value
        let tx_value: U256 = match p.value.parse() {
            Ok(v) => v,
            Err(_) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Invalid value: {}", p.value)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        // Parse calldata
        let calldata: ethers::types::Bytes = match hex::decode(p.data.trim_start_matches("0x")) {
            Ok(d) => d.into(),
            Err(e) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Invalid calldata hex: {}", e)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        // Fetch nonce if not provided
        let nonce = match p.nonce {
            Some(n) => U256::from(n),
            None => match rpc.get_transaction_count(from_address).await {
                Ok(n) => n,
                Err(e) => {
                    return ToolResult {
                        success: false,
                        content: String::new(),
                        error: Some(format!("Failed to fetch nonce: {}", e)),
                        metadata: None,
                        retry_after_secs: None,
                    }
                }
            },
        };

        // Fetch gas prices if not provided
        let (max_fee, priority_fee) = match (&p.max_fee_per_gas, &p.max_priority_fee_per_gas) {
            (Some(mf), Some(pf)) => {
                let mf: U256 = mf.parse().unwrap_or(U256::from(1_000_000_000u64));
                let pf: U256 = pf.parse().unwrap_or(U256::from(100_000_000u64));
                (mf, pf)
            }
            _ => match rpc.estimate_eip1559_fees().await {
                Ok(fees) => fees,
                Err(e) => {
                    return ToolResult {
                        success: false,
                        content: String::new(),
                        error: Some(format!("Failed to estimate gas fees: {}", e)),
                        metadata: None,
                        retry_after_secs: None,
                    }
                }
            },
        };

        // Estimate gas if not provided
        let gas = match &p.gas {
            Some(g) => g.parse().unwrap_or(U256::from(250_000u64)),
            None => {
                // Default reasonable gas limit for swaps
                U256::from(350_000u64)
            }
        };

        log::info!(
            "[sign_raw_tx] Signing tx: to={}, value={}, gas={}, nonce={}, chain={}",
            p.to, p.value, gas, nonce, p.chain_id
        );

        // Build EIP-1559 transaction
        let tx_req = Eip1559TransactionRequest::new()
            .from(from_address)
            .to(to_address)
            .value(tx_value)
            .nonce(nonce)
            .gas(gas)
            .max_fee_per_gas(max_fee)
            .max_priority_fee_per_gas(priority_fee)
            .chain_id(p.chain_id)
            .data(calldata);

        // Sign the transaction
        let typed_tx: TypedTransaction = tx_req.into();
        let signature = match wallet_provider.sign_transaction(&typed_tx).await {
            Ok(sig) => sig,
            Err(e) => {
                return ToolResult {
                    success: false,
                    content: String::new(),
                    error: Some(format!("Failed to sign transaction: {}", e)),
                    metadata: None,
                    retry_after_secs: None,
                }
            }
        };

        // RLP-encode the signed transaction
        let signed_tx = typed_tx.rlp_signed(&signature);
        let signed_tx_hex = format!("0x{}", hex::encode(&signed_tx));

        // Compute tx hash
        let tx_hash = format!("0x{}", hex::encode(ethers::utils::keccak256(&signed_tx)));

        log::info!(
            "[sign_raw_tx] Signed tx hash={}, nonce={}",
            tx_hash, nonce
        );

        ToolResult {
            success: true,
            content: json!({
                "signed_tx": signed_tx_hex,
                "tx_hash": tx_hash,
                "from": from_str,
                "to": p.to,
                "nonce": nonce.as_u64(),
                "chain_id": p.chain_id,
            })
            .to_string(),
            error: None,
            metadata: None,
            retry_after_secs: None,
        }
    }
}
