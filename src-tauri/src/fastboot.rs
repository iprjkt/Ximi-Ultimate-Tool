use crate::adb::DeviceInfo;
use crate::utils::{
    blocking, detect_fastboot, new_task_id, run_fastboot, run_streaming, LineSink, SerialGuard, TaskProgress,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tauri::Emitter;

fn flash_line_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"fastboot(?:\s+[\$%\*]+)?(?:\s+-s\s+\S+)?\s+flash\s+([^\s]+)\s+(.+)$").unwrap()
    })
}

/// Serials are interpolated into script lines, so only allow what real serials contain.
fn validate_serial(serial: &str) -> Result<(), String> {
    if !serial.is_empty() && serial.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-')) {
        Ok(())
    } else {
        Err(format!("Invalid device serial: {}", serial))
    }
}

fn clean_serial(serial: &Option<String>) -> Result<Option<String>, String> {
    match serial.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => {
            validate_serial(s)?;
            Ok(Some(s.to_string()))
        }
        None => Ok(None),
    }
}

/// Forward every output line to the flasher log, the progress bar and the generic task event.
fn flasher_sink(app: &tauri::AppHandle, id: &str) -> LineSink {
    let (app, id) = (app.clone(), id.to_string());
    Arc::new(move |line, pct| {
        let _ = app.emit("fastboot-log", line.to_string());
        if let Some(p) = pct {
            let _ = app.emit("fastboot-progress", p);
        }
        let _ = app.emit("task-progress", TaskProgress { id: id.clone(), percent: pct, line: Some(line.to_string()) });
    })
}

/// Make scripts find fastboot even when it is not on PATH (e.g. C:\platform-tools).
fn path_with_fastboot() -> Option<std::ffi::OsString> {
    let fb = PathBuf::from(detect_fastboot());
    let dir = fb.parent().filter(|d| !d.as_os_str().is_empty())?.to_path_buf();
    let mut paths = vec![dir];
    if let Some(cur) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&cur));
    }
    std::env::join_paths(paths).ok()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartitionFlashItem {
    pub index: usize,
    pub partition: String,
    pub image_file: String,
    pub is_dangerous: bool,
    pub raw_command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RomInfo {
    pub folder_path: String,
    pub scripts: Vec<String>,
    pub default_script: String,
    pub partitions: Vec<PartitionFlashItem>,
}

pub fn is_partition_dangerous(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("preloader")
        || lower.contains("nvram")
        || lower.contains("nvdata")
        || lower.contains("protect")
        || lower.contains("sec1")
        || lower.contains("proinfo")
        || lower.contains("efuse")
        || lower.contains("persist")
        || lower.contains("devinfo")
        || lower == "misc"
}

fn parse_fastboot_devices(stdout: &str) -> Vec<DeviceInfo> {
    stdout
        .lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }
            Some(DeviceInfo {
                serial: parts[0].to_string(),
                state: parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "fastboot".to_string()),
                info: "Fastboot Mode".to_string(),
            })
        })
        .collect()
}

#[tauri::command]
pub async fn get_fastboot_devices() -> Result<Vec<DeviceInfo>, String> {
    blocking(|| {
        let (code, stdout, stderr) = run_fastboot(None, &["devices"])?;
        if code != 0 {
            return Err(format!("fastboot devices failed: {}", stderr.trim()));
        }
        Ok(parse_fastboot_devices(&stdout))
    })
    .await
}

async fn run_fastboot_task(
    app: tauri::AppHandle,
    serial: Option<String>,
    mut rest: Vec<String>,
    task_id: Option<String>,
) -> Result<String, String> {
    let serial = clean_serial(&serial)?;
    let _guard = SerialGuard::acquire(serial.as_deref())?;
    let id = new_task_id(task_id);
    let sink = flasher_sink(&app, &id);
    let mut args = Vec::new();
    if let Some(s) = &serial {
        args.push("-s".to_string());
        args.push(s.clone());
    }
    args.append(&mut rest);
    let _ = app.emit("fastboot-progress", 0u32);
    let (code, out) = run_streaming(detect_fastboot(), args, None, id, sink).await?;
    if code == 0 {
        let _ = app.emit("fastboot-progress", 100u32);
        Ok(out)
    } else {
        Err(out)
    }
}

#[tauri::command]
pub async fn flash_partition(
    app: tauri::AppHandle,
    serial: Option<String>,
    partition: String,
    file_path: String,
    disable_verity: bool,
    task_id: Option<String>,
) -> Result<String, String> {
    if !Path::new(&file_path).is_file() {
        return Err(format!("Image file not found: {}", file_path));
    }
    let mut rest = Vec::new();
    if disable_verity && partition.starts_with("vbmeta") {
        rest.push("--disable-verity".to_string());
        rest.push("--disable-verification".to_string());
    }
    rest.extend(["flash".to_string(), partition, file_path]);
    run_fastboot_task(app, serial, rest, task_id).await
}

