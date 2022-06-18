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

#[derive(Debug, Clone)]
pub struct Condition {
    keyvals: HashMap<String, Vec<String>>,
}

impl Condition {
    pub fn matches(&self) -> bool {
        self.keyvals.iter().all(|(_key, values)| {
            values.iter().any(|_value| true)
        })
    }
}

impl TryFrom<&json::JsonValue> for Condition {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        if !value.is_object() {
            return Err(json::Error::wrong_type("expected condition to be key-values"));
        }
        let keyvals = value.entries().map(|(k, v)| {
            match v.as_str() {
                // TODO: Handle multiple values
                Some(s) => Ok((k.to_string(), vec![s.to_string()])),
                None => Err(json::Error::wrong_type("expected value to be a string")),
            }
        }).collect::<Result<HashMap<_,_>,_>>()?;
        Ok(Condition{keyvals})
    }
}

#[derive(Debug, Clone)]
pub struct ConditionMap {
    operators: HashMap<String, Condition>,
}

impl ConditionMap {
    pub fn matches(&self) -> bool {
        self.operators.iter().all(|(_op, condition)| {
            condition.matches()
        })
    }
}

impl TryFrom<&json::JsonValue> for ConditionMap {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let operators = value.entries().map(|(k, v)| {
            Condition::try_from(v).map(|condition| (k.to_string(), condition))
        }).collect::<Result<_, _>>()?;
        Ok(ConditionMap{operators})
    }
}