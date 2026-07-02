use rmcp::{
    ErrorData, ServerHandler,
    handler::server::wrapper::{Json, Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::{
    D2CliListReport, D2FormatArgs, D2FormatReport, D2McpError, D2RenderArgs, D2RenderReport,
    D2StatusReport, D2ValidateArgs, D2ValidateReport, d2_status, format_d2, list_d2_layouts,
    list_d2_themes, render_d2, validate_d2,
};

#[derive(Debug, Clone)]
pub struct D2McpServer;

#[tool_router]
impl D2McpServer {
    #[tool(
        name = "d2_status",
        title = "D2 Status",
        description = "Report D2 CLI availability and this MCP server's safety limits. This tool is read-only.",
        annotations(
            title = "D2 Status",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn d2_status(&self) -> Json<D2StatusReport> {
        Json(d2_status())
    }

    #[tool(
        name = "d2_layouts",
        title = "List D2 Layout Engines",
        description = "List layout engines available to the configured D2 CLI. This helps agents choose a valid layout without shell access.",
        annotations(
            title = "List D2 Layout Engines",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn d2_layouts(&self) -> Result<Json<D2CliListReport>, ErrorData> {
        list_d2_layouts().map(Json).map_err(error_data)
    }

    #[tool(
        name = "d2_themes",
        title = "List D2 Themes",
        description = "List themes available to the configured D2 CLI. This helps agents choose a valid theme without shell access.",
        annotations(
            title = "List D2 Themes",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn d2_themes(&self) -> Result<Json<D2CliListReport>, ErrorData> {
        list_d2_themes().map(Json).map_err(error_data)
    }

    #[tool(
        name = "d2_validate",
        title = "Validate D2 Source",
        description = "Validate D2 source text through the official d2 CLI. Invalid diagrams return structured validation status instead of reading arbitrary files.",
        annotations(
            title = "Validate D2 Source",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn d2_validate(
        &self,
        Parameters(args): Parameters<D2ValidateArgs>,
    ) -> Result<Json<D2ValidateReport>, ErrorData> {
        validate_d2(args).map(Json).map_err(error_data)
    }

    #[tool(
        name = "d2_format",
        title = "Format D2 Source",
        description = "Format D2 source text through d2 fmt using a temporary file and return the formatted source. This does not mutate user files.",
        annotations(
            title = "Format D2 Source",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn d2_format(
        &self,
        Parameters(args): Parameters<D2FormatArgs>,
    ) -> Result<Json<D2FormatReport>, ErrorData> {
        format_d2(args).map(Json).map_err(error_data)
    }

    #[tool(
        name = "d2_render",
        title = "Render D2 Diagram",
        description = "Render D2 source text to SVG or PNG through the official d2 CLI. Output files are restricted to D2_MCP_WORKDIR and remote assets are blocked unless explicitly allowed.",
        annotations(
            title = "Render D2 Diagram",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    fn d2_render(
        &self,
        Parameters(args): Parameters<D2RenderArgs>,
    ) -> Result<Json<D2RenderReport>, ErrorData> {
        render_d2(args).map(Json).map_err(error_data)
    }
}

#[tool_handler]
impl ServerHandler for D2McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("d2-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions("D2 MCP validates, formats, and renders D2 diagrams through the official d2 CLI. It accepts source text, writes outputs only inside D2_MCP_WORKDIR, blocks remote assets by default, and never runs arbitrary shell commands.")
    }
}

fn error_data(error: D2McpError) -> ErrorData {
    ErrorData::invalid_params(error.to_string(), None)
}
