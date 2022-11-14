use super::operator::Operator;

use anyhow::anyhow;

/*
In this implementation, ...IfExists is represented by ForAnyValue, since they
are functionally equivalent.
 */

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Quantifier {
	// Returns true if every value for the context key is true.
	// This is trivially true if there are no values or the value resolves to
	// a null data set. Use of ForAllValues with Allow is discouraged because
	// it is overly permissive.
	ForAllValues(Operator),
	// Returns true if at least one value in the context key is true. This
	// also represents ...IfExists for single-valued keys.
	ForAnyValue(Operator),
	// Returns true if the emptiness of the set matches the condition target.
	Null,
}

impl Quantifier {
	pub fn matches(&self, values: Option<&Vec<String>>, targets: &Vec<String>) -> anyhow::Result<bool> {
		match self {
			Self::ForAllValues(op) => matches_all(op, values, targets),
			Self::ForAnyValue(op) => matches_any(op, values, targets),
			Self::Null => matches_null(values, targets),
		}
	}

}

fn matches_all(op: &Operator, values: Option<&Vec<String>>, targets: &Vec<String>) -> anyhow::Result<bool> {
	let values = match values {
		Some(v) => v,
		None => return Ok(true),
	};
	values.iter().try_fold(true, |result, value| {
		if !result {
			return Ok(result);
		}
		targets.iter().try_fold(false, |found, target| {
			if found {
				Ok(found)
			} else {
				op.matches(value, target)
			}
		})
	})
}

fn matches_any(op: &Operator, values: Option<&Vec<String>>, targets: &Vec<String>) -> anyhow::Result<bool> {
	let values = match values {
		Some(v) => v,
		None => return Ok(false),
	};
	values.iter().try_fold(false, |result, value| {
		if result {
			return Ok(result);
		}
		targets.iter().try_fold(false, |found, target| {
			if found {
				Ok(found)
			} else {
				op.matches(value, target)
			}
		})
	})
}

fn matches_null(values: Option<&Vec<String>>, targets: &Vec<String>) -> anyhow::Result<bool> {
	if targets.len() == 1 {
		Ok(values.is_none() == (&targets[0] == "true"))
	} else {
		Err(anyhow!("Null condition must take exactly one argument"))
	}
}

#[cfg(test)]
mod test {
	use super::Quantifier;
	use super::super::operator::Operator;

	#[test]
	fn forall_empty() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAllValues(op);
		let targets = vec!["a".to_string()];
		assert!(quant.matches(None, &targets).unwrap());
	}

	#[test]
	fn forall_single_target_all() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAllValues(op);
		let targets = vec!["a".to_string()];
		let values = vec!["a".to_string(), "a".to_string()];
		assert!(quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forall_single_target_not_all() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAllValues(op);
		let targets = vec!["a".to_string()];
		let values = vec!["a".to_string(), "b".to_string()];
		assert!(! quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forall_multi_targets_all() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAllValues(op);
		let targets = vec!["a".to_string(), "b".to_string(), "c".to_string()];
		let values = vec!["a".to_string(), "b".to_string()];
		assert!(quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forall_multi_targets_not_all() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAllValues(op);
		let targets = vec!["a".to_string(), "b".to_string()];
		let values = vec!["a".to_string(), "b".to_string(), "c".to_string()];
		assert!(! quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forany_empty() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAnyValue(op);
		let targets = vec!["a".to_string()];
		assert!(! quant.matches(None, &targets).unwrap());
	}

	#[test]
	fn forany_single_target_some() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAnyValue(op);
		let targets = vec!["a".to_string()];
		let values = vec!["a".to_string(), "b".to_string()];
		assert!(quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forany_single_target_none() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAnyValue(op);
		let targets = vec!["a".to_string()];
		let values = vec!["b".to_string(), "c".to_string()];
		assert!(! quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forany_multi_target_some() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAnyValue(op);
		let targets = vec!["a".to_string(), "b".to_string()];
		let values = vec!["a".to_string(), "b".to_string(), "c".to_string()];
		assert!(quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn forany_multi_target_none() {
		let op = Operator::StringEquals;
		let quant = Quantifier::ForAnyValue(op);
		let targets = vec!["a".to_string(), "b".to_string()];
		let values = vec!["c".to_string(), "d".to_string(), "e".to_string()];
		assert!(! quant.matches(Some(&values), &targets).unwrap());
	}

	#[test]
	fn null_checks_empty() {
		let quant = Quantifier::Null;
		let target_true = vec!["true".to_string()];
		let target_false = vec!["false".to_string()];
		let non_empty = Some(vec!["value".to_string()]);
		assert!(quant.matches(None, &target_true).unwrap());
		assert!(quant.matches(non_empty.as_ref(), &target_false).unwrap());
		assert!(! quant.matches(None, &target_false).unwrap());
		assert!(! quant.matches(non_empty.as_ref(), &target_true).unwrap());
	}

	#[test]
	fn null_takes_single_target() {
		let quant = Quantifier::Null;
		let targets_zero = Vec::<String>::new();
		let targets_multi = vec!["a".to_string(), "b".to_string()];
		assert!(quant.matches(None, &targets_zero).is_err());
		assert!(quant.matches(None, &targets_multi).is_err());
	}
}
