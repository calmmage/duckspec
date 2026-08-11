//! Shared application core for duckboard and ducktui.
//!
//! Owns chat session persistence, scope orientation, slash/meta/fast-response
//! helpers, agent catalog/turn driving, and filesystem watching — without any
//! UI framework dependency.

pub mod agent;
pub mod chat_store;
pub mod fast_response;
pub mod inputs_ledger;
pub mod meta_card;
pub mod paths;
pub mod scope;
pub mod session_sharing;
pub mod slash_commands;
pub mod transcript;
pub mod watcher;

pub mod test_support;
