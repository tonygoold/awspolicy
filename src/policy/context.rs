use std::collections::HashMap;

use crate::aws::ARN;

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

    fn try_context_from(value: &json::JsonValue) -> json::Result<ResourceContext> {
        value.entries().map(|(key, value)| {
            let values = if let Some(value) = value.as_str() {
                Ok(vec![value.to_string()])
            } else if value.is_array() {
                value.members().map(|value| value.as_str().map(String::from).ok_or_else(|| json::Error::wrong_type("expected array of string values")))
                    .collect::<json::Result<Vec<_>>>()
            } else {
                Err(json::Error::wrong_type("expected resource property to be a string or array of strings"))
            }?;
            Ok((key.to_string(), values))
        }).collect::<json::Result<HashMap<_, _>>>()
    }

    fn try_resources_from(value: &json::JsonValue) -> json::Result<HashMap<ARN, ResourceContext>> {
        if value.is_null() {
            return Ok(HashMap::new());
        } else if !value.is_object() {
            return Err(json::Error::wrong_type("expected resources to be an object"));
        }

        value.entries().map(|(key, value)| {
            let arn = ARN::try_from(key)
                .map_err(|_| json::Error::wrong_type("expected an ARN"))?;
            let context = Self::try_context_from(value)?;
            Ok((arn, context))
        }).collect::<json::Result<HashMap<_, _>>>()
    }
}

impl TryFrom<&json::JsonValue> for Context {
    type Error = json::Error;

    fn try_from(value: &json::JsonValue) -> json::Result<Self> {
        if !value.is_object() {
            return Err(json::Error::wrong_type("expected object at root of context"));
        }
        let global = Self::try_context_from(&value["global"])?;
        let resources = Self::try_resources_from(&value["resources"])?;
        Ok(Context{ global, resources })
    }
}

impl TryFrom<&str> for Context {
    type Error = json::Error;

    fn try_from(value: &str) -> json::Result<Self> {
        let value = json::parse(value)?;
        Self::try_from(&value)
    }
}
