use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;
use tokio::sync::oneshot;
#[cfg(test)]
use std::time::Duration;

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

// ---------------------------------------------------------------------------
// Tool discovery (resolved once, then cached)
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn tool_candidates(exe: &str) -> Vec<PathBuf> {
    let mut v = vec![PathBuf::from(format!(r"C:\platform-tools\{}.exe", exe))];
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        v.push(
            PathBuf::from(local)
                .join("Android")
                .join("Sdk")
                .join("platform-tools")
                .join(format!("{}.exe", exe)),
        );
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn tool_candidates(_exe: &str) -> Vec<PathBuf> {
    Vec::new()
}

fn resolve_tool(name: &str) -> String {
    if let Ok(p) = which::which(name) {
        return p.to_string_lossy().to_string();
    }
    for c in tool_candidates(name) {
        if c.exists() {
            return c.to_string_lossy().to_string();
        }
    }
    name.to_string()
}

pub fn detect_adb() -> String {
    static ADB: OnceLock<String> = OnceLock::new();
    ADB.get_or_init(|| resolve_tool("adb")).clone()
}

pub fn detect_fastboot() -> String {
    static FB: OnceLock<String> = OnceLock::new();
    FB.get_or_init(|| resolve_tool("fastboot")).clone()
}

// ---------------------------------------------------------------------------
// Process helpers. Every spawn goes through here so that no console window
// can ever pop up on Windows (CREATE_NO_WINDOW).
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// A std `Command` that never opens a console window.
#[allow(unused_mut)]
pub fn std_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// A tokio `Command` that never opens a console window and is killed if dropped.
pub fn tokio_command(program: &str) -> TokioCommand {
    let mut cmd = TokioCommand::new(program);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.kill_on_drop(true);
    cmd
}

fn run_capture(mut cmd: Command, label: &str) -> Result<(i32, String, String), String> {
    cmd.stdin(Stdio::null());
    match cmd.output() {
        Ok(out) => Ok((
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )),
        Err(e) => Err(format!("Failed to execute {}: {}", label, e)),
    }
}

pub fn run_adb_cmd(args: &[&str]) -> Result<(i32, String, String), String> {
    let mut cmd = std_command(&detect_adb());
    cmd.args(args);
    run_capture(cmd, &format!("adb {}", args.join(" ")))
}

pub fn run_fastboot_cmd(args: &[&str]) -> Result<(i32, String, String), String> {
    let mut cmd = std_command(&detect_fastboot());
    cmd.args(args);
    run_capture(cmd, &format!("fastboot {}", args.join(" ")))
}

/// `-s <serial>` prefix, skipped when the serial is missing or blank.
pub fn serial_args(serial: Option<&str>) -> Vec<String> {
    match serial.map(str::trim) {
        Some(s) if !s.is_empty() => vec!["-s".to_string(), s.to_string()],
        _ => Vec::new(),
    }
}

/// Run `adb [-s serial] <rest...>`.
pub fn run_adb(serial: Option<&str>, rest: &[&str]) -> Result<(i32, String, String), String> {
    let mut full = serial_args(serial);
    full.extend(rest.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = full.iter().map(String::as_str).collect();
    run_adb_cmd(&refs)
}

/// Run `fastboot [-s serial] <rest...>`.
pub fn run_fastboot(serial: Option<&str>, rest: &[&str]) -> Result<(i32, String, String), String> {
    let mut full = serial_args(serial);
    full.extend(rest.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = full.iter().map(String::as_str).collect();
    run_fastboot_cmd(&refs)
}

/// Run blocking work off the UI thread / async workers.
pub async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| format!("Background task failed: {}", e))?
}

/// Quote a string for a POSIX shell (Android `sh`).
pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

// ---------------------------------------------------------------------------
// Streaming task runner with progress + cancellation
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct TaskProgress {
    pub id: String,
    pub percent: Option<u32>,
    pub line: Option<String>,
}

fn tasks() -> &'static Mutex<HashMap<String, oneshot::Sender<()>>> {
    static TASKS: OnceLock<Mutex<HashMap<String, oneshot::Sender<()>>>> = OnceLock::new();
    TASKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn busy_serials() -> &'static Mutex<std::collections::HashSet<String>> {
    static BUSY: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    BUSY.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

/// Prevents two long operations (flash/push/...) from hitting one device at once.
pub struct SerialGuard(String);

impl SerialGuard {
    pub fn acquire(serial: Option<&str>) -> Result<Self, String> {
        let key = serial.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("default").to_string();
        let mut set = busy_serials().lock().unwrap_or_else(|p| p.into_inner());
        if !set.insert(key.clone()) {
            return Err("Another operation is already running on this device.".to_string());
        }
        Ok(SerialGuard(key))
    }
}

impl Drop for SerialGuard {
    fn drop(&mut self) {
        busy_serials().lock().unwrap_or_else(|p| p.into_inner()).remove(&self.0);
    }
}

#[tauri::command]
pub fn cancel_task(task_id: String) -> bool {
    let tx = tasks().lock().unwrap_or_else(|p| p.into_inner()).remove(&task_id);
    match tx {
        Some(tx) => tx.send(()).is_ok(),
        None => false,
    }
}

pub fn cancel_all_tasks() {
    let all: Vec<_> = tasks().lock().unwrap_or_else(|p| p.into_inner()).drain().collect();
    for (_, tx) in all {
        let _ = tx.send(());
    }
}

/// Extract a 0-100 progress value from adb / fastboot output lines.
/// Handles `[ 42%] /path`, `Sending sparse 'super' 3/12 (...)` and `Writing 'x'`.
pub fn parse_progress(line: &str) -> Option<u32> {
    static PCT: OnceLock<Regex> = OnceLock::new();
    static SPARSE: OnceLock<Regex> = OnceLock::new();
    let pct = PCT.get_or_init(|| Regex::new(r"\[\s*(\d{1,3})%\]").unwrap());
    let sparse = SPARSE.get_or_init(|| Regex::new(r"(?i)sparse.*?\s(\d+)/(\d+)").unwrap());
    if let Some(c) = pct.captures(line) {
        return c[1].parse::<u32>().ok().map(|v| v.min(100));
    }
    if let Some(c) = sparse.captures(line) {
        let n: u32 = c[1].parse().ok()?;
        let t: u32 = c[2].parse().ok()?;
        return (n * 100).checked_div(t).map(|v| v.min(100));
    }
    None
}

pub type LineSink = Arc<dyn Fn(&str, Option<u32>) + Send + Sync>;

/// Kill a process and everything it spawned (flash scripts start fastboot as a child).
fn kill_tree(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill").args(["-KILL", &format!("-{}", pid)]).status();
    }
    #[cfg(windows)]
    {
        let _ = std_command("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).status();
    }
}

/// Spawn `program args...`, stream every output line (split on `\n` and `\r`,
/// because adb/fastboot redraw progress with `\r`) to `on_line`, and allow
/// cancellation through `cancel_task(task_id)`.
/// Returns the exit code and the last lines of combined output.
pub async fn run_streaming(
    program: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    task_id: String,
    on_line: LineSink,
) -> Result<(i32, String), String> {
    run_streaming_env(program, args, cwd, task_id, on_line, None).await
}

/// Like `run_streaming`, with an optional replacement `PATH` for the child process.
pub async fn run_streaming_env(
    program: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    task_id: String,
    on_line: LineSink,
    path_env: Option<std::ffi::OsString>,
) -> Result<(i32, String), String> {
    let mut cmd = tokio_command(&program);
    if let Some(p) = path_env {
        cmd.env("PATH", p);
    }
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    #[cfg(unix)]
    cmd.process_group(0); // own process group so a cancel can take down scripts and their children
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", program, e))?;
    let pid = child.id();

    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    tasks().lock().unwrap_or_else(|p| p.into_inner()).insert(task_id.clone(), cancel_tx);

    let tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::with_capacity(64)));

    let mut readers = Vec::new();
    for stream in [
        child.stdout.take().map(|s| Box::new(s) as Box<dyn tokio::io::AsyncRead + Unpin + Send>),
        child.stderr.take().map(|s| Box::new(s) as Box<dyn tokio::io::AsyncRead + Unpin + Send>),
    ]
    .into_iter()
    .flatten()
    {
        let sink = on_line.clone();
        let tail = tail.clone();
        readers.push(tokio::spawn(async move {
            let mut stream = stream;
            let mut buf = [0u8; 8192];
            let mut cur: Vec<u8> = Vec::new();
            let flush = |cur: &mut Vec<u8>| {
                if cur.is_empty() {
                    return;
                }
                let line = String::from_utf8_lossy(cur).trim().to_string();
                cur.clear();
                if line.is_empty() {
                    return;
                }
                sink(&line, parse_progress(&line));
                let mut t = tail.lock().unwrap_or_else(|p| p.into_inner());
                if t.len() >= 60 {
                    t.pop_front();
                }
                t.push_back(line);
            };
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        for &b in &buf[..n] {
                            if b == b'\n' || b == b'\r' {
                                flush(&mut cur);
                            } else if cur.len() < 8192 {
                                cur.push(b);
                            }
                        }
                    }
                }
            }
            flush(&mut cur);
        }));
    }

    let mut cancelled = false;
    let status = tokio::select! {
        s = child.wait() => s.map_err(|e| format!("Process failed: {}", e)),
        _ = cancel_rx => {
            cancelled = true;
            if let Some(pid) = pid {
                kill_tree(pid);
            }
            let _ = child.kill().await;
            child.wait().await.map_err(|e| format!("Process failed: {}", e))
        }
    };
    tasks().lock().unwrap_or_else(|p| p.into_inner()).remove(&task_id);
    // Make sure every trailing line has been delivered before returning.
    for r in readers {
        if cancelled {
            r.abort();
        } else {
            // a grandchild that inherited the pipes must not keep us waiting forever
            let _ = tokio::time::timeout(std::time::Duration::from_secs(3), r).await;
        }
    }
    let status = status?;
    if cancelled {
        return Err("Cancelled".to_string());
    }
    let out = tail.lock().unwrap_or_else(|p| p.into_inner()).iter().cloned().collect::<Vec<_>>().join("\n");
    Ok((status.code().unwrap_or(-1), out))
}

