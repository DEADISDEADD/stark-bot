//! Database model modules - additional methods not in sqlite.rs
//!
//! Note: Core methods are in sqlite.rs. These modules add specialized functionality.
//! Modules with duplicate methods have been disabled to avoid conflicts.

// Disabled - all methods already in sqlite.rs:
// mod auth_sessions;
// mod api_keys;
// mod channels;
// mod agent_settings;
// mod identities;
// mod memories;
// mod tool_configs;

// Enabled - contain unique methods:
mod chat_sessions;  // compaction methods
mod skills;         // get_enabled_skill_by_name
mod cron_jobs;      // all cron job methods
mod heartbeat;      // heartbeat config methods
mod gmail;          // gmail config methods
