use crate::models::{McpServerConfig, ResourceDefinition, ToolDefinition};
use crate::services::mcp::McpService;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cached capabilities for a single MCP server.
#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    pub config: Option<McpServerConfig>,
    pub tools: HashMap<String, ToolDefinition>,
    pub resources: HashMap<String, ResourceDefinition>,
}

/// Host-side registry — populated once, queried deterministically.
#[derive(Clone)]
pub struct CapabilityRegistry {
    servers: Arc<RwLock<HashMap<String, ServerCapabilities>>>,
}

/// A resolved, validated tool call ready for execution.
#[derive(Debug, Clone)]
pub struct ResolvedCall {
    pub server_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// LLM-facing tool specs and a reverse lookup map.
#[derive(Debug, Clone, Default)]
pub struct LlmToolSpecBundle {
    pub tools: Vec<serde_json::Value>,
    pub tool_map: HashMap<String, (String, String)>, // tool_id -> (server_id, tool_name)
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Hydrate the registry from live MCP connections.
    /// Called once at startup / when MCP config changes.
    pub async fn refresh(&self, mcp_service: &McpService) -> Result<(), String> {
        let server_configs = mcp_service.list_servers().await;
        let statuses = mcp_service.status(None).await;
        let connected: HashSet<String> = statuses
            .into_iter()
            .filter(|s| s.connected)
            .map(|s| s.id)
            .collect();

        let mut new_map: HashMap<String, ServerCapabilities> = HashMap::new();

        for server in &server_configs {
            if !server.enabled {
                continue;
            }

            // Connect if not already connected
            if !connected.contains(&server.id) {
                if let Err(e) = mcp_service.connect(&server.id).await {
                    eprintln!(
                        "[CapabilityRegistry] Failed to connect '{}': {}",
                        server.id, e
                    );
                    continue;
                }
            }

            let mut caps = ServerCapabilities {
                config: Some(server.clone()),
                ..Default::default()
            };

            // Cache tools
            match mcp_service.tools_list(&server.id).await {
                Ok(tools) => {
                    for tool in tools {
                        if let Some(name) = tool.get("name").and_then(|v| v.as_str()) {
                            caps.tools.insert(name.to_string(), tool);
                        }
                    }
                }
                Err(e) => eprintln!(
                    "[CapabilityRegistry] tools_list failed for '{}': {}",
                    server.id, e
                ),
            }

            // Cache resources
            match mcp_service.resources_list(&server.id).await {
                Ok(resources) => {
                    for res in resources {
                        if let Some(uri) = res.get("uri").and_then(|v| v.as_str()) {
                            caps.resources.insert(uri.to_string(), res);
                        }
                    }
                }
                Err(e) => eprintln!(
                    "[CapabilityRegistry] resources_list failed for '{}': {}",
                    server.id, e
                ),
            }

            new_map.insert(server.id.clone(), caps);
        }

        let mut guard = self.servers.write().await;
        *guard = new_map;
        Ok(())
    }

    // ── Validation (hard, deterministic) ──────────────────────────

    pub async fn has_server(&self, id: &str) -> bool {
        self.servers.read().await.contains_key(id)
    }

