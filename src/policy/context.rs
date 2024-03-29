use crate::aws::ARN;

use std::collections::HashMap;
use std::str::FromStr;

use anyhow::anyhow;

pub type ResourceContext = HashMap<String, Vec<String>>;

pub struct Context {
    global: ResourceContext,
    resources: HashMap<ARN, ResourceContext>,
}

impl Context {
    pub fn new() -> Self {
        Context{
            global: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn globals(&self) -> &ResourceContext {
        &self.global
    }

    pub fn resource(&self, arn: &ARN) -> Option<&ResourceContext> {
        self.resources.get(arn)
    }

    fn try_context_from(value: &json::JsonValue) -> anyhow::Result<ResourceContext> {
        value.entries().map(|(key, value)| {
            let values = if let Some(value) = value.as_str() {
                Ok(vec![value.to_string()])
            } else if value.is_array() {
                value.members().map(|value| value.as_str().map(String::from).ok_or_else(|| anyhow!("expected array of string values")))
                    .collect::<anyhow::Result<Vec<_>>>()
            } else {
                Err(anyhow!("expected resource property to be a string or array of strings"))
            }?;
            Ok((key.to_string(), values))
        }).collect::<anyhow::Result<HashMap<_, _>>>()
    }

    fn try_resources_from(value: &json::JsonValue) -> anyhow::Result<HashMap<ARN, ResourceContext>> {
        if value.is_null() {
            return Ok(HashMap::new());
        } else if !value.is_object() {
            return Err(anyhow!("expected resources to be an object"));
        }

        value.entries().map(|(key, value)| {
            let arn: ARN = key.parse()
                .map_err(|_| anyhow!("expected an ARN"))?;
            let context = Self::try_context_from(value)?;
            Ok((arn, context))
        }).collect::<anyhow::Result<HashMap<_, _>>>()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<&json::JsonValue> for Context {
    type Error = anyhow::Error;

    fn try_from(value: &json::JsonValue) -> anyhow::Result<Self> {
        if !value.is_object() {
            return Err(anyhow!("expected object at root of context"));
        }
        let global = Self::try_context_from(&value["global"])?;
        let resources = Self::try_resources_from(&value["resources"])?;
        Ok(Context{ global, resources })
    }
}

impl FromStr for Context {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        let value = json::parse(value)?;
        Self::try_from(&value)
    }
}
