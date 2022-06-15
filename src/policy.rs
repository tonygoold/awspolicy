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

#[derive(Debug, Clone)]
pub enum ResourceConstraint {
    Any,
    Pattern(ARN),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub sid: String,
    pub effect: Effect,
    pub action: ActionConstraint,
    pub resource: ResourceConstraint,
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
        // TODO: ActionConstraint or [ActionConstraint]
        let action = match value["Action"].as_str() {
            Some("*") => ActionConstraint::Any,
            Some(val) => Action::try_from(val).map(ActionConstraint::Pattern).map_err(|_| json::Error::wrong_type("expected Action to be an action pattern"))?,
            None => return Err(json::Error::wrong_type("expected Action to be a string")),
        };
        // TODO: ResourceConstraint or [ResourceConstraint]
        let resource = match value["Resource"].as_str() {
            Some("*") => ResourceConstraint::Any,
            Some(val) => ARN::try_from(val).map(ResourceConstraint::Pattern).map_err(|_| json::Error::wrong_type("expected Resource to be an ARN pattern"))?,
            None => return Err(json::Error::wrong_type("expected Resource to be a string")),
        };
        Ok(Statement{sid, effect, action, resource})
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