    pub async fn has_tool(&self, server_id: &str, tool_name: &str) -> bool {
        self.servers
            .read()
            .await
            .get(server_id)
            .map(|caps| caps.tools.contains_key(tool_name))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub async fn has_resource(&self, server_id: &str, uri: &str) -> bool {
        self.servers
            .read()
            .await
            .get(server_id)
            .map(|caps| caps.resources.contains_key(uri))
            .unwrap_or(false)
    }

    /// Hard-validate a call before execution. Returns Err if anything is wrong.
    pub async fn validate_call(&self, call: &ResolvedCall) -> Result<(), String> {
        if !self.has_server(&call.server_id).await {
            return Err(format!("Server '{}' not in registry", call.server_id));
        }
        if !self.has_tool(&call.server_id, &call.tool_name).await {
            return Err(format!(
                "Tool '{}' not found on server '{}'",
                call.tool_name, call.server_id
            ));
        }
        Ok(())
    }

    // ── Matching (deterministic dispatch) ─────────────────────────

    /// Given a query, find the best matching tools across allowed servers.
    /// Returns a list of (score, server_id, tool_name) ranked by relevance.
    pub async fn search_tools(
        &self,
        query: &str,
        allowed_server_ids: &[String],
        limit: usize,
    ) -> Vec<(f64, String, String)> {
        let guard = self.servers.read().await;
        let query_lower = query.to_ascii_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut matches: Vec<(f64, String, String)> = Vec::new();

        for (server_id, caps) in guard.iter() {
            if !allowed_server_ids.is_empty()
                && !allowed_server_ids.iter().any(|id| id == server_id)
            {
                continue;
            }

            for (tool_name, tool_def) in &caps.tools {
                let description = tool_def
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let haystack = format!("{} {}", tool_name, description).to_ascii_lowercase();

                let mut score = 0.0;
                for word in &query_words {
                    if haystack.contains(*word) {
                        score += 1.0;
                    }
                }
                score /= query_words.len().max(1) as f64;

                if score > 0.0 {
                    matches.push((score, server_id.clone(), tool_name.clone()));
                }
            }
        }

        matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(limit);
        matches
    }

    /// Default implementation for backward compatibility or simple cases
    pub async fn match_tool(
        &self,
        query: &str,
        allowed_server_ids: &[String],
    ) -> Option<ResolvedCall> {
        let results = self.search_tools(query, allowed_server_ids, 1).await;
        results
            .first()
            .map(|(_, server_id, tool_name)| ResolvedCall {
                server_id: server_id.clone(),
                tool_name: tool_name.clone(),
                arguments: serde_json::json!({}),
            })
    }

    /// Build a compact summary for the LLM intent-classification prompt.
    /// Only names + short descriptions — NOT the full schema.
    pub async fn summary_for_prompt(&self, allowed_server_ids: &[String]) -> String {
        let guard = self.servers.read().await;
        let mut lines = Vec::new();

        for (server_id, caps) in guard.iter() {
            if !allowed_server_ids.is_empty()
                && !allowed_server_ids.iter().any(|id| id == server_id)
            {
                continue;
            }

            lines.push(format!("Server: {}", server_id));
            for (name, def) in &caps.tools {
                let desc = def
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no description)");
                lines.push(format!("  tool: {} — {}", name, desc));
            }
            for (uri, def) in &caps.resources {
                let desc = def
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no description)");
                lines.push(format!("  resource: {} — {}", uri, desc));
            }
        }

        if lines.is_empty() {
            "No MCP capabilities available.".to_string()
        } else {
            lines.join("\n")
        }
    }

    /// Build a compact JSON summary for the LLM intent-classification prompt.
    /// Can be filtered by a specific list of relevant tools.
    pub async fn summary_for_prompt_json(
        &self,
        allowed_server_ids: &[String],
        filter: Option<&[(String, String)]>, // (server_id, tool_name)
    ) -> String {
        let guard = self.servers.read().await;

        let mut servers_val = Vec::new();
        for (server_id, caps) in guard.iter() {
            if !allowed_server_ids.is_empty()
                && !allowed_server_ids.iter().any(|id| id == server_id)
            {
                continue;
            }

            let mut tools_list = Vec::new();
            for (name, def) in &caps.tools {
                // Apply filter if provided
                if let Some(f) = filter {
                    if !f.iter().any(|(srv, t)| srv == server_id && t == name) {
                        continue;
                    }
                }

                let desc = def
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no description)");
                tools_list.push(serde_json::json!({ "name": name, "description": desc }));
            }

            let mut resources_list = Vec::new();
            // Don't filter resources for now, or maybe only include if server is in filter
            if filter.is_none()
                || filter
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|(srv, _)| srv == server_id)
            {
                for (uri, def) in &caps.resources {
                    let desc = def
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("(no description)");
                    resources_list.push(serde_json::json!({ "uri": uri, "description": desc }));
                }
            }

