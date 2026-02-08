# AGENTS.md

## Repository expectations

[mcp]
### mcp_servers-context7
- Capabilities: Context7 MCP pulls up-to-date, version-specific documentation and code examples straight from the source — and places them directly into your prompt.
- Notes: Always use Context7 MCP when I need library/API documentation, code generation, setup or configuration steps without me having to explicitly ask.

[mcp]
### sequential‑thinking
Enables structured, step‑by‑step reasoning for complex problems through an iterative chain‑of‑thought approach.
- Capabilities:
  * Break down complex problems into sequential steps  
  * Revise and refine thoughts as understanding deepens  
  * Branch into alternative reasoning paths  
  * Adjust the total number of thoughts dynamically  
  * Generate and verify solution hypotheses  
- Notes: Useful for multi‑stage analysis, planning, and reflective problem solving. :contentReference[oaicite:0]{index=0}

[mcp]
### sonarcloud
Integrates AI agents with SonarQube (Cloud or Server) for real, on‑demand code quality and security analysis.
- Capabilities:
  * Connect to SonarQube Cloud or Server via Model Context Protocol  
  * Analyze code snippets and files using SonarQube rules  
  * Surface issues and quality gate status directly within AI workflows  
  * Support for multiple MCP clients (Copilot, Cursor, VS Code, etc.)
- Notes: Requires SONARQUBE_TOKEN and SONARQUBE_ORG/SONARQUBE_URL configuration. :contentReference[oaicite:10]{index=10}

- Breakdown bigger problems into smaller ones using `sequential-thinking`