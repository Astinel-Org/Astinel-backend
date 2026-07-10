//! CLI interface for the Sentinel security scanner.
//!
//! Provides a production-quality CLI for scanning Soroban smart contracts
//! with deterministic analysis.

pub mod app;
pub mod commands;
pub mod config;
pub mod errors;
pub mod exit;
pub mod output;
pub mod paths;
pub mod progress;
pub mod scan;
pub mod terminal;
