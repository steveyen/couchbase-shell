//! The `collections get` command fetches all of the collection names from the server.

use crate::state::State;
use couchbase::{CollectionSpec, CreateCollectionOptions};

use crate::cli::util::bucket_name_from_args;
use async_trait::async_trait;
use log::debug;
use nu_cli::{CommandArgs, OutputStream};
use nu_errors::ShellError;
use nu_protocol::{Signature, SyntaxShape};
use std::sync::Arc;
use std::time::Duration;

pub struct CollectionsCreate {
    state: Arc<State>,
}

impl CollectionsCreate {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl nu_cli::WholeStreamCommand for CollectionsCreate {
    fn name(&self) -> &str {
        "collections create"
    }

    fn signature(&self) -> Signature {
        Signature::build("collections create")
            .required_named(
                "name",
                SyntaxShape::String,
                "the name of the collection",
                None,
            )
            .named(
                "bucket",
                SyntaxShape::String,
                "the name of the bucket",
                None,
            )
            .named("scope", SyntaxShape::String, "the name of the scope", None)
            .named(
                "max-expiry",
                SyntaxShape::String,
                "the maximum expiry for documents in this collection",
                None,
            )
    }

    fn usage(&self) -> &str {
        "Creates collections through the HTTP API"
    }

    async fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        collections_create(self.state.clone(), args).await
    }
}

async fn collections_create(
    state: Arc<State>,
    args: CommandArgs,
) -> Result<OutputStream, ShellError> {
    let args = args.evaluate_once().await?;

    let collection = match args.get("name") {
        Some(v) => match v.as_string() {
            Ok(uname) => uname,
            Err(e) => return Err(e),
        },
        None => return Err(ShellError::unexpected("name is required")),
    };

    let bucket = bucket_name_from_args(&args, state.active_cluster())?;
    let scope_name = match args.get("scope").map(|c| c.as_string().ok()).flatten() {
        Some(name) => name,
        None => match state.active_cluster().active_scope() {
            Some(s) => s,
            None => {
                return Err(ShellError::untagged_runtime_error(format!(
                    "Could not auto-select a scope - please use --scope instead"
                )));
            }
        },
    };
    let expiry = match args.get("max-expiry") {
        Some(v) => match v.as_u64() {
            Ok(e) => e,
            Err(e) => return Err(e),
        },
        None => 0,
    };

    debug!(
        "Running collections create for {:?} on bucket {:?}, scope {:?}",
        &collection, &bucket, &scope_name
    );

    let mgr = state.active_cluster().bucket(bucket.as_str()).collections();
    let result = mgr
        .create_collection(
            CollectionSpec::new(collection, scope_name, Duration::from_secs(expiry)),
            CreateCollectionOptions::default(),
        )
        .await;

    match result {
        Ok(_) => Ok(OutputStream::empty()),
        Err(e) => Err(ShellError::untagged_runtime_error(format!("{}", e))),
    }
}
