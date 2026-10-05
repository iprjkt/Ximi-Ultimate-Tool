//! Built-in screen mirroring (no scrcpy): `adb exec-out screenrecord` produces a raw
//! H.264 stream that is forwarded, as binary, to the webview and decoded there with
//! WebCodecs. Input goes through one persistent `adb shell` session.

use crate::utils::{blocking, detect_adb, run_adb, serial_args, std_command};
use serde::Serialize;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::ipc::{Channel, InvokeResponseBody, Response};
use tauri::Emitter;

struct MirrorSession {
    stop: Arc<AtomicBool>,
    video: Arc<Mutex<Option<Child>>>,
    shell: Child,
    shell_in: Arc<Mutex<Option<ChildStdin>>>,
}

fn slot() -> &'static Mutex<Option<MirrorSession>> {
    static S: OnceLock<Mutex<Option<MirrorSession>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

#[derive(Serialize, Clone)]
pub struct MirrorInfo {
    pub width: u32,
    pub height: u32,
    pub device_width: u32,
    pub device_height: u32,
}

/// Parse `wm size` output; an override size wins over the physical size.
pub fn parse_wm_size(out: &str) -> Option<(u32, u32)> {
    let line = out.lines().rev().find(|l| l.contains("size:"))?;
    let dims = line.split(':').nth(1)?.trim();
    let (w, h) = dims.split_once('x')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

/// Scale so the longer side is at most `max_side`, keeping aspect, aligned to 8 px.
pub fn scaled_size(w: u32, h: u32, max_side: u32) -> (u32, u32) {
    let longest = w.max(h) as f64;
    let scale = if longest > max_side as f64 { max_side as f64 / longest } else { 1.0 };
    let align = |v: f64| (((v * scale) / 8.0).round() as u32).max(1) * 8;
    (align(w as f64), align(h as f64))
}

pub fn stop_mirror_blocking() {
    let session = slot().lock().unwrap_or_else(|p| p.into_inner()).take();
    if let Some(mut s) = session {
        s.stop.store(true, Ordering::SeqCst);
        if let Some(mut c) = s.video.lock().unwrap_or_else(|p| p.into_inner()).take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        s.shell_in.lock().unwrap_or_else(|p| p.into_inner()).take();
        let _ = s.shell.kill();
        let _ = s.shell.wait();
    }
}

#[tauri::command]
pub async fn start_mirror(
    app: tauri::AppHandle,
    serial: Option<String>,
    max_size: Option<u32>,
    bitrate_mbps: Option<u32>,
    on_frame: Channel<InvokeResponseBody>,
) -> Result<MirrorInfo, String> {
    blocking(move || {
        stop_mirror_blocking();

        let (_, wm, _) = run_adb(serial.as_deref(), &["shell", "wm", "size"])?;
        let (dw, dh) = parse_wm_size(&wm).ok_or("Could not read the device screen size. Is the device authorized?")?;
        let (w, h) = scaled_size(dw, dh, max_size.unwrap_or(1280).clamp(320, 2560));
        let bitrate = (bitrate_mbps.unwrap_or(8).clamp(1, 40) * 1_000_000).to_string();
        let size_arg = format!("{}x{}", w, h);

        // persistent shell for input events
        let mut shell_cmd = std_command(&detect_adb());
        shell_cmd.args(serial_args(serial.as_deref()));
        shell_cmd.arg("shell").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
        let mut shell = shell_cmd.spawn().map_err(|e| format!("Failed to open input shell: {}", e))?;
        let shell_in = Arc::new(Mutex::new(shell.stdin.take()));

        let stop = Arc::new(AtomicBool::new(false));
        let video: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

        {
            let (stop, video, app) = (stop.clone(), video.clone(), app.clone());
            let serial_args_v = serial_args(serial.as_deref());
            std::thread::spawn(move || {
                let mut end_reason = String::from("Mirror stopped");
                while !stop.load(Ordering::SeqCst) {
                    let mut cmd = std_command(&detect_adb());
                    cmd.args(&serial_args_v);
                    cmd.args(["exec-out", "screenrecord", "--output-format=h264", "--size", &size_arg, "--bit-rate", &bitrate, "-"]);
                    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
                    let mut child = match cmd.spawn() {
                        Ok(c) => c,
                        Err(e) => {
                            end_reason = format!("Failed to start screenrecord: {}", e);
                            break;
                        }
                    };
                    let mut out = child.stdout.take();
                    let err = child.stderr.take();
                    *video.lock().unwrap_or_else(|p| p.into_inner()) = Some(child);

                    let err_text = Arc::new(Mutex::new(String::new()));
                    if let Some(mut e) = err {
                        let t = err_text.clone();
                        std::thread::spawn(move || {
                            let mut s = String::new();
                            let _ = e.read_to_string(&mut s);
                            *t.lock().unwrap_or_else(|p| p.into_inner()) = s;
                        });
                    }

                    let started = Instant::now();
                    let mut total = 0usize;
                    if let Some(o) = out.as_mut() {
                        let mut buf = vec![0u8; 64 * 1024];
                        loop {
                            match o.read(&mut buf) {
                                Ok(0) | Err(_) => break,
                                Ok(n) => {
                                    total += n;
                                    if on_frame.send(InvokeResponseBody::Raw(buf[..n].to_vec())).is_err() {
                                        stop.store(true, Ordering::SeqCst);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if let Some(mut c) = video.lock().unwrap_or_else(|p| p.into_inner()).take() {
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    // screenrecord stops by itself after 3 minutes: just restart it.
                    // If it died right away without data, report why instead of looping.
                    if total < 1024 || started.elapsed() < Duration::from_secs(3) {
                        std::thread::sleep(Duration::from_millis(150));
                        let e = err_text.lock().unwrap_or_else(|p| p.into_inner()).trim().to_string();
                        end_reason = if e.is_empty() { "screenrecord exited unexpectedly".to_string() } else { e };
                        break;
                    }
                }
                let _ = app.emit("mirror-ended", end_reason);
            });
        }

        *slot().lock().unwrap_or_else(|p| p.into_inner()) = Some(MirrorSession { stop, video, shell, shell_in });
        Ok(MirrorInfo { width: w, height: h, device_width: dw, device_height: dh })
    })
    .await
}

#[tauri::command]
pub async fn stop_mirror() -> Result<(), String> {
    blocking(|| {
        stop_mirror_blocking();
        Ok(())
    })
    .await
}

fn send_shell(cmd: &str) -> Result<(), String> {
    let guard = slot().lock().unwrap_or_else(|p| p.into_inner());
    let session = guard.as_ref().ok_or("Mirror is not running")?;
    let mut stdin = session.shell_in.lock().unwrap_or_else(|p| p.into_inner());
    let w = stdin.as_mut().ok_or("Input shell closed")?;
    w.write_all(cmd.as_bytes())
        .and_then(|_| w.write_all(b"\n"))
        .and_then(|_| w.flush())
        .map_err(|e| format!("Input failed: {}", e))
}

#[tauri::command]
pub async fn mirror_tap(x: u32, y: u32) -> Result<(), String> {
    send_shell(&format!("input tap {} {}", x, y))
}

#[tauri::command]
pub async fn mirror_swipe(x1: u32, y1: u32, x2: u32, y2: u32, duration_ms: u32) -> Result<(), String> {
    send_shell(&format!("input swipe {} {} {} {} {}", x1, y1, x2, y2, duration_ms.clamp(10, 5000)))
}

/// Android keycode (3 = HOME, 4 = BACK, 187 = APP_SWITCH, 26 = POWER, 24/25 = volume ...).
#[tauri::command]
pub async fn mirror_key(keycode: u32) -> Result<(), String> {
    send_shell(&format!("input keyevent {}", keycode))
}

/// One PNG frame; used by the webview when WebCodecs is unavailable.
#[tauri::command]
pub async fn capture_frame(serial: Option<String>) -> Result<Response, String> {
    blocking(move || {
        let mut cmd = std_command(&detect_adb());
        cmd.args(serial_args(serial.as_deref()));
        cmd.args(["exec-out", "screencap", "-p"]).stdin(Stdio::null());
        let out = cmd.output().map_err(|e| format!("Failed to run adb: {}", e))?;
        if !out.status.success() || out.stdout.len() < 8 {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        Ok(Response::new(out.stdout))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wm_size_prefers_override() {
        assert_eq!(parse_wm_size("Physical size: 1080x2400\n"), Some((1080, 2400)));
        assert_eq!(
            parse_wm_size("Physical size: 1080x2400\nOverride size: 720x1600\n"),
            Some((720, 1600))
        );
        assert_eq!(parse_wm_size("garbage"), None);
    }

    #[test]
    fn scaling_keeps_aspect_and_alignment() {
        let (w, h) = scaled_size(1080, 2400, 1280);
        assert!(h <= 1280 + 8 && w % 8 == 0 && h % 8 == 0);
        assert_eq!(scaled_size(720, 1280, 2560), (720, 1280));
    }
}
