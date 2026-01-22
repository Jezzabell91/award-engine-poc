//! Employee model and related types.
//!
//! This module defines the Employee struct and EmploymentType enum
//! for representing workers in the award interpretation system.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the type of employment arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmploymentType {
    /// Full-time employment (typically 38 hours per week).
    FullTime,
    /// Part-time employment (less than 38 hours per week with regular pattern).
    PartTime,
    /// Casual employment (no guaranteed hours, includes casual loading).
    Casual,
}

/// Represents an employee subject to award interpretation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Employee {
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
    pub base_hourly_rate: Option<Decimal>,
    /// Tags for categorizing employees (e.g., qualifications, departments).
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Employee {
    /// Returns true if the employee is a casual worker.
    ///
    /// # Examples
    ///
    /// ```
    /// use award_engine::models::{Employee, EmploymentType};
    /// use chrono::NaiveDate;
    ///
    /// let casual = Employee {
    ///     id: "emp_001".to_string(),
    ///     employment_type: EmploymentType::Casual,
    ///     classification_code: "dce_level_3".to_string(),
    ///     date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
    ///     employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
    ///     base_hourly_rate: None,
    ///     tags: vec![],
    /// };
    /// assert!(casual.is_casual());
    /// ```
    pub fn is_casual(&self) -> bool {
        self.employment_type == EmploymentType::Casual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_employee(employment_type: EmploymentType) -> Employee {
        Employee {
            id: "emp_001".to_string(),
            employment_type,
            classification_code: "dce_level_3".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            base_hourly_rate: None,
            tags: vec![],
        }
    }

    #[test]
    fn test_deserialize_fulltime_employee() {
        let json = r#"{
            "id": "emp_001",
            "employment_type": "full_time",
            "classification_code": "dce_level_3",
            "date_of_birth": "1990-01-15",
            "employment_start_date": "2023-06-01",
            "tags": []
        }"#;

        let employee: Employee = serde_json::from_str(json).unwrap();
        assert_eq!(employee.id, "emp_001");
        assert_eq!(employee.employment_type, EmploymentType::FullTime);
        assert_eq!(employee.classification_code, "dce_level_3");
        assert_eq!(
            employee.date_of_birth,
            NaiveDate::from_ymd_opt(1990, 1, 15).unwrap()
        );
        assert_eq!(
            employee.employment_start_date,
            NaiveDate::from_ymd_opt(2023, 6, 1).unwrap()
        );
        assert!(employee.tags.is_empty());
    }

    #[test]
    fn test_deserialize_casual_employee() {
        let json = r#"{
            "id": "emp_002",
            "employment_type": "casual",
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-05-20",
            "employment_start_date": "2024-01-15",
            "base_hourly_rate": "30.50",
            "tags": ["qualified", "night_shift"]
        }"#;

        let employee: Employee = serde_json::from_str(json).unwrap();
        assert_eq!(employee.employment_type, EmploymentType::Casual);
        assert_eq!(employee.base_hourly_rate, Some(Decimal::new(3050, 2)));
        assert_eq!(employee.tags, vec!["qualified", "night_shift"]);
    }

    #[test]
    fn test_deserialize_part_time_employee() {
        let json = r#"{
            "id": "emp_003",
            "employment_type": "part_time",
            "classification_code": "dce_level_3",
            "date_of_birth": "1992-08-10",
            "employment_start_date": "2022-03-01",
            "tags": []
        }"#;

        let employee: Employee = serde_json::from_str(json).unwrap();
        assert_eq!(employee.employment_type, EmploymentType::PartTime);
    }

    #[test]
    fn test_serialize_employee() {
        let employee = create_test_employee(EmploymentType::FullTime);
        let json = serde_json::to_string(&employee).unwrap();

        // Deserialize back and verify round-trip
        let deserialized: Employee = serde_json::from_str(&json).unwrap();
        assert_eq!(employee, deserialized);
    }

    #[test]
    fn test_is_casual_returns_true_for_casual() {
        let employee = create_test_employee(EmploymentType::Casual);
        assert!(employee.is_casual());
    }

    #[test]
    fn test_is_casual_returns_false_for_fulltime() {
        let employee = create_test_employee(EmploymentType::FullTime);
        assert!(!employee.is_casual());
    }

    #[test]
    fn test_is_casual_returns_false_for_parttime() {
        let employee = create_test_employee(EmploymentType::PartTime);
        assert!(!employee.is_casual());
    }

    #[test]
    fn test_employment_type_serialization() {
        assert_eq!(
            serde_json::to_string(&EmploymentType::FullTime).unwrap(),
            "\"full_time\""
        );
        assert_eq!(
            serde_json::to_string(&EmploymentType::PartTime).unwrap(),
            "\"part_time\""
        );
        assert_eq!(
            serde_json::to_string(&EmploymentType::Casual).unwrap(),
            "\"casual\""
        );
    }

    #[test]
    fn test_employee_with_base_hourly_rate() {
        let mut employee = create_test_employee(EmploymentType::FullTime);
        employee.base_hourly_rate = Some(Decimal::new(3254, 2)); // 32.54

        let json = serde_json::to_string(&employee).unwrap();
        let deserialized: Employee = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.base_hourly_rate, Some(Decimal::new(3254, 2)));
    }
}
