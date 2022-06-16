mod constraint;
mod statement;

use statement::Statement;
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
        let statements = if statements.is_object() {
            Statement::try_from(statements).map(|statement| vec![statement])?
        } else if statements.is_array() {
            statements.members().map(Statement::try_from).collect::<Result<Vec<_>,_>>()?
        } else {
            return Err(json::Error::wrong_type("expected Statements to be an object or array"));
        };
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
