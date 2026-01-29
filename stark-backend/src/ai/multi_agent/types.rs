//! Multi-agent system types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The current mode/phase of the multi-agent system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Initial phase - determines which mode to use
    Initializer,
    /// Exploration phase - gather information and grow context
    Explore,
    /// Planning phase - create a detailed plan from gathered context
    Plan,
    /// Execution phase - perform the planned actions
    Perform,
}

impl Default for AgentMode {
    fn default() -> Self {
        AgentMode::Initializer
    }
}

impl std::fmt::Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMode::Initializer => write!(f, "initializer"),
            AgentMode::Explore => write!(f, "explore"),
            AgentMode::Plan => write!(f, "plan"),
            AgentMode::Perform => write!(f, "perform"),
        }
    }
}

impl AgentMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "initializer" | "init" => Some(AgentMode::Initializer),
            "explore" | "exploration" => Some(AgentMode::Explore),
            "plan" | "planning" => Some(AgentMode::Plan),
            "perform" | "execute" | "execution" => Some(AgentMode::Perform),
            _ => None,
        }
    }
}

/// Task status in the task list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Blocked => write!(f, "blocked"),
        }
    }
}

/// A task in the multi-task plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: String,
    /// Short subject/title
    pub subject: String,
    /// Detailed description
    pub description: String,
    /// Current status
    pub status: TaskStatus,
    /// Task IDs that must complete before this can start
    pub blocked_by: Vec<String>,
    /// Task IDs that this task blocks
    pub blocks: Vec<String>,
    /// Tool to use for this task (if applicable)
    pub tool: Option<String>,
    /// Tool parameters (if applicable)
    pub tool_params: Option<serde_json::Value>,
    /// Result output after completion
    pub result: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Whether this task can run in parallel with others
    pub parallelizable: bool,
    /// Priority (lower = higher priority)
    pub priority: u32,
}

impl Task {
    pub fn new(id: impl Into<String>, subject: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            description: description.into(),
            status: TaskStatus::Pending,
            blocked_by: Vec::new(),
            blocks: Vec::new(),
            tool: None,
            tool_params: None,
            result: None,
            error: None,
            parallelizable: true,
            priority: 100,
        }
    }

    /// Check if this task is ready to execute (not blocked)
    pub fn is_ready(&self, completed_tasks: &[String]) -> bool {
        self.status == TaskStatus::Pending
            && self.blocked_by.iter().all(|dep| completed_tasks.contains(dep))
    }
}

/// Context accumulated during the multi-agent flow
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentContext {
    /// Original user request
    pub original_request: String,

    /// Notes gathered during exploration
    pub exploration_notes: Vec<String>,

    /// Key findings from exploration
    pub findings: Vec<Finding>,

    /// Multi-task plan
    pub tasks: Vec<Task>,

    /// Plan summary/goal
    pub plan_summary: Option<String>,

    /// Current mode
    pub mode: AgentMode,

    /// Number of iterations in current mode
    pub mode_iterations: u32,

    /// Total iterations across all modes
    pub total_iterations: u32,

    /// Whether the agent believes it has enough context
    pub context_sufficient: bool,

    /// Whether the plan is ready for execution
    pub plan_ready: bool,

    /// Scratchpad for agent notes during execution
    pub scratchpad: String,
}

impl AgentContext {
    /// Get tasks that are ready to execute
    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        let completed: Vec<String> = self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect();

        self.tasks
            .iter()
            .filter(|t| t.is_ready(&completed))
            .collect()
    }

    /// Get the next task to work on (respects priority and dependencies)
    pub fn get_next_task(&self) -> Option<&Task> {
        self.get_ready_tasks()
            .into_iter()
            .min_by_key(|t| t.priority)
    }

    /// Get task by ID
    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get mutable task by ID
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Check if all tasks are complete
    pub fn all_tasks_complete(&self) -> bool {
        self.tasks.iter().all(|t| t.status == TaskStatus::Completed || t.status == TaskStatus::Failed)
    }

    /// Get completion stats
    pub fn get_stats(&self) -> TaskStats {
        let mut stats = TaskStats::default();
        for task in &self.tasks {
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::InProgress => stats.in_progress += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Failed => stats.failed += 1,
                TaskStatus::Blocked => stats.blocked += 1,
            }
        }
        stats.total = self.tasks.len();
        stats
    }

    /// Update blocked status for all tasks
    pub fn update_blocked_status(&mut self) {
        let completed: Vec<String> = self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect();

        for task in &mut self.tasks {
            if task.status == TaskStatus::Pending || task.status == TaskStatus::Blocked {
                let is_blocked = !task.blocked_by.iter().all(|dep| completed.contains(dep));
                task.status = if is_blocked { TaskStatus::Blocked } else { TaskStatus::Pending };
            }
        }
    }
}

/// Task completion statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
    pub blocked: usize,
}

impl std::fmt::Display for TaskStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} complete ({} pending, {} in progress, {} blocked, {} failed)",
            self.completed, self.total, self.pending, self.in_progress, self.blocked, self.failed
        )
    }
}

/// A finding discovered during exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub content: String,
    pub relevance: Relevance,
    /// Related file paths
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Relevance {
    High,
    Medium,
    Low,
}

/// Transition decision from the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    pub from: AgentMode,
    pub to: AgentMode,
    pub reason: String,
}
