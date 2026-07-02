use d2_mcp::D2McpServer;
use rmcp::{
    ClientHandler,
    model::{CallToolRequestParams, ClientInfo},
    service::ServiceExt,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[derive(Debug, Clone, Default)]
struct SmokeClient;

impl ClientHandler for SmokeClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

#[tokio::test]
async fn d2_mcp_surface_smoke() -> anyhow::Result<()> {
    if std::process::Command::new("d2")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipping d2 MCP smoke because d2 binary is not installed");
        return Ok(());
    }

    let output_dir = Path::new(".d2-mcp-test");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }

    let (server_transport, client_transport) = tokio::io::duplex(256 * 1024);
    let server_handle = tokio::spawn(async move {
        D2McpServer.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });

    let client = SmokeClient.serve(client_transport).await?;
    let tools = client.list_tools(Default::default()).await?;
    let tool_names: Vec<_> = tools.tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(tool_names.len(), 4);
    assert!(tool_names.contains(&"d2_status"));
    assert!(tool_names.contains(&"d2_validate"));
    assert!(tool_names.contains(&"d2_format"));
    assert!(tool_names.contains(&"d2_render"));

    for tool in &tools.tools {
        assert!(
            tool.output_schema.is_some(),
            "{} must expose outputSchema",
            tool.name
        );
    }

    let status = client
        .call_tool(CallToolRequestParams::new("d2_status"))
        .await?;
    assert_eq!(status.is_error, Some(false));
    let status_body = structured(&status);
    assert_eq!(status_body["status"], "ready");
    assert_eq!(status_body["reads_arbitrary_files"], false);
    assert_eq!(status_body["writes_outside_workdir"], false);

    let render = client
        .call_tool(
            CallToolRequestParams::new("d2_render").with_arguments(arguments(json!({
                "source": "client -> d2_mcp -> svg",
                "format": "svg",
                "output_path": ".d2-mcp-test/smoke.svg",
                "overwrite": true,
                "inline_svg": true
            }))),
        )
        .await?;
    assert_eq!(render.is_error, Some(false));
    let render_body = structured(&render);
    assert_eq!(render_body["status"], "rendered");
    assert_eq!(render_body["format"], "svg");
    assert_eq!(render_body["output_path"], ".d2-mcp-test/smoke.svg");
    assert!(render_body["output_bytes"].as_i64().unwrap_or_default() > 1000);
    assert!(
        render_body["inline_svg"]
            .as_str()
            .unwrap_or_default()
            .contains("<svg")
    );
    assert!(output_dir.join("smoke.svg").is_file());

    let escaped = client
        .call_tool(
            CallToolRequestParams::new("d2_render").with_arguments(arguments(json!({
                "source": "a -> b",
                "format": "svg",
                "output_path": "../escape.svg"
            }))),
        )
        .await;
    assert!(escaped.is_err());

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(output_dir)?;
    Ok(())
}

fn arguments(value: Value) -> serde_json::Map<String, Value> {
    value.as_object().expect("arguments object").clone()
}

fn structured(result: &rmcp::model::CallToolResult) -> &Value {
    result
        .structured_content
        .as_ref()
        .expect("structured tool response")
}
