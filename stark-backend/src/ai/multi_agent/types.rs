//! Agent types

use serde::{Deserialize, Serialize};

use crate::tools::types::ToolGroup;

/// The specialized mode/persona of the agent
/// Controls which tools and skills are available (acts as a "toolbox")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentSubtype {
    /// Finance/DeFi specialist - crypto swaps, transfers, web3 operations
    #[default]
    Finance,
    /// Code engineer - software development, git, code editing
    CodeEngineer,
    /// Secretary - social media, marketing, messaging, scheduling
    Secretary,
}

impl AgentSubtype {
    /// Get all available subtypes
    pub fn all() -> Vec<AgentSubtype> {
        vec![
            AgentSubtype::Finance,
            AgentSubtype::CodeEngineer,
            AgentSubtype::Secretary,
        ]
    }

    /// Get the tool groups allowed for this subtype
    /// Note: System, Web, and Filesystem are always available as "core" tools
    pub fn allowed_tool_groups(&self) -> Vec<ToolGroup> {
        // Core groups available to all subtypes
        let mut groups = vec![
            ToolGroup::System,     // set_agent_subtype, subagent
            ToolGroup::Web,        // web_fetch
            ToolGroup::Filesystem, // read_file, list_files
        ];

        // Add subtype-specific groups
        match self {
            AgentSubtype::Finance => {
                groups.push(ToolGroup::Finance); // web3_tx, token_lookup, x402_*, etc.
            }
            AgentSubtype::CodeEngineer => {
                groups.push(ToolGroup::Development); // edit_file, grep, glob, git, etc.
                groups.push(ToolGroup::Exec);        // exec command
            }
            AgentSubtype::Secretary => {
                groups.push(ToolGroup::Messaging); // agent_send
                groups.push(ToolGroup::Social);    // twitter, scheduling tools
            }
        }

        groups
    }

    /// Get the skill tags allowed for this subtype
    pub fn allowed_skill_tags(&self) -> Vec<&'static str> {
        match self {
            AgentSubtype::Finance => vec!["crypto", "defi", "transfer", "swap", "finance", "wallet", "token"],
            AgentSubtype::CodeEngineer => vec!["development", "git", "testing", "debugging", "review", "code", "github"],
            AgentSubtype::Secretary => vec!["social", "marketing", "messaging", "twitter", "scheduling", "communication", "social-media"],
        }
    }

    /// Human-readable label for UI display
    pub fn label(&self) -> &'static str {
        match self {
            AgentSubtype::Finance => "Finance",
            AgentSubtype::CodeEngineer => "CodeEngineer",
            AgentSubtype::Secretary => "Secretary",
        }
    }

    /// Get description of what this subtype does
    pub fn description(&self) -> &'static str {
        match self {
            AgentSubtype::Finance => "Crypto swaps, transfers, DeFi operations, token lookups",
            AgentSubtype::CodeEngineer => "Code editing, git operations, testing, debugging",
            AgentSubtype::Secretary => "Social media, messaging, scheduling, marketing",
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentSubtype::Finance => "finance",
            AgentSubtype::CodeEngineer => "code_engineer",
            AgentSubtype::Secretary => "secretary",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "finance" | "defi" | "crypto" | "swap" | "transfer" => Some(AgentSubtype::Finance),
            "code_engineer" | "codeengineer" | "code" | "dev" | "developer" | "git" => {
                Some(AgentSubtype::CodeEngineer)
            }
            "secretary" | "social" | "marketing" | "messaging" | "twitter" => {
                Some(AgentSubtype::Secretary)
            }
            _ => None,
        }
    }

    /// Get emoji for this subtype
    pub fn emoji(&self) -> &'static str {
        match self {
            AgentSubtype::Finance => "💰",
            AgentSubtype::CodeEngineer => "🛠️",
            AgentSubtype::Secretary => "📱",
        }
    }
}

impl std::fmt::Display for AgentSubtype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The current mode of the agent (simplified - single mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Active assistant mode - handles all tasks
    Assistant,
}

impl Default for AgentMode {
    fn default() -> Self {
        AgentMode::Assistant
    }
}

impl std::fmt::Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMode::Assistant => write!(f, "assistant"),
        }
    }
}

impl AgentMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "assistant" | "explore" | "plan" | "perform" | "execute" => Some(AgentMode::Assistant),
            _ => None,
        }
    }

    /// Check if skills are available in this mode
    pub fn allows_skills(&self) -> bool {
        true // Always allow skills
    }

    /// Check if action tools (swap, transfer, etc.) are available in this mode
    pub fn allows_action_tools(&self) -> bool {
        true // Always allow action tools
    }

    /// Human-readable label for UI display
    pub fn label(&self) -> &'static str {
        match self {
            AgentMode::Assistant => "Assistant",
        }
    }
}

/// Context accumulated during the agent session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentContext {
    /// Original user request
    pub original_request: String,

    /// Notes gathered during the session
    pub exploration_notes: Vec<String>,

    /// Current mode (always Assistant)
    pub mode: AgentMode,

    /// Current agent subtype/specialization
    #[serde(default)]
    pub subtype: AgentSubtype,

    /// Number of iterations in current session
    pub mode_iterations: u32,

    /// Total iterations
    pub total_iterations: u32,

    /// Scratchpad for agent notes
    pub scratchpad: String,

    /// Currently active skill context
    #[serde(default)]
    pub active_skill: Option<ActiveSkill>,

    /// Total actual tool calls made (excludes orchestrator tools)
    #[serde(default)]
    pub actual_tool_calls: u32,

    /// Number of times the agent tried to respond without calling tools
    #[serde(default)]
    pub no_tool_warnings: u32,
}

/// Active skill context that persists across turns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSkill {
    /// Name of the skill
    pub name: String,
    /// Skill instructions/body
    pub instructions: String,
    /// When the skill was activated
    pub activated_at: String,
    /// Number of actual tool calls made since this skill was activated
    #[serde(default)]
    pub tool_calls_made: u32,
}

/// Mode transition (kept for API compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    pub from: AgentMode,
    pub to: AgentMode,
    pub reason: String,
}
