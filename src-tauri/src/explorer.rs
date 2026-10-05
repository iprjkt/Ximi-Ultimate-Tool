use crate::utils::{
    blocking, format_bytes, new_task_id, run_adb, run_adb_streaming, sh_quote, SerialGuard,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Largest file the built-in text editor will load (bigger files go through IPC as one string).
const MAX_EDIT_BYTES: u64 = 2 * 1024 * 1024;

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
    let full_cmd = if root_mode { format!("su -c {}", sh_quote(cmd)) } else { cmd.to_string() };
    run_adb(serial, &["shell", &full_cmd])
}

fn err_text(stdout: String, stderr: String) -> String {
    let t = if stdout.trim().is_empty() { stderr } else { stdout };
    t.trim().to_string()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

#[tauri::command]
pub fn get_home_dir() -> String {
    home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "/".to_string())
}

fn format_time(t: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = t.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(unix)]
fn pc_permissions(meta: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode();
    let mut s = String::with_capacity(10);
    s.push(if meta.is_dir() { 'd' } else if meta.file_type().is_symlink() { 'l' } else { '-' });
    for shift in [6, 3, 0] {
        let bits = (mode >> shift) & 0o7;
        s.push(if bits & 4 != 0 { 'r' } else { '-' });
        s.push(if bits & 2 != 0 { 'w' } else { '-' });
        s.push(if bits & 1 != 0 { 'x' } else { '-' });
    }
    s
}

#[cfg(not(unix))]
fn pc_permissions(meta: &fs::Metadata) -> String {
    let d = if meta.is_dir() { 'd' } else { '-' };
    if meta.permissions().readonly() {
        format!("{}r--r--r--", d)
    } else {
        format!("{}rw-rw-rw-", d)
    }
}

fn sort_items(items: &mut [FileItem]) {
    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
}

#[tauri::command]
pub async fn list_pc_directory(dir_path: String) -> Result<Vec<FileItem>, String> {
    blocking(move || {
        let mut resolved = PathBuf::from(&dir_path);
        if dir_path.trim().is_empty() {
            resolved = home_dir().unwrap_or(resolved);
        } else if let Some(rest) = dir_path.strip_prefix('~') {
            if let Some(home) = home_dir() {
                resolved = home.join(rest.trim_start_matches(['/', '\\']));
            }
        }
        if !resolved.exists() {
            return Err(format!("Directory does not exist: {}", resolved.display()));
        }
        let entries = fs::read_dir(&resolved).map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut items = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else { continue };
            let is_link = meta.file_type().is_symlink();
            let is_dir = if is_link { path.is_dir() } else { meta.is_dir() };
            let link_target = if is_link {
                fs::read_link(&path).map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
            } else {
                String::new()
            };
            let size = if is_dir { 0 } else { meta.len() };
            items.push(FileItem {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_dir,
                is_link,
                link_target,
                size,
                size_formatted: if is_dir { "--".to_string() } else { format_bytes(size) },
                permissions: pc_permissions(&meta),
                owner: "-".into(),
                group: "-".into(),
                date: meta.modified().ok().map(format_time).unwrap_or_else(|| "-".to_string()),
            });
        }
        sort_items(&mut items);
        Ok(items)
    })
    .await
}

fn parse_android_ls(stdout: &str, clean: &str) -> Vec<FileItem> {
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
            (
                parts[2].to_string(),
                parts[3].to_string(),
                parts[4].parse::<u64>().unwrap_or(0),
                format!("{} {}", parts[5], parts[6]),
                parts[7..].join(" "),
            )
        } else {
            (
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].parse::<u64>().unwrap_or(0),
                parts[4].to_string(),
                parts[5..].join(" "),
            )
        };

        let (name, link_target) = match raw_name.split_once(" -> ") {
            Some((n, t)) => (n.to_string(), t.to_string()),
            None => (raw_name, String::new()),
        };
        if name == "." || name == ".." {
            continue;
        }

        items.push(FileItem {
            path: format!("{}/{}", clean, name).replace("//", "/"),
            name,
            is_dir,
            is_link,
            link_target,
            size,
            size_formatted: if is_dir { "--".to_string() } else { format_bytes(size) },
            permissions: perms.to_string(),
            owner,
            group,
            date: date_val,
        });
    }
    sort_items(&mut items);
    items
}

