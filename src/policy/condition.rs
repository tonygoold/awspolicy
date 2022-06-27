use crate::aws::glob_matches;
use std::collections::HashMap;
use json;

// "Condition" : { "{condition-operator}" : { "{condition-key}" : "{condition-value}" }}
/*
"Condition": {
    "StringEquals": {
        "foo": "bar",
        "baz": ["alpha", "beta", "gamma"]
    }
}
 */

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
    pub fn matches(&self, target: &str, value: &str) -> bool {
        match *self {
            Self::StringEquals => target == value,
            Self::StringNotEquals => target != value,
            Self::StringEqualsIgnoreCase => target.to_lowercase() == value.to_lowercase(),
            Self::StringNotEqualsIgnoreCase => target.to_lowercase() != value.to_lowercase(),
            Self::StringLike => glob_matches(target, value),
            Self::StringNotLike => !glob_matches(target, value),

            Self::NumericEquals => false,
            Self::NumericNotEquals => false,
            Self::NumericLessThan => false,
            Self::NumericLessThanEquals => false,
            Self::NumericGreaterThan => false,
            Self::NumericGreaterThanEquals => false,

            Self::DateEquals => false,
            Self::DateNotEquals => false,
            Self::DateLessThan => false,
            Self::DateLessThanEquals => false,
            Self::DateGreaterThan => false,
            Self::DateGreaterThanEquals => false,

            Self::Bool => false,

            Self::BinaryEquals => false,

            Self::IpAddress => false,
            Self::NotIpAddress => false,

            Self::ArnEquals => false,
            Self::ArnLike => false,
            Self::ArnNotEquals => false,
            Self::ArnNotLike => false,

            // Condition value must be "true" or "false"
            Self::Null => false,
        }
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
    // TODO: Genericize value_map parameter
    pub fn matches(&self, value_map: &HashMap<String, Vec<String>>) -> bool {
        self.conditions.iter().all(|(op, target_map)| {
            target_map.iter().all(|(key, targets)| {
                let values = match value_map.get(key) {
                    Some(values) => if *op == ConditionOperator::Null {
                        return &targets[0] == "false";
                    } else {
                        values
                    }
                    None => if *op == ConditionOperator::Null {
                        return &targets[0] == "true";
                    } else {
                        // TODO: Handle ...IfExists suffix
                        return false;
                    }
                };
                // TODO: Handle ForAllValues:/ForAnyValues: prefixes
                if values.len() != 1 {
                    return false;
                }
                let value = &values[0];
                targets.iter().any(|target| {
                    op.matches(target, value)
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
