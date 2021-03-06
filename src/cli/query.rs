use super::util::convert_couchbase_rows_json_to_nu_stream;
use crate::state::State;
use async_trait::async_trait;
use couchbase::QueryOptions;
use log::debug;
use nu_cli::{CommandArgs, OutputStream};
use nu_errors::ShellError;
use nu_protocol::{Signature, SyntaxShape};
use std::sync::Arc;

pub struct Query {
    state: Arc<State>,
}

impl Query {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl nu_cli::WholeStreamCommand for Query {
    fn name(&self) -> &str {
        "query"
    }

    fn signature(&self) -> Signature {
        Signature::build("query").required("statement", SyntaxShape::String, "the query statement")
    }

    fn usage(&self) -> &str {
        "Performs a n1ql query"
    }

    async fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        run(self.state.clone(), args).await
    }
}

async fn run(state: Arc<State>, args: CommandArgs) -> Result<OutputStream, ShellError> {
    let args = args.evaluate_once().await?;
    let ctrl_c = args.ctrl_c.clone();
    let statement = args.nth(0).expect("need statement").as_string()?;

    debug!("Running n1ql query {}", &statement);
    let result = state
        .active_cluster()
        .cluster()
        .query(statement, QueryOptions::default())
        .await;

    match result {
        Ok(mut r) => convert_couchbase_rows_json_to_nu_stream(ctrl_c, r.rows()),
        Err(e) => Err(ShellError::untagged_runtime_error(format!("{}", e))),
    }
}
