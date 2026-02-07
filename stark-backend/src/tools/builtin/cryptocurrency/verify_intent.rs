//! Transaction intent verification — safety layer before tx queueing
//!
//! Every transaction-creating tool (send_eth, web3_function_call,
//! web3_preset_function_call, bridge_usdc) calls `verify_intent()` BEFORE
//! `tx_queue.queue()`. The check is embedded inside each tool so the AI
//! agent cannot skip it.
//!
//! ## Steps
//! 1. Read `original_user_message` from `context.extra`
//! 2. Run deterministic checks (fast, no network)
//! 3. Run isolated AI verification call
//! 4. Return `Ok(())` or `Err(reason)`

use crate::ai::{AiClient, Message, MessageRole};
use crate::tools::types::ToolContext;
use serde_json::Value;

/// Describes the transaction about to be queued.
#[derive(Debug, Clone)]
pub struct TransactionIntent {
    pub tx_type: String,
    pub to: String,
    pub value: String,
    pub value_display: String,
    pub network: String,
    pub function_name: Option<String>,
    pub abi_name: Option<String>,
    pub preset_name: Option<String>,
    pub destination_chain: Option<String>,
    pub calldata: Option<String>,
    pub description: String,
}

// ─── Public entry point ──────────────────────────────────────────────────────

/// Verify that a transaction intent matches the user's original request.
///
/// `ai_override` — pass a pre-built client in tests to skip the DB lookup.
pub async fn verify_intent(
    intent: &TransactionIntent,
    context: &ToolContext,
    ai_override: Option<&AiClient>,
) -> Result<(), String> {
    // 1. Run deterministic checks first (cheap, no network)
    run_deterministic_checks(intent, context)?;

    // 2. Read original user message
    let user_message = context
        .extra
        .get("original_user_message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if user_message.is_empty() {
        log::warn!("[verify_intent] No original_user_message in context — skipping AI check");
        // Still pass; deterministic checks already ran.
        return Ok(());
    }

    // 3. Obtain an AI client
    let owned_client: Option<AiClient>;
    let client: &AiClient = match ai_override {
        Some(c) => c,
        None => {
            owned_client = build_client_from_db(context);
            match owned_client.as_ref() {
                Some(c) => c,
                None => {
                    log::warn!("[verify_intent] Could not build AI client — skipping AI check");
                    return Ok(());
                }
            }
        }
    };

    // 4. Run AI verification
    let prompt = format_verification_prompt(intent, &user_message);
    let messages = vec![
        Message {
            role: MessageRole::System,
            content: VERIFICATION_SYSTEM_PROMPT.to_string(),
        },
        Message {
            role: MessageRole::User,
            content: prompt,
        },
    ];

    let ai_response = client.generate_text(messages).await;

    match ai_response {
        Ok(text) => parse_verification_response(&text),
        Err(e) => {
            // Fail-closed: AI error blocks the transaction
            Err(format!(
                "Intent verification failed (AI error): {}. Transaction blocked for safety.",
                e
            ))
        }
    }
}

// ─── Deterministic checks ────────────────────────────────────────────────────

/// Fast, offline checks that catch obvious problems.
fn run_deterministic_checks(
    intent: &TransactionIntent,
    context: &ToolContext,
) -> Result<(), String> {
    let to_lower = intent.to.to_lowercase();

    // 1. Zero-address recipient
    if to_lower == "0x0000000000000000000000000000000000000000" {
        return Err(
            "Transaction blocked: recipient is the zero address (0x0000...0000). \
             Sending to the zero address will burn funds permanently."
                .to_string(),
        );
    }

    // 2. Self-send detection (only for plain ETH transfers)
    if intent.tx_type == "eth_transfer" {
        if let Some(wallet_addr) = context.registers.get("wallet_address") {
            if let Some(addr_str) = wallet_addr.as_str() {
                if addr_str.to_lowercase() == to_lower {
                    return Err(
                        "Transaction blocked: you are sending ETH to your own wallet. \
                         This wastes gas with no effect. Please verify the recipient."
                            .to_string(),
                    );
                }
            }
        }
    }

    // 3. Recipient address should appear in registers or context bank
    //    (anti-hallucination check)
    if intent.tx_type == "eth_transfer" {
        let address_in_registers = address_exists_in_registers(&to_lower, context);
        let address_in_context_bank = address_exists_in_context_bank(&to_lower, context);

        if !address_in_registers && !address_in_context_bank {
            return Err(format!(
                "Transaction blocked: recipient address {} was not found in any register \
                 or in the context bank. This may indicate a hallucinated address. \
                 Use register_set to store the address first.",
                intent.to
            ));
        }
    }

    Ok(())
}

