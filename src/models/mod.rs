//! Core data models for the Award Interpretation Engine.
//!
//! This module contains all the domain models used throughout the engine.

mod calculation_result;
mod employee;
mod pay_period;
mod shift;

pub use calculation_result::{
    AllowancePayment, AuditStep, AuditTrace, AuditWarning, CalculationResult, PayCategory, PayLine,
    PayTotals,
};
pub use employee::{Employee, EmploymentType};
pub use pay_period::{PayPeriod, PublicHoliday};
pub use shift::{Break, Shift};
