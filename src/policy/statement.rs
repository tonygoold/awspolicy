use crate::aws::ARN;
use crate::iam::{Action, Principal};
use super::condition::ConditionSet;
use super::constraint::{ActionConstraint, PrincipalConstraint, ResourceConstraint};
use super::context::Context;

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
pub enum PrincipalClause {
    None,
    Principal(Vec<PrincipalConstraint>),
    NotPrincipal(Vec<PrincipalConstraint>),
}

#[derive(Debug, Clone)]
pub enum ActionClause {
    Action(Vec<ActionConstraint>),
    NotAction(Vec<ActionConstraint>),
}

#[derive(Debug, Clone)]
pub enum ResourceClause {
    Resource(Vec<ResourceConstraint>),
    NotResource(Vec<ResourceConstraint>),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub sid: Option<String>,
    pub effect: Effect,
    pub principals: PrincipalClause,
    pub actions: ActionClause,
    pub resources: ResourceClause,
    pub conditions: Option<ConditionSet>,
}

impl Statement {
    fn matches_conditions(&self, resource: &ARN, context: &Context) -> anyhow::Result<bool> {
        let conditions = match &self.conditions {
            Some(conditions) => conditions,
            None => return Ok(true),
        };
        let mut key_values = context.globals().clone();
        if let Some(rsrc_values) = context.resource(resource) {
            key_values.extend(rsrc_values.clone().into_iter());
        }
        let matches = conditions.matches(&key_values)?;
        Ok(matches)
    }

    pub fn check_action(&self, action: &Action, resource: &ARN, context: &Context) -> anyhow::Result<CheckResult> {
        let matches_action = match &self.actions {
            ActionClause::Action(actions) => actions.iter().any(|constraint| constraint.matches(action)),
            ActionClause::NotAction(actions) => !actions.iter().any(|constraint| constraint.matches(action)),
        };
        if !matches_action {
            return Ok(CheckResult::Unspecified);
        }

        let matches_resource = match &self.resources {
            ResourceClause::Resource(resources) => resources.iter().any(|constraint| constraint.matches(resource)),
            ResourceClause::NotResource(resources) => !resources.iter().any(|constraint| constraint.matches(resource)),
        };
        if !matches_resource {
            return Ok(CheckResult::Unspecified);
        }

        if !self.matches_conditions(resource, context)? {
            return Ok(CheckResult::Unspecified);
        }

        Ok(match self.effect {
            Effect::Allow => CheckResult::Allow,
            Effect::Deny => CheckResult::Deny,
        })
    }

    pub fn check(&self, principal: &Principal, action: &Action, resource: &ARN, context: &Context) -> anyhow::Result<CheckResult> {
        let matches_principals = match &self.principals {
            PrincipalClause::None => true,
            PrincipalClause::Principal(principals) => principals.iter().any(|constraint| constraint.matches(principal)),
            PrincipalClause::NotPrincipal(principals) => !principals.iter().any(|constraint| constraint.matches(principal)),
        };
        if matches_principals {
            self.check_action(action, resource, context)
        } else {
            Ok(CheckResult::Unspecified)
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
            value.members().map(ActionConstraint::try_from).collect::<json::Result<Vec<_>>>()
        }
    }

    fn parse_aws_principal(value: &json::JsonValue) -> json::Result<PrincipalConstraint> {
        let value = value.as_str().ok_or_else(|| json::Error::wrong_type("expected AWS principal to be a string"))?;
        if value == "*" {
            return Ok(PrincipalConstraint::AWSAny);
        }
        let re = regex::Regex::new("^\\d+$").map_err(|_| json::Error::wrong_type("unable to compile regular expression"))?;
        let account = if re.is_match(value) {
            let mut arn = String::from("arn:aws:iam::");
            arn.push_str(value);
            arn.push_str(":root");
            Some(arn)
        } else {
            None
        };
        ARN::try_from(account.as_deref().unwrap_or(value)).map_err(|_| json::Error::wrong_type("expected AWS principal to be an ARN or '*'"))
            .map(Principal::AWS)
            .map(PrincipalConstraint::Pattern)
    }

    fn parse_aws_principals(value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
        if value.is_string() {
            Self::parse_aws_principal(value).map(|constraint| vec![constraint])
        } else {
            value.members().map(Self::parse_aws_principal).collect::<json::Result<Vec<_>>>()
        }
    }

    fn parse_federated_principal(value: &json::JsonValue) -> json::Result<PrincipalConstraint> {
        let value = value.as_str().ok_or_else(|| json::Error::wrong_type("expected Federated principal to be a string"))?;
        Ok(PrincipalConstraint::Pattern(Principal::Federated(value.to_string())))
    }

    fn parse_federated_principals(value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
        if value.is_string() {
            Self::parse_federated_principal(value).map(|constraint| vec![constraint])
        } else {
            value.members().map(Self::parse_federated_principal).collect::<json::Result<Vec<_>>>()
        }
    }

