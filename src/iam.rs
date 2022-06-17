use crate::aws::ARN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalKind {
    User,
    Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionParseError {
    InvalidFormat,
}

#[derive(Debug, Clone)]
pub struct Principal {
    pub arn: ARN,
    pub kind: PrincipalKind,
}

impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} ({:?})", &self.arn, self.kind))
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

impl TryFrom<&str> for Action {
    type Error = ActionParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let separator = value.find(':').ok_or(ActionParseError::InvalidFormat)?;
        Ok(Action{value: value.into(), separator})
    }
}