#[tauri::command]
pub async fn list_android_directory(
    serial: Option<String>,
    dir_path: String,
    root_mode: bool,
) -> Result<Vec<FileItem>, String> {
    blocking(move || {
        // Trailing slash so symlinked folders (like /sdcard) list their contents
        let clean = dir_path.trim_end_matches('/');
        let target = if clean.is_empty() { "/".to_string() } else { format!("{}/", clean) };
        let cmd = format!("ls -la {}", sh_quote(&target));

        let (mut code, mut stdout, mut stderr) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
        if code != 0 && stdout.trim().is_empty() && root_mode {
            // root not available: retry as the shell user
            (code, stdout, stderr) = exec_android_shell(serial.as_deref(), &cmd, false)?;
        }
        if code != 0 && stdout.trim().is_empty() {
            return Err(stderr.trim().to_string());
        }
        Ok(parse_android_ls(&stdout, clean))
    })
    .await
}

fn file_name_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string())
}

/// Push `local` to the exact remote path `target`, via /data/local/tmp when root is required.
async fn push_to(
    app: tauri::AppHandle,
    serial: Option<String>,
    local: String,
    target: String,
    root_mode: bool,
    id: String,
) -> Result<String, String> {
    let direct = !root_mode || target.starts_with("/sdcard") || target.starts_with("/storage");
    if direct {
        let (code, out) = run_adb_streaming(app, serial.as_deref(), &["push", &local, &target], id).await?;
        return if code == 0 { Ok(format!("Pushed successfully: {}", target)) } else { Err(out) };
    }

    // Protected path: push to a temp location, then move it with root.
    let tmp = format!("/data/local/tmp/_ximi_{}", file_name_of(&local));
    let (code, out) = run_adb_streaming(app, serial.as_deref(), &["push", &local, &tmp], id).await?;
    if code != 0 {
        return Err(format!("Failed to push to temporary dir: {}", out));
    }
    let move_cmd = format!("cp -r {} {} && rm -rf {}", sh_quote(&tmp), sh_quote(&target), sh_quote(&tmp));
    let ser = serial.clone();
    let (m_code, m_out, m_err) = blocking(move || exec_android_shell(ser.as_deref(), &move_cmd, true)).await?;
    if m_code == 0 {
        Ok(format!("Pushed successfully with root: {}", target))
    } else {
        Err(err_text(m_out, m_err))
    }
}

#[tauri::command]
pub async fn transfer_pc_to_android(
    app: tauri::AppHandle,
    serial: Option<String>,
    local_path: String,
    remote_dir: String,
    root_mode: bool,
    task_id: Option<String>,
) -> Result<String, String> {
    if !Path::new(&local_path).exists() {
        return Err(format!("Local file not found: {}", local_path));
    }
    let _guard = SerialGuard::acquire(serial.as_deref())?;
    let target = format!("{}/{}", remote_dir.trim_end_matches('/'), file_name_of(&local_path));
    push_to(app, serial, local_path, target, root_mode, new_task_id(task_id)).await
}

