use crate::aws::{glob_matches, ARN};
use crate::iam::{Action, Principal};

#[derive(Debug, Clone)]
pub enum ActionConstraint {
    Any,
    Pattern(Action),
}

impl ActionConstraint {
    pub fn matches(&self, action: &Action) -> bool {
        match self {
            Self::Any => true,
            Self::Pattern(pattern) => glob_matches(pattern.service(), action.service()) && glob_matches(pattern.action(), action.action()),
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

impl PrincipalConstraint {
    fn matches_aws(arn: &ARN, other: &Principal) -> bool {
        if let Principal::AWS(other) = other {
            glob_matches(arn.raw(), other.raw())
        } else {
            false
        }
    }

    fn matches_federated(s: &str, other: &Principal) -> bool {
        if let Principal::Federated(other) = other {
            glob_matches(s, other)
        } else {
            false
        }
    }

    fn matches_service(s: &str, other: &Principal) -> bool {
        if let Principal::Service(other) = other {
            glob_matches(s, other)
        } else {
            false
        }
    }

    fn matches_canonicaluser(s: &str, other: &Principal) -> bool {
        if let Principal::CanonicalUser(other) = other {
            glob_matches(s, other)
        } else {
            false
        }
    }

    pub fn matches(&self, other: &Principal) -> bool {
        match self {
            Self::Any => true,
            Self::AWSAny => match other {
                Principal::AWS(_) => true,
                _ => false,
            }
            Self::Pattern(principal) => match principal {
                Principal::AWS(arn) => Self::matches_aws(arn, other),
                Principal::Federated(s) => Self::matches_federated(s, other),
                Principal::Service(s) => Self::matches_service(s, other),
                Principal::CanonicalUser(s) => Self::matches_canonicaluser(s, other),
            }
        }
    }
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
            Self::Pattern(pattern) => glob_matches(pattern.raw(), resource.raw()),
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