#[tauri::command]
pub async fn boot_image(
    app: tauri::AppHandle,
    serial: Option<String>,
    file_path: String,
    task_id: Option<String>,
) -> Result<String, String> {
    if !Path::new(&file_path).is_file() {
        return Err(format!("Image file not found: {}", file_path));
    }
    run_fastboot_task(app, serial, vec!["boot".to_string(), file_path], task_id).await
}

#[tauri::command]
pub async fn reboot_fastboot(serial: Option<String>, mode: String) -> Result<String, String> {
    blocking(move || {
        let serial = clean_serial(&serial)?;
        let mut rest = vec!["reboot"];
        match mode.as_str() {
            "bootloader" | "recovery" | "edl" => rest.push(mode.as_str()),
            "system" | "" => {}
            other => return Err(format!("Unknown reboot mode: {}", other)),
        }
        let (code, _stdout, stderr) = run_fastboot(serial.as_deref(), &rest)?;
        if code == 0 { Ok(format!("Rebooted fastboot to {}", mode)) } else { Err(stderr) }
    })
    .await
}

#[tauri::command]
pub async fn parse_rom_directory(folder_path: String, script_name: Option<String>) -> Result<RomInfo, String> {
    blocking(move || parse_rom_directory_sync(folder_path, script_name)).await
}

pub fn parse_rom_directory_sync(folder_path: String, script_name: Option<String>) -> Result<RomInfo, String> {
    let dir = Path::new(&folder_path);
    if !dir.is_dir() {
        return Err("Specified ROM path is not a valid directory.".to_string());
    }

    let is_windows = cfg!(target_os = "windows");
    let preferred_ext = if is_windows { ".bat" } else { ".sh" };

    // Collect all flash scripts
    let mut all_scripts = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("flash_") && (file_name.ends_with(".sh") || file_name.ends_with(".bat")) {
                    all_scripts.push(file_name.to_string());
                }
            }
        }
    }

    // Filter scripts according to host OS (.bat for Windows, .sh for Linux/macOS)
    let mut available_scripts: Vec<String> = all_scripts
        .iter()
        .filter(|s| s.ends_with(preferred_ext))
        .cloned()
        .collect();

    // Fallback if no scripts match the current OS extension
    if available_scripts.is_empty() {
        available_scripts = all_scripts;
    }

    // Sort so flash_all is first, then flash_all_except_*, then flash_all_lock
    available_scripts.sort_by(|a, b| {
        let score = |s: &str| {
            if s.starts_with("flash_all.") {
                0
            } else if s.contains("except") {
                1
            } else if s.contains("lock") {
                2
            } else {
                3
            }
        };
        score(a).cmp(&score(b))
    });

    let default_script = if let Some(ref custom_script) = script_name {
        if available_scripts.contains(custom_script) {
            custom_script.clone()
        } else {
            available_scripts.first().cloned().unwrap_or_else(|| {
                if is_windows { "flash_all.bat".to_string() } else { "flash_all.sh".to_string() }
            })
        }
    } else {
        available_scripts
            .first()
            .cloned()
            .unwrap_or_else(|| {
                if is_windows { "flash_all.bat".to_string() } else { "flash_all.sh".to_string() }
            })
    };

    let mut items = Vec::new();

    // Prefer parsing the selected/default script
    let script_path = dir.join(&default_script);
    if script_path.exists() {
        if let Ok(content) = fs::read_to_string(&script_path) {
            let re_flash = flash_line_regex();
            let mut idx = 1;
            for line in content.lines() {
                let trimmed = line.trim();
                let lower = trimmed.to_lowercase();
                if lower.starts_with('#') || lower.starts_with("rem") || lower.starts_with("::") || trimmed.is_empty() {
                    continue;
                }
                if let Some(caps) = re_flash.captures(trimmed) {
                    let part = caps[1].trim().to_string();
                    let raw_img = caps[2].trim();
                    let img_part = raw_img.split("||").next().unwrap_or(raw_img).trim();
                    let img_part = img_part.split('|').next().unwrap_or(img_part).trim();
                    let clean_filename = img_part
                        .rsplit(['/', '\\'])
                        .next()
                        .unwrap_or(img_part)
                        .replace(['`', '"', '\''], "")
                        .trim()
                        .to_string();

                    let is_dang = is_partition_dangerous(&part);
                    items.push(PartitionFlashItem {
                        index: idx,
                        partition: part,
                        image_file: clean_filename,
                        is_dangerous: is_dang,
                        raw_command: trimmed.to_string(),
                    });
                    idx += 1;
                }
            }
        }
    }

    // Fallback if no flash lines found in script: search images/ directory
    if items.is_empty() {
        let mut scan_dir = dir.to_path_buf();
        let images_sub = dir.join("images");
        if images_sub.is_dir() {
            scan_dir = images_sub;
        }

        if let Ok(entries) = fs::read_dir(&scan_dir) {
            let mut idx = 1;
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "img" || ext == "bin" {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let part_name = file_stem.to_string();
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                        let is_dang = is_partition_dangerous(&part_name);
                        items.push(PartitionFlashItem {
                            index: idx,
                            partition: part_name.clone(),
                            image_file: file_name,
                            is_dangerous: is_dang,
                            raw_command: format!("fastboot flash {} {}", part_name, path.display()),
                        });
                        idx += 1;
                    }
                }
            }
        }
    }

    Ok(RomInfo {
        folder_path,
        scripts: available_scripts,
        default_script,
        partitions: items,
    })
}