#[tauri::command]
pub async fn transfer_android_to_pc(
    app: tauri::AppHandle,
    serial: Option<String>,
    remote_path: String,
    local_dir: String,
    root_mode: bool,
    task_id: Option<String>,
) -> Result<String, String> {
    let _guard = SerialGuard::acquire(serial.as_deref())?;
    let id = new_task_id(task_id);
    let file_name = file_name_of(&remote_path);
    let local_dest = Path::new(&local_dir).join(&file_name);
    let local_str = local_dest.to_string_lossy().to_string();

    let (code, out) =
        run_adb_streaming(app.clone(), serial.as_deref(), &["pull", &remote_path, &local_str], id.clone()).await?;
    if code == 0 {
        return Ok(format!("Pulled successfully to {}", local_dest.display()));
    }
    if !root_mode {
        return Err(out);
    }

    // Protected path: copy it somewhere readable with root, pull it, clean up.
    let tmp = format!("/data/local/tmp/_ximi_pull_{}", file_name);
    let cp_cmd = format!("cp -r {} {} && chmod -R 777 {}", sh_quote(&remote_path), sh_quote(&tmp), sh_quote(&tmp));
    let ser = serial.clone();
    blocking(move || exec_android_shell(ser.as_deref(), &cp_cmd, true)).await?;
    let result = run_adb_streaming(app, serial.as_deref(), &["pull", &tmp, &local_str], id).await;
    let ser = serial.clone();
    let rm_cmd = format!("rm -rf {}", sh_quote(&tmp));
    let _ = blocking(move || exec_android_shell(ser.as_deref(), &rm_cmd, true)).await;

    match result? {
        (0, _) => Ok(format!("Pulled successfully via root to {}", local_dest.display())),
        (_, out) => Err(out),
    }
}

#[tauri::command]
pub async fn create_item(
    serial: Option<String>,
    path: String,
    is_dir: bool,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    blocking(move || {
        if !is_android {
            if is_dir {
                fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            } else {
                fs::File::create(&path).map_err(|e| e.to_string())?;
            }
            return Ok("Created successfully on PC".to_string());
        }
        let cmd = if is_dir { format!("mkdir -p {}", sh_quote(&path)) } else { format!("touch {}", sh_quote(&path)) };
        let (code, out, err) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
        if code == 0 { Ok("Created successfully on Android".to_string()) } else { Err(err_text(out, err)) }
    })
    .await
}

#[tauri::command]
pub async fn delete_item(
    serial: Option<String>,
    path: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    blocking(move || {
        if !is_android {
            let p = Path::new(&path);
            if p.is_dir() {
                fs::remove_dir_all(p).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(p).map_err(|e| e.to_string())?;
            }
            return Ok("Deleted from PC".to_string());
        }
        // Refuse to wipe the filesystem root by accident
        if path.trim_matches('/').is_empty() {
            return Err("Refusing to delete the root directory".to_string());
        }
        let cmd = format!("rm -rf {}", sh_quote(&path));
        let (code, out, err) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
        if code == 0 { Ok("Deleted from Android".to_string()) } else { Err(err_text(out, err)) }
    })
    .await
}

#[tauri::command]
pub async fn rename_item(
    serial: Option<String>,
    old_path: String,
    new_path: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    blocking(move || {
        if !is_android {
            fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
            return Ok("Renamed successfully on PC".to_string());
        }
        let cmd = format!("mv {} {}", sh_quote(&old_path), sh_quote(&new_path));
        let (code, out, err) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
        if code == 0 { Ok("Renamed successfully on Android".to_string()) } else { Err(err_text(out, err)) }
    })
    .await
}

fn too_large() -> String {
    format!("File is too large to edit as text (limit {} MB).", MAX_EDIT_BYTES / 1024 / 1024)
}

