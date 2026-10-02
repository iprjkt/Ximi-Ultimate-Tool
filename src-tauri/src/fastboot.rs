use crate::adb::DeviceInfo;
use crate::utils::run_fastboot_cmd;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

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

#[tauri::command]
pub fn get_fastboot_devices() -> Result<Vec<DeviceInfo>, String> {
    let (code, stdout, stderr) = run_fastboot_cmd(&["devices"])?;
    if code != 0 {
        return Err(format!("fastboot devices failed: {}", stderr));
    }

    let mut devices = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            let serial = parts[0].to_string();
            let state = if parts.len() > 1 {
                parts[1].to_string()
            } else {
                "fastboot".to_string()
            };
            devices.push(DeviceInfo {
                serial,
                state,
                info: "Fastboot Mode".to_string(),
            });
        }
    }
    Ok(devices)
}

#[tauri::command]
pub fn flash_partition(
    serial: Option<String>,
    partition: String,
    file_path: String,
    disable_verity: bool,
) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    if disable_verity && partition.starts_with("vbmeta") {
        args.extend_from_slice(&["--disable-verity", "--disable-verification"]);
    }
    args.extend_from_slice(&["flash", &partition, &file_path]);

    let (code, stdout, stderr) = run_fastboot_cmd(&args)?;
    let output = format!("{}\n{}", stdout, stderr).trim().to_string();
    if code == 0 {
        Ok(output)
    } else {
        Err(output)
    }
}

#[tauri::command]
pub fn boot_image(serial: Option<String>, file_path: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["boot", &file_path]);

    let (code, stdout, stderr) = run_fastboot_cmd(&args)?;
    let output = format!("{}\n{}", stdout, stderr).trim().to_string();
    if code == 0 {
        Ok(output)
    } else {
        Err(output)
    }
}

#[tauri::command]
pub fn reboot_fastboot(serial: Option<String>, mode: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.push("reboot");
    match mode.as_str() {
        "bootloader" => args.push("bootloader"),
        "recovery" => args.push("recovery"),
        "edl" => args.push("edl"),
        _ => {}
    }

    let (code, _stdout, stderr) = run_fastboot_cmd(&args)?;
    if code == 0 {
        Ok(format!("Rebooted fastboot to {}", mode))
    } else {
        Err(stderr)
    }
}

