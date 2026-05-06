use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::error::{GdbError, Result};

pub struct GdbSession {
    pub id: String,
    pub child: Child,
    pub stdin: ChildStdin,
    pub reader: BufReader<ChildStdout>,
    pub ready: bool,
    pub target: Option<String>,
    pub working_dir: Option<String>,
}

impl GdbSession {
    pub async fn spawn(gdb_path: &str, working_dir: Option<&str>) -> Result<(Self, String)> {
        let working_dir_owned = working_dir.map(|s| s.to_string());

        let mut child = Command::new(gdb_path)
            .arg("--interpreter=mi")
            .envs(std::env::vars())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| GdbError::GdbProcessError(format!("Failed to spawn GDB: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| GdbError::GdbProcessError("Failed to open GDB stdin".into()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GdbError::GdbProcessError("Failed to open GDB stdout".into()))?;

        let reader = BufReader::new(stdout);

        let mut session = GdbSession {
            id: chrono::Utc::now().timestamp_millis().to_string(),
            child,
            stdin,
            reader,
            ready: false,
            target: None,
            working_dir: working_dir_owned,
        };

        let startup_output = session.wait_for_ready().await?;

        Ok((session, startup_output))
    }

    async fn wait_for_ready(&mut self) -> Result<String> {
        let timeout_duration = std::time::Duration::from_secs(10);
        let mut output = String::new();

        let result = tokio::time::timeout(timeout_duration, async {
            loop {
                let mut line = String::new();
                let n = self.reader.read_line(&mut line).await.map_err(|e| {
                    GdbError::GdbProcessError(format!("Failed to read GDB output: {}", e))
                })?;

                if n == 0 {
                    return Err(GdbError::GdbProcessError(
                        "GDB process closed unexpectedly".into(),
                    ));
                }

                output.push_str(&line);

                if line.contains("(gdb)") || line.contains("^done") {
                    self.ready = true;
                    return Ok::<(), GdbError>(());
                }
            }
        })
        .await;

        match result {
            Err(_) => Err(GdbError::GdbTimeout),
            Ok(inner) => {
                inner?;
                Ok(output)
            }
        }
    }

    pub async fn execute_command(&mut self, command: &str) -> Result<String> {
        if !self.ready {
            let _ = self.wait_for_ready().await;
            if !self.ready {
                return Err(GdbError::GdbNotReady);
            }
        }

        self.stdin
            .write_all(format!("{}\n", command).as_bytes())
            .await
            .map_err(|e| GdbError::GdbProcessError(format!("Failed to write to GDB: {}", e)))?;

        self.stdin
            .flush()
            .await
            .map_err(|e| GdbError::GdbProcessError(format!("Failed to flush GDB stdin: {}", e)))?;

        let timeout_duration = std::time::Duration::from_secs(10);
        let mut output = String::new();

        let result = tokio::time::timeout(timeout_duration, async {
            loop {
                let mut line = String::new();
                let n = self.reader.read_line(&mut line).await.map_err(|e| {
                    GdbError::GdbProcessError(format!("Failed to read GDB output: {}", e))
                })?;

                if n == 0 {
                    return Err(GdbError::GdbProcessError(
                        "GDB process closed unexpectedly".into(),
                    ));
                }

                output.push_str(&line);

                if line.contains("(gdb)") || line.contains("^done") || line.contains("^error") {
                    return Ok::<(), GdbError>(());
                }
            }
        })
        .await;

        match result {
            Err(_) => Err(GdbError::GdbTimeout)?,
            Ok(inner) => inner?,
        }

        Ok(output)
    }

    pub async fn terminate(&mut self) -> Result<()> {
        if self.ready {
            let _ = self.execute_command("quit").await;
        }
        self.child.kill().await.ok();
        Ok(())
    }
}

pub fn normalize_path(program: &str, working_dir: Option<&str>) -> String {
    match working_dir {
        Some(wd) if !Path::new(program).is_absolute() => {
            let p = Path::new(wd).join(program);
            p.to_string_lossy().into_owned()
        }
        _ => program.to_string(),
    }
}
