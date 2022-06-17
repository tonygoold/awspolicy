use crate::aws::ARN;
use crate::iam::Action;
use super::constraint::{ActionConstraint, PrincipalConstraint, ResourceConstraint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    Allow,
    Deny,
    Unspecified,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub sid: String,
    pub effect: Effect,
    pub principals: Vec<PrincipalConstraint>,
    pub actions: Vec<ActionConstraint>,
    pub resources: Vec<ResourceConstraint>,
}

impl Statement {
    pub fn check(&self, action: &Action, resource: &ARN) -> CheckResult {
        let matches_action = self.actions.iter().any(|constraint| constraint.matches(action));
        if !matches_action {
            return CheckResult::Unspecified;
        }
        let matches_resource = self.resources.iter().any(|constraint| constraint.matches(resource));
        if !matches_resource {
            return CheckResult::Unspecified;
        }
        match self.effect {
            Effect::Allow => CheckResult::Allow,
            Effect::Deny => CheckResult::Deny,
        }
    }

    fn parse_actions(value: &json::JsonValue) -> json::Result<Vec<ActionConstraint>> {
        if value.is_string() {
            ActionConstraint::try_from(value).map(|action| vec![action])
        } else {
            value.members().map(ActionConstraint::try_from).collect::<Result<Vec<_>,_>>()
        }
    }

    fn parse_principals(value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
        // TODO: Implement this
        Ok(vec![])
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
        let principals = Self::parse_principals(&value["Principal"])?;
        let resources = Self::parse_resources(&value["Resource"])?;
        Ok(Statement{sid, effect, actions, principals, resources})
    }
}
