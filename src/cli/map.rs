#![cfg(not(target_os = "windows"))]

use super::util::{convert_json_value_to_nu_value, convert_nu_value_to_json_value};
use crate::state::State;
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::StreamExt;
use jq_rs;
use nu_cli::{CommandArgs, OutputStream};
use nu_errors::ShellError;
use nu_protocol::{ReturnSuccess, Signature, SyntaxShape};
use nu_source::Tag;
use serde_json::{Map as JsonMap, Value};
use std::sync::Arc;

pub struct Map {
    state: Arc<State>,
}

impl Map {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl nu_cli::WholeStreamCommand for Map {
    fn name(&self) -> &str {
        "map"
    }

    fn signature(&self) -> Signature {
        Signature::build("map").required(
            "pattern",
            SyntaxShape::String,
            "the transformation pattern, using jq syntax",
        )
    }

    fn usage(&self) -> &str {
        "Map from one table structure to another. Much flexible, so wow."
    }

    async fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        map(self.state.clone(), args).await
    }
}

async fn map(_state: Arc<State>, args: CommandArgs) -> Result<OutputStream, ShellError> {
    let mut args = args.evaluate_once().await?;

    let pattern = args.nth(0).unwrap().as_string()?;

    let stream = stream! {
        while let Some(item) = args.input.next().await {
            if let Ok(converted_json_value) = convert_nu_value_to_json_value(&item) {
                let encoded_json = serde_json::to_string(&converted_json_value);
                if encoded_json.is_err() {
                    yield Err(ShellError::unexpected("Could not turn json value into encoded format"));
                }
                let modified = jq_rs::run(&pattern, encoded_json.unwrap().as_str());
                if modified.is_err() {
                    yield Err(ShellError::unexpected("Could not run map operation, likely the pattern is malformed or unsupported"));
                }
                let modified_json_value: Result<JsonMap<String, Value>, serde_json::Error> = serde_json::from_str(modified.unwrap().as_str());
                if modified_json_value.is_err() {
                    yield Err(ShellError::unexpected("Could not turn mapped data back into tabular format, use a different pattern"));
                }
                if let Ok(decoded) = convert_json_value_to_nu_value(&Value::Object(modified_json_value.unwrap()), Tag::default()){
                    yield ReturnSuccess::value(decoded)
                } else{
                    yield Err(ShellError::unexpected("Could not convert encoded value into nu value"));
                }
            } else {
                yield Err(ShellError::unexpected("Could not convert nu value into encoded format"));
            }
        }
    };
    Ok(OutputStream::new(stream))
}
