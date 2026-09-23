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
    pub script_name: String,
    pub partitions: Vec<PartitionFlashItem>,
}

fn get_dangerous_partitions() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    s.insert("preloader");
    s.insert("preloader_a");
    s.insert("preloader_b");
    s.insert("persist");
    s.insert("devinfo");
    s.insert("misc");
    s.insert("nvram");
    s.insert("nvdata");
    s.insert("sec1");
    s.insert("proinfo");
    s.insert("protect1");
    s.insert("protect2");
    s
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
pub fn parse_rom_directory(folder_path: String) -> Result<RomInfo, String> {
    let dir = Path::new(&folder_path);
    if !dir.is_dir() {
        return Err("Specified ROM path is not a valid directory.".to_string());
    }

    // Try finding flash_all.sh (Linux/macOS) or flash_all.bat (Windows)
    let candidates = [
        "flash_all.sh",
        "flash_all.bat",
        "flash_all_except_storage.sh",
        "flash_all_except_storage.bat",
    ];

    let mut found_script = None;
    for cand in candidates {
        let p = dir.join(cand);
        if p.exists() {
            found_script = Some((cand.to_string(), p));
            break;
        }
    }

    let dangerous_set = get_dangerous_partitions();
    let mut items = Vec::new();

    if let Some((script_name, script_path)) = found_script {
        if let Ok(content) = fs::read_to_string(&script_path) {
            let re_flash = Regex::new(r"fastboot\s+(?:-s\s+\S+\s+)?flash\s+([^\s]+)\s+([^\s\r\n]+)").unwrap();
            let mut idx = 1;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with("rem") || trimmed.is_empty() {
                    continue;
                }
                if let Some(caps) = re_flash.captures(trimmed) {
                    let part = caps[1].to_string();
                    let raw_img = caps[2].to_string();
                    let is_dang = dangerous_set.contains(part.as_str());
                    items.push(PartitionFlashItem {
                        index: idx,
                        partition: part,
                        image_file: raw_img,
                        is_dangerous: is_dang,
                        raw_command: trimmed.to_string(),
                    });
                    idx += 1;
                }
            }
        }
        return Ok(RomInfo {
            folder_path,
            script_name,
            partitions: items,
        });
    }

    // Fallback: search for *.img files in folder or images/ subdirectory
    let mut scan_dir = dir.to_path_buf();
    let images_sub = dir.join("images");
    if images_sub.is_dir() {
        scan_dir = images_sub;
    }

    if let Ok(entries) = fs::read_dir(&scan_dir) {
        let mut idx = 1;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("img") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let part_name = file_stem.to_string();
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let is_dang = dangerous_set.contains(part_name.as_str());
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

    Ok(RomInfo {
        folder_path,
        script_name: "Manual images list".to_string(),
        partitions: items,
    })
}
