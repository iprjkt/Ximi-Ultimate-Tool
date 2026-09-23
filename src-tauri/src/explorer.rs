use crate::utils::{format_bytes, run_adb_cmd};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub link_target: String,
    pub size: u64,
    pub size_formatted: String,
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub date: String,
}

fn exec_android_shell(
    serial: Option<&str>,
    cmd: &str,
    root_mode: bool,
) -> Result<(i32, String, String), String> {
    let mut args = Vec::new();
    if let Some(s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    let escaped = cmd.replace('"', "\\\"");
    let full_cmd = if root_mode {
        format!("su -c \"{}\"", escaped)
    } else {
        cmd.to_string()
    };
    args.extend_from_slice(&["shell", &full_cmd]);
    run_adb_cmd(&args)
}

#[tauri::command]
pub fn list_pc_directory(dir_path: String) -> Result<Vec<FileItem>, String> {
    let mut resolved_path = PathBuf::from(&dir_path);
    if dir_path.starts_with('~') {
        if let Some(home) = dirs_home() {
            resolved_path = home.join(dir_path.trim_start_matches("~/").trim_start_matches('~'));
        }
    }
    if !resolved_path.exists() {
        return Err(format!("Directory does not exist: {}", resolved_path.display()));
    }

    let mut items = Vec::new();
    let entries = match fs::read_dir(&resolved_path) {
        Ok(e) => e,
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_link = meta.file_type().is_symlink();
        let is_dir = if is_link {
            path.is_dir()
        } else {
            meta.is_dir()
        };

        let link_target = if is_link {
            fs::read_link(&path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let size = if is_dir { 0 } else { meta.len() };
        let size_formatted = if is_dir {
            "--".to_string()
        } else {
            format_bytes(size)
        };

        let date_str = meta
            .modified()
            .ok()
            .and_then(|t| {
                t.duration_since(SystemTime::UNIX_EPOCH).ok().map(|d| {
                    let secs = d.as_secs();
                    format_epoch_to_date(secs)
                })
            })
            .unwrap_or_else(|| "-".to_string());

        items.push(FileItem {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            is_link,
            link_target,
            size,
            size_formatted,
            permissions: if is_dir { "drwxr-xr-x".into() } else { "-rw-r--r--".into() },
            owner: "user".into(),
            group: "user".into(),
            date: date_str,
        });
    }

    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(items)
}

#[tauri::command]
pub fn list_android_directory(
    serial: Option<String>,
    dir_path: String,
    root_mode: bool,
) -> Result<Vec<FileItem>, String> {
    let s_ref = serial.as_deref();

    // Critical fix: ensure trailing slash so symlinked folders (like /sdcard) list contents
    let clean = dir_path.trim_end_matches('/');
    let target_path = if clean.is_empty() {
        "/".to_string()
    } else {
        format!("{}/", clean)
    };

    let cmd = format!("ls -la '{}'", target_path);
    let (code, stdout, stderr) = exec_android_shell(s_ref, &cmd, root_mode)?;

    if code != 0 && stdout.trim().is_empty() {
        if root_mode {
            // Fallback without root
            return list_android_directory(serial, dir_path, false);
        }
        return Err(stderr);
    }

    let mut items = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total ") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let perms = parts[0];
        let is_dir = perms.starts_with('d');
        let is_link = perms.starts_with('l');

        let (owner, group, size, date_val, raw_name) = if parts.len() >= 8 {
            let o = parts[2].to_string();
            let g = parts[3].to_string();
            let sz = parts[4].parse::<u64>().unwrap_or(0);
            let dt = format!("{} {}", parts[5], parts[6]);
            let nm = parts[7..].join(" ");
            (o, g, sz, dt, nm)
        } else {
            let o = parts[1].to_string();
            let g = parts[2].to_string();
            let sz = parts[3].parse::<u64>().unwrap_or(0);
            let dt = parts[4].to_string();
            let nm = parts[5..].join(" ");
            (o, g, sz, dt, nm)
        };

        let mut name = raw_name.clone();
        let mut link_target = String::new();
        if raw_name.contains(" -> ") {
            let n_parts: Vec<&str> = raw_name.split(" -> ").collect();
            name = n_parts[0].to_string();
            link_target = n_parts.get(1).unwrap_or(&"").to_string();
        }

        if name == "." || name == ".." {
            continue;
        }

        let full_path = format!("{}/{}", clean, name).replace("//", "/");
        let size_fmt = if is_dir {
            "--".to_string()
        } else {
            format_bytes(size)
        };

        items.push(FileItem {
            name,
            path: full_path,
            is_dir,
            is_link,
            link_target,
            size,
            size_formatted: size_fmt,
            permissions: perms.to_string(),
            owner,
            group,
            date: date_val,
        });
    }

    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(items)
}

#[tauri::command]
pub fn transfer_pc_to_android(
    serial: Option<String>,
    local_path: String,
    remote_dir: String,
    root_mode: bool,
) -> Result<String, String> {
    let s_ref = serial.as_deref();
    let file_name = Path::new(&local_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());

    let dest_clean = remote_dir.trim_end_matches('/');
    let target_dest = format!("{}/{}", dest_clean, file_name);

    if !root_mode || dest_clean.starts_with("/sdcard") || dest_clean.starts_with("/storage") {
        let mut args = Vec::new();
        if let Some(s) = s_ref {
            args.extend_from_slice(&["-s", s]);
        }
        args.extend_from_slice(&["push", &local_path, &target_dest]);
        let (code, _stdout, stderr) = run_adb_cmd(&args)?;
        if code == 0 {
            return Ok(format!("Pushed successfully: {}", target_dest));
        } else {
            return Err(stderr);
        }
    }

    // Protected path under root: push to /data/local/tmp/ then move with root
    let tmp_remote = format!("/data/local/tmp/{}", file_name);
    let mut args = Vec::new();
    if let Some(s) = s_ref {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["push", &local_path, &tmp_remote]);
    let (code, _, stderr) = run_adb_cmd(&args)?;
    if code != 0 {
        return Err(format!("Failed to push to temporary dir: {}", stderr));
    }

    let move_cmd = format!(
        "cp -r '{}' '{}' && chmod -R 644 '{}' && rm -rf '{}'",
        tmp_remote, target_dest, target_dest, tmp_remote
    );
    let (m_code, _, m_err) = exec_android_shell(s_ref, &move_cmd, true)?;
    if m_code == 0 {
        Ok(format!("Pushed successfully with root: {}", target_dest))
    } else {
        Err(m_err)
    }
}

#[tauri::command]
pub fn transfer_android_to_pc(
    serial: Option<String>,
    remote_path: String,
    local_dir: String,
    root_mode: bool,
) -> Result<String, String> {
    let s_ref = serial.as_deref();
    let file_name = Path::new(&remote_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());

    let local_dest = Path::new(&local_dir).join(&file_name);

    let mut args = Vec::new();
    if let Some(s) = s_ref {
        args.extend_from_slice(&["-s", s]);
    }
    let local_dest_str = local_dest.to_string_lossy().to_string();
    args.extend_from_slice(&["pull", &remote_path, &local_dest_str]);

    let (code, _stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 {
        return Ok(format!("Pulled successfully to {}", local_dest.display()));
    }

    if root_mode {
        // Protected root path: copy to /data/local/tmp/ with 777 perms first
        let tmp_remote = format!("/data/local/tmp/_ximi_pull_{}", file_name);
        let cp_cmd = format!(
            "cp -r '{}' '{}' && chmod -R 777 '{}'",
            remote_path, tmp_remote, tmp_remote
        );
        let _ = exec_android_shell(s_ref, &cp_cmd, true);

        let mut pull_args = Vec::new();
        if let Some(s) = s_ref {
            pull_args.extend_from_slice(&["-s", s]);
        }
        pull_args.extend_from_slice(&["pull", &tmp_remote, &local_dest_str]);
        let (p_code, _, p_err) = run_adb_cmd(&pull_args)?;

        let _ = exec_android_shell(s_ref, &format!("rm -rf '{}'", tmp_remote), true);

        if p_code == 0 {
            return Ok(format!("Pulled successfully via root to {}", local_dest.display()));
        } else {
            return Err(p_err);
        }
    }

    Err(stderr)
}

#[tauri::command]
pub fn create_item(
    serial: Option<String>,
    path: String,
    is_dir: bool,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        if is_dir {
            fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        } else {
            fs::File::create(&path).map_err(|e| e.to_string())?;
        }
        return Ok("Created successfully on PC".to_string());
    }

    let cmd = if is_dir {
        format!("mkdir -p '{}'", path)
    } else {
        format!("touch '{}'", path)
    };
    let (code, stdout, stderr) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
    if code == 0 {
        Ok("Created successfully on Android".to_string())
    } else {
        Err(if stdout.is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn delete_item(
    serial: Option<String>,
    path: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        let p = Path::new(&path);
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(p).map_err(|e| e.to_string())?;
        }
        return Ok("Deleted from PC".to_string());
    }

    let cmd = format!("rm -rf '{}'", path);
    let (code, stdout, stderr) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
    if code == 0 {
        Ok("Deleted from Android".to_string())
    } else {
        Err(if stdout.is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn rename_item(
    serial: Option<String>,
    old_path: String,
    new_path: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
        return Ok("Renamed successfully on PC".to_string());
    }

    let cmd = format!("mv '{}' '{}'", old_path, new_path);
    let (code, stdout, stderr) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
    if code == 0 {
        Ok("Renamed successfully on Android".to_string())
    } else {
        Err(if stdout.is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn read_file_text(
    serial: Option<String>,
    path: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        return fs::read_to_string(&path).map_err(|e| e.to_string());
    }

    let cmd = format!("cat '{}'", path);
    let (code, stdout, stderr) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
    if code == 0 {
        Ok(stdout)
    } else {
        Err(stderr)
    }
}

#[tauri::command]
pub fn write_file_text(
    serial: Option<String>,
    path: String,
    content: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        let mut file = fs::File::create(&path).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        return Ok("Saved successfully on PC".to_string());
    }

    // Write to a temporary file locally, then push to Android
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("_ximi_edit_{}.txt", SystemTime::now().elapsed().unwrap().as_millis()));
    let mut file = fs::File::create(&temp_file).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;

    let res = transfer_pc_to_android(
        serial,
        temp_file.to_string_lossy().to_string(),
        path,
        root_mode,
    );
    let _ = fs::remove_file(temp_file);
    res
}

#[tauri::command]
pub fn extract_zip_archive(
    serial: Option<String>,
    zip_path: String,
    dst_folder: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        let file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        archive.extract(&dst_folder).map_err(|e| e.to_string())?;
        return Ok("Extracted successfully on PC".to_string());
    }

    let cmd = format!("unzip -o '{}' -d '{}'", zip_path, dst_folder);
    let (code, _stdout, stderr) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
    if code == 0 {
        Ok("Extracted successfully on Android".to_string())
    } else {
        let cmd2 = format!("toybox unzip -o '{}' -d '{}'", zip_path, dst_folder);
        let (code2, _, stderr2) = exec_android_shell(serial.as_deref(), &cmd2, root_mode)?;
        if code2 == 0 {
            Ok("Extracted successfully on Android".to_string())
        } else {
            Err(if stderr.is_empty() { stderr2 } else { stderr })
        }
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn format_epoch_to_date(epoch_secs: u64) -> String {
    let days = epoch_secs / 86400;
    // Simple approximate human date for display
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    let rem_secs = epoch_secs % 86400;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, mins)
}