/// Convenience: stream an adb command, emitting `task-progress` events.
pub async fn run_adb_streaming(
    app: tauri::AppHandle,
    serial: Option<&str>,
    rest: &[&str],
    task_id: String,
) -> Result<(i32, String), String> {
    use tauri::Emitter;
    let mut args = serial_args(serial);
    args.extend(rest.iter().map(|s| s.to_string()));
    let id = task_id.clone();
    let sink: LineSink = Arc::new(move |line, pct| {
        let _ = app.emit(
            "task-progress",
            TaskProgress { id: id.clone(), percent: pct, line: Some(line.to_string()) },
        );
    });
    run_streaming(detect_adb(), args, None, task_id, sink).await
}

pub fn new_task_id(given: Option<String>) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(1);
    given
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("task-{}", N.fetch_add(1, Ordering::Relaxed)))
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

#[tauri::command]
pub async fn pick_file(
    title: Option<String>,
    filter_name: Option<String>,
    filter_pattern: Option<String>,
) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(ref t) = title {
        dialog = dialog.set_title(t);
    }
    if let (Some(name), Some(pat)) = (filter_name, filter_pattern) {
        let clean_patterns: Vec<&str> = pat
            .split(&[',', ';', ' '][..])
            .map(|s| s.trim_start_matches("*."))
            .filter(|s| !s.is_empty())
            .collect();
        if !clean_patterns.is_empty() {
            dialog = dialog.add_filter(&name, &clean_patterns);
        }
    }
    let res = dialog.pick_file().await;
    Ok(res.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn pick_folder(title: Option<String>) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(ref t) = title {
        dialog = dialog.set_title(t);
    }
    let res = dialog.pick_folder().await;
    Ok(res.map(|f| f.path().to_string_lossy().to_string()))
}

