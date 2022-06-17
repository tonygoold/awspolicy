#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ARNParseError {
    InvalidFormat,
    MissingPrefix,
}

#[derive(Clone)]
pub struct ARN {
    value: String,
    separators: Vec<usize>,
}

impl ARN {
    pub fn new(service: &str, region: &str, account: &str, resource: &str) -> Self {
        let (sep0, sep1) = (3, 7);
        let sep2 = sep1 + 1 + service.len();
        let sep3 = sep2 + 1 + region.len();
        let sep4 = sep3 + 1 + account.len();
        let separators = vec![sep0, sep1, sep2, sep3, sep4];
        let mut value = String::new();
        value.reserve(sep4 + 1 + resource.len());
        value.push_str("arn:aws:");
        value.push_str(service);
        value.push(':');
        value.push_str(region);
        value.push(':');
        value.push_str(account);
        value.push(':');
        value.push_str(resource);
        ARN {value, separators}
    }

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

    pub fn raw(&self) -> &str {
        &self.value
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
