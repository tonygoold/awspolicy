use crate::aws::glob_matches;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

use chrono::DateTime;
use ipnetwork::IpNetwork;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionError {
    TypeMismatch,
    TooManyValues,
    NotImplemented
}

impl std::fmt::Display for ConditionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for ConditionError {}

fn cmp_numbers(lhs: &str, rhs: &str) -> anyhow::Result<Ordering> {
    let lhs = f64::from_str(lhs).map_err(|_| ConditionError::TypeMismatch)?;
    let rhs = f64::from_str(rhs).map_err(|_| ConditionError::TypeMismatch)?;
    let result = lhs.partial_cmp(&rhs).ok_or(ConditionError::TypeMismatch)?;
    Ok(result)
}

fn cmp_dates(lhs: &str, rhs: &str) -> anyhow::Result<Ordering> {
    let lhs = DateTime::parse_from_rfc3339(lhs).map_err(|_| ConditionError::TypeMismatch)?;
    let rhs = DateTime::parse_from_rfc3339(rhs).map_err(|_| ConditionError::TypeMismatch)?;
    Ok(lhs.cmp(&rhs))
}

fn bools_eq(lhs: &str, rhs: &str) -> anyhow::Result<bool> {
    let lhs = bool::from_str(lhs).map_err(|_| ConditionError::TypeMismatch)?;
    let rhs = bool::from_str(rhs).map_err(|_| ConditionError::TypeMismatch)?;
    Ok(lhs == rhs)
}

fn base64s_eq(lhs: &str, rhs: &str) -> anyhow::Result<bool> {
    let lhs = base64::decode(lhs).map_err(|_| ConditionError::TypeMismatch)?;
    let rhs = base64::decode(rhs).map_err(|_| ConditionError::TypeMismatch)?;
    Ok(lhs == rhs)
}

fn ip_in_cidr(lhs: &str, rhs: &str) -> anyhow::Result<bool> {
    let lhs = IpAddr::from_str(lhs).map_err(|_| ConditionError::TypeMismatch)?;
    let rhs = IpNetwork::from_str(rhs).map_err(|_| ConditionError::TypeMismatch)?;
    Ok(rhs.contains(lhs))
}

pub type ConditionValues = HashMap<String, Vec<String>>;

// TODO: Implement "IfExists" suffix for everything but Null
// TODO: Implement "ForAllValues" and "ForAnyValue" set operators
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ConditionOperator {
    StringEquals,
    StringNotEquals,
    StringEqualsIgnoreCase,
    StringNotEqualsIgnoreCase,
    StringLike,
    StringNotLike,

    NumericEquals,
    NumericNotEquals,
    NumericLessThan,
    NumericLessThanEquals,
    NumericGreaterThan,
    NumericGreaterThanEquals,

    DateEquals,
    DateNotEquals,
    DateLessThan,
    DateLessThanEquals,
    DateGreaterThan,
    DateGreaterThanEquals,

    Bool,

    BinaryEquals,

    IpAddress,
    NotIpAddress,

    ArnEquals,
    ArnLike,
    ArnNotEquals,
    ArnNotLike,

    // Condition value must be "true" or "false"
    Null,
}

impl ConditionOperator {
    pub fn matches(&self, value: &str, target: &str) -> anyhow::Result<bool> {
        match *self {
            Self::StringEquals => Ok(target == value),
            Self::StringNotEquals => Ok(target != value),
            Self::StringEqualsIgnoreCase => Ok(target.to_lowercase() == value.to_lowercase()),
            Self::StringNotEqualsIgnoreCase => Ok(target.to_lowercase() != value.to_lowercase()),
            Self::StringLike => Ok(glob_matches(target, value)),
            Self::StringNotLike => Ok(!glob_matches(target, value)),

            Self::NumericEquals => Ok(cmp_numbers(value, target)? == Ordering::Equal),
            Self::NumericNotEquals => Ok(cmp_numbers(value, target)? != Ordering::Equal),
            Self::NumericLessThan => Ok(cmp_numbers(value, target)? == Ordering::Less),
            Self::NumericLessThanEquals => Ok(cmp_numbers(value, target)? != Ordering::Greater),
            Self::NumericGreaterThan => Ok(cmp_numbers(value, target)? == Ordering::Greater),
            Self::NumericGreaterThanEquals => Ok(cmp_numbers(value, target)? != Ordering::Less),

            Self::DateEquals => Ok(cmp_dates(value, target)? == Ordering::Equal),
            Self::DateNotEquals => Ok(cmp_dates(value, target)? != Ordering::Equal),
            Self::DateLessThan => Ok(cmp_dates(value, target)? == Ordering::Less),
            Self::DateLessThanEquals => Ok(cmp_dates(value, target)? != Ordering::Greater),
            Self::DateGreaterThan => Ok(cmp_dates(value, target)? == Ordering::Greater),
            Self::DateGreaterThanEquals => Ok(cmp_dates(value, target)? != Ordering::Less),

            // Ok(_?) needed because of ConditionError vs. anyhow::Error result type mismatch
            Self::Bool => Ok(bools_eq(value, target)?),

            // Ok(_?) needed because of ConditionError vs. anyhow::Error result type mismatch
            Self::BinaryEquals => Ok(base64s_eq(value, target)?),

            Self::IpAddress => Ok(ip_in_cidr(value, target)?),
            Self::NotIpAddress => Ok(!ip_in_cidr(value, target)?),

            Self::ArnEquals => Err(ConditionError::NotImplemented),
            Self::ArnLike => Err(ConditionError::NotImplemented),
            Self::ArnNotEquals => Err(ConditionError::NotImplemented),
            Self::ArnNotLike => Err(ConditionError::NotImplemented),

            // Condition value must be "true" or "false"
            // This case should be handled outside this function, because we
            // assume non-null by this point
            Self::Null => Err(ConditionError::NotImplemented),
        }.map_err(anyhow::Error::from)
    }
}

impl TryFrom<&str> for ConditionOperator {
    type Error = json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let op = match value {
            "StringEquals" => Self::StringEquals,
            "StringNotEquals" => Self::StringNotEquals,
            "StringEqualsIgnoreCase" => Self::StringEqualsIgnoreCase,
            "StringNotEqualsIgnoreCase" => Self::StringNotEqualsIgnoreCase,
            "StringLike" => Self::StringLike,
            "StringNotLike" => Self::StringNotLike,
            "NumericEquals" => Self::NumericEquals,
            "NumericNotEquals" => Self::NumericNotEquals,
            "NumericLessThan" => Self::NumericLessThan,
            "NumericLessThanEquals" => Self::NumericLessThanEquals,
            "NumericGreaterThan" => Self::NumericGreaterThan,
            "NumericGreaterThanEquals" => Self::NumericGreaterThanEquals,
            "DateEquals" => Self::DateEquals,
            "DateNotEquals" => Self::DateNotEquals,
            "DateLessThan" => Self::DateLessThan,
            "DateLessThanEquals" => Self::DateLessThanEquals,
            "DateGreaterThan" => Self::DateGreaterThan,
            "DateGreaterThanEquals" => Self::DateGreaterThanEquals,
            "Bool" => Self::Bool,
            "BinaryEquals" => Self::BinaryEquals,
            "IpAddress" => Self::IpAddress,
            "NotIpAddress" => Self::NotIpAddress,
            "ArnEquals" => Self::ArnEquals,
            "ArnLike" => Self::ArnLike,
            "ArnNotEquals" => Self::ArnNotEquals,
            "ArnNotLike" => Self::ArnNotLike,
            "Null" => Self::Null,
            _ => return Err(json::Error::wrong_type("unrecognized condition operator")),
        };
        Ok(op)
    }
}

#[derive(Debug, Clone)]
pub struct ConditionSet {
    conditions: HashMap<ConditionOperator, ConditionValues>,
}

impl ConditionSet {
    pub fn new() -> Self {
        ConditionSet{ conditions: HashMap::new() }
    }

    pub fn insert(&mut self, entry: (ConditionOperator, ConditionValues)) -> Option<ConditionValues> {
        let (op, values) = entry;
        self.conditions.insert(op, values)
    }