/// Replace the first `fastboot [%*|$*]` on a script line with `fastboot [-s serial]`.
fn inject_serial(line: &str, serial: Option<&str>) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r#"\bfastboot(?:\s+(?:%\*|\$\*|"\$@"|\$@))?"#).unwrap());
    let replacement = match serial {
        Some(s) => format!("fastboot -s {}", s),
        None => "fastboot".to_string(),
    };
    re.replace(line, regex::NoExpand(&replacement)).to_string()
}

fn is_fatal_fastboot_line(line: &str) -> bool {
    let l = line.to_lowercase();
    ["flash", "erase", "set_active", "delete-logical", "create-logical", "resize-logical"]
        .iter()
        .any(|k| l.contains(&format!(" {} ", k)) || l.ends_with(&format!(" {}", k)))
}

#[tauri::command]
pub async fn flash_rom(
    app: tauri::AppHandle,
    serial: Option<String>,
    folder_path: String,
    script_name: String,
    excluded_partitions: Vec<String>,
    task_id: Option<String>,
) -> Result<String, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.is_dir() {
        return Err("Invalid ROM directory".to_string());
    }
    let script_file = dir.join(&script_name);
    if script_name.contains(['/', '\\']) || !script_file.is_file() {
        return Err(format!("Script {} not found in ROM folder", script_name));
    }
    let serial = clean_serial(&serial)?;
    let _guard = SerialGuard::acquire(serial.as_deref())?;
    let id = new_task_id(task_id);
    let is_windows = cfg!(target_os = "windows");
    let excluded: HashSet<String> = excluded_partitions.into_iter().collect();
    let sink = flasher_sink(&app, &id);
    let path_env = path_with_fastboot();

    let (shell, shell_flag) = if is_windows { ("cmd", "/c") } else { ("bash", "-c") };

    let content = fs::read_to_string(&script_file).map_err(|e| format!("Failed to read script: {}", e))?;
    let re_flash = flash_line_regex();

    // Partitions excluded -> run the script line by line, skipping excluded flashes.
    if !excluded.is_empty() {
        let lines: Vec<&str> = content
            .lines()
            .map(str::trim)
            .filter(|l| {
                let low = l.to_lowercase();
                !l.is_empty() && !low.starts_with('#') && !low.starts_with("rem ") && !low.starts_with("::") && !low.starts_with("echo") && !low.starts_with('@') && low.contains("fastboot")
            })
            .collect();
        let total = lines.len().max(1);

        for (idx, line) in lines.iter().enumerate() {
            if let Some(caps) = re_flash.captures(line) {
                let part = caps[1].trim();
                if excluded.contains(part) {
                    sink(&format!("[SKIP] Skipping excluded partition: {}", part), None);
                    continue;
                }
            }
            let pct = (idx as u32 * 100) / total as u32;
            let _ = app.emit("fastboot-progress", pct);
            sink(&format!("> {}", line), None);

            let mut cmd_line = inject_serial(line, serial.as_deref());
            if is_windows {
                cmd_line = cmd_line.replace("%~dp0", &format!("{}\\", dir.display()));
            }
            let args = vec![shell_flag.to_string(), cmd_line];
            // env PATH tweak is applied through a wrapper command below
            let (code, _out) = run_script_line(shell, args, &dir, path_env.clone(), id.clone(), sink.clone()).await?;
            if code != 0 {
                if is_fatal_fastboot_line(line) {
                    let msg = format!("Command failed (exit {}): {}", code, line);
                    sink(&format!("[ERROR] {}", msg), None);
                    return Err(msg);
                }
                sink(&format!("[WARN] Command returned status code: {}", code), None);
            }
        }
        let _ = app.emit("fastboot-progress", 100u32);
        sink("[SUCCESS] Flashing sequence completed!", None);
        return Ok("Flashing completed successfully!".to_string());
    }

    // Whole script. Progress = flashed partitions / flash lines found in the script.
    let total_flashes = content.lines().filter(|l| re_flash.is_match(l.trim())).count().max(1) as u32;
    let done = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let progress_sink: LineSink = {
        let (inner, app, done) = (sink.clone(), app.clone(), done.clone());
        Arc::new(move |line, pct| {
            inner(line, pct);
            if line.starts_with("Writing ") || line.starts_with("Finished.") {
                let n = done.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                let _ = app.emit("fastboot-progress", (n * 100 / total_flashes).min(99));
            }
        })
    };

    sink(&format!("[START] Executing script: {} in {}", script_name, folder_path), None);
    let mut args: Vec<String> = Vec::new();
    if is_windows {
        args.push("/c".into());
    }
    args.push(script_file.to_string_lossy().to_string());
    if let Some(s) = &serial {
        args.push("-s".into());
        args.push(s.clone());
    }
    let program = if is_windows { "cmd" } else { "bash" };
    let (code, _out) = run_script_line(program, args, &dir, path_env, id, progress_sink).await?;
    let _ = app.emit("fastboot-progress", 100u32);
    if code == 0 {
        sink("[SUCCESS] Full ROM flash finished successfully!", None);
        Ok("ROM flashed successfully!".to_string())
    } else {
        let err_msg = format!("Script exited with status code: {}", code);
        sink(&format!("[ERROR] {}", err_msg), None);
        Err(err_msg)
    }
}

