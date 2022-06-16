use crate::aws::ARN;
use crate::iam::Action;

#[derive(Debug, Clone)]
pub enum ActionConstraint {
    Any,
    Pattern(Action),
}

impl TryFrom<&json::JsonValue> for ActionConstraint {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let value = value.as_str()
            .ok_or_else(|| json::Error::wrong_type("expected Action to be a string"))?;
        if value == "*" {
            return Ok(ActionConstraint::Any);
        }
        Action::try_from(value).map(ActionConstraint::Pattern)
            .map_err(|_| json::Error::wrong_type("expected Action to be an action pattern"))
    }
}

#[derive(Debug, Clone)]
pub enum ResourceConstraint {
    Any,
    Pattern(ARN),
}

impl TryFrom<&json::JsonValue> for ResourceConstraint {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let value = value.as_str()
            .ok_or_else(|| json::Error::wrong_type("expected Resource to be a string"))?;
        if value == "*" {
            return Ok(ResourceConstraint::Any);
        }
        ARN::try_from(value).map(ResourceConstraint::Pattern)
            .map_err(|_| json::Error::wrong_type("expected Resource to be an ARN pattern"))
    }
}