    // TODO: Genericize value_map parameter
    pub fn matches(&self, value_map: &HashMap<String, Vec<String>>) -> anyhow::Result<bool> {
        self.conditions.iter().fold(Ok(true), |result, (op, target_map)| {
            // Use !result.contains(true) when stable
            match result {
                Err(_) | Ok(false) => return result,
                _ => (),
            };
            target_map.iter().fold(Ok(true), |result, (key, targets)| {
                // Use !result.contains(true) when stable
                match result {
                    Err(_) | Ok(false) => return result,
                    _ => (),
                };
                let values = match value_map.get(key) {
                    Some(values) => if *op == ConditionOperator::Null {
                        return Ok(&targets[0] == "false");
                    } else {
                        values
                    }
                    None => if *op == ConditionOperator::Null {
                        return Ok(&targets[0] == "true");
                    } else {
                        // TODO: Handle ...IfExists suffix
                        return Err(ConditionError::NotImplemented).map_err(anyhow::Error::from);
                    }
                };
                // TODO: Handle ForAllValues:/ForAnyValues: prefixes
                if values.len() != 1 {
                    return Err(ConditionError::TooManyValues).map_err(anyhow::Error::from);
                }
                let value = &values[0];
                targets.iter().fold(Ok(false), |result, target| {
                    if let Ok(false) = result {
                        op.matches(value, target)
                    } else {
                        result
                    }
                })
            })
        })
    }

    fn try_from_values(values: &json::JsonValue) -> Result<ConditionValues, json::Error> {
        values.entries().map(|(key, values)| {
            if let Some(s) = values.as_str() {
                return Ok((key.to_string(), vec![s.to_string()]));
            }
            values.members().map(|value| {
                value.as_str()
                    .ok_or_else(|| json::Error::wrong_type("expected condition values to be strings"))
                    .map(String::from)

            }).collect::<Result<Vec<_>, _>>().map(|values| (key.to_string(), values))
        }).collect()
    }
}

impl TryFrom<&json::JsonValue> for ConditionSet {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        value.entries().map(|(key, value)| {
            let operator = ConditionOperator::try_from(key)?;
            let values = Self::try_from_values(value)?;
            Ok((operator, values))
        }).collect::<Result<HashMap<_, _>, _>>()
            .map(|conditions| ConditionSet { conditions })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{ConditionOperator, ConditionSet, ConditionValues};

    fn single_value(key: &str, value: &str) -> ConditionValues {
        ConditionValues::from([(key.to_string(), vec![value.to_string()])])
    }

    #[test]
    fn op_string_equals() {
        let cases = [
            (ConditionOperator::StringEquals, true),
            (ConditionOperator::StringNotEquals, false),
        ];
        for (op, expected) in cases {
            assert_eq!(expected, op.matches("test", "test").unwrap());
            assert_eq!(expected, op.matches("test?", "test?").unwrap());
            assert_eq!(expected, op.matches("test*", "test*").unwrap());

            assert_ne!(expected, op.matches("TEST", "test").unwrap());
            assert_ne!(expected, op.matches("testa", "test?").unwrap());
            assert_ne!(expected, op.matches("testa", "test*").unwrap());
        }
    }

    #[test]
    fn op_string_equals_ignore_case() {
        let cases = [
            (ConditionOperator::StringEqualsIgnoreCase, true),
            (ConditionOperator::StringNotEqualsIgnoreCase, false),
        ];
        for (op, expected) in cases {
            assert_eq!(expected, op.matches("test", "test").unwrap());
            assert_eq!(expected, op.matches("TEST", "test").unwrap());
        }
    }

    #[test]
    fn op_string_like() {
        let cases = [
            (ConditionOperator::StringLike, true),
            (ConditionOperator::StringNotLike, false),
        ];
        for (op, expected) in cases {
            assert_eq!(expected, op.matches("test", "t?st").unwrap());
            assert_eq!(expected, op.matches("tst", "t*st").unwrap());
            assert_eq!(expected, op.matches("test", "t*st").unwrap());
            assert_eq!(expected, op.matches("teest", "t*st").unwrap());

            assert_ne!(expected, op.matches("tst", "t?st").unwrap());
            assert_ne!(expected, op.matches("teest", "t?st").unwrap());
        }
    }

    #[test]
    fn op_num_compare() {
        use ConditionOperator::{
            NumericEquals,
            NumericNotEquals,
            NumericLessThan,
            NumericLessThanEquals,
            NumericGreaterThan,
            NumericGreaterThanEquals,
        };
        // lhs, right, less-than, equal
        let cases = [
            ("1", "2", true, false),
            ("2", "2", false, true),
            ("3", "2", false, false),
            ("1.0", "2", true, false),
            ("2.0", "2", false, true),
            ("3.0", "2", false, false),
            ("1", "2.0", true, false),
            ("2", "2.0", false, true),
            ("3", "2.0", false, false),
            ("1.0", "2.0", true, false),
            ("2.0", "2.0", false, true),
            ("3.0", "2.0", false, false),
        ];
        for (lhs, rhs, less_than, equals) in cases {
            assert_eq!(equals, NumericEquals.matches(lhs, rhs).unwrap());
            assert_eq!(!equals, NumericNotEquals.matches(lhs, rhs).unwrap());
            assert_eq!(less_than, NumericLessThan.matches(lhs, rhs).unwrap());
            assert_eq!(less_than || equals, NumericLessThanEquals.matches(lhs, rhs).unwrap());
            assert_eq!(!(less_than || equals), NumericGreaterThan.matches(lhs, rhs).unwrap());
            assert_eq!(!less_than, NumericGreaterThanEquals.matches(lhs, rhs).unwrap());
        }
    }

