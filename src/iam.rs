use crate::aws::ARN;

use std::str::FromStr;

// Do these distinctions matter for evaluating policies?
// Would simple string matching be sufficient?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    AWS(ARN),
    Federated(String),
    Service(String),
    CanonicalUser(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionParseError {
    InvalidFormat,
}

impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Principal::AWS(arn) => f.write_fmt(format_args!("AWS: {}", arn)),
            Principal::Federated(id) => f.write_fmt(format_args!("Federated: {}", id)),
            Principal::Service(id) => f.write_fmt(format_args!("Service: {}", id)),
            Principal::CanonicalUser(id) => f.write_fmt(format_args!("CanonicalUser: {}", id)),
        }
    }
}

#[derive(Clone)]
pub struct Action {
    value: String,
    separator: usize,
}

impl Action {
    pub fn new(service: &str, action: &str) -> Self {
        let mut value = String::new();
        let separator = service.len();
        value.reserve(separator + action.len() + 1);
        value.push_str(service);
        value.push(':');
        value.push_str(action);
        Action{value, separator}
    }

    pub fn service(&self) -> &str {
        &self.value[..self.separator]
    }

    pub fn action(&self) -> &str {
        &self.value[self.separator + 1 ..]
    }
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

impl FromStr for Action {
    type Err = ActionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let separator = value.find(':').ok_or(ActionParseError::InvalidFormat)?;
        Ok(Action{value: value.into(), separator})
    }
}
