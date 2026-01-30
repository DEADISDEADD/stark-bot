//! Multi-agent specific tools for task management
//!
//! These tools are designed for OpenAI-compatible APIs (Kimi, etc.)

use crate::tools::{PropertySchema, ToolDefinition, ToolGroup, ToolInputSchema};
use std::collections::HashMap;

// =============================================================================
// ASSISTANT TOOLS
// =============================================================================

/// Create the `add_note` tool
pub fn add_note_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "note".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Note content to remember".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "add_note".to_string(),
        description: "Add a note to the scratchpad. Use for observations or information to remember.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["note".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `complete_task` tool
pub fn complete_task_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "summary".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Summary of what was accomplished".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "follow_up".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Any recommended follow-up actions (optional)".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "complete_task".to_string(),
        description: "Signal that the user's request is complete and provide a summary.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["summary".to_string()],
        },
        group: ToolGroup::System,
    }
}

// =============================================================================
// TOOL SETS
// =============================================================================

/// Get tools for the assistant mode
pub fn get_tools_for_mode(_mode: super::types::AgentMode) -> Vec<ToolDefinition> {
    // Single mode now - always return the same tools
    vec![
        add_note_tool(),
        complete_task_tool(),
    ]
}

/// Get all multi-agent tools (for reference)
pub fn get_all_tools() -> Vec<ToolDefinition> {
    vec![
        add_note_tool(),
        complete_task_tool(),
    ]
}