    #[test]
    fn op_num_invalid() {
        use ConditionOperator::{
            NumericEquals,
            NumericNotEquals,
            NumericLessThan,
            NumericLessThanEquals,
            NumericGreaterThan,
            NumericGreaterThanEquals,
        };
        let cases = [
            ("1", "1.1.1"),
            ("1.1.1", "1"),
            ("1.1.1", "1.1.1"),
        ];
        for (lhs, rhs) in cases {
            assert!(NumericEquals.matches(lhs, rhs).is_err());
            assert!(NumericNotEquals.matches(lhs, rhs).is_err());
            assert!(NumericLessThan.matches(lhs, rhs).is_err());
            assert!(NumericLessThanEquals.matches(lhs, rhs).is_err());
            assert!(NumericGreaterThan.matches(lhs, rhs).is_err());
            assert!(NumericGreaterThanEquals.matches(lhs, rhs).is_err());
        }
    }

    #[test]
    fn op_date_compare() {
        use ConditionOperator::{
            DateEquals,
            DateNotEquals,
            DateLessThan,
            DateLessThanEquals,
            DateGreaterThan,
            DateGreaterThanEquals,
        };
        let cases = [
            ("2020-04-01T00:00:01Z", "2020-04-01T00:00:02Z", true, false),
            ("2020-04-01T00:00:02Z", "2020-04-01T00:00:02Z", false, true),
            ("2020-04-01T00:00:03Z", "2020-04-01T00:00:02Z", false, false),
            ("2020-04-01T00:00:02+01:00", "2020-04-01T00:00:02Z", true, false),
            ("2020-04-01T00:00:02+00:00", "2020-04-01T00:00:02Z", false, true),
            ("2020-04-01T00:00:02-01:00", "2020-04-01T00:00:02Z", false, false),
        ];
        for (lhs, rhs, less_than, equals) in cases {
            assert_eq!(equals, DateEquals.matches(lhs, rhs).unwrap());
            assert_eq!(!equals, DateNotEquals.matches(lhs, rhs).unwrap());
            assert_eq!(less_than, DateLessThan.matches(lhs, rhs).unwrap());
            assert_eq!(less_than || equals, DateLessThanEquals.matches(lhs, rhs).unwrap());
            assert_eq!(!(less_than || equals), DateGreaterThan.matches(lhs, rhs).unwrap());
            assert_eq!(!less_than, DateGreaterThanEquals.matches(lhs, rhs).unwrap());
        }
    }

    #[test]
    fn op_date_invalid() {
        use ConditionOperator::{
            DateEquals,
            DateNotEquals,
            DateLessThan,
            DateLessThanEquals,
            DateGreaterThan,
            DateGreaterThanEquals,
        };
        // Values missing timezones are invalid
        let cases = [
            ("2020-04-01T00:00:02", "2020-04-01T00:00:02Z"),
            ("2020-04-01T00:00:02Z", "2020-04-01T00:00:02"),
            ("2020-04-01T00:00:02", "2020-04-01T00:00:02"),
        ];
        for (lhs, rhs) in cases {
            assert!(DateEquals.matches(lhs, rhs).is_err());
            assert!(DateNotEquals.matches(lhs, rhs).is_err());
            assert!(DateLessThan.matches(lhs, rhs).is_err());
            assert!(DateLessThanEquals.matches(lhs, rhs).is_err());
            assert!(DateGreaterThan.matches(lhs, rhs).is_err());
            assert!(DateGreaterThanEquals.matches(lhs, rhs).is_err());
        }
    }

    #[test]
    fn op_bool_equals() {
        use ConditionOperator::Bool;
        let cases = [
            ("true", "true", true),
            ("true", "false", false),
            ("false", "true", false),
            ("false", "false", true),
        ];
        for (lhs, rhs, equals) in cases {
            assert_eq!(equals, Bool.matches(lhs, rhs).unwrap());
        }
    }