#[tauri::command]
pub async fn read_file_text(
    serial: Option<String>,
    path: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    blocking(move || {
        if !is_android {
            let len = fs::metadata(&path).map_err(|e| e.to_string())?.len();
            if len > MAX_EDIT_BYTES {
                return Err(too_large());
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if bytes.contains(&0) {
                return Err("This looks like a binary file and cannot be edited as text.".to_string());
            }
            return Ok(String::from_utf8_lossy(&bytes).to_string());
        }
        // `head -c` keeps a huge remote file from ever being streamed to the PC
        let cmd = format!("head -c {} {}", MAX_EDIT_BYTES + 1, sh_quote(&path));
        let (code, out, err) = exec_android_shell(serial.as_deref(), &cmd, root_mode)?;
        if code != 0 && out.is_empty() {
            return Err(err.trim().to_string());
        }
        if out.len() as u64 > MAX_EDIT_BYTES {
            return Err(too_large());
        }
        if out.contains('\0') {
            return Err("This looks like a binary file and cannot be edited as text.".to_string());
        }
        Ok(out)
    })
    .await
}

#[tauri::command]
pub async fn write_file_text(
    app: tauri::AppHandle,
    serial: Option<String>,
    path: String,
    content: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    if !is_android {
        let p = path.clone();
        blocking(move || {
            let mut file = fs::File::create(&p).map_err(|e| e.to_string())?;
            file.write_all(content.as_bytes()).map_err(|e| e.to_string())
        })
        .await?;
        return Ok("Saved successfully on PC".to_string());
    }

    // Stage locally, then push to the exact remote file path
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_file = std::env::temp_dir().join(format!("_ximi_edit_{}_{}", std::process::id(), nanos));
    let temp = temp_file.clone();
    blocking(move || fs::write(&temp, content.as_bytes()).map_err(|e| e.to_string())).await?;

    // Keep the remote file name so the root-mode staging copy lands on the right file
    let staged = temp_file.with_file_name(file_name_of(&path));
    let result = match fs::rename(&temp_file, &staged) {
        Ok(()) => {
            let r = push_to(app, serial, staged.to_string_lossy().to_string(), path, root_mode, new_task_id(None)).await;
            let _ = fs::remove_file(&staged);
            r
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_file);
            Err(e.to_string())
        }
    };
    result.map(|_| "Saved successfully on Android".to_string())
}

#[tauri::command]
pub async fn extract_zip_archive(
    serial: Option<String>,
    zip_path: String,
    dst_folder: String,
    is_android: bool,
    root_mode: bool,
) -> Result<String, String> {
    blocking(move || {
        if !is_android {
            let file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            archive.extract(&dst_folder).map_err(|e| e.to_string())?;
            return Ok("Extracted successfully on PC".to_string());
        }
        let (z, d) = (sh_quote(&zip_path), sh_quote(&dst_folder));
        let (code, _o, stderr) = exec_android_shell(serial.as_deref(), &format!("unzip -o {} -d {}", z, d), root_mode)?;
        if code == 0 {
            return Ok("Extracted successfully on Android".to_string());
        }
        let (code2, _, stderr2) =
            exec_android_shell(serial.as_deref(), &format!("toybox unzip -o {} -d {}", z, d), root_mode)?;
        if code2 == 0 {
            Ok("Extracted successfully on Android".to_string())
        } else {
            Err(if stderr.trim().is_empty() { stderr2 } else { stderr }.trim().to_string())
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_ls_is_parsed() {
        let out = "total 8\ndrwxrwx--x  3 root sdcard_rw 3452 2024-01-01 10:00 Download\n-rw-rw----  1 root sdcard_rw 1234 2024-02-03 11:22 my file.txt\nlrwxrwxrwx  1 root root 10 2024-01-01 10:00 sdcard -> /storage/self/primary\n";
        let items = parse_android_ls(out, "/sdcard");
        assert_eq!(items.len(), 3);
        assert!(items[0].is_dir);
        assert_eq!(items[0].path, "/sdcard/Download");
        let f = items.iter().find(|i| i.name == "my file.txt").unwrap();
        assert_eq!(f.size, 1234);
        let l = items.iter().find(|i| i.is_link).unwrap();
        assert_eq!(l.link_target, "/storage/self/primary");
    }

    #[test]
    fn file_name_fallback() {
        assert_eq!(file_name_of("/a/b/c.txt"), "c.txt");
    }
}
