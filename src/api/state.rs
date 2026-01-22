//! Application state for the Award Interpretation Engine API.
//!
//! This module defines the shared application state that is available
//! to all request handlers.

use std::sync::Arc;

use crate::config::ConfigLoader;

/// Shared application state.
///
/// Contains resources that are shared across all request handlers,
/// such as the loaded award configuration.
#[derive(Clone)]
pub struct AppState {
    /// The loaded award configuration.
    config: Arc<ConfigLoader>,
}

impl AppState {
    /// Creates a new application state with the given configuration loader.
    pub fn new(config: ConfigLoader) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Returns a reference to the configuration loader.
    pub fn config(&self) -> &ConfigLoader {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_is_clone() {
        // Verify AppState can be cloned (required for axum state)
        fn assert_clone<T: Clone>() {}
        assert_clone::<AppState>();
    }
}