    #[test]
    fn op_bool_invalid() {
        use ConditionOperator::Bool;
        let cases = [
            ("true", "tree"),
            ("tree", "true"),
            ("tree", "tree"),
        ];
        for (lhs, rhs) in cases {
            assert!(Bool.matches(lhs, rhs).is_err());
        }
    }

    #[test]
    fn op_binary_equals() {
        use ConditionOperator::BinaryEquals;
        // TODO: Verify AWS allows padding to be omitted.
        let cases = [
            ("dGVzdA==", "dGVzdA==", true),
            ("dGVzdA==", "dGVzdA=", true),
            ("dGVzdA==", "dGVzdA", true),
            ("dGVzdA=", "dGVzdA==", true),
            ("dGVzdA", "dGVzdA==", true),
            ("dGVzdA=", "dGVzdA=", true),
            ("dGVzdA=", "dGVzdA", true),
            ("dGVzdA", "dGVzdA=", true),
            ("dGVzdA", "dGVzdA", true),
            ("dGVzdA==", "dGVzdC4=", false),
            ("dGVzdC4=", "dGVzdA==", false),
        ];
        for (lhs, rhs, equals) in cases {
            assert_eq!(equals, BinaryEquals.matches(lhs, rhs).unwrap());
        }
    }

    #[test]
    fn op_binary_invalid() {
        use ConditionOperator::BinaryEquals;
        let cases = [
            ("dGVzdA==", "dGVzdAB"),
            ("dGVzdAB", "dGVzdA=="),
            ("dGVzdAB", "dGVzdAB"),
        ];
        for (lhs, rhs) in cases {
            assert!(BinaryEquals.matches(lhs, rhs).is_err());
        }
    }

    #[test]
    fn op_ipaddress() {
        use ConditionOperator::{IpAddress, NotIpAddress};
        let cases = [
            ("203.0.113.64", "203.0.113.0/24", true),
            ("203.0.112.1", "203.0.113.0/24", false),
            ("203.0.114.1", "203.0.113.0/24", false),
            ("2001:DB8:1234:5678::1", "2001:DB8:1234:5678::/64", true),
            ("2001:DB8:1234:5678:FFFF:FFFF:FFFF:1", "2001:DB8:1234:5678::/64", true),
            ("2001:DB8:1234:5677::1", "2001:DB8:1234:5678::/64", false),
            ("2001:DB8:1234:5679::1", "2001:DB8:1234:5678::/64", false),
        ];
        for (lhs, rhs, contains) in cases {
            assert_eq!(contains, IpAddress.matches(lhs, rhs).unwrap());
            assert_ne!(contains, NotIpAddress.matches(lhs, rhs).unwrap());
        }
    }

    #[test]
    fn op_ipaddress_invalid() {
        use ConditionOperator::{IpAddress, NotIpAddress};
        let cases = [
            // 256 out of range
            ("256.0.113.64", "203.0.113.0/24"),
            ("203.0.113.64", "256.0.113.0/24"),
            // 33 not a valid netmask
            ("203.0.113.64", "203.0.113.0/33"),
            // Value can't be a CIDR
            ("203.0.113.64/31", "203.0.113.0/24"),
            // Can't have multiple :: in an address
            ("2001:DB8::1234:5678::1", "2001:DB8:1234:5678::/64"),
            ("2001:DB8:1234:5678::1", "2001:DB8::1234:5678::/64"),
            // 129 not a valid netmask
            ("2001:DB8:1234:5678::1", "2001:DB8:1234:5678::/129"),
            // Value can't be a CIDR
            ("2001:DB8:1234:5678::1/126", "2001:DB8:1234:5678::/64"),
        ];
        for (lhs, rhs) in cases {
            assert!(IpAddress.matches(lhs, rhs).is_err());
            assert!(NotIpAddress.matches(lhs, rhs).is_err());
        }
    }

    #[test]
    fn condition_set_string_equals() {
        let mut set = ConditionSet::new();
        set.insert((ConditionOperator::StringEquals, single_value("test:Property", "foo")));
        let values = single_value("test:Property", "foo");
        assert!(set.matches(&values).unwrap());

        let values = single_value("test:Property", "bar");
        assert!(!set.matches(&values).unwrap());

        let values = HashMap::new();
        assert!(set.matches(&values).is_err());
    }
}
