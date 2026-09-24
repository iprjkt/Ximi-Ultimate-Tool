use std::process::Command;

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

pub fn detect_adb() -> String {
    if let Ok(p) = which::which("adb") {
        return p.to_string_lossy().to_string();
    }
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\platform-tools\adb.exe",
            r"%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe",
        ];
        for c in candidates {
            if std::path::Path::new(c).exists() {
                return c.to_string();
            }
        }
    }
    "adb".to_string()
}

pub fn detect_fastboot() -> String {
    if let Ok(p) = which::which("fastboot") {
        return p.to_string_lossy().to_string();
    }
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\platform-tools\fastboot.exe",
            r"%LOCALAPPDATA%\Android\Sdk\platform-tools\fastboot.exe",
        ];
        for c in candidates {
            if std::path::Path::new(c).exists() {
                return c.to_string();
            }
        }
    }
    "fastboot".to_string()
}

pub fn run_adb_cmd(args: &[&str]) -> Result<(i32, String, String), String> {
    let adb = detect_adb();
    let mut cmd = Command::new(&adb);
    cmd.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    match cmd.output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Ok((code, stdout, stderr))
        }
        Err(e) => Err(format!("Failed to execute adb {}: {}", args.join(" "), e)),
    }
}

pub fn run_fastboot_cmd(args: &[&str]) -> Result<(i32, String, String), String> {
    let fb = detect_fastboot();
    let mut cmd = Command::new(&fb);
    cmd.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    match cmd.output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Ok((code, stdout, stderr))
        }
        Err(e) => Err(format!("Failed to execute fastboot {}: {}", args.join(" "), e)),
    }
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
pub fn pick_file(
    title: Option<String>,
    filter_name: Option<String>,
    filter_pattern: Option<String>,
) -> Result<Option<String>, String> {
    if which::which("zenity").is_ok() {
        let mut cmd = Command::new("zenity");
        cmd.arg("--file-selection");
        if let Some(ref t) = title {
            cmd.arg(format!("--title={}", t));
        }
        if let (Some(ref name), Some(ref pat)) = (filter_name.as_ref(), filter_pattern.as_ref()) {
            cmd.arg(format!("--file-filter={} | {}", name, pat));
            cmd.arg("--file-filter=All files | *");
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
            return Ok(None);
        }
    }

    if which::which("kdialog").is_ok() {
        let mut cmd = Command::new("kdialog");
        cmd.arg("--getopenfilename");
        cmd.arg(".");
        if let Some(ref pat) = filter_pattern {
            cmd.arg(pat);
        }
        if let Some(ref t) = title {
            cmd.arg(format!("--title={}", t));
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
            return Ok(None);
        }
    }

    Err("No compatible file chooser found (please install zenity or kdialog)".to_string())
}

#[tauri::command]
pub fn pick_folder(title: Option<String>) -> Result<Option<String>, String> {
    if which::which("zenity").is_ok() {
        let mut cmd = Command::new("zenity");
        cmd.arg("--file-selection").arg("--directory");
        if let Some(ref t) = title {
            cmd.arg(format!("--title={}", t));
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
            return Ok(None);
        }
    }

    if which::which("kdialog").is_ok() {
        let mut cmd = Command::new("kdialog");
        cmd.arg("--getexistingdirectory");
        if let Some(ref t) = title {
            cmd.arg(format!("--title={}", t));
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
            return Ok(None);
        }
    }

    Err("No compatible folder chooser found (please install zenity or kdialog)".to_string())
}
