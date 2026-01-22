//! Award Interpretation Engine for Australian Awards
//!
//! This crate provides functionality for interpreting the Aged Care Award 2010 (MA000018)
//! and calculating pay based on shifts, employee classifications, and award rules.
//!
//! # HTTP API
//!
//! The engine can be exposed via HTTP API using the [`api`] module:
//!
//! ```no_run
//! use award_engine::api::{create_router, AppState};
//! use award_engine::config::ConfigLoader;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ConfigLoader::load("./config/ma000018").expect("Failed to load config");
//!     let state = AppState::new(config);
//!     let router = create_router(state);
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, router).await.unwrap();
//! }
//! ```

#![warn(missing_docs)]

pub mod api;
pub mod calculation;
pub mod config;
pub mod error;
pub mod models;
