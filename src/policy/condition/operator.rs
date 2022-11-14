use crate::aws::glob_matches;
use super::{
  cmp_numbers,
  cmp_dates,
  bools_eq,
  base64s_eq,
  ip_in_cidr,
  arn_eq,
  arn_like,
};

use std::cmp::Ordering;
use std::ops::Not;
use std::str::FromStr;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Operator {
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

    // The Null condition is omitted here because it is treated as a
    // quantifier, similar to ...IfExists.
}

impl Operator {
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

            Self::Bool => bools_eq(value, target),

            Self::BinaryEquals => base64s_eq(value, target),

            Self::IpAddress => ip_in_cidr(value, target),
            Self::NotIpAddress => ip_in_cidr(value, target).map(bool::not),

            Self::ArnEquals => arn_eq(value, target),
            Self::ArnLike => arn_like(value, target),
            Self::ArnNotEquals => arn_eq(value, target).map(bool::not),
            Self::ArnNotLike => arn_like(value, target).map(bool::not),
        }
    }
}

impl FromStr for Operator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let op = match s {
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
            _ => return Err(anyhow!("unrecognized condition operator")),
        };
        Ok(op)
    }
}
