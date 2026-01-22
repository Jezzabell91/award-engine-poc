//! Core data models for the Award Interpretation Engine.
//!
//! This module contains all the domain models used throughout the engine.

mod employee;
mod shift;

pub use employee::{Employee, EmploymentType};
pub use shift::{Break, Shift};