/// Native "save as" dialog; returns the chosen path.
#[tauri::command]
pub async fn pick_save_path(
    title: Option<String>,
    default_name: Option<String>,
) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(ref t) = title {
        dialog = dialog.set_title(t);
    }
    if let Some(ref n) = default_name {
        dialog = dialog.set_file_name(n);
    }
    Ok(dialog.save_file().await.map(|f| f.path().to_string_lossy().to_string()))
}

/// Write a UTF-8 text file (used by log export).
#[tauri::command]
pub async fn save_text_file(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to write {}: {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_adb_percent() {
        assert_eq!(parse_progress("[ 42%] /sdcard/super.img"), Some(42));
        assert_eq!(parse_progress("[100%] /sdcard/a.apk"), Some(100));
        assert_eq!(parse_progress("Performing Streamed Install"), None);
    }

    #[test]
    fn progress_fastboot_sparse() {
        assert_eq!(
            parse_progress("Sending sparse 'super' 3/12 (262140 KB)    OKAY [  6.2s]"),
            Some(25)
        );
        assert_eq!(parse_progress("Sending 'boot_a' (65536 KB)"), None);
    }

    #[test]
    fn quoting() {
        assert_eq!(sh_quote("a'b"), "'a'\\''b'");
        assert_eq!(sh_quote("/sdcard/x y"), "'/sdcard/x y'");
    }

    #[test]
    fn serial_blank_is_skipped() {
        assert!(serial_args(Some("  ")).is_empty());
        assert!(serial_args(None).is_empty());
        assert_eq!(serial_args(Some("abc")), vec!["-s", "abc"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn streaming_splits_cr_and_reports_progress() {
        let seen = Arc::new(Mutex::new(Vec::<(String, Option<u32>)>::new()));
        let s2 = seen.clone();
        let sink: LineSink = Arc::new(move |l, p| s2.lock().unwrap().push((l.to_string(), p)));
        let (code, tail) = run_streaming(
            "sh".into(),
            vec!["-c".into(), "printf '[ 10%%] a\\r[ 60%%] a\\r[100%%] a\\n'; echo done >&2; exit 3".into()],
            None,
            "t-stream".into(),
            sink,
        )
        .await
        .unwrap();
        assert_eq!(code, 3);
        assert!(tail.contains("done"));
        let pcts: Vec<_> = seen.lock().unwrap().iter().filter_map(|(_, p)| *p).collect();
        assert_eq!(pcts, vec![10, 60, 100]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn streaming_can_be_cancelled() {
        let sink: LineSink = Arc::new(|_, _| {});
        let task = tokio::spawn(run_streaming(
            "sh".into(),
            vec!["-c".into(), "sleep 30".into()],
            None,
            "t-cancel".into(),
            sink,
        ));
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(cancel_task("t-cancel".into()));
        let res = tokio::time::timeout(Duration::from_secs(5), task).await.expect("did not stop").unwrap();
        assert_eq!(res.unwrap_err(), "Cancelled");
    }
}
