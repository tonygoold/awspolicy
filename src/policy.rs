pub mod condition;
pub mod constraint;
pub mod context;
pub mod statement;

use crate::aws::ARN;
use crate::iam::{Action, Principal};
use context::Context;
use statement::{Effect, Statement};
use json;

pub use statement::CheckResult;

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

#[derive(Debug, Clone)]
pub struct Policy {
    pub version: Option<String>,
    pub id: Option<String>,
    pub statements: Vec<Statement>,
}

impl Policy {
    /*
    See https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html#policy-eval-denyallow
    */
    pub fn check_action(&self, action: &Action, resource: &ARN, context: &Context) -> anyhow::Result<CheckResult> {
        self.statements.iter().fold(Ok(CheckResult::Unspecified), |result, stmt| {
            match result {
                // An explicit deny in any policy overrides any allows
                Ok(CheckResult::Deny) => result,
                Ok(CheckResult::Unspecified) => stmt.check_action(action, resource, context),
                // If there is an explict allow, we only need to evaluate policies that would
                // override this with an explicit deny
                Ok(CheckResult::Allow) => if stmt.effect == Effect::Deny {
                    match stmt.check_action(action, resource, context) {
                        // An explicit deny overrides any other result
                        Ok(CheckResult::Deny) => Ok(CheckResult::Deny),
                        // The previous explicit allow takes precedence
                        Ok(_) => Ok(CheckResult::Allow),
                        Err(err) => Err(err),
                    }
                } else {
                    result
                }
                Err(_) => result,
            }
        })
    }

    pub fn check(&self, principal: &Principal, action: &Action, resource: &ARN, context: &Context) -> anyhow::Result<CheckResult> {
        self.statements.iter().fold(Ok(CheckResult::Unspecified), |result, stmt| {
            match result {
                // An explicit deny in any policy overrides any allows
                Ok(CheckResult::Deny) => result,
                Ok(CheckResult::Unspecified) => stmt.check(principal, action, resource, context),
                // If there is an explict allow, we only need to evaluate policies that would
                // override this with an explicit deny
                Ok(CheckResult::Allow) => if stmt.effect == Effect::Deny {
                    match stmt.check(principal, action, resource, context) {
                        // An explicit deny overrides any other result
                        Ok(CheckResult::Deny) => Ok(CheckResult::Deny),
                        // The previous explicit allow takes precedence
                        Ok(_) => Ok(CheckResult::Allow),
                        Err(err) => Err(err),
                    }
                } else {
                    result
                }
                Err(_) => result,
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
        Self::try_from(&value)
    }
}
