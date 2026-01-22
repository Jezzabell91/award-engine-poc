//! Configuration loading and management for the Award Interpretation Engine.
//!
//! This module provides functionality to load award configurations from YAML files,
//! including award metadata, classifications, rates, and penalty information.
//!
//! # Example
//!
//! ```no_run
//! use award_engine::config::ConfigLoader;
//!
//! let config = ConfigLoader::load("./config/ma000018").unwrap();
//! println!("Loaded award: {}", config.award().name);
//! ```

mod loader;
mod types;

pub use loader::ConfigLoader;
pub use types::{
    AllowanceRates, AwardConfig, AwardMetadata, Classification, ClassificationRate, OvertimeConfig,
    OvertimeRates, OvertimeSection, Penalties, PenaltyConfig, PenaltyRates, RateConfig,
};
