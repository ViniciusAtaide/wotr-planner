//! Starting the server and calling its tools as a client would.

use std::sync::LazyLock;

use mcp::{Planner, Server};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ReadResourceRequestParams, ReadResourceResult, Resource,
    ServerCapabilities,
};
use rmcp::service::{RoleClient, RunningService, ServiceError};
use rmcp::{ServiceExt, serde_json};
use serde_json::{Value, json};

/// The real game data, read and checked once for every test.
static PLANNER: LazyLock<Planner> =
    LazyLock::new(|| Planner::load(&seeds::game_data_dir()).expect("libs/game-data loads"));

/// A connected client: the server runs on a task at the pipe's other end until the client goes.
pub struct Client(RunningService<RoleClient, ()>);

impl Client {
    /// A server on the real game data.
    pub async fn start() -> Self {
        Self::start_on(PLANNER.clone()).await
    }

    /// A server on `rules::test_world`, the engine tests' small invented world.
    pub async fn on_the_test_world() -> Self {
        let planner =
            Planner::from_data(rules::test_world::world()).expect("the test world checks");
        Self::start_on(planner).await
    }

    /// A server on this planner's game data, for example the test world's.
    pub async fn start_on(planner: Planner) -> Self {
        let (server_end, client_end) = tokio::io::duplex(64 * 1024);
        let server = Server::new(planner);
        tokio::spawn(async move {
            server
                .serve(server_end)
                .await
                .expect("the server starts")
                .waiting()
                .await
        });
        Self(().serve(client_end).await.expect("the client connects"))
    }

    /// The server's `instructions`, from its `initialize` reply.
    pub fn instructions(&self) -> String {
        self.0
            .peer_info()
            .and_then(|info| info.instructions.clone())
            .unwrap_or_default()
    }

    /// The server's capabilities, from its `initialize` reply.
    pub fn capabilities(&self) -> ServerCapabilities {
        self.0
            .peer_info()
            .map(|info| info.capabilities.clone())
            .unwrap_or_default()
    }

    /// The resources the server lists.
    pub async fn resources(&self) -> Vec<Resource> {
        self.0
            .list_all_resources()
            .await
            .expect("the resources list")
    }

    /// Reads one resource by its URI.
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, ServiceError> {
        self.0
            .read_resource(ReadResourceRequestParams::new(uri))
            .await
    }

    /// The tools' names, sorted.
    pub async fn tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .0
            .list_all_tools()
            .await
            .expect("the tools list")
            .into_iter()
            .map(|tool| tool.name.into_owned())
            .collect();
        names.sort();
        names
    }

    /// Calls a tool with these arguments (a JSON object).
    pub async fn call(&self, tool: &str, arguments: Value) -> CallToolResult {
        let Value::Object(arguments) = arguments else {
            panic!("tool arguments are an object");
        };
        self.0
            .call_tool(CallToolRequestParams::new(tool.to_owned()).with_arguments(arguments))
            .await
            .expect("the call gets a result")
    }

    /// A successful call's structured content.
    pub async fn ok(&self, tool: &str, arguments: Value) -> Value {
        let result = self.call(tool, arguments).await;
        assert_ne!(result.is_error, Some(true), "{tool} failed: {result:?}");
        result
            .structured_content
            .expect("the tool returns structured content")
    }

    /// The `search` matches for a query, as `(kind, key)` pairs in the order returned.
    pub async fn search(&self, arguments: Value) -> Vec<(String, String)> {
        self.ok("search", arguments).await["matches"]
            .as_array()
            .expect("matches is a list")
            .iter()
            .map(|hit| (text(&hit["kind"]), text(&hit["key"])))
            .collect()
    }

    pub async fn search_for(&self, query: &str) -> Vec<(String, String)> {
        self.search(json!({ "query": query })).await
    }
}

/// A Build's code, as the planner's links carry it.
pub fn code(build: &rules::Build) -> String {
    rules::encode_build(build)
}

/// The Build a code holds, decoded with the test world's keys (the server's own codes always decode).
pub fn decode(code: &str) -> rules::Build {
    rules::test_world::engine()
        .open_link(code)
        .expect("the code opens")
}

/// A failed call's code: the text before its first colon (`stale_link`, `no_such_choice`, …).
pub fn error_code(result: &CallToolResult) -> String {
    let text = error_text(result);
    text.split_once(':')
        .map_or(text.as_str(), |(code, _)| code)
        .to_owned()
}

/// The `choice` addresses of a list of `{choice, …}` (a `todo`, a level's `picks`).
pub fn choices(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["choice"]))
        .collect()
}

/// The item of a list of `{choice, …}` at an address.
pub fn at<'a>(list: &'a Value, choice: &str) -> &'a Value {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .find(|item| item["choice"] == choice)
        .unwrap_or_else(|| panic!("no {choice} in {list}"))
}

/// A JSON string's text.
pub fn text(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or_else(|| panic!("{value} is a string"))
        .to_owned()
}

/// A failed call's message.
pub fn error_text(result: &CallToolResult) -> String {
    assert_eq!(result.is_error, Some(true), "the call failed: {result:?}");
    result
        .content
        .iter()
        .filter_map(|content| content.as_text())
        .map(|text| text.text.as_str())
        .collect()
}
