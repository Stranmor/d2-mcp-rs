mod mcp;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::{OsStr, OsString},
    fmt, fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use tempfile::NamedTempFile;
use wait_timeout::ChildExt;

pub use mcp::D2McpServer;

const DEFAULT_MAX_SOURCE_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_RENDER_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_TIMEOUT_SECONDS: u64 = 20;
const MAX_TIMEOUT_SECONDS: u64 = 120;
const MAX_DIAGNOSTIC_BYTES: usize = 32 * 1024;
const MAX_INLINE_SVG_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2StatusReport {
    pub status: D2BackendStatus,
    pub d2_binary: String,
    pub d2_version: Option<String>,
    pub workdir: String,
    pub max_source_bytes: i64,
    pub max_render_bytes: i64,
    pub default_timeout_seconds: i64,
    pub max_timeout_seconds: i64,
    pub supported_output_formats: Vec<D2OutputFormat>,
    pub remote_assets_default: RemoteAssetPolicy,
    pub reads_arbitrary_files: bool,
    pub writes_outside_workdir: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum D2BackendStatus {
    Ready,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemoteAssetPolicy {
    BlockedUnlessExplicitlyAllowed,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum D2OutputFormat {
    Svg,
    Png,
}

impl D2OutputFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Png => "png",
        }
    }

    fn cli_value(self) -> &'static str {
        self.extension()
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2ValidateArgs {
    pub source: String,
    #[serde(default)]
    pub allow_remote_assets: bool,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2ValidateReport {
    pub status: D2ValidationStatus,
    pub source_bytes: i64,
    pub source_sha256: String,
    pub elapsed_ms: i64,
    pub d2_version: Option<String>,
    pub diagnostics: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum D2ValidationStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2FormatArgs {
    pub source: String,
    #[serde(default)]
    pub allow_remote_assets: bool,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2FormatReport {
    pub status: D2FormatStatus,
    pub source_bytes: i64,
    pub formatted_bytes: i64,
    pub changed: bool,
    pub formatted_source: String,
    pub formatted_sha256: String,
    pub elapsed_ms: i64,
    pub d2_version: Option<String>,
    pub diagnostics: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum D2FormatStatus {
    Formatted,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2RenderArgs {
    pub source: String,
    pub format: D2OutputFormat,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    pub inline_svg: bool,
    #[serde(default)]
    pub allow_remote_assets: bool,
    #[serde(default)]
    pub theme: Option<i64>,
    #[serde(default)]
    pub dark_theme: Option<i64>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub sketch: bool,
    #[serde(default)]
    pub pad: Option<i64>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct D2RenderReport {
    pub status: D2RenderStatus,
    pub format: D2OutputFormat,
    pub source_bytes: i64,
    pub source_sha256: String,
    pub output_path: String,
    pub output_bytes: i64,
    pub output_sha256: String,
    pub inline_svg: Option<String>,
    pub elapsed_ms: i64,
    pub d2_version: Option<String>,
    pub diagnostics: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum D2RenderStatus {
    Rendered,
}

pub fn d2_status() -> D2StatusReport {
    let config = D2Config::from_env();
    let version = d2_version(&config.d2_binary);

    D2StatusReport {
        status: if version.is_some() {
            D2BackendStatus::Ready
        } else {
            D2BackendStatus::Unavailable
        },
        d2_binary: config.d2_binary,
        d2_version: version,
        workdir: config.workdir.display().to_string(),
        max_source_bytes: usize_to_i64(config.max_source_bytes),
        max_render_bytes: usize_to_i64(config.max_render_bytes),
        default_timeout_seconds: u64_to_i64(DEFAULT_TIMEOUT_SECONDS),
        max_timeout_seconds: u64_to_i64(MAX_TIMEOUT_SECONDS),
        supported_output_formats: vec![D2OutputFormat::Svg, D2OutputFormat::Png],
        remote_assets_default: RemoteAssetPolicy::BlockedUnlessExplicitlyAllowed,
        reads_arbitrary_files: false,
        writes_outside_workdir: false,
    }
}

pub fn validate_d2(args: D2ValidateArgs) -> Result<D2ValidateReport, D2McpError> {
    let config = D2Config::from_env();
    validate_source(&args.source, args.allow_remote_assets, &config)?;
    let timeout = timeout_from_args(args.timeout_seconds)?;
    let mut input = source_tempfile(&args.source)?;
    let start = std::time::Instant::now();
    let output = run_d2(
        CommandSpec::new(&config.d2_binary).arg("validate").arg(input.path()).timeout(timeout),
    )?;
    input.flush()?;

    let diagnostics = diagnostics_from(&output);
    let status =
        if output.exit_success { D2ValidationStatus::Valid } else { D2ValidationStatus::Invalid };

    Ok(D2ValidateReport {
        status,
        source_bytes: usize_to_i64(args.source.len()),
        source_sha256: sha256_hex(args.source.as_bytes()),
        elapsed_ms: u128_to_i64(start.elapsed().as_millis()),
        d2_version: d2_version(&config.d2_binary),
        diagnostics,
    })
}

pub fn format_d2(args: D2FormatArgs) -> Result<D2FormatReport, D2McpError> {
    let config = D2Config::from_env();
    validate_source(&args.source, args.allow_remote_assets, &config)?;
    let timeout = timeout_from_args(args.timeout_seconds)?;
    let input = source_tempfile(&args.source)?;
    let start = std::time::Instant::now();
    let output =
        run_d2(CommandSpec::new(&config.d2_binary).arg("fmt").arg(input.path()).timeout(timeout))?;
    if !output.exit_success {
        return Err(D2McpError::D2Failed(diagnostics_from(&output)));
    }
    let formatted_source = fs::read_to_string(input.path())?;
    let changed = formatted_source != args.source;

    Ok(D2FormatReport {
        status: D2FormatStatus::Formatted,
        source_bytes: usize_to_i64(args.source.len()),
        formatted_bytes: usize_to_i64(formatted_source.len()),
        changed,
        formatted_sha256: sha256_hex(formatted_source.as_bytes()),
        formatted_source,
        elapsed_ms: u128_to_i64(start.elapsed().as_millis()),
        d2_version: d2_version(&config.d2_binary),
        diagnostics: diagnostics_from(&output),
    })
}

pub fn render_d2(args: D2RenderArgs) -> Result<D2RenderReport, D2McpError> {
    let config = D2Config::from_env();
    validate_source(&args.source, args.allow_remote_assets, &config)?;
    validate_render_args(&args)?;
    let timeout = timeout_from_args(args.timeout_seconds)?;
    let input = source_tempfile(&args.source)?;
    let source_sha256 = sha256_hex(args.source.as_bytes());
    let output_path = resolve_output_path(
        &config.workdir,
        args.output_path.as_deref(),
        args.format,
        &source_sha256,
    )?;
    if output_path.exists() && !args.overwrite {
        return Err(D2McpError::InvalidInput(format!(
            "output path already exists: {}; pass overwrite=true or choose a different relative output_path",
            relative_display(&config.workdir, &output_path)
        )));
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let start = std::time::Instant::now();
    let mut spec = CommandSpec::new(&config.d2_binary)
        .arg(input.path())
        .arg(&output_path)
        .arg("--stdout-format")
        .arg(args.format.cli_value())
        .arg("--timeout")
        .arg(timeout.as_secs().to_string())
        .timeout(timeout.checked_add(Duration::from_secs(2)).unwrap_or(timeout));
    if let Some(theme) = args.theme {
        spec = spec.arg("--theme").arg(theme.to_string());
    }
    if let Some(dark_theme) = args.dark_theme {
        spec = spec.arg("--dark-theme").arg(dark_theme.to_string());
    }
    if let Some(layout) = args.layout.as_deref() {
        spec = spec.arg("--layout").arg(layout);
    }
    if args.sketch {
        spec = spec.arg("--sketch");
    }
    if let Some(pad) = args.pad {
        spec = spec.arg("--pad").arg(pad.to_string());
    }

    let output = run_d2(spec)?;
    if !output.exit_success {
        return Err(D2McpError::D2Failed(diagnostics_from(&output)));
    }
    let rendered = fs::read(&output_path)?;
    if rendered.len() > config.max_render_bytes {
        let _ = fs::remove_file(&output_path);
        return Err(D2McpError::InvalidInput(format!(
            "rendered output is {} bytes, limit is {} bytes",
            rendered.len(),
            config.max_render_bytes
        )));
    }

    let inline_svg = if args.inline_svg && args.format == D2OutputFormat::Svg {
        if rendered.len() <= MAX_INLINE_SVG_BYTES {
            Some(String::from_utf8(rendered.clone()).map_err(|err| {
                D2McpError::InvalidInput(format!("rendered SVG is not valid UTF-8: {err}"))
            })?)
        } else {
            None
        }
    } else {
        None
    };

    Ok(D2RenderReport {
        status: D2RenderStatus::Rendered,
        format: args.format,
        source_bytes: usize_to_i64(args.source.len()),
        source_sha256,
        output_path: relative_display(&config.workdir, &output_path),
        output_bytes: usize_to_i64(rendered.len()),
        output_sha256: sha256_hex(&rendered),
        inline_svg,
        elapsed_ms: u128_to_i64(start.elapsed().as_millis()),
        d2_version: d2_version(&config.d2_binary),
        diagnostics: diagnostics_from(&output),
    })
}

#[derive(Debug, Clone)]
struct D2Config {
    d2_binary: String,
    workdir: PathBuf,
    max_source_bytes: usize,
    max_render_bytes: usize,
}

impl D2Config {
    fn from_env() -> Self {
        let d2_binary = env::var("D2_MCP_D2_BIN").unwrap_or_else(|_| "d2".to_string());
        let workdir = env::var_os("D2_MCP_WORKDIR")
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            d2_binary,
            workdir,
            max_source_bytes: env_usize("D2_MCP_MAX_SOURCE_BYTES", DEFAULT_MAX_SOURCE_BYTES),
            max_render_bytes: env_usize("D2_MCP_MAX_RENDER_BYTES", DEFAULT_MAX_RENDER_BYTES),
        }
    }
}

fn env_usize(name: &str, default_value: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

fn validate_source(
    source: &str,
    allow_remote_assets: bool,
    config: &D2Config,
) -> Result<(), D2McpError> {
    let bytes = source.as_bytes();
    if bytes.is_empty() {
        return Err(D2McpError::InvalidInput("source is empty".to_string()));
    }
    if bytes.len() > config.max_source_bytes {
        return Err(D2McpError::InvalidInput(format!(
            "source is {} bytes, limit is {} bytes",
            bytes.len(),
            config.max_source_bytes
        )));
    }
    if !allow_remote_assets && contains_remote_asset_reference(source) {
        return Err(D2McpError::InvalidInput(
            "source contains http:// or https:// references; pass allow_remote_assets=true only when remote asset fetches are expected".to_string(),
        ));
    }
    Ok(())
}

fn contains_remote_asset_reference(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    lower.contains("http://") || lower.contains("https://")
}

fn validate_render_args(args: &D2RenderArgs) -> Result<(), D2McpError> {
    if let Some(theme) = args.theme
        && !(0..=300).contains(&theme)
    {
        return Err(D2McpError::InvalidInput("theme must be between 0 and 300".to_string()));
    }
    if let Some(dark_theme) = args.dark_theme
        && !(-1..=300).contains(&dark_theme)
    {
        return Err(D2McpError::InvalidInput("dark_theme must be between -1 and 300".to_string()));
    }
    if let Some(pad) = args.pad
        && !(0..=1000).contains(&pad)
    {
        return Err(D2McpError::InvalidInput("pad must be between 0 and 1000".to_string()));
    }
    if let Some(layout) = args.layout.as_deref()
        && (layout.is_empty()
            || layout.len() > 32
            || !layout.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
    {
        return Err(D2McpError::InvalidInput(
            "layout must be 1-32 chars and contain only ASCII letters, digits, '-' or '_'"
                .to_string(),
        ));
    }
    Ok(())
}

fn timeout_from_args(value: Option<u64>) -> Result<Duration, D2McpError> {
    let seconds = value.unwrap_or(DEFAULT_TIMEOUT_SECONDS);
    if seconds == 0 || seconds > MAX_TIMEOUT_SECONDS {
        return Err(D2McpError::InvalidInput(format!(
            "timeout_seconds must be between 1 and {MAX_TIMEOUT_SECONDS}"
        )));
    }
    Ok(Duration::from_secs(seconds))
}

fn source_tempfile(source: &str) -> Result<NamedTempFile, D2McpError> {
    let mut file = NamedTempFile::with_suffix(".d2")?;
    file.write_all(source.as_bytes())?;
    file.flush()?;
    Ok(file)
}

fn resolve_output_path(
    workdir: &Path,
    output_path: Option<&str>,
    format: D2OutputFormat,
    source_sha256: &str,
) -> Result<PathBuf, D2McpError> {
    let relative_path = output_path.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(".d2-mcp-output").join(format!(
            "{}.{}",
            &source_sha256[..16],
            format.extension()
        ))
    });
    validate_relative_path(&relative_path)?;
    Ok(workdir.join(relative_path))
}

fn validate_relative_path(path: &Path) -> Result<(), D2McpError> {
    if path.as_os_str().is_empty() {
        return Err(D2McpError::InvalidInput("output_path is empty".to_string()));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {},
            Component::CurDir => {},
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(D2McpError::InvalidInput(
                    "output_path must be relative and stay inside D2_MCP_WORKDIR".to_string(),
                ));
            },
        }
    }
    Ok(())
}

fn relative_display(workdir: &Path, output_path: &Path) -> String {
    output_path.strip_prefix(workdir).unwrap_or(output_path).display().to_string()
}

#[derive(Debug)]
struct CommandSpec {
    program: String,
    args: Vec<OsString>,
    timeout: Duration,
}

impl CommandSpec {
    fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: Vec::new(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        }
    }

    fn arg<T: AsRef<OsStr>>(mut self, arg: T) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug)]
struct CommandOutput {
    exit_success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_d2(spec: CommandSpec) -> Result<CommandOutput, D2McpError> {
    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| D2McpError::Backend(format!("failed to start {}: {err}", spec.program)))?;

    match child.wait_timeout(spec.timeout)? {
        Some(_) => {
            let output = child.wait_with_output()?;
            Ok(CommandOutput {
                exit_success: output.status.success(),
                stdout: output.stdout,
                stderr: output.stderr,
            })
        },
        None => {
            let _ = child.kill();
            let _ = child.wait();
            Err(D2McpError::Timeout(format!("d2 exceeded {}s timeout", spec.timeout.as_secs())))
        },
    }
}

fn d2_version(binary: &str) -> Option<String> {
    let output =
        run_d2(CommandSpec::new(binary).arg("--version").timeout(Duration::from_secs(5))).ok()?;
    if !output.exit_success {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text.trim();
    if version.is_empty() { None } else { Some(version.to_string()) }
}

fn diagnostics_from(output: &CommandOutput) -> String {
    let mut diagnostic = String::new();
    if !output.stdout.is_empty() {
        diagnostic.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !diagnostic.is_empty() {
            diagnostic.push('\n');
        }
        diagnostic.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    truncate_diagnostic(&diagnostic)
}

fn truncate_diagnostic(text: &str) -> String {
    if text.len() <= MAX_DIAGNOSTIC_BYTES {
        return text.trim().to_string();
    }
    let mut truncated = text[..MAX_DIAGNOSTIC_BYTES].to_string();
    truncated.push_str("\n[truncated]");
    truncated
}

fn sha256_hex(payload: &[u8]) -> String {
    let digest = Sha256::digest(payload);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn usize_to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn u128_to_i64(value: u128) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[derive(Debug)]
pub enum D2McpError {
    InvalidInput(String),
    Backend(String),
    D2Failed(String),
    Timeout(String),
    Io(std::io::Error),
}

impl fmt::Display for D2McpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
            Self::Backend(message) => write!(formatter, "backend error: {message}"),
            Self::D2Failed(message) => write!(formatter, "d2 failed: {message}"),
            Self::Timeout(message) => write!(formatter, "timeout: {message}"),
            Self::Io(error) => write!(formatter, "io error: {error}"),
        }
    }
}

impl std::error::Error for D2McpError {}

impl From<std::io::Error> for D2McpError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_source() {
        let config = D2Config::from_env();
        let err = validate_source("", false, &config).expect_err("empty source rejected");
        assert!(err.to_string().contains("source is empty"));
    }

    #[test]
    fn rejects_remote_assets_by_default() {
        let config = D2Config::from_env();
        let err = validate_source("x: {icon: https://example.com/a.svg}", false, &config)
            .expect_err("remote asset rejected");
        assert!(err.to_string().contains("allow_remote_assets"));
    }

    #[test]
    fn rejects_parent_dir_output_path() {
        let err = resolve_output_path(
            Path::new("/tmp/work"),
            Some("../escape.svg"),
            D2OutputFormat::Svg,
            "0123456789abcdef",
        )
        .expect_err("parent dir rejected");
        assert!(err.to_string().contains("inside D2_MCP_WORKDIR"));
    }

    #[test]
    fn creates_default_output_path_inside_workdir() {
        let path = resolve_output_path(
            Path::new("/tmp/work"),
            None,
            D2OutputFormat::Png,
            "0123456789abcdef9999",
        )
        .expect("default output path");
        assert_eq!(path, Path::new("/tmp/work/.d2-mcp-output/0123456789abcdef.png"));
    }
}