#[tauri::command]
pub fn parse_rom_directory(folder_path: String, script_name: Option<String>) -> Result<RomInfo, String> {
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
            let re_flash = Regex::new(r"fastboot(?:\s+[\$%\*]+)?(?:\s+-s\s+\S+)?\s+flash\s+([^\s]+)\s+(.+)$").unwrap();
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
                        .split('/')
                        .last()
                        .unwrap_or(img_part)
                        .split('\\')
                        .last()
                        .unwrap_or(img_part)
                        .replace('`', "")
                        .replace('"', "")
                        .replace('\'', "")
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

#[tauri::command]
pub async fn flash_rom(
    app: tauri::AppHandle,
    serial: Option<String>,
    folder_path: String,
    script_name: String,
    excluded_partitions: Vec<String>,
) -> Result<String, String> {
    use std::process::Stdio;
    use tauri::Emitter;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command as TokioCommand;

    let dir = Path::new(&folder_path);
    if !dir.is_dir() {
        return Err("Invalid ROM directory".to_string());
    }

    let script_file = dir.join(&script_name);
    if !script_file.exists() {
        return Err(format!("Script {} not found in ROM folder", script_name));
    }

    let is_windows = cfg!(target_os = "windows");
    let excluded_set: HashSet<String> = excluded_partitions.into_iter().collect();

    // If partitions were excluded, parse the script and execute line-by-line skipping excluded ones
    if !excluded_set.is_empty() {
        let content = fs::read_to_string(&script_file).map_err(|e| format!("Failed to read script: {}", e))?;
        let re_flash = Regex::new(r"fastboot(?:\s+[\$%\*]+)?(?:\s+-s\s+\S+)?\s+flash\s+([^\s]+)\s+(.+)$").unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let lower = trimmed.to_lowercase();
            if trimmed.is_empty() || lower.starts_with('#') || lower.starts_with("rem") || lower.starts_with("::") {
                continue;
            }

            if let Some(caps) = re_flash.captures(trimmed) {
                let part = caps[1].trim();
                if excluded_set.contains(part) {
                    let _ = app.emit("fastboot-log", format!("[SKIP] Skipping excluded partition: {}", part));
                    continue;
                }
            }

            let pct = ((idx + 1) as f32 / total_lines as f32 * 100.0) as u32;
            let _ = app.emit("fastboot-progress", pct);
            let _ = app.emit("fastboot-log", format!("> {}", trimmed));

            let mut cmd = if is_windows {
                let mut c = TokioCommand::new("cmd");
                c.arg("/c");
                c
            } else {
                let mut c = TokioCommand::new("bash");
                c.arg("-c");
                c
            };

            let cmd_with_serial = if is_windows {
                let mut s_cmd = trimmed
                    .replace("%~dp0", &format!("{}\\", dir.display()))
                    .replace("pause", "echo [CONTINUE]");
                if let Some(ref s) = serial {
                    if s_cmd.contains("fastboot %*") {
                        s_cmd = s_cmd.replace("fastboot %*", &format!("fastboot -s {}", s));
                    } else if s_cmd.contains("fastboot $*") {
                        s_cmd = s_cmd.replace("fastboot $*", &format!("fastboot -s {}", s));
                    } else {
                        s_cmd = s_cmd.replace("fastboot ", &format!("fastboot -s {} ", s));
                    }
                } else {
                    s_cmd = s_cmd.replace("fastboot %*", "fastboot")
                                 .replace("fastboot $*", "fastboot");
                }
                s_cmd
            } else {
                let mut s_cmd = trimmed.to_string();
                if let Some(ref s) = serial {
                    if s_cmd.contains("fastboot $*") {
                        s_cmd = s_cmd.replace("fastboot $*", &format!("fastboot -s {}", s));
                    } else if s_cmd.contains("fastboot %*") {
                        s_cmd = s_cmd.replace("fastboot %*", &format!("fastboot -s {}", s));
                    } else {
                        s_cmd = s_cmd.replace("fastboot ", &format!("fastboot -s {} ", s));
                    }
                } else {
                    s_cmd = s_cmd.replace("fastboot $*", "fastboot")
                                 .replace("fastboot %*", "fastboot");
                }
                s_cmd
            };

            cmd.arg(&cmd_with_serial);
            cmd.current_dir(dir);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| format!("Failed to execute command: {}", e))?;
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            if let Some(out) = stdout {
                let app_c = app.clone();
                tokio::spawn(async move {
                    let mut reader = BufReader::new(out).lines();
                    while let Ok(Some(l)) = reader.next_line().await {
                        let _ = app_c.emit("fastboot-log", l);
                    }
                });
            }
            if let Some(err) = stderr {
                let app_c = app.clone();
                tokio::spawn(async move {
                    let mut reader = BufReader::new(err).lines();
                    while let Ok(Some(l)) = reader.next_line().await {
                        let _ = app_c.emit("fastboot-log", l);
                    }
                });
            }

            let status = child.wait().await.map_err(|e| format!("Command execution failed: {}", e))?;
            if !status.success() {
                let _ = app.emit("fastboot-log", format!("[WARN] Command returned status code: {:?}", status.code()));
            }
        }
        let _ = app.emit("fastboot-progress", 100);
        let _ = app.emit("fastboot-log", "[SUCCESS] Flashing sequence completed!".to_string());
        return Ok("Flashing completed successfully!".to_string());
    }

    // Direct script execution
    let mut cmd = if is_windows {
        let mut c = TokioCommand::new("cmd");
        c.arg("/c").arg(&script_file);
        if let Some(ref s) = serial {
            c.arg("-s").arg(s);
        }
        c
    } else {
        let mut c = TokioCommand::new("bash");
        c.arg(&script_file);
        if let Some(ref s) = serial {
            c.arg("-s").arg(s);
        }
        c
    };

    cmd.current_dir(dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let _ = app.emit("fastboot-log", format!("[START] Executing script: {} in {}", script_name, folder_path));
    let mut child = cmd.spawn().map_err(|e| format!("Failed to execute script: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(out) = stdout {
        let app_c = app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(l)) = reader.next_line().await {
                let _ = app_c.emit("fastboot-log", l);
            }
        });
    }

    if let Some(err) = stderr {
        let app_c = app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(l)) = reader.next_line().await {
                let _ = app_c.emit("fastboot-log", l);
            }
        });
    }

    let status = child.wait().await.map_err(|e| format!("Script failed: {}", e))?;
    let _ = app.emit("fastboot-progress", 100);
    if status.success() {
        let _ = app.emit("fastboot-log", "[SUCCESS] Full ROM flash finished successfully!".to_string());
        Ok("ROM flashed successfully!".to_string())
    } else {
        let err_msg = format!("Script exited with status code: {:?}", status.code());
        let _ = app.emit("fastboot-log", format!("[ERROR] {}", err_msg));
        Err(err_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rom_directory_os_filtering() {
        let test_rom_path = "/run/media/fxxyz73/sigeonpex/ROM_Fastboot_Recovery_ISO/tanzanite_eea_global_images_OS2.0.214.0.VOGEUXM_15.0";
        if !Path::new(test_rom_path).exists() {
            return;
        }

        let res = parse_rom_directory(test_rom_path.to_string(), None);
        assert!(res.is_ok(), "Failed to parse ROM: {:?}", res.err());
        let rom_info = res.unwrap();

        // On Linux, should only contain .sh scripts
        #[cfg(not(target_os = "windows"))]
        {
            assert!(!rom_info.scripts.is_empty());
            for script in &rom_info.scripts {
                assert!(script.ends_with(".sh"), "Expected .sh script on Linux, found: {}", script);
            }
            assert_eq!(rom_info.default_script, "flash_all.sh");
        }

        // On Windows, should only contain .bat scripts
        #[cfg(target_os = "windows")]
        {
            assert!(!rom_info.scripts.is_empty());
            for script in &rom_info.scripts {
                assert!(script.ends_with(".bat"), "Expected .bat script on Windows, found: {}", script);
            }
            assert_eq!(rom_info.default_script, "flash_all.bat");
        }

        // Verify partitions are parsed
        assert!(!rom_info.partitions.is_empty(), "Partitions should not be empty");
        println!("Loaded scripts: {:?}", rom_info.scripts);
        println!("Default script: {}", rom_info.default_script);
        println!("Loaded partitions count: {}", rom_info.partitions.len());
    }
}

