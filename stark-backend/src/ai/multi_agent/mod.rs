//! Multi-agent system for complex task handling
//!
//! This module implements a multi-phase agent architecture with multi-task support:
//!
//! 1. **Initializer** - Analyzes the request and selects the appropriate mode
//! 2. **Explore** - Gathers information and builds context
//! 3. **Plan** - Creates a multi-task execution plan with dependencies
//! 4. **Perform** - Executes tasks (supports parallel execution)
//!
//! ## Flow
//!
//! ```text
//! Request → Initializer → [Explore] → Plan → Perform → Response
//!                ↓
//!           (simple task)
//!                ↓
//!             Perform → Response
//! ```
//!
//! ## Multi-Task Features
//!
//! - Tasks can have dependencies (`blocked_by`)
//! - Tasks are automatically unblocked when dependencies complete
//! - Parallel execution of independent tasks
//! - Priority-based ordering for ready tasks
//!
//! ## Example Task Flow
//!
//! ```text
//! Plan Mode:
//!   create_task(id="1", subject="Setup database")
//!   create_task(id="2", subject="Create models", blocked_by=["1"])
//!   create_task(id="3", subject="Write tests", blocked_by=["2"])
//!   ready_to_perform()
//!
//! Perform Mode:
//!   start_task("1") → complete_task("1")  // Unblocks "2"
//!   start_task("2") → complete_task("2")  // Unblocks "3"
//!   start_task("3") → complete_task("3")
//!   finish_execution()
//! ```

pub mod orchestrator;
pub mod tools;
pub mod types;

pub use orchestrator::{Orchestrator, ProcessResult};
pub use types::{AgentContext, AgentMode, Finding, Relevance, Task, TaskStats, TaskStatus};