            if !tools_list.is_empty() || !resources_list.is_empty() {
                servers_val.push(serde_json::json!({
                    "id": server_id,
                    "tools": tools_list,
                    "resources": resources_list
                }));
            }
        }

        if servers_val.is_empty() {
            serde_json::json!({ "note": "No matching/available MCP capabilities found." })
                .to_string()
        } else {
            serde_json::json!({ "servers": servers_val }).to_string()
        }
    }

    pub async fn available_server_ids(&self) -> Vec<String> {
        self.servers.read().await.keys().cloned().collect()
    }

    /// Build LLM tool specs (OpenAI-style) plus a tool_id -> (server, tool) map.
    /// If max_tools > 0 and total tools exceed max_tools, uses a lightweight
    /// search-based filter to select the top-N tools for the query.
    pub async fn llm_tools_for_query(
        &self,
        query: &str,
        allowed_server_ids: &[String],
        max_tools: usize,
    ) -> LlmToolSpecBundle {
        let total_tools = {
            let guard = self.servers.read().await;
            guard
                .iter()
                .filter(|(server_id, _)| {
                    allowed_server_ids.is_empty()
                        || allowed_server_ids.iter().any(|id| id == *server_id)
                })
                .map(|(_, caps)| caps.tools.len())
                .sum::<usize>()
        };

        let filtered: Option<HashSet<(String, String)>> =
            if max_tools > 0 && total_tools > max_tools {
                let matches = self
                    .search_tools(query, allowed_server_ids, max_tools)
                    .await;
                Some(
                    matches
                        .into_iter()
                        .map(|(_, server_id, tool_name)| (server_id, tool_name))
                        .collect(),
                )
            } else {
                None
            };

        let guard = self.servers.read().await;
        let mut bundle = LlmToolSpecBundle::default();

        for (server_id, caps) in guard.iter() {
            if !allowed_server_ids.is_empty()
                && !allowed_server_ids.iter().any(|id| id == server_id)
            {
                continue;
            }

            for (tool_name, tool_def) in &caps.tools {
                if let Some(filter) = &filtered {
                    if !filter.contains(&(server_id.clone(), tool_name.clone())) {
                        continue;
                    }
                }

                let tool_id = encode_tool_id(server_id, tool_name);
                let desc = tool_def
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no description)");
                let parameters = tool_def
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({ "type": "object", "properties": {} }));

                bundle
                    .tool_map
                    .insert(tool_id.clone(), (server_id.clone(), tool_name.clone()));
                bundle.tools.push(serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool_id,
                        "description": format!("[{}] {}", server_id, desc),
                        "parameters": parameters
                    }
                }));
            }
        }

        bundle
    }

    /// Return the cached tool definition for a given server + tool.
    pub async fn get_tool_def(&self, server_id: &str, tool_name: &str) -> Option<ToolDefinition> {
        self.servers
            .read()
            .await
            .get(server_id)
            .and_then(|caps| caps.tools.get(tool_name).cloned())
    }

    /// Build arguments from the query string using the tool's inputSchema.
    /// Finds the first required string parameter and injects the query there.
    /// Falls back to `{"query": query}` when schema inspection fails.
    pub fn build_arguments_from_query(tool_def: &ToolDefinition, query: &str) -> serde_json::Value {
        if let Some(schema) = tool_def.get("inputSchema").and_then(|s| s.as_object()) {
            if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
                let required: Vec<String> = schema
                    .get("required")
                    .and_then(|r| r.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                // Priority 1: find a required string parameter
                for req_name in &required {
                    if let Some(prop) = props.get(req_name).and_then(|p| p.as_object()) {
                        if prop
                            .get("type")
                            .and_then(|t| t.as_str())
                            .map(|t| t == "string")
                            .unwrap_or(false)
                        {
                            return serde_json::json!({ req_name: query });
                        }
                    }
                }

                // Priority 2: find well-known parameter names
                let well_known = ["query", "q", "search", "input", "text", "prompt", "message"];
                for name in &well_known {
                    if props.contains_key(*name) {
                        return serde_json::json!({ (*name): query });
                    }
                }

                // Priority 3: first string property
                for (name, prop) in props {
                    if prop
                        .get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "string")
                        .unwrap_or(false)
                    {
                        return serde_json::json!({ name: query });
                    }
                }
            }
        }

        // Ultimate fallback
        serde_json::json!({ "query": query })
    }
}

fn encode_tool_id(server_id: &str, tool_name: &str) -> String {
    fn sanitize(input: &str) -> String {
        input
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    format!("mcp__{}__{}", sanitize(server_id), sanitize(tool_name))
}

#[cfg(test)]
mod tests {
    use super::{CapabilityRegistry, ServerCapabilities};
    use std::collections::HashMap;

    #[tokio::test]
    async fn llm_tools_for_query_builds_tool_ids() {
        let registry = CapabilityRegistry::new();
        let mut guard = registry.servers.write().await;

        let mut tools = HashMap::new();
        tools.insert(
            "search".to_string(),
            serde_json::json!({
                "name": "search",
                "description": "Search tool",
                "inputSchema": { "type": "object", "properties": { "q": { "type": "string" } } }
            }),
        );

        guard.insert(
            "server1".to_string(),
            ServerCapabilities {
                config: None,
                tools,
                resources: HashMap::new(),
            },
        );
        drop(guard);

        let bundle = registry
            .llm_tools_for_query("search", &vec!["server1".to_string()], 0)
            .await;
        assert!(bundle.tool_map.contains_key("mcp__server1__search"));
        assert_eq!(bundle.tools.len(), 1);
        assert_eq!(
            bundle.tools[0]["function"]["name"],
            serde_json::json!("mcp__server1__search")
        );
    }
}
