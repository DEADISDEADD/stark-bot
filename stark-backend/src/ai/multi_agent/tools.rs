//! Multi-agent specific tools for mode transitions and task management
//!
//! These tools are designed for OpenAI-compatible APIs (Kimi, etc.)

use crate::tools::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use serde_json::{json, Value};
use std::collections::HashMap;

// =============================================================================
// INITIALIZER MODE TOOLS
// =============================================================================

/// Create the `select_mode` tool for the Initializer agent
pub fn select_mode_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "mode".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "The mode to transition to".to_string(),
            default: None,
            items: None,
            enum_values: Some(vec![
                "explore".to_string(),
                "plan".to_string(),
                "perform".to_string(),
            ]),
        },
    );
    properties.insert(
        "reasoning".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Why this mode is appropriate for the request".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "select_mode".to_string(),
        description: "Select the agent mode. Use 'explore' to gather information first, 'plan' to create a multi-step plan, or 'perform' for simple direct actions.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["mode".to_string(), "reasoning".to_string()],
        },
        group: ToolGroup::System,
    }
}

// =============================================================================
// EXPLORE MODE TOOLS
// =============================================================================

/// Create the `add_finding` tool for recording discoveries
pub fn add_finding_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "category".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Category of finding".to_string(),
            default: None,
            items: None,
            enum_values: Some(vec![
                "code_pattern".to_string(),
                "file_structure".to_string(),
                "dependency".to_string(),
                "constraint".to_string(),
                "risk".to_string(),
                "other".to_string(),
            ]),
        },
    );
    properties.insert(
        "content".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "What you discovered".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "relevance".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "How relevant to the task".to_string(),
            default: Some(json!("medium")),
            items: None,
            enum_values: Some(vec![
                "high".to_string(),
                "medium".to_string(),
                "low".to_string(),
            ]),
        },
    );
    properties.insert(
        "files".to_string(),
        PropertySchema {
            schema_type: "array".to_string(),
            description: "Related file paths".to_string(),
            default: Some(json!([])),
            items: Some(Box::new(PropertySchema {
                schema_type: "string".to_string(),
                description: "File path".to_string(),
                default: None,
                items: None,
                enum_values: None,
            })),
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "add_finding".to_string(),
        description: "Record an important discovery during exploration. Findings are used to inform the planning phase.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["category".to_string(), "content".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `ready_to_plan` tool
pub fn ready_to_plan_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "summary".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Summary of what was learned and why you're ready to plan".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "ready_to_plan".to_string(),
        description: "Signal that exploration is complete. Call this when you have gathered enough context to create a plan.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["summary".to_string()],
        },
        group: ToolGroup::System,
    }
}

// =============================================================================
// PLAN MODE TOOLS - Multi-task support
// =============================================================================

/// Create the `create_task` tool for adding tasks to the plan
pub fn create_task_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "id".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Unique task ID (e.g., 'task-1', 'setup-db')".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "subject".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Short task title (imperative form, e.g., 'Create user model')".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "description".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Detailed description of what needs to be done".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "blocked_by".to_string(),
        PropertySchema {
            schema_type: "array".to_string(),
            description: "IDs of tasks that must complete before this one can start".to_string(),
            default: Some(json!([])),
            items: Some(Box::new(PropertySchema {
                schema_type: "string".to_string(),
                description: "Task ID".to_string(),
                default: None,
                items: None,
                enum_values: None,
            })),
            enum_values: None,
        },
    );
    properties.insert(
        "tool".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Primary tool to use for this task (optional)".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "priority".to_string(),
        PropertySchema {
            schema_type: "integer".to_string(),
            description: "Priority (1=highest, 100=default). Lower numbers execute first among ready tasks.".to_string(),
            default: Some(json!(100)),
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "create_task".to_string(),
        description: "Add a task to the execution plan. Tasks can have dependencies and run in parallel when possible.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["id".to_string(), "subject".to_string(), "description".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `set_plan_summary` tool
pub fn set_plan_summary_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "summary".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "One-line summary of what the plan accomplishes".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "set_plan_summary".to_string(),
        description: "Set the overall plan summary/goal.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["summary".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `ready_to_perform` tool
pub fn ready_to_perform_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "confirmation".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Confirm the plan is complete and ready for execution".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "ready_to_perform".to_string(),
        description: "Signal that planning is complete and execution can begin. Must have at least one task created.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["confirmation".to_string()],
        },
        group: ToolGroup::System,
    }
}

// =============================================================================
// PERFORM MODE TOOLS
// =============================================================================

/// Create the `start_task` tool
pub fn start_task_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "task_id".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "ID of the task to start working on".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "start_task".to_string(),
        description: "Mark a task as in-progress. Call this before working on a task.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["task_id".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `complete_task` tool
pub fn complete_task_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "task_id".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "ID of the completed task".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "result".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "What was accomplished".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "complete_task".to_string(),
        description: "Mark a task as completed successfully.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["task_id".to_string(), "result".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `fail_task` tool
pub fn fail_task_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "task_id".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "ID of the failed task".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "error".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "What went wrong".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "fail_task".to_string(),
        description: "Mark a task as failed. Include error details.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["task_id".to_string(), "error".to_string()],
        },
        group: ToolGroup::System,
    }
}

/// Create the `finish_execution` tool
pub fn finish_execution_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "summary".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Final summary of what was accomplished".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );
    properties.insert(
        "follow_up".to_string(),
        PropertySchema {
            schema_type: "string".to_string(),
            description: "Any recommended follow-up actions".to_string(),
            default: None,
            items: None,
            enum_values: None,
        },
    );

    ToolDefinition {
        name: "finish_execution".to_string(),
        description: "Signal that all tasks are complete and provide a final summary.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties,
            required: vec!["summary".to_string()],
        },
        group: ToolGroup::System,
    }
}

// =============================================================================
// SHARED TOOLS (available in multiple modes)
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

/// Create the `get_task_list` tool
pub fn get_task_list_tool() -> ToolDefinition {
    ToolDefinition {
        name: "get_task_list".to_string(),
        description: "Get the current task list with status. Shows which tasks are ready, blocked, in-progress, or complete.".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        group: ToolGroup::System,
    }
}

// =============================================================================
// TOOL SETS PER MODE
// =============================================================================

/// Get tools for a specific agent mode
pub fn get_tools_for_mode(mode: super::types::AgentMode) -> Vec<ToolDefinition> {
    use super::types::AgentMode;

    match mode {
        AgentMode::Initializer => vec![
            select_mode_tool(),
        ],
        AgentMode::Explore => vec![
            add_finding_tool(),
            add_note_tool(),
            ready_to_plan_tool(),
        ],
        AgentMode::Plan => vec![
            create_task_tool(),
            set_plan_summary_tool(),
            add_note_tool(),
            get_task_list_tool(),
            ready_to_perform_tool(),
        ],
        AgentMode::Perform => vec![
            start_task_tool(),
            complete_task_tool(),
            fail_task_tool(),
            add_note_tool(),
            get_task_list_tool(),
            finish_execution_tool(),
        ],
    }
}

/// Get all multi-agent tools (for reference)
pub fn get_all_tools() -> Vec<ToolDefinition> {
    vec![
        // Initializer
        select_mode_tool(),
        // Explore
        add_finding_tool(),
        ready_to_plan_tool(),
        // Plan
        create_task_tool(),
        set_plan_summary_tool(),
        ready_to_perform_tool(),
        // Perform
        start_task_tool(),
        complete_task_tool(),
        fail_task_tool(),
        finish_execution_tool(),
        // Shared
        add_note_tool(),
        get_task_list_tool(),
    ]
}
