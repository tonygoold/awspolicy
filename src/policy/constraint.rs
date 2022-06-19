use crate::aws::ARN;
use crate::iam::{Action, Principal};
use regex::{escape, Regex};

fn pattern_matches(pattern: &str, target: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == target;
    }
    let mut i = pattern.split('*');
    let mut p = String::from('^');
    if let Some(s) = i.next() {
        p.push_str(&escape(s));
    } else {
        return false;
    };
    i.for_each(|s| {
        p.push_str(".*");
        p.push_str(&escape(s));
    });
    p.push('$');
    if let Ok(r) = Regex::new(&p) {
        r.is_match(target)
    } else {
        // TODO: Report error or crash
        false
    }
}

#[derive(Debug, Clone)]
pub enum ActionConstraint {
    Any,
    Pattern(Action),
}

impl ActionConstraint {
    pub fn matches(&self, action: &Action) -> bool {
        match self {
            Self::Any => true,
            Self::Pattern(pattern) => pattern_matches(pattern.service(), action.service()) && pattern_matches(pattern.action(), action.action()),
        }
    }
}

impl TryFrom<&json::JsonValue> for ActionConstraint {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let value = value.as_str()
            .ok_or_else(|| json::Error::wrong_type("expected Action to be a string"))?;
        if value == "*" {
            return Ok(Self::Any);
        }
        Action::try_from(value).map(Self::Pattern)
            .map_err(|_| json::Error::wrong_type("expected Action to be an action pattern"))
    }
}

// TODO: You can specify multiple principals, including of different types.
#[derive(Debug, Clone)]
pub enum PrincipalConstraint {
    Any,
    AWSAny,
    Pattern(Principal),
}

#[derive(Debug, Clone)]
pub enum ResourceConstraint {
    Any,
    Pattern(ARN),
}

impl ResourceConstraint {
    pub fn matches(&self, resource: &ARN) -> bool {
        match self {
            Self::Any => true,
            Self::Pattern(pattern) => pattern_matches(pattern.raw(), resource.raw()),
        }
    }
}

impl TryFrom<&json::JsonValue> for ResourceConstraint {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let value = value.as_str()
            .ok_or_else(|| json::Error::wrong_type("expected Resource to be a string"))?;
        if value == "*" {
            return Ok(Self::Any);
        }
        ARN::try_from(value).map(Self::Pattern)
            .map_err(|_| json::Error::wrong_type("expected Resource to be an ARN pattern"))
    }
}

/*
Examples of Principal constraints:
{ "AWS": "arn:aws:iam::123456789012:root" }
{ "AWS": "arn:aws:iam::123456789012:user/user-name" }
{ "AWS": "arn:aws:iam::123456789012:role/role-name" }
{ "AWS": "123456789012" }
{ "AWS": "arn:aws:sts::123456789012:assumed-role/role-name/role-session-name" }
{ "AWS": "arn:aws:sts::123456789012:federated-user/user-name" }
{ "CanonicalUser": "79a59df900b949e55d96a1e698fbacedfd6e09d98eacf8f8d5218e7cd47ef2be" }
{
  "AWS": [
    "arn:aws:iam::123456789012:root",
    "999999999999"
  ],
  "CanonicalUser": "79a59df900b949e55d96a1e698fbacedfd6e09d98eacf8f8d5218e7cd47ef2be"
}
{ "Federated": "cognito-identity.amazonaws.com" }
{ "Federated": "www.amazon.com" }
{ "Federated": "graph.facebook.com" }
{ "Federated": "accounts.google.com" }
{ "Federated": "arn:aws:iam::AWS-account-ID:saml-provider/provider-name" }
{
    "Service": [
        "ecs.amazonaws.com",
        "elasticloadbalancing.amazonaws.com"
   ]
}

When you specify users in a Principal element, you cannot use a wildcard (*)
to mean "all users". Principals must always name specific users.

We strongly recommend that you do not use a wildcard (*) in the Principal
element of a resource-based policy with an Allow effect unless you intend to
grant public or anonymous access. Otherwise, specify intended principals,
services, or AWS accounts in the Principal element and then further restrict
access in the Condition element. This is especially true for IAM role trust
policies, because they allow other principals to become a principal in your
account.
 */
