use super::operator::Operator;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Nullable {
	// The value must be non-null and match the operator
	Expect(Operator),
	// If the value is null, consider it a match, otherwise match the operator
	IfExists(Operator),
	// Only the value's existence is checked
	IsNull,
}

impl Nullable {
	pub fn matches(&self, value: Option<&str>, target: &str) -> anyhow::Result<bool> {
		match *self {
			Self::Expect(operator) => if let Some(value) = value {
				operator.matches(value, target)
			} else {
				Ok(false)
			}
			Self::IsNull => match target {
				"true" => Ok(value.is_none()),
				"false" => Ok(value.is_some()),
				_ => Err(anyhow!("Null target must be either 'true' or 'false'")),
			},
			Self::IfExists(operator) => if let Some(value) = value {
				operator.matches(value, target)
			} else {
				Ok(true)
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::Nullable;
	use super::super::operator::Operator;

	#[test]
	fn expect_is_required() {
		let op = Nullable::Expect(Operator::StringEquals);
		assert!(! op.matches(None, "target").unwrap());
	}

	#[test]
	fn expect_evaluates_operator() {
		let op = Nullable::Expect(Operator::StringEquals);
		assert!(op.matches(Some("target"), "target").unwrap());
		assert!(! op.matches(Some("other"), "target").unwrap());
	}

	#[test]
	fn if_exists_is_optional() {
		let op = Nullable::IfExists(Operator::StringEquals);
		assert!(op.matches(None, "target").unwrap());
	}

	#[test]
	fn if_exists_evaluates_operator() {
		let op = Nullable::IfExists(Operator::StringEquals);
		assert!(op.matches(Some("target"), "target").unwrap());
		assert!(! op.matches(Some("other"), "target").unwrap());
	}

	#[test]
	fn is_null_checks_existence() {
		let op = Nullable::IsNull;
		assert!(op.matches(None, "true").unwrap());
		assert!(op.matches(Some("value"), "false").unwrap());
		assert!(! op.matches(None, "false").unwrap());
		assert!(! op.matches(Some("value"), "true").unwrap());
	}
}
