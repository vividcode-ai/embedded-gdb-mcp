use rmcp::{
    tool, tool_handler, tool_router, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    ErrorData as McpError,
    service::RequestContext,
    RoleServer,
};
use tracing::{debug, error, info};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::session::{GdbSession, normalize_path};

// =============================================================================
// Argument Types for all 17 tools
// =============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbStartArgs {
    /// Path to the GDB executable (optional, defaults to "gdb")
    #[serde(default = "default_gdb_path")]
    pub gdb_path: String,
    /// Working directory for GDB (optional)
    pub working_dir: Option<String>,
}
fn default_gdb_path() -> String { "gdb".to_string() }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbLoadArgs {
    /// GDB session ID
    pub session_id: String,
    /// Path to the program to debug
    pub program: String,
    /// Command-line arguments for the program (optional)
    pub arguments: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbCommandArgs {
    /// GDB session ID
    pub session_id: String,
    /// GDB command to execute
    pub command: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbTerminateArgs {
    /// GDB session ID
    pub session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbListSessionsArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbAttachArgs {
    /// GDB session ID
    pub session_id: String,
    /// Process ID to attach to
    pub pid: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbLoadCoreArgs {
    /// GDB session ID
    pub session_id: String,
    /// Path to the program executable
    pub program: String,
    /// Path to the core dump file
    pub core_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbSetBreakpointArgs {
    /// GDB session ID
    pub session_id: String,
    /// Breakpoint location (e.g., function name, file:line)
    pub location: String,
    /// Breakpoint condition (optional)
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbContinueArgs {
    /// GDB session ID
    pub session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbStepArgs {
    /// GDB session ID
    pub session_id: String,
    /// Step by instructions instead of source lines (optional)
    #[serde(default)]
    pub instructions: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbNextArgs {
    /// GDB session ID
    pub session_id: String,
    /// Step by instructions instead of source lines (optional)
    #[serde(default)]
    pub instructions: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbFinishArgs {
    /// GDB session ID
    pub session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbBacktraceArgs {
    /// GDB session ID
    pub session_id: String,
    /// Show variables in each frame (optional)
    #[serde(default)]
    pub full: bool,
    /// Maximum number of frames to show (optional)
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbPrintArgs {
    /// GDB session ID
    pub session_id: String,
    /// Expression to evaluate
    pub expression: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbExamineArgs {
    /// GDB session ID
    pub session_id: String,
    /// Memory address or expression
    pub expression: String,
    /// Display format (e.g., "x" for hex, "i" for instruction)
    #[serde(default = "default_examine_format")]
    pub format: String,
    /// Number of units to display
    #[serde(default = "default_examine_count")]
    pub count: u32,
}
fn default_examine_format() -> String { "x".to_string() }
fn default_examine_count() -> u32 { 1 }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbInfoRegistersArgs {
    /// GDB session ID
    pub session_id: String,
    /// Specific register to display (optional)
    pub register: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GdbListSourceArgs {
    /// GDB session ID
    pub session_id: String,
    /// Source location (e.g., function name, file:line, optional)
    pub location: Option<String>,
    /// Number of lines to show (optional, default is 10)
    #[serde(default = "default_line_count")]
    pub line_count: u32,
}
fn default_line_count() -> u32 { 10 }

#[derive(Debug, Serialize)]
struct SessionInfo {
    id: String,
    target: String,
    working_dir: String,
}

#[derive(Debug, Serialize)]
struct SourceInfo {
    file_path: String,
    line_start: u32,
    line_end: u32,
    current_line: u32,
}

// =============================================================================
// Main Handler Struct
// =============================================================================

#[derive(Clone)]
pub struct EmbeddedGdbToolHandler {
    tool_router: ToolRouter<EmbeddedGdbToolHandler>,
    sessions: Arc<RwLock<HashMap<String, Arc<RwLock<GdbSession>>>>>,
}

impl EmbeddedGdbToolHandler {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for EmbeddedGdbToolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl EmbeddedGdbToolHandler {
    // =========================================================================
    // gdb_start - Start a new GDB session
    // =========================================================================

    #[tool(description = "Start a new GDB session")]
    async fn gdb_start(
        &self,
        Parameters(args): Parameters<GdbStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Starting GDB session with path: {}", args.gdb_path);

        let result = GdbSession::spawn(&args.gdb_path, args.working_dir.as_deref()).await;

        match result {
            Ok((session, startup_output)) => {
                let session_id = session.id.clone();
                let session = Arc::new(RwLock::new(session));

                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.clone(), session);
                }

                let msg = format!(
                    "GDB session started with ID: {}\n\nOutput:\n{}",
                    session_id, startup_output
                );

                info!("Started GDB session: {}", session_id);
                Ok(CallToolResult::success(vec![Content::text(msg)]))
            }
            Err(e) => {
                let err_msg = format!("Failed to start GDB: {}", e);
                error!("{}", err_msg);
                Err(McpError::internal_error(err_msg, None))
            }
        }
    }

    // =========================================================================
    // gdb_load - Load a program into GDB
    // =========================================================================

    #[tool(description = "Load a program into GDB")]
    async fn gdb_load(
        &self,
        Parameters(args): Parameters<GdbLoadArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let working_dir = session.working_dir.clone();
        let normalized_path = normalize_path(&args.program, working_dir.as_deref());

        session.target = Some(normalized_path.clone());

        let load_output = execute_or_err(&mut session, &format!("file \"{}\"", normalized_path)).await?;

        let mut args_output = String::new();
        if let Some(prog_args) = &args.arguments {
            if !prog_args.is_empty() {
                args_output = execute_or_err(
                    &mut session,
                    &format!("set args {}", prog_args.join(" ")),
                )
                .await?;
            }
        }

        let msg = format!(
            "Program loaded: {}\n\nOutput:\n{}{}",
            normalized_path,
            load_output,
            if args_output.is_empty() {
                String::new()
            } else {
                format!("\n{}", args_output)
            }
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_command - Execute a GDB command
    // =========================================================================

    #[tool(description = "Execute a GDB command")]
    async fn gdb_command(
        &self,
        Parameters(args): Parameters<GdbCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let output = execute_or_err(&mut session, &args.command).await?;

        let msg = format!("Command: {}\n\nOutput:\n{}", args.command, output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_terminate - Terminate a GDB session
    // =========================================================================

    #[tool(description = "Terminate a GDB session")]
    async fn gdb_terminate(
        &self,
        Parameters(args): Parameters<GdbTerminateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&args.session_id)
        };

        match session_arc {
            Some(session_arc) => {
                let mut session = session_arc.write().await;
                let _ = session.terminate().await;

                let msg = format!("GDB session terminated: {}", args.session_id);
                info!("{}", msg);
                Ok(CallToolResult::success(vec![Content::text(msg)]))
            }
            None => Err(McpError::internal_error(
                format!("No active GDB session with ID: {}", args.session_id),
                None,
            )),
        }
    }

    // =========================================================================
    // gdb_list_sessions - List all active GDB sessions
    // =========================================================================

    #[tool(description = "List all active GDB sessions")]
    async fn gdb_list_sessions(
        &self,
        Parameters(_args): Parameters<GdbListSessionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let sessions = self.sessions.read().await;

        let session_list: Vec<SessionInfo> = sessions
            .iter()
            .map(|(id, session_arc)| {
                let session = session_arc.try_read().ok();
                SessionInfo {
                    id: id.clone(),
                    target: session
                        .as_ref()
                        .and_then(|s| s.target.clone())
                        .unwrap_or_else(|| "No program loaded".to_string()),
                    working_dir: session
                        .as_ref()
                        .and_then(|s| s.working_dir.clone())
                        .unwrap_or_else(|| std::env::current_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| "unknown".to_string())),
                }
            })
            .collect();

        let json_str = serde_json::to_string_pretty(&session_list)
            .unwrap_or_else(|_| "[]".to_string());

        let msg = format!(
            "Active GDB Sessions ({}):\n\n{}",
            session_list.len(),
            json_str
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_attach - Attach to a running process
    // =========================================================================

    #[tool(description = "Attach to a running process")]
    async fn gdb_attach(
        &self,
        Parameters(args): Parameters<GdbAttachArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let output = execute_or_err(&mut session, &format!("attach {}", args.pid)).await?;

        let msg = format!("Attached to process {}\n\nOutput:\n{}", args.pid, output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_load_core - Load a core dump file
    // =========================================================================

    #[tool(description = "Load a core dump file")]
    async fn gdb_load_core(
        &self,
        Parameters(args): Parameters<GdbLoadCoreArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let file_output = execute_or_err(&mut session, &format!("file \"{}\"", args.program)).await?;
        let core_output =
            execute_or_err(&mut session, &format!("core-file \"{}\"", args.core_path)).await?;
        let backtrace_output = execute_or_err(&mut session, "backtrace").await?;

        let msg = format!(
            "Core file loaded: {}\n\nOutput:\n{}\n{}\n\nBacktrace:\n{}",
            args.core_path, file_output, core_output, backtrace_output
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_set_breakpoint - Set a breakpoint
    // =========================================================================

    #[tool(description = "Set a breakpoint")]
    async fn gdb_set_breakpoint(
        &self,
        Parameters(args): Parameters<GdbSetBreakpointArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let output = execute_or_err(&mut session, &format!("break {}", args.location)).await?;

        let mut condition_output = String::new();
        if let Some(condition) = &args.condition {
            let re = regex::Regex::new(r"Breakpoint (\d+)").unwrap();
            if let Some(caps) = re.captures(&output) {
                if let Some(bp_num) = caps.get(1) {
                    condition_output =
                        execute_or_err(&mut session, &format!("condition {} {}", bp_num.as_str(), condition)).await?;
                }
            }
        }

        let msg = format!(
            "Breakpoint set at: {}{}\n\nOutput:\n{}{}",
            args.location,
            args.condition
                .as_ref()
                .map(|c| format!(" with condition: {}", c))
                .unwrap_or_default(),
            output,
            if condition_output.is_empty() {
                String::new()
            } else {
                format!("\n{}", condition_output)
            }
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_continue - Continue program execution
    // =========================================================================

    #[tool(description = "Continue program execution")]
    async fn gdb_continue(
        &self,
        Parameters(args): Parameters<GdbContinueArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let output = execute_or_err(&mut session, "continue").await?;

        let msg = format!("Continued execution\n\nOutput:\n{}", output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_step - Step program execution
    // =========================================================================

    #[tool(description = "Step program execution")]
    async fn gdb_step(
        &self,
        Parameters(args): Parameters<GdbStepArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let command = if args.instructions { "stepi" } else { "step" };
        let output = execute_or_err(&mut session, command).await?;

        let step_type = if args.instructions {
            "instruction"
        } else {
            "line"
        };
        let msg = format!("Stepped {}\n\nOutput:\n{}", step_type, output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_next - Step over function calls
    // =========================================================================

    #[tool(description = "Step over function calls")]
    async fn gdb_next(
        &self,
        Parameters(args): Parameters<GdbNextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let command = if args.instructions { "nexti" } else { "next" };
        let output = execute_or_err(&mut session, command).await?;

        let step_type = if args.instructions {
            "instruction"
        } else {
            "function call"
        };
        let msg = format!("Stepped over {}\n\nOutput:\n{}", step_type, output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_finish - Execute until the current function returns
    // =========================================================================

    #[tool(description = "Execute until the current function returns")]
    async fn gdb_finish(
        &self,
        Parameters(args): Parameters<GdbFinishArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let output = execute_or_err(&mut session, "finish").await?;

        let msg = format!("Finished current function\n\nOutput:\n{}", output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_backtrace - Show call stack
    // =========================================================================

    #[tool(description = "Show call stack")]
    async fn gdb_backtrace(
        &self,
        Parameters(args): Parameters<GdbBacktraceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let mut command = if args.full {
            "backtrace full".to_string()
        } else {
            "backtrace".to_string()
        };

        if let Some(limit) = args.limit {
            command.push_str(&format!(" {}", limit));
        }

        let output = execute_or_err(&mut session, &command).await?;

        let msg = format!(
            "Backtrace{}{}:\n\n{}",
            if args.full { " (full)" } else { "" },
            if args.limit.is_some() {
                format!(" (limit: {})", args.limit.unwrap())
            } else {
                String::new()
            },
            output
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_print - Print value of expression
    // =========================================================================

    #[tool(description = "Print value of expression")]
    async fn gdb_print(
        &self,
        Parameters(args): Parameters<GdbPrintArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let output = execute_or_err(&mut session, &format!("print {}", args.expression)).await?;

        let msg = format!("Print {}:\n\n{}", args.expression, output);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_examine - Examine memory
    // =========================================================================

    #[tool(description = "Examine memory")]
    async fn gdb_examine(
        &self,
        Parameters(args): Parameters<GdbExamineArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let command = format!("x/{}{} {}", args.count, args.format, args.expression);
        let output = execute_or_err(&mut session, &command).await?;

        let msg = format!(
            "Examine {} (format: {}, count: {}):\n\n{}",
            args.expression, args.format, args.count, output
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_info_registers - Display registers
    // =========================================================================

    #[tool(description = "Display registers")]
    async fn gdb_info_registers(
        &self,
        Parameters(args): Parameters<GdbInfoRegistersArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let command = match &args.register {
            Some(reg) => format!("info registers {}", reg),
            None => "info registers".to_string(),
        };
        let output = execute_or_err(&mut session, &command).await?;

        let msg = format!(
            "Register info{}:\n\n{}",
            args.register
                .as_ref()
                .map(|r| format!(" for {}", r))
                .unwrap_or_default(),
            output
        );

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // =========================================================================
    // gdb_list_source - List source code with VS Code integration
    // =========================================================================

    #[tool(description = "List source code at current location or specified location, with VS Code integration")]
    async fn gdb_list_source(
        &self,
        Parameters(args): Parameters<GdbListSourceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session_arc = get_session(&self.sessions, &args.session_id).await?;
        let mut session = session_arc.write().await;

        let command = match &args.location {
            Some(loc) => format!("list {}", loc),
            None => "list".to_string(),
        };
        let output = execute_or_err(&mut session, &command).await?;

        // Try to parse source info
        let source_info = parse_source_info(&mut session, &output).await;

        let loc_str = args
            .location
            .as_ref()
            .map(|l| format!(" at {}", l))
            .unwrap_or_default();

        if let Some(info) = source_info {
            let vscode_uri = format!("vscode://file{}:{}", info.file_path, info.line_start);
            let text = format!("Source code{}:\n\n{}", loc_str, output);
            let extra = serde_json::json!({
                "type": "source_location",
                "filePath": info.file_path,
                "lineStart": info.line_start,
                "vscodeUri": vscode_uri,
                "lineEnd": info.line_end,
                "currentLine": info.current_line
            });
            let extra_str = serde_json::to_string(&extra).unwrap_or_default();
            Ok(CallToolResult::success(vec![Content::text(text), Content::text(extra_str)]))
        } else {
            let msg = format!("Source code{}:\n\n{}", loc_str, output);
            Ok(CallToolResult::success(vec![Content::text(msg)]))
        }
    }
}

// =============================================================================
// ServerHandler Implementation
// =============================================================================

#[tool_handler]
impl ServerHandler for EmbeddedGdbToolHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "GDB debugging MCP server. Provides 17 tools for gdb debugging: \
                gdb_start, gdb_load, gdb_command, gdb_terminate, gdb_list_sessions, \
                gdb_attach, gdb_load_core, gdb_set_breakpoint, gdb_continue, gdb_step, \
                gdb_next, gdb_finish, gdb_backtrace, gdb_print, gdb_examine, \
                gdb_info_registers, gdb_list_source."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("GDB MCP server initialized");
        Ok(self.get_info())
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

async fn get_session(
    sessions: &Arc<RwLock<HashMap<String, Arc<RwLock<GdbSession>>>>>,
    session_id: &str,
) -> Result<Arc<RwLock<GdbSession>>, McpError> {
    let sessions = sessions.read().await;
    match sessions.get(session_id) {
        Some(session) => Ok(session.clone()),
        None => Err(McpError::internal_error(
            format!("No active GDB session with ID: {}", session_id),
            None,
        )),
    }
}

async fn execute_or_err(
    session: &mut GdbSession,
    command: &str,
) -> Result<String, McpError> {
    session
        .execute_command(command)
        .await
        .map_err(|e| McpError::internal_error(format!("{}", e), None))
}

async fn parse_source_info(
    session: &mut GdbSession,
    list_output: &str,
) -> Option<SourceInfo> {
    if list_output.trim().is_empty() {
        return None;
    }

    // Try info line first
    let info_line_output = session.execute_command("info line").await.ok()?;

    let re = regex::Regex::new(r#"Line (\d+) of "([^"]+)""#).ok()?;
    let (file_path, current_line) = if let Some(caps) = re.captures(&info_line_output) {
        (
            caps.get(2)?.as_str().to_string(),
            caps.get(1)?.as_str().parse::<u32>().ok()?,
        )
    } else {
        // Fallback to info source
        let info_output = session.execute_command("info source").await.ok()?;
        let file_re = regex::Regex::new(r"Current source file is (.+?)(?: |$)").ok()?;
        if let Some(caps) = file_re.captures(&info_output) {
            (caps.get(1)?.as_str().to_string(), 0)
        } else {
            return None;
        }
    };

    // Extract line numbers from list output
    let line_re = regex::Regex::new(r"^\s*(\d+)\s+").ok()?;
    let source_lines: Vec<u32> = list_output
        .lines()
        .filter_map(|line| {
            line_re.captures(line).and_then(|caps| {
                caps.get(1)?.as_str().parse::<u32>().ok()
            })
        })
        .collect();

    if source_lines.is_empty() {
        return None;
    }

    let line_start = *source_lines.first()?;
    let line_end = *source_lines.last()?;

    Some(SourceInfo {
        file_path,
        line_start,
        line_end,
        current_line,
    })
}
