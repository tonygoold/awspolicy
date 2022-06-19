mod condition;
mod constraint;
mod statement;

use crate::aws::ARN;
use crate::iam::{Action, Principal};
use statement::{CheckResult, Statement};
use json;

// This was an earlier version of the policy language. You might see this
// version on older existing policies. Do not use this version for any new
// policies or when you update any existing policies. Newer features, such as
// policy variables, will not work with your policy. For example, variables
// such as ${aws:username} aren't recognized as variables and are instead
// treated as literal strings in the policy.
pub const VERSION_2008_10_17: &str = "2008-10-17";

// This is the current version of the policy language, and you should always
// include a Version element and set it to 2012-10-17. Otherwise, you cannot
// use features such as policy variables that were introduced with this
// version.
pub const VERSION_2012_10_17: &str = "2012-10-17";

/*
See https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html
for the JSON policy grammar and https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements.html
for a description of each element.
 */

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
    pub version: Option<String>,
    pub id: Option<String>,
    pub statements: Vec<Statement>,
}

impl Policy {
    pub fn check_action(&self, action: &Action, resource: &ARN) -> CheckResult {
        self.statements.iter().fold(CheckResult::Unspecified, |result, stmt| {
            if result == CheckResult::Unspecified {
                stmt.check_action(action, resource)
            } else {
                result
            }
        })
    }

    pub fn check(&self, principal: &Principal, action: &Action, resource: &ARN) -> CheckResult {
        self.statements.iter().fold(CheckResult::Unspecified, |result, stmt| {
            if result == CheckResult::Unspecified {
                stmt.check(principal, action, resource)
            } else {
                result
            }
        })
    }
}

impl TryFrom<&json::JsonValue> for Policy {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let version = &value["Version"];
        let version = if let Some(v) = version.as_str() {
            // TODO: Introduce proper error type (or use a crate like anyhow)
            match v {
                VERSION_2008_10_17 | VERSION_2012_10_17 => Some(v.to_string()),
                _ => return Err(json::Error::wrong_type("unsupported Version")),
            }
        } else if version.is_null() {
            None
        } else {
            return Err(json::Error::wrong_type("expected Version to be a string"));
        };
        let id = value["Id"].as_str().map(|s| s.to_string());
        let statements = &value["Statement"];
        let statements = if statements.is_object() {
            Statement::try_from(statements).map(|statement| vec![statement])?
        } else if statements.is_array() {
            statements.members().map(Statement::try_from).collect::<Result<Vec<_>,_>>()?
        } else {
            return Err(json::Error::wrong_type("expected Statements to be an object or array"));
        };
        Ok(Policy{version, id, statements})
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