    fn parse_service_principal(value: &json::JsonValue) -> json::Result<PrincipalConstraint> {
        let value = value.as_str().ok_or_else(|| json::Error::wrong_type("expected Federated principal to be a string"))?;
        Ok(PrincipalConstraint::Pattern(Principal::Service(value.to_string())))
    }

    fn parse_service_principals(value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
        if value.is_string() {
            Self::parse_service_principal(value).map(|constraint| vec![constraint])
        } else {
            value.members().map(Self::parse_service_principal).collect::<json::Result<Vec<_>>>()
        }
    }

    fn parse_canonicaluser_principal(value: &json::JsonValue) -> json::Result<PrincipalConstraint> {
        let value = value.as_str().ok_or_else(|| json::Error::wrong_type("expected Federated principal to be a string"))?;
        Ok(PrincipalConstraint::Pattern(Principal::CanonicalUser(value.to_string())))
    }

    fn parse_canonicaluser_principals(value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
        if value.is_string() {
            Self::parse_canonicaluser_principal(value).map(|constraint| vec![constraint])
        } else {
            value.members().map(Self::parse_canonicaluser_principal).collect::<json::Result<Vec<_>>>()
        }
    }

    fn parse_principals(value: &json::JsonValue) -> json::Result<Vec<PrincipalConstraint>> {
        if value.is_null() {
            return Ok(vec![]);
        } else if let Some(s) = value.as_str() {
            match s {
                "*" => return Ok(vec![PrincipalConstraint::Any]),
                _ => return Err(json::Error::wrong_type("expected Principal to be non-string value except '*'")),
            }
        } else if !value.is_object() {
            return Err(json::Error::wrong_type("expected Principal to be an object when it is not '*'"));
        }

        value.entries().map(|(key, value)| {
            match key {
                "AWS" => Self::parse_aws_principals(value),
                "Federated" => Self::parse_federated_principals(value),
                "Service" => Self::parse_service_principals(value),
                "CanonicalUser" => Self::parse_canonicaluser_principals(value),
                _ => Err(json::Error::wrong_type("expected Principal to be *, AWS, Federated, Service, or CanonicalUser")),
            }
        }).fold(Ok(Vec::new()), |acc, value| {
            if let Ok(other) = value {
                acc.map(|mut constraints| {
                    constraints.extend_from_slice(&other);
                    constraints
                })
            } else {
                value
            }
        })
    }

    fn parse_resources(value: &json::JsonValue) -> json::Result<Vec<ResourceConstraint>> {
        if value.is_string() {
            ResourceConstraint::try_from(value).map(|resource| vec![resource])
        } else {
            value.members().map(ResourceConstraint::try_from).collect::<Result<Vec<_>,_>>()
        }
    }

    fn parse_conditions(value: &json::JsonValue) -> json::Result<Option<ConditionSet>> {
        if value.is_null() {
            Ok(None)
        } else if value.is_object() {
            ConditionSet::try_from(value).map(Some)
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
        // According to https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-reference-policy-checks.html#access-analyzer-reference-policy-checks-error-unsupported-element-combination
        // Principal/NotPrincipal, Action/NotAction, and Resource/NotResource
        // are mutually exclusive, and it is an error to include both.
        let action = &value["Action"];
        let not_action = &value["NotAction"];
        let actions = match(action.is_null(), not_action.is_null()) {
            (true, true) => return Err(json::Error::wrong_type("missing Action or NotAction")),
            (false, true) => ActionClause::Action(Self::parse_actions(action)?),
            (true, false) => ActionClause::NotAction(Self::parse_actions(action)?),
            (false, false) => return Err(json::Error::wrong_type("cannot have both Action and NotAction in same statement")),
        };
        let principal = &value["Principal"];
        let not_principal = &value["NotPrincipal"];
        let principals = match (principal.is_null(), not_principal.is_null()) {
            (true, true) => PrincipalClause::None,
            (false, true) => PrincipalClause::Principal(Self::parse_principals(principal)?),
            (true, false) => PrincipalClause::NotPrincipal(Self::parse_principals(not_principal)?),
            (false, false) => return Err(json::Error::wrong_type("cannot have both Principal and NotPrincipal in same statement")),
        };
        let resource = &value["Resource"];
        let not_resource = &value["NotResource"];
        let resources = match(resource.is_null(), not_resource.is_null()) {
            (true, true) => return Err(json::Error::wrong_type("missing Resource or NotResource")),
            (false, true) => ResourceClause::Resource(Self::parse_resources(resource)?),
            (true, false) => ResourceClause::NotResource(Self::parse_resources(not_resource)?),
            (false, false) => return Err(json::Error::wrong_type("cannot have both Resource and NotResource in same statement")),
        };
        let conditions = Self::parse_conditions(&value["Condition"])?;
        Ok(Statement{
            sid,
            effect,
            actions,
            principals,
            resources,
            conditions
        })
    }
}