/// `run_streaming` plus a PATH that contains the detected fastboot directory.
async fn run_script_line(
    program: &str,
    args: Vec<String>,
    dir: &Path,
    path_env: Option<std::ffi::OsString>,
    id: String,
    sink: LineSink,
) -> Result<(i32, String), String> {
    crate::utils::run_streaming_env(program.to_string(), args, Some(dir.to_path_buf()), id, sink, path_env).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_rom() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ximi_rom_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("images")).unwrap();
        fs::write(
            dir.join("flash_all.sh"),
            "#!/bin/bash\nfastboot $* erase userdata\nfastboot $* flash crclist \"$(dirname \"$0\")\"/images/crclist.txt\nfastboot $* flash super `dirname $0`/images/super.img\nfastboot $* flash preloader images/preloader.img\nfastboot $* reboot\n",
        )
        .unwrap();
        fs::write(dir.join("flash_all.bat"), "fastboot %* flash boot %~dp0images\\boot.img\n").unwrap();
        fs::write(dir.join("flash_all_lock.sh"), "fastboot $* oem lock\n").unwrap();
        dir
    }

    #[test]
    fn rom_directory_is_parsed() {
        let dir = fixture_rom();
        let info = parse_rom_directory_sync(dir.to_string_lossy().to_string(), None).unwrap();
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(info.default_script, "flash_all.sh");
            assert!(info.scripts.iter().all(|s| s.ends_with(".sh")));
            let parts: Vec<_> = info.partitions.iter().map(|p| p.partition.as_str()).collect();
            assert_eq!(parts, vec!["crclist", "super", "preloader"]);
            assert_eq!(info.partitions[1].image_file, "super.img");
            assert!(info.partitions[2].is_dangerous);
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn serial_injection_into_script_lines() {
        assert_eq!(inject_serial("fastboot $* flash boot boot.img", Some("abc123")), "fastboot -s abc123 flash boot boot.img");
        assert_eq!(inject_serial("fastboot %* erase userdata", None), "fastboot erase userdata");
        assert_eq!(inject_serial("fastboot reboot", Some("abc")), "fastboot -s abc reboot");
    }

    #[test]
    fn serial_validation() {
        assert!(validate_serial("1a2b3c4d").is_ok());
        assert!(validate_serial("192.168.1.5:5555").is_ok());
        assert!(validate_serial("x; rm -rf /").is_err());
        assert!(validate_serial("").is_err());
    }

    #[test]
    fn fatal_lines() {
        assert!(is_fatal_fastboot_line("fastboot -s a flash boot boot.img"));
        assert!(is_fatal_fastboot_line("fastboot erase userdata"));
        assert!(!is_fatal_fastboot_line("fastboot getvar product"));
        assert!(!is_fatal_fastboot_line("fastboot reboot"));
    }

    #[test]
    fn fastboot_device_list() {
        let d = parse_fastboot_devices("1a2b3c\tfastboot\n\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].serial, "1a2b3c");
    }
}