/// Check whether `addr` (lowercase) appears as a value in any register.
fn address_exists_in_registers(addr: &str, context: &ToolContext) -> bool {
    for key in context.registers.keys() {
        if let Some(val) = context.registers.get(&key) {
            if let Some(s) = val.as_str() {
                if s.to_lowercase() == addr {
                    return true;
                }
            }
        }
    }
    false
}

/// Check whether `addr` (lowercase) appears in the context bank's eth_address items.
fn address_exists_in_context_bank(addr: &str, context: &ToolContext) -> bool {
    for item in context.context_bank.items() {
        if item.item_type == "eth_address" && item.value.to_lowercase() == addr {
            return true;
        }
    }
    false
}

// ─── AI verification ─────────────────────────────────────────────────────────

const VERIFICATION_SYSTEM_PROMPT: &str = "\
You are a transaction safety verifier. Your job is to compare a user's original request \
with the transaction that was constructed, and determine whether they match.

Respond with EXACTLY one of these formats (no extra text):
  APPROVED
  REJECTED: <one-line reason>
  NEED_INFO: <what is missing>

Rules:
- APPROVED means the transaction clearly matches what the user asked for.
- REJECTED means there is a mismatch in recipient, amount, network, or operation type.
- NEED_INFO means the user's request is too vague to confirm the transaction.
- When in doubt, use REJECTED. It is always safer to block than to allow.
- Do NOT add any explanation beyond the single-line reason.";

fn format_verification_prompt(intent: &TransactionIntent, user_message: &str) -> String {
    let mut prompt = String::new();
    prompt.push_str("## User's original message\n");
    prompt.push_str(user_message);
    prompt.push_str("\n\n## Constructed transaction\n");
    prompt.push_str(&format!("Type: {}\n", intent.tx_type));
    prompt.push_str(&format!("To: {}\n", intent.to));
    prompt.push_str(&format!("Value: {} ({})\n", intent.value, intent.value_display));
    prompt.push_str(&format!("Network: {}\n", intent.network));

    if let Some(ref name) = intent.function_name {
        prompt.push_str(&format!("Function: {}\n", name));
    }
    if let Some(ref abi) = intent.abi_name {
        prompt.push_str(&format!("ABI: {}\n", abi));
    }
    if let Some(ref preset) = intent.preset_name {
        prompt.push_str(&format!("Preset: {}\n", preset));
    }
    if let Some(ref dest) = intent.destination_chain {
        prompt.push_str(&format!("Destination chain: {}\n", dest));
    }

    prompt.push_str(&format!("\nDescription: {}\n", intent.description));
    prompt.push_str("\nDoes this transaction match the user's request?");
    prompt
}

/// Parse the AI verifier response.  Fail-closed: anything unparseable is a rejection.
fn parse_verification_response(response: &str) -> Result<(), String> {
    let trimmed = response.trim();

    if trimmed == "APPROVED" {
        log::info!("[verify_intent] APPROVED");
        return Ok(());
    }

    if let Some(reason) = trimmed.strip_prefix("REJECTED:") {
        let reason = reason.trim();
        log::warn!("[verify_intent] REJECTED: {}", reason);
        return Err(format!(
            "Transaction rejected by safety verifier: {}",
            reason
        ));
    }

    if let Some(info) = trimmed.strip_prefix("NEED_INFO:") {
        let info = info.trim();
        log::warn!("[verify_intent] NEED_INFO: {}", info);
        return Err(format!(
            "Transaction blocked — more information needed: {}",
            info
        ));
    }

    // Unparseable = fail-closed
    log::warn!(
        "[verify_intent] Unparseable response (blocked): {}",
        trimmed
    );
    Err(format!(
        "Transaction blocked: safety verifier returned an unexpected response. \
         Please try again or rephrase your request."
    ))
}

