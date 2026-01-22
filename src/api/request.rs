//! Request types for the Award Interpretation Engine API.
//!
//! This module defines the JSON request structures for the `/calculate` endpoint.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::{Break, Employee, EmploymentType, PayPeriod, PublicHoliday, Shift};

/// Request body for the `/calculate` endpoint.
///
/// Contains all information needed to calculate pay for an employee's shifts
/// within a pay period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationRequest {
    /// The employee information.
    pub employee: EmployeeRequest,
    /// The pay period for the calculation.
    pub pay_period: PayPeriodRequest,
    /// The shifts worked during the pay period.
    pub shifts: Vec<ShiftRequest>,
}

/// Employee information in a calculation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeRequest {
    /// Unique identifier for the employee.
    pub id: String,
    /// The type of employment arrangement.
    pub employment_type: EmploymentType,
    /// The award classification code (e.g., "dce_level_3").
    pub classification_code: String,
    /// The employee's date of birth.
    pub date_of_birth: NaiveDate,
    /// The date the employee started employment.
    pub employment_start_date: NaiveDate,
    /// Optional override for the base hourly rate.
    #[serde(default)]
    pub base_hourly_rate: Option<Decimal>,
    /// Tags for categorizing employees (e.g., qualifications, departments).
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Pay period information in a calculation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPeriodRequest {
    /// The start date of the pay period (inclusive).
    pub start_date: NaiveDate,
    /// The end date of the pay period (inclusive).
    pub end_date: NaiveDate,
    /// Public holidays that fall within this pay period.
    #[serde(default)]
    pub public_holidays: Vec<PublicHolidayRequest>,
}

/// Public holiday information in a calculation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicHolidayRequest {
    /// The date of the public holiday.
    pub date: NaiveDate,
    /// The name of the public holiday.
    pub name: String,
    /// The region where this holiday applies.
    #[serde(default = "default_region")]
    pub region: String,
}

fn default_region() -> String {
    "national".to_string()
}

/// Shift information in a calculation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftRequest {
    /// Unique identifier for the shift.
    pub id: String,
    /// The date of the shift.
    pub date: NaiveDate,
    /// The start time of the shift.
    pub start_time: NaiveDateTime,
    /// The end time of the shift.
    pub end_time: NaiveDateTime,
    /// Breaks taken during the shift.
    #[serde(default)]
    pub breaks: Vec<BreakRequest>,
}

/// Break information in a calculation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakRequest {
    /// The start time of the break.
    pub start_time: NaiveDateTime,
    /// The end time of the break.
    pub end_time: NaiveDateTime,
    /// Whether the break is paid.
    #[serde(default)]
    pub is_paid: bool,
}

impl From<EmployeeRequest> for Employee {
    fn from(req: EmployeeRequest) -> Self {
        Employee {
            id: req.id,
            employment_type: req.employment_type,
            classification_code: req.classification_code,
            date_of_birth: req.date_of_birth,
            employment_start_date: req.employment_start_date,
            base_hourly_rate: req.base_hourly_rate,
            tags: req.tags,
        }
    }
}

impl From<PayPeriodRequest> for PayPeriod {
    fn from(req: PayPeriodRequest) -> Self {
        PayPeriod {
            start_date: req.start_date,
            end_date: req.end_date,
            public_holidays: req.public_holidays.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PublicHolidayRequest> for PublicHoliday {
    fn from(req: PublicHolidayRequest) -> Self {
        PublicHoliday {
            date: req.date,
            name: req.name,
            region: req.region,
        }
    }
}

impl From<ShiftRequest> for Shift {
    fn from(req: ShiftRequest) -> Self {
        Shift {
            id: req.id,
            date: req.date,
            start_time: req.start_time,
            end_time: req.end_time,
            breaks: req.breaks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BreakRequest> for Break {
    fn from(req: BreakRequest) -> Self {
        Break {
            start_time: req.start_time,
            end_time: req.end_time,
            is_paid: req.is_paid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_calculation_request() {
        let json = r#"{
            "employee": {
                "id": "emp_001",
                "employment_type": "full_time",
                "classification_code": "dce_level_3",
                "date_of_birth": "1985-03-15",
                "employment_start_date": "2020-01-01",
                "tags": []
            },
            "pay_period": {
                "start_date": "2026-01-13",
                "end_date": "2026-01-19",
                "public_holidays": []
            },
            "shifts": [
                {
                    "id": "shift_001",
                    "date": "2026-01-13",
                    "start_time": "2026-01-13T09:00:00",
                    "end_time": "2026-01-13T17:00:00",
                    "breaks": []
                }
            ]
        }"#;

        let request: CalculationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.employee.id, "emp_001");
        assert_eq!(request.employee.employment_type, EmploymentType::FullTime);
        assert_eq!(request.shifts.len(), 1);
        assert_eq!(request.shifts[0].id, "shift_001");
    }

    #[test]
    fn test_deserialize_casual_employee_with_tags() {
        let json = r#"{
            "employee": {
                "id": "emp_002",
                "employment_type": "casual",
                "classification_code": "dce_level_3",
                "date_of_birth": "1990-07-22",
                "employment_start_date": "2024-06-01",
                "tags": ["laundry_allowance"]
            },
            "pay_period": {
                "start_date": "2026-01-13",
                "end_date": "2026-01-19",
                "public_holidays": []
            },
            "shifts": []
        }"#;

        let request: CalculationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.employee.employment_type, EmploymentType::Casual);
        assert!(request.employee.tags.contains(&"laundry_allowance".to_string()));
    }

    #[test]
    fn test_employee_conversion() {
        let req = EmployeeRequest {
            id: "emp_001".to_string(),
            employment_type: EmploymentType::FullTime,
            classification_code: "dce_level_3".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            base_hourly_rate: None,
            tags: vec!["laundry_allowance".to_string()],
        };

        let employee: Employee = req.into();
        assert_eq!(employee.id, "emp_001");
        assert!(employee.tags.contains(&"laundry_allowance".to_string()));
    }
}
