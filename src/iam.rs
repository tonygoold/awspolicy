#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalKind {
    User,
    Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ARNParseError {
    InvalidFormat,
    MissingPrefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionParseError {
    InvalidFormat,
}

#[derive(Clone)]
pub struct ARN {
    value: String,
    separators: Vec<usize>,
}

impl ARN {
    pub fn service(&self) -> &str {
        &self.value[self.separators[1] + 1 .. self.separators[2]]
    }

    pub fn region(&self) -> &str {
        &self.value[self.separators[2] + 1 .. self.separators[3]]
    }

    pub fn account(&self) -> &str {
        &self.value[self.separators[3] + 1 .. self.separators[4]]
    }

    pub fn resource(&self) -> &str {
        &self.value[self.separators[4] + 1 ..]
    }
}

impl std::fmt::Debug for ARN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

impl std::fmt::Display for ARN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

impl TryFrom<&str> for ARN {
    type Error = ARNParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !value.starts_with("arn:") {
            return Err(ARNParseError::MissingPrefix);
        }
        let separators: Vec<usize> = value.char_indices().filter_map(|(i, c)| {
            if c == ':' {
                Some(i)
            } else {
                None
            }
        }).collect();
        // "arn":"aws":service:region:account:resource
        if separators.len() != 5 {
            return Err(ARNParseError::InvalidFormat);
        }
        Ok(ARN{value: value.into(), separators})
    }
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
