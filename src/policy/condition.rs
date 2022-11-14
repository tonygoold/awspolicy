pub mod global;
pub mod nullable;
pub mod operator;
pub mod quantifier;

use crate::aws::ARN;
use quantifier::Quantifier;

use super::constraint::ResourceConstraint;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

use anyhow::anyhow;
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

fn arn_eq(lhs: &str, rhs: &str) -> anyhow::Result<bool> {
    let lhs: ARN = lhs.parse().map_err(|_| ConditionError::TypeMismatch)?;
    let rhs: ARN = rhs.parse().map_err(|_| ConditionError::TypeMismatch)?;
    Ok(lhs == rhs)
}

fn arn_like(value: &str, pattern: &str) -> anyhow::Result<bool> {
    let value: ARN = value.parse().map_err(|_| ConditionError::TypeMismatch)?;
    if pattern == "*" {
        return Ok(true);
    }
    let pattern = pattern.parse().map(ResourceConstraint::Pattern)
        .map_err(|_| ConditionError::TypeMismatch)?;
    Ok(pattern.matches(&value))
}

pub type ConditionValues = HashMap<String, Vec<String>>;

#[derive(Debug, Clone)]
pub struct ConditionList {
    conditions: HashMap<Quantifier, ConditionValues>,
}

impl ConditionList {
    pub fn new() -> Self {
        ConditionList{ conditions: HashMap::new() }
    }

    pub fn insert(&mut self, entry: (Quantifier, ConditionValues)) -> Option<ConditionValues> {
        let (op, values) = entry;
        self.conditions.insert(op, values)
    }

    pub fn matches(&self, value_map: &HashMap<String, Vec<String>>) -> anyhow::Result<bool> {
        self.conditions.iter().try_fold(true, |result, (op, target_map)| {
            // Short-circuit on the first failure to match
            if !result {
                return Ok(result);
            }

            target_map.iter().try_fold(true, |result, (key, targets)| {
                // Short-circuit on the first failure to match
                if !result {
                    return Ok(result);
                }

                let values = value_map.get(key);
                op.matches(values, targets)
            })
        })
    }
    fn try_from_values(values: &json::JsonValue) -> anyhow::Result<ConditionValues> {
        values.entries().map(|(key, values)| {
            if let Some(s) = values.as_str() {
                return Ok((key.to_string(), vec![s.to_string()]));
            }
            values.members().map(|value| {
                value.as_str()
                    .ok_or_else(|| anyhow!("expected condition values to be strings"))
                    .map(String::from)

            }).collect::<anyhow::Result<Vec<_>>>().map(|values| (key.to_string(), values))
        }).collect()
    }
}

impl Default for ConditionList {
    fn default() -> Self { ConditionList::new() }
}

impl TryFrom<&json::JsonValue> for ConditionList {
    type Error = anyhow::Error;

    fn try_from(value: &json::JsonValue) -> anyhow::Result<Self> {
        value.entries().map(|(key, value)| {
            let mut op_str = key;
            // The default for single-valued is to assume ForAny
            let mut for_any = true;
            if let Some(op) = key.strip_suffix("IfExists") {
                op_str = op;
                for_any = false;
            }

            if let Some(op) = key.strip_prefix("ForAny:") {
                op_str = op;
                for_any = true;
            } else if let Some(op) = key.strip_prefix("ForAll:") {
                op_str = op;
                for_any = false;
            }

            let operator = op_str.parse()?;
            let is_null = op_str == "Null";
            let values = Self::try_from_values(value)?;
            let quant = match (for_any, is_null) {
                (_, true) => Quantifier::Null,
                (true, _) => Quantifier::ForAnyValue(operator),
                (false, _) => Quantifier::ForAllValues(operator),
            };
            Ok((quant, values))
        }).collect::<Result<HashMap<_, _>, _>>()
            .map(|conditions| ConditionList { conditions })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{ConditionList, ConditionValues};
    use super::operator::Operator;
    use super::quantifier::Quantifier;

    fn single_value(key: &str, value: &str) -> ConditionValues {
        ConditionValues::from([(key.to_string(), vec![value.to_string()])])
    }

    #[test]
    fn op_string_equals() {
        let cases = [
            (Operator::StringEquals, true),
            (Operator::StringNotEquals, false),
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
            (Operator::StringEqualsIgnoreCase, true),
            (Operator::StringNotEqualsIgnoreCase, false),
        ];
        for (op, expected) in cases {
            assert_eq!(expected, op.matches("test", "test").unwrap());
            assert_eq!(expected, op.matches("TEST", "test").unwrap());
        }
    }

    #[test]
    fn op_string_like() {
        let cases = [
            (Operator::StringLike, true),
            (Operator::StringNotLike, false),
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
        use Operator::{
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
        use Operator::{
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
        use Operator::{
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
        use Operator::{
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
        use Operator::Bool;
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
        use Operator::Bool;
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
        use Operator::BinaryEquals;
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
        use Operator::BinaryEquals;
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
        use Operator::{IpAddress, NotIpAddress};
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
        use Operator::{IpAddress, NotIpAddress};
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
    fn op_arn() {
        use Operator::{ArnEquals, ArnNotEquals, ArnLike, ArnNotLike};
        let cases = [
            ("arn:aws:iam::123456789012:user/Alice", "arn:aws:iam::123456789012:user/Alice", true, true),
            ("arn:aws:iam::123456789012:user/Alice", "arn:aws:iam::123456789012:user/Bob", false, false),
            ("arn:aws:iam::123456789012:user/Alice", "arn:aws:iam::123456789012:user/*", false, true),
            ("arn:aws:iam::123456789012:user/Alice", "arn:aws:iam::*:user/Bob", false, false),
            ("arn:aws:iam::123456789012:user/Alice", "arn:aws:iam::*:user/Alice", false, true),
            // Not sure this counts as valid. It should never happen in practice.
            ("arn:aws:iam::*:user/Alice", "arn:aws:iam::*:user/Alice", true, true),
        ];
        for (lhs, rhs, equals, like) in cases {
            assert_eq!(equals, ArnEquals.matches(lhs, rhs).unwrap());
            assert_ne!(equals, ArnNotEquals.matches(lhs, rhs).unwrap());
            assert_eq!(like, ArnLike.matches(lhs, rhs).unwrap());
            assert_ne!(like, ArnNotLike.matches(lhs, rhs).unwrap());
        }
    }

    #[test]
    fn condition_list_string_equals() {
        let mut set = ConditionList::new();
        let quant = Quantifier::ForAnyValue(Operator::StringEquals);
        set.insert((quant, single_value("test:Property", "foo")));
        let values = single_value("test:Property", "foo");
        assert!(set.matches(&values).unwrap());

        let values = single_value("test:Property", "bar");
        assert!(!set.matches(&values).unwrap());

        let values = HashMap::new();
        assert!(!set.matches(&values).unwrap());
    }
}