// ─── DB helper ───────────────────────────────────────────────────────────────

/// Build an AiClient from DB settings (same pattern as save_session_memory).
fn build_client_from_db(context: &ToolContext) -> Option<AiClient> {
    let db = context.database.as_ref()?;
    let settings = db.get_active_agent_settings().ok()??;
    AiClient::from_settings(&settings).ok()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::context_bank::ContextBankItem;
    use crate::tools::RegisterStore;

    // ── parse_verification_response ──────────────────────────────────

    #[test]
    fn test_parse_approved() {
        assert!(parse_verification_response("APPROVED").is_ok());
        assert!(parse_verification_response("  APPROVED  ").is_ok());
    }

    #[test]
    fn test_parse_rejected() {
        let err = parse_verification_response("REJECTED: wrong recipient").unwrap_err();
        assert!(err.contains("wrong recipient"), "got: {}", err);
    }

    #[test]
    fn test_parse_need_info() {
        let err = parse_verification_response("NEED_INFO: which address?").unwrap_err();
        assert!(err.contains("which address?"), "got: {}", err);
    }

    #[test]
    fn test_parse_garbage_fails_closed() {
        let err = parse_verification_response("sure thing buddy").unwrap_err();
        assert!(err.contains("unexpected response"), "got: {}", err);
    }

    #[test]
    fn test_parse_empty_fails_closed() {
        let err = parse_verification_response("").unwrap_err();
        assert!(err.contains("unexpected response"), "got: {}", err);
    }

    // ── deterministic checks ─────────────────────────────────────────

    fn make_intent(tx_type: &str, to: &str) -> TransactionIntent {
        TransactionIntent {
            tx_type: tx_type.to_string(),
            to: to.to_string(),
            value: "1000000000000000".to_string(),
            value_display: "0.001 ETH".to_string(),
            network: "base".to_string(),
            function_name: None,
            abi_name: None,
            preset_name: None,
            destination_chain: None,
            calldata: None,
            description: "test tx".to_string(),
        }
    }

    #[test]
    fn test_zero_address_blocked() {
        let intent = make_intent(
            "eth_transfer",
            "0x0000000000000000000000000000000000000000",
        );
        let ctx = ToolContext::new();
        let err = run_deterministic_checks(&intent, &ctx).unwrap_err();
        assert!(err.contains("zero address"), "got: {}", err);
    }

    #[test]
    fn test_self_send_blocked() {
        let wallet = "0xAbCdEf1234567890AbCdEf1234567890AbCdEf12";
        let intent = make_intent("eth_transfer", wallet);

        let registers = RegisterStore::new();
        registers.set(
            "wallet_address",
            serde_json::json!(wallet),
            "wallet_provider",
        );
        // Also add send_to so register-exists check passes
        registers.set("send_to", serde_json::json!(wallet), "register_set");
        let ctx = ToolContext::new().with_registers(registers);

        let err = run_deterministic_checks(&intent, &ctx).unwrap_err();
        assert!(err.contains("own wallet"), "got: {}", err);
    }

    #[test]
    fn test_address_not_in_registers_or_context_blocked() {
        let intent = make_intent(
            "eth_transfer",
            "0x1111111111111111111111111111111111111111",
        );
        let ctx = ToolContext::new();
        let err = run_deterministic_checks(&intent, &ctx).unwrap_err();
        assert!(err.contains("not found in any register"), "got: {}", err);
    }

    #[test]
    fn test_address_in_register_passes() {
        let addr = "0x1111111111111111111111111111111111111111";
        let intent = make_intent("eth_transfer", addr);

        let registers = RegisterStore::new();
        registers.set("send_to", serde_json::json!(addr), "register_set");
        let ctx = ToolContext::new().with_registers(registers);

        assert!(run_deterministic_checks(&intent, &ctx).is_ok());
    }

    #[test]
    fn test_address_in_context_bank_passes() {
        let addr = "0x1111111111111111111111111111111111111111";
        let intent = make_intent("eth_transfer", addr);

        let mut ctx = ToolContext::new();
        ctx.context_bank.add(ContextBankItem {
            value: addr.to_string(),
            item_type: "eth_address".to_string(),
            label: None,
        });

        assert!(run_deterministic_checks(&intent, &ctx).is_ok());
    }

    #[test]
    fn test_contract_call_skips_register_check() {
        // Contract calls don't require the "to" address to be in registers
        // because the contract address comes from ABI files, not user input
        let intent = make_intent(
            "contract_call",
            "0x1111111111111111111111111111111111111111",
        );
        let ctx = ToolContext::new();
        assert!(run_deterministic_checks(&intent, &ctx).is_ok());
    }

    // ── format_verification_prompt ───────────────────────────────────

    #[test]
    fn test_format_verification_prompt_basic() {
        let intent = make_intent(
            "eth_transfer",
            "0x1111111111111111111111111111111111111111",
        );
        let prompt = format_verification_prompt(&intent, "send 0.001 ETH to alice");
        assert!(prompt.contains("send 0.001 ETH to alice"));
        assert!(prompt.contains("eth_transfer"));
        assert!(prompt.contains("0x1111"));
        assert!(prompt.contains("0.001 ETH"));
    }

    #[test]
    fn test_format_verification_prompt_with_function() {
        let mut intent = make_intent(
            "contract_call",
            "0x1111111111111111111111111111111111111111",
        );
        intent.function_name = Some("transfer".to_string());
        intent.abi_name = Some("erc20".to_string());
        let prompt = format_verification_prompt(&intent, "send 100 USDC");
        assert!(prompt.contains("transfer"));
        assert!(prompt.contains("erc20"));
    }

    // ── integration test with MockAiClient ───────────────────────────

    use crate::ai::{MockAiClient, AiResponse};

    fn mock_client(responses: Vec<&str>) -> AiClient {
        let ai_responses: Vec<Result<AiResponse, _>> = responses
            .into_iter()
            .map(|text| Ok(AiResponse::text(text.to_string())))
            .collect();
        AiClient::Mock(MockAiClient::new(ai_responses))
    }

    #[tokio::test]
    async fn test_verify_intent_approved_by_mock() {
        let mock = mock_client(vec!["APPROVED"]);
        let addr = "0x1111111111111111111111111111111111111111";
        let intent = make_intent("eth_transfer", addr);

        let registers = RegisterStore::new();
        registers.set("send_to", serde_json::json!(addr), "register_set");
        let mut ctx = ToolContext::new().with_registers(registers);
        ctx.extra.insert(
            "original_user_message".to_string(),
            serde_json::json!("send 0.001 ETH to 0x1111"),
        );

        let result = verify_intent(&intent, &ctx, Some(&mock)).await;
        assert!(result.is_ok(), "Expected APPROVED, got: {:?}", result);
    }

    #[tokio::test]
    async fn test_verify_intent_rejected_by_mock() {
        let mock = mock_client(vec!["REJECTED: amount mismatch"]);
        let addr = "0x1111111111111111111111111111111111111111";
        let intent = make_intent("eth_transfer", addr);

        let registers = RegisterStore::new();
        registers.set("send_to", serde_json::json!(addr), "register_set");
        let mut ctx = ToolContext::new().with_registers(registers);
        ctx.extra.insert(
            "original_user_message".to_string(),
            serde_json::json!("send 0.001 ETH"),
        );

        let result = verify_intent(&intent, &ctx, Some(&mock)).await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("amount mismatch"),
            "Expected mismatch reason"
        );
    }

    #[tokio::test]
    async fn test_verify_intent_no_user_message_still_passes() {
        // When original_user_message is missing, AI check is skipped
        // but deterministic checks still run
        let addr = "0x1111111111111111111111111111111111111111";
        let intent = make_intent("eth_transfer", addr);

        let registers = RegisterStore::new();
        registers.set("send_to", serde_json::json!(addr), "register_set");
        let ctx = ToolContext::new().with_registers(registers);

        let result = verify_intent(&intent, &ctx, None).await;
        assert!(result.is_ok(), "Should pass without AI check: {:?}", result);
    }
}
