use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct Config {
    discord: DiscordConfig,
    mcp: Option<McpConfig>,
}

#[derive(Debug, Deserialize)]
struct DiscordConfig {
    bot_token: String,
    channel_id: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct McpConfig {
    disable_schema_url: Option<bool>,
}

impl Config {
    fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw = std::fs::read_to_string(path)?;
        let config = toml::from_str(&raw)?;
        Ok(config)
    }

    fn disable_schema_url(&self) -> bool {
        self.mcp
            .as_ref()
            .and_then(|mcp| mcp.disable_schema_url)
            .unwrap_or(false)
    }
}

#[derive(Clone)]
struct DiscordClient {
    http: reqwest::Client,
    base_url: String,
    bot_token: String,
    channel_id: String,
}

impl DiscordClient {
    fn new(config: &DiscordConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: config.base_url.clone(),
            bot_token: config.bot_token.clone(),
            channel_id: config.channel_id.clone(),
        }
    }

    async fn send_message(&self, content: &str) -> Result<(), McpError> {
        if content.trim().is_empty() {
            return Err(McpError::invalid_params(
                "content is empty or contains only whitespace",
                None,
            ));
        }

        let url = format!("{}/channels/{}/messages", self.base_url, self.channel_id);
        let payload = SendMessagePayload {
            content: content.to_string(),
        };

        let response = self
            .http
            .post(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bot {}", self.bot_token),
            )
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Discord API request failed: {e}"), None)
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(McpError::internal_error(
                "Discord API error",
                Some(serde_json::json!({ "status": status })),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
struct SendMessagePayload {
    content: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
struct SendMessageRequest {
    content: String,
}

#[derive(Clone)]
struct DiscordMcp {
    discord: DiscordClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DiscordMcp {
    fn new(config: Config) -> Self {
        let disable_schema_url = config.disable_schema_url();
        let mut tool_router = Self::tool_router();
        if disable_schema_url {
            for route in tool_router.map.values_mut() {
                route.attr.input_schema = Arc::new(strip_schema_url(&route.attr.input_schema));
                if let Some(output_schema) = &route.attr.output_schema {
                    route.attr.output_schema = Some(Arc::new(strip_schema_url(output_schema)));
                }
            }
        }

        Self {
            discord: DiscordClient::new(&config.discord),
            tool_router,
        }
    }

    #[tool(
        name = "send_message",
        description = "Send a message to the configured Discord channel"
    )]
    async fn send_message(
        &self,
        params: Parameters<SendMessageRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.discord.send_message(&params.0.content).await?;
        Ok(CallToolResult::success(vec![Content::text("ok")]))
    }
}

#[tool_handler]
impl ServerHandler for DiscordMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Discord MCP server that sends messages to a configured channel.".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn strip_schema_url(schema: &rmcp::model::JsonObject) -> rmcp::model::JsonObject {
    let mut schema = schema.clone();
    schema.remove("$schema");
    if let Some(Value::Object(metadata)) = schema.get_mut("metadata") {
        metadata.remove("$schema");
    }
    schema
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("config.toml")?;
    let service = DiscordMcp::new(config).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
