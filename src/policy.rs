use crate::iam::{ARN, Action};
use json;

const POLICY: &str = r#"{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "VisualEditor0",
            "Effect": "Allow",
            "Action": "route53:ChangeResourceRecordSets",
            "Resource": "arn:aws:route53:::hostedzone/*"
        },
        {
            "Sid": "VisualEditor1",
            "Effect": "Allow",
            "Action": [
                "route53:ListHostedZones",
                "route53:ListHostedZonesByName"
            ],
            "Resource": "*"
        },
        {
            "Sid": "VisualEditor2",
            "Effect": "Allow",
            "Action": "route53:GetChange",
            "Resource": "arn:aws:route53:::change/*"
        }
    ]
}"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Allow,
    Deny,
}

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

#[derive(Debug, Clone)]
pub struct Policy {
    pub version: String,
    pub statements: Vec<Statement>,
}

impl TryFrom<&json::JsonValue> for Policy {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let version = value["Version"].as_str()
            .ok_or_else(|| json::Error::wrong_type("expected Version to be a string"))?
            .to_string();
        let statements = &value["Statement"];
        // TODO: Statement or [Statement]
        if !statements.is_array() {
            return Err(json::Error::wrong_type("expected Statements to be an array"));
        }
        let statements = statements.members().map(Statement::try_from).collect::<Result<Vec<_>,_>>()?;
        Ok(Policy{version, statements})
    }
}

impl TryFrom<&str> for Policy {
    type Error = json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = json::parse(value)?;
        Policy::try_from(&value)
    }
}

pub fn sample_policy() -> Result<Policy, json::Error> {
    Policy::try_from(POLICY)
}
