use super::constraint::{ActionConstraint, ResourceConstraint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub sid: String,
    pub effect: Effect,
    pub actions: Vec<ActionConstraint>,
    pub resources: Vec<ResourceConstraint>,
}

impl Statement {
    fn parse_actions(value: &json::JsonValue) -> json::Result<Vec<ActionConstraint>> {
        if value.is_string() {
            ActionConstraint::try_from(value).map(|action| vec![action])
        } else {
            value.members().map(ActionConstraint::try_from).collect::<Result<Vec<_>,_>>()
        }
    }

    fn parse_resources(value: &json::JsonValue) -> json::Result<Vec<ResourceConstraint>> {
        if value.is_string() {
            ResourceConstraint::try_from(value).map(|resource| vec![resource])
        } else {
            value.members().map(ResourceConstraint::try_from).collect::<Result<Vec<_>,_>>()
        }
    }
}

impl TryFrom<&json::JsonValue> for Statement {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let sid = value["Sid"].as_str()
            .ok_or_else(|| json::Error::wrong_type("expected Sid to be a string"))?
            .to_string();
        let effect = match value["Effect"].as_str() {
            Some("Allow") => Effect::Allow,
            Some("Deny") => Effect::Deny,
            Some(_) => return Err(json::Error::wrong_type("expected Effect to be Allow or Deny")),
            _ => return Err(json::Error::wrong_type("expected Effect to be a string")),
        };
        let actions = Self::parse_actions(&value["Action"])?;
        let resources = Self::parse_resources(&value["Resource"])?;
        Ok(Statement{sid, effect, actions, resources})
    }
}
