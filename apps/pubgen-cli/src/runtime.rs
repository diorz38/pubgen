//! Shared DesignCraft Runtime abstractions for headless & live execution.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// Common contract for DesignCraft runtime interactions.
pub trait DesignCraftRuntime {
    /// Probe available engine commands and capabilities.
    fn capabilities(&mut self) -> Result<Vec<String>, String>;

    /// Execute a registered engine or UI command.
    fn execute(&mut self, command: &str, params: Value) -> Result<Value, String>;

    /// Check if the runtime connection is healthy.
    #[allow(dead_code)]
    fn is_healthy(&mut self) -> bool;
}

/// A CLI runtime that manages an external spawned DesignCraft child process
/// communicating over a localhost TCP control socket.
#[allow(dead_code)]
pub struct CliDesignCraftRuntime {
    exe: PathBuf,
    port: u16,
    child: ChildGuard,
    stream: TcpStream,
    next_id: u64,
}

#[allow(dead_code)]
impl CliDesignCraftRuntime {
    pub fn spawn(exe: &Path, timeout: Duration) -> Result<Self, String> {
        let port = free_port()?;
        let mut child = ChildGuard::spawn(exe, port).map_err(|e| format!("spawn {}: {e}", exe.display()))?;

        let stream = connect_with_timeout(port, &mut child, timeout)?;

        Ok(Self { exe: exe.to_path_buf(), port, child, stream, next_id: 1 })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn exe(&self) -> &Path {
        &self.exe
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let req = json!({
            "id": id,
            "method": method,
            "params": params,
        });

        let mut line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        line.push('\n');

        let mut writer = BufWriter::new(&self.stream);
        writer.write_all(line.as_bytes()).map_err(|e| format!("send {method}: {e}"))?;
        writer.flush().map_err(|e| format!("flush {method}: {e}"))?;

        let mut reader = BufReader::new(&self.stream);
        let mut resp_str = String::new();
        reader.read_line(&mut resp_str).map_err(|e| format!("recv {method}: {e}"))?;

        let resp: Value = serde_json::from_str(&resp_str).map_err(|e| format!("invalid response for {method}: {e}"))?;

        if let Some(err) = resp.get("error").filter(|e| !e.is_null()) {
            return Err(format!("runtime error: {err}"));
        }

        Ok(resp.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl DesignCraftRuntime for CliDesignCraftRuntime {
    fn capabilities(&mut self) -> Result<Vec<String>, String> {
        let res = self.call("engine.commands", json!({}))?;
        let cmds = res.as_array().ok_or("engine.commands response was not an array")?;

        let mut names = Vec::new();
        for item in cmds {
            if let Some(id) = item.get("id").and_then(Value::as_str) {
                names.push(id.to_string());
            }
        }
        Ok(names)
    }

    fn execute(&mut self, command: &str, params: Value) -> Result<Value, String> {
        self.call(
            "engine.execute",
            json!({
                "command": command,
                "params": params,
            }),
        )
    }

    fn is_healthy(&mut self) -> bool {
        self.call("engine.commands", json!({})).is_ok()
    }
}

/// A mock runtime for offline validation, dry-run, and unit testing.
pub struct MockDesignCraftRuntime {
    pub registered_commands: Vec<String>,
    pub execution_log: Vec<(String, Value)>,
}

impl Default for MockDesignCraftRuntime {
    fn default() -> Self {
        Self {
            registered_commands: vec!["pubgen.compileProject".to_string(), "file.open".to_string(), "file.exportPdf".to_string()],
            execution_log: Vec::new(),
        }
    }
}

impl DesignCraftRuntime for MockDesignCraftRuntime {
    fn capabilities(&mut self) -> Result<Vec<String>, String> {
        Ok(self.registered_commands.clone())
    }

    fn execute(&mut self, command: &str, params: Value) -> Result<Value, String> {
        self.execution_log.push((command.to_string(), params.clone()));
        if command == "pubgen.compileProject" {
            let path = params.get("path").and_then(Value::as_str).unwrap_or("pubgen_project.json");
            let pdf = params.get("pdf").and_then(Value::as_str).unwrap_or("output.pdf");
            Ok(json!({
                "status": "success",
                "compilerVersion": "0.17.0",
                "pages": 14,
                "elapsedMs": 42,
                "document": path,
                "pdf": pdf,
                "chapters": [{"chapter": 1, "pages": 14}],
                "warnings": [],
                "errors": []
            }))
        } else {
            Ok(json!({"status": "ok"}))
        }
    }

    fn is_healthy(&mut self) -> bool {
        true
    }
}

/// Guard structure ensuring child processes are terminated when dropped.
pub struct ChildGuard(Option<Child>);

impl ChildGuard {
    pub fn spawn(exe: &Path, port: u16) -> std::io::Result<Self> {
        Command::new(exe).arg("--control").arg(port.to_string()).spawn().map(|c| Self(Some(c)))
    }

    pub fn kill(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill();
    }
}

fn free_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("no free port: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);
    Ok(port)
}

fn connect_with_timeout(port: u16, child: &mut ChildGuard, timeout: Duration) -> Result<TcpStream, String> {
    let poll_interval = Duration::from_millis(200);
    let start = Instant::now();

    while start.elapsed() < timeout {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.set_read_timeout(Some(timeout));
            let _ = stream.set_write_timeout(Some(timeout));
            return Ok(stream);
        }

        if let Some(ref mut c) = child.0
            && let Ok(Some(status)) = c.try_wait()
        {
            return Err(format!("designcraft exited prematurely with {status}"));
        }

        std::thread::sleep(poll_interval);
    }

    Err(format!("timed out after {:.1}s connecting to port {port}", timeout.as_secs_f32()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_runtime_capabilities_and_execution() {
        let mut runtime = MockDesignCraftRuntime::default();
        let caps = runtime.capabilities().unwrap();
        assert!(caps.contains(&"pubgen.compileProject".to_string()));

        let res = runtime.execute("pubgen.compileProject", json!({"path": "project.json", "pdf": "test.pdf"})).unwrap();
        assert_eq!(res.get("pages").and_then(Value::as_u64), Some(14));
        assert_eq!(res.get("status").and_then(Value::as_str), Some("success"));
        assert!(runtime.is_healthy());
    }

    #[test]
    fn test_free_port_binds_and_releases() {
        let port = free_port().unwrap();
        assert!(port > 1024);
    }
}
