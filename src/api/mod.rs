//! HTTP API module for the Award Interpretation Engine.
//!
//! This module provides the REST API endpoints for calculating pay
//! based on the Aged Care Award 2010.

mod handlers;
mod request;
mod response;
mod state;

pub use handlers::create_router;
pub use request::CalculationRequest;
pub use response::ApiError;
pub use state::AppState;
