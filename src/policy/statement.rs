use crate::aws::ARN;
use crate::iam::Action;
use super::condition::ConditionMap;
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
    pub sid: Option<String>,
    pub effect: Effect,
    pub principals: Vec<PrincipalConstraint>,
    pub actions: Vec<ActionConstraint>,
    pub resources: Vec<ResourceConstraint>,
    pub conditions: Option<ConditionMap>,
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
        if !self.conditions.as_ref().map_or(true, |conditions| conditions.matches()) {
            return CheckResult::Unspecified;
        }
        match self.effect {
            Effect::Allow => CheckResult::Allow,
            Effect::Deny => CheckResult::Deny,
        }
    }

    fn parse_effect(value: &json::JsonValue) -> json::Result<Effect> {
        match value.as_str() {
            Some("Allow") => Ok(Effect::Allow),
            Some("Deny") => Ok(Effect::Deny),
            Some(_) => Err(json::Error::wrong_type("expected Effect to be Allow or Deny")),
            _ => Err(json::Error::wrong_type("expected Effect to be a string")),
        }
    }

    fn parse_actions(value: &json::JsonValue) -> json::Result<Vec<ActionConstraint>> {
        if value.is_string() {
            ActionConstraint::try_from(value).map(|action| vec![action])
        } else {
            value.members().map(ActionConstraint::try_from).collect::<Result<Vec<_>,_>>()
        }
    }

    fn parse_principals(_value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
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

    fn parse_conditions(value: &json::JsonValue) -> json::Result<Option<ConditionMap>> {
        if value.is_null() {
            Ok(None)
        } else if value.is_object() {
            ConditionMap::try_from(value).map(Some)
        } else {
            Err(json::Error::wrong_type("expected Condition to be an object"))
        }
    }
}

impl TryFrom<&json::JsonValue> for Statement {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let sid = &value["Sid"];
        let sid = if let Some(s) = sid.as_str() {
            Some(s.to_string())
        } else if sid.is_null() {
            None
        } else {
            return Err(json::Error::wrong_type("expected Sid to be a string"));
        };
        let effect = Self::parse_effect(&value["Effect"])?;
        let actions = Self::parse_actions(&value["Action"])?;
        let principals = Self::parse_principals(&value["Principal"])?;
        let resources = Self::parse_resources(&value["Resource"])?;
        let conditions = Self::parse_conditions(&value["Condition"])?;
        Ok(Statement{sid, effect, actions, principals, resources, conditions})
    }
}
