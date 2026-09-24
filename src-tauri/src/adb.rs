use crate::utils::{detect_adb, run_adb_cmd};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub serial: String,
    pub state: String,
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceSpecs {
    pub market_name: String,
    pub model: String,
    pub device: String,
    pub brand: String,
    pub android_ver: String,
    pub security_patch: String,
    pub hyperos_version: String,
    pub hyperos_short: String,
    pub cpu: String,
    pub ram: String,
    pub storage: String,
    pub battery: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BloatwareItem {
    pub package: String,
    pub name: String,
    pub category: String,
    pub risk: String, // "Safe", "Caution", "Optional"
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageItem {
    pub package: String,
    pub name: String,
    pub category: String,
    pub risk: String,
    pub description: String,
    pub is_installed: bool,
}

pub fn get_curated_bloatware() -> Vec<BloatwareItem> {
    vec![
        // Analytics & Tracking
        BloatwareItem {
            package: "com.miui.analytics".into(),
            name: "Xiaomi Analytics".into(),
            category: "Analytics".into(),
            risk: "Safe".into(),
            description: "Telemetry, logs, and tracking system.".into(),
        },
        BloatwareItem {
            package: "com.xiaomi.joyose".into(),
            name: "Joyose".into(),
            category: "Analytics".into(),
            risk: "Safe".into(),
            description: "Game throttle, analytics, and telemetry service.".into(),
        },
        BloatwareItem {
            package: "com.miui.daemon".into(),
            name: "MIUI Daemon".into(),
            category: "Analytics".into(),
            risk: "Safe".into(),
            description: "Background diagnostic & monitoring service.".into(),
        },
        BloatwareItem {
            package: "com.miui.msa.global".into(),
            name: "MIUI System Ads (Global)".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Service responsible for push ads in MIUI apps.".into(),
        },
        BloatwareItem {
            package: "com.miui.bugreport".into(),
            name: "Mi Bug Report".into(),
            category: "Analytics".into(),
            risk: "Safe".into(),
            description: "Crash report and feedback uploader.".into(),
        },
        BloatwareItem {
            package: "com.miui.miservice".into(),
            name: "Services & Feedback".into(),
            category: "Analytics".into(),
            risk: "Safe".into(),
            description: "User feedback and log collection service.".into(),
        },
        BloatwareItem {
            package: "com.xiaomi.xmsf".into(),
            name: "Xiaomi Service Framework".into(),
            category: "Analytics".into(),
            risk: "Caution".into(),
            description: "Framework for Xiaomi cloud & push notifications.".into(),
        },
        BloatwareItem {
            package: "com.xiaomi.finddevice".into(),
            name: "Find Device".into(),
            category: "Analytics".into(),
            risk: "Caution".into(),
            description: "Xiaomi Cloud Find Device feature.".into(),
        },
        // Ads & Commercial Apps
        BloatwareItem {
            package: "com.xiaomi.mipicks".into(),
            name: "GetApps".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Xiaomi App Store with frequent promotional ads.".into(),
        },
        BloatwareItem {
            package: "com.miui.hybrid".into(),
            name: "Quick Apps".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Mini apps service running ads in background.".into(),
        },
        BloatwareItem {
            package: "com.miui.hybrid.accessory".into(),
            name: "Quick Apps Accessory".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Companion service for Quick Apps.".into(),
        },
        BloatwareItem {
            package: "com.mipay.wallet.id".into(),
            name: "Mi Pay ID".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Payment wallet service.".into(),
        },
        BloatwareItem {
            package: "com.mipay.wallet.in".into(),
            name: "Mi Pay IN".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Payment wallet service.".into(),
        },
        BloatwareItem {
            package: "com.miui.micredit".into(),
            name: "Mi Credit".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Loan and financial marketing app.".into(),
        },
        BloatwareItem {
            package: "com.android.browser".into(),
            name: "Mi Browser".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Xiaomi browser preloaded with promotional feed.".into(),
        },
        BloatwareItem {
            package: "com.miui.videoplayer".into(),
            name: "Mi Video".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Video player bundled with online trending feed.".into(),
        },
        BloatwareItem {
            package: "com.miui.player".into(),
            name: "Mi Music".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Music player bundled with promotional music feed.".into(),
        },
        BloatwareItem {
            package: "com.miui.yellowpage".into(),
            name: "Yellow Pages".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Caller ID and business directory lookup.".into(),
        },
        BloatwareItem {
            package: "com.miui.android.fashiongallery".into(),
            name: "Wallpaper Carousel".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Lock screen ads and news carousel.".into(),
        },
        BloatwareItem {
            package: "com.mfashiongallery.emag".into(),
            name: "Glance Carousel".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Glance lockscreen news and ads.".into(),
        },
        // Preloaded & Games
        BloatwareItem {
            package: "com.xiaomi.glgm".into(),
            name: "Xiaomi Game Center".into(),
            category: "Games".into(),
            risk: "Safe".into(),
            description: "Promotional games marketplace.".into(),
        },
        BloatwareItem {
            package: "com.xiaomi.midrop".into(),
            name: "ShareMe / Mi Drop".into(),
            category: "System".into(),
            risk: "Optional".into(),
            description: "File sharing tool.".into(),
        },
        BloatwareItem {
            package: "com.miui.cleanmaster".into(),
            name: "Clean Master Cleaner".into(),
            category: "System".into(),
            risk: "Safe".into(),
            description: "MIUI security cleaner database/engine.".into(),
        },
        BloatwareItem {
            package: "com.miui.compass".into(),
            name: "Compass".into(),
            category: "System".into(),
            risk: "Safe".into(),
            description: "Default compass app.".into(),
        },
        BloatwareItem {
            package: "com.miui.notes".into(),
            name: "Mi Notes".into(),
            category: "System".into(),
            risk: "Optional".into(),
            description: "Stock notes app.".into(),
        },
        BloatwareItem {
            package: "com.miui.weather2".into(),
            name: "Mi Weather".into(),
            category: "System".into(),
            risk: "Optional".into(),
            description: "Stock weather application.".into(),
        },
        BloatwareItem {
            package: "com.miui.calculator".into(),
            name: "Mi Calculator".into(),
            category: "System".into(),
            risk: "Optional".into(),
            description: "Stock calculator app.".into(),
        },
        BloatwareItem {
            package: "com.miui.screenrecorder".into(),
            name: "Screen Recorder".into(),
            category: "System".into(),
            risk: "Optional".into(),
            description: "Built-in screen recording utility.".into(),
        },
        BloatwareItem {
            package: "com.miui.voiceassist".into(),
            name: "Mi AI Voice".into(),
            category: "System".into(),
            risk: "Safe".into(),
            description: "Voice assistant (often China/Asian variants).".into(),
        },
        // Facebook Services
        BloatwareItem {
            package: "com.facebook.katana".into(),
            name: "Facebook".into(),
            category: "Facebook".into(),
            risk: "Safe".into(),
            description: "Preinstalled Facebook main app.".into(),
        },
        BloatwareItem {
            package: "com.facebook.system".into(),
            name: "Facebook App Installer".into(),
            category: "Facebook".into(),
            risk: "Safe".into(),
            description: "Silent background APK installer for FB.".into(),
        },
        BloatwareItem {
            package: "com.facebook.appmanager".into(),
            name: "Facebook App Manager".into(),
            category: "Facebook".into(),
            risk: "Safe".into(),
            description: "Background updater for Facebook products.".into(),
        },
        BloatwareItem {
            package: "com.facebook.services".into(),
            name: "Facebook Services".into(),
            category: "Facebook".into(),
            risk: "Safe".into(),
            description: "Background sync and tracking service.".into(),
        },
        // Google Preinstalled
        BloatwareItem {
            package: "com.google.android.apps.tachyon".into(),
            name: "Google Meet / Duo".into(),
            category: "Google".into(),
            risk: "Safe".into(),
            description: "Video calling service.".into(),
        },
        BloatwareItem {
            package: "com.google.android.videos".into(),
            name: "Google TV / Play Movies".into(),
            category: "Google".into(),
            risk: "Safe".into(),
            description: "Movie rental and purchase store.".into(),
        },
        BloatwareItem {
            package: "com.google.android.music".into(),
            name: "Google Play Music".into(),
            category: "Google".into(),
            risk: "Safe".into(),
            description: "Legacy Google Music app.".into(),
        },
        BloatwareItem {
            package: "com.google.android.apps.photos".into(),
            name: "Google Photos".into(),
            category: "Google".into(),
            risk: "Optional".into(),
            description: "Cloud photo backup and gallery.".into(),
        },
        BloatwareItem {
            package: "com.google.android.apps.docs".into(),
            name: "Google Drive / Docs".into(),
            category: "Google".into(),
            risk: "Optional".into(),
            description: "Google cloud docs utility.".into(),
        },
        BloatwareItem {
            package: "com.google.android.youtube".into(),
            name: "YouTube".into(),
            category: "Google".into(),
            risk: "Optional".into(),
            description: "Stock YouTube video client.".into(),
        },
        BloatwareItem {
            package: "com.google.android.apps.youtube.music".into(),
            name: "YouTube Music".into(),
            category: "Google".into(),
            risk: "Safe".into(),
            description: "Streaming music service.".into(),
        },
        BloatwareItem {
            package: "com.google.android.feedback".into(),
            name: "Google Feedback".into(),
            category: "Google".into(),
            risk: "Safe".into(),
            description: "Google bug and feedback uploader.".into(),
        },
    ]
}

fn get_prop(serial: Option<&str>, prop: &str) -> String {
    let mut args = Vec::new();
    if let Some(s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["shell", "getprop", prop]);
    if let Ok((0, out, _)) = run_adb_cmd(&args) {
        out.trim().to_string()
    } else {
        String::new()
    }
}

#[tauri::command]
pub fn get_adb_devices() -> Result<Vec<DeviceInfo>, String> {
    let (code, stdout, stderr) = run_adb_cmd(&["devices", "-l"])?;
    if code != 0 {
        return Err(format!("adb devices failed: {}", stderr));
    }

    let mut devices = Vec::new();
    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let serial = parts[0].to_string();
            let state = parts[1].to_string();
            let info = if parts.len() > 2 {
                parts[2..].join(" ")
            } else {
                String::new()
            };
            devices.push(DeviceInfo { serial, state, info });
        }
    }
    Ok(devices)
}

#[tauri::command]
pub fn get_device_specs(serial: Option<String>) -> Result<DeviceSpecs, String> {
    let s_ref = serial.as_deref();

    let hyperos_incremental = get_prop(s_ref, "ro.mi.os.version.incremental");
    let hyperos_name = get_prop(s_ref, "ro.miui.ui.version.name");
    let android_ver = get_prop(s_ref, "ro.build.version.release");
    let security_patch = get_prop(s_ref, "ro.build.version.security_patch");

    let market_name = get_prop(s_ref, "ro.product.marketname");
    let model = get_prop(s_ref, "ro.product.model");
    let device = get_prop(s_ref, "ro.product.device");
    let brand = get_prop(s_ref, "ro.product.brand");

    let final_market = if !market_name.is_empty() {
        market_name
    } else if !model.is_empty() {
        model.clone()
    } else if !device.is_empty() {
        device.clone()
    } else {
        "Xiaomi Device".to_string()
    };

    let (hyperos_ver, hyperos_short) = if !hyperos_incremental.is_empty() {
        let re = Regex::new(r"^(\d+\.\d+\.\d+\.\d+|\d+\.\d+\.\d+)").unwrap();
        let short = if let Some(caps) = re.captures(&hyperos_incremental) {
            caps[1].to_string()
        } else {
            hyperos_incremental.clone()
        };
        (hyperos_incremental, short)
    } else if !hyperos_name.is_empty() {
        (hyperos_name, "1.0.0.0".to_string())
    } else {
        ("Xiaomi HyperOS 1.0".to_string(), "1.0.0.0".to_string())
    };

    // CPU / SoC
    let mut soc = get_prop(s_ref, "ro.soc.model");
    if soc.is_empty() {
        soc = get_prop(s_ref, "ro.board.platform");
    }
    if soc.is_empty() {
        soc = get_prop(s_ref, "ro.hardware");
    }
    if soc.is_empty() {
        soc = "Octa-core Processor".to_string();
    }

    // RAM
    let mut ram_str = "8.0GB".to_string();
    let mut mem_args = Vec::new();
    if let Some(s) = s_ref {
        mem_args.extend_from_slice(&["-s", s]);
    }
    mem_args.extend_from_slice(&["shell", "cat", "/proc/meminfo"]);
    if let Ok((0, mem_out, _)) = run_adb_cmd(&mem_args) {
        for line in mem_out.lines() {
            if line.contains("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        let gb = kb / (1024.0 * 1024.0);
                        ram_str = if gb <= 4.3 {
                            "4.0GB".to_string()
                        } else if gb <= 6.3 {
                            "6.0GB".to_string()
                        } else if gb <= 8.5 {
                            "8.0GB".to_string()
                        } else if gb <= 12.5 {
                            "12.0GB".to_string()
                        } else if gb <= 16.5 {
                            "16.0GB".to_string()
                        } else {
                            format!("{:.1}GB", gb)
                        };
                    }
                }
                break;
            }
        }
    }

    // Storage
    let mut storage_str = "72.5GB/256GB".to_string();
    let mut df_args = Vec::new();
    if let Some(s) = s_ref {
        df_args.extend_from_slice(&["-s", s]);
    }
    df_args.extend_from_slice(&["shell", "df -h /data"]);
    if let Ok((0, df_out, _)) = run_adb_cmd(&df_args) {
        let lines: Vec<&str> = df_out.lines().collect();
        if lines.len() >= 2 {
            let parts: Vec<&str> = lines[1].split_whitespace().collect();
            if parts.len() >= 4 {
                let size = parts[1];
                let used = parts[2];
                storage_str = format!("{}/{}", used, size);
            }
        }
    }

    // Battery
    let mut battery_str = "5000mAh (typ)".to_string();
    let mut batt_args = Vec::new();
    if let Some(s) = s_ref {
        batt_args.extend_from_slice(&["-s", s]);
    }
    batt_args.extend_from_slice(&["shell", "dumpsys battery"]);
    if let Ok((0, batt_out, _)) = run_adb_cmd(&batt_args) {
        let mut level = "100";
        for line in batt_out.lines() {
            if line.contains("level:") {
                if let Some(l) = line.split(':').nth(1) {
                    level = l.trim();
                }
            }
        }
        battery_str = format!("5000mAh ({}%)", level);
        if final_market.contains("14") {
            battery_str = format!("5500mAh(typ) ({}%)", level);
        }
    }

    Ok(DeviceSpecs {
        market_name: final_market,
        model: if model.is_empty() { device.clone() } else { model },
        device: if device.is_empty() { "hyperos".to_string() } else { device },
        brand: if brand.is_empty() { "Xiaomi".to_string() } else { brand },
        android_ver: if android_ver.is_empty() { "14".to_string() } else { android_ver },
        security_patch: if security_patch.is_empty() { "-".to_string() } else { security_patch },
        hyperos_version: hyperos_ver,
        hyperos_short,
        cpu: soc,
        ram: ram_str,
        storage: storage_str,
        battery: battery_str,
    })
}

#[tauri::command]
pub fn get_packages(serial: Option<String>, filter_mode: String) -> Result<Vec<PackageItem>, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["shell", "pm", "list", "packages"]);
    match filter_mode.as_str() {
        "3rd" => args.push("-3"),
        "system" => args.push("-s"),
        "disabled" => args.push("-d"),
        _ => {}
    }

    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code != 0 {
        return Err(format!("Failed to list packages: {}", stderr));
    }

    let installed_set: std::collections::HashSet<String> = stdout
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("package:") {
                Some(trimmed.replace("package:", "").trim().to_string())
            } else {
                None
            }
        })
        .collect();

    let bloat_db = get_curated_bloatware();
    let bloat_map: HashMap<String, BloatwareItem> = bloat_db
        .into_iter()
        .map(|b| (b.package.clone(), b))
        .collect();

    let mut result = Vec::new();

    if filter_mode == "recommended" {
        for b in get_curated_bloatware() {
            let is_inst = installed_set.contains(&b.package);
            result.push(PackageItem {
                package: b.package,
                name: b.name,
                category: b.category,
                risk: b.risk,
                description: b.description,
                is_installed: is_inst,
            });
        }
    } else {
        for pkg in &installed_set {
            if let Some(b) = bloat_map.get(pkg) {
                result.push(PackageItem {
                    package: pkg.clone(),
                    name: b.name.clone(),
                    category: b.category.clone(),
                    risk: b.risk.clone(),
                    description: b.description.clone(),
                    is_installed: true,
                });
            } else {
                let name = pkg.split('.').last().unwrap_or(pkg).to_string();
                result.push(PackageItem {
                    package: pkg.clone(),
                    name,
                    category: "App".to_string(),
                    risk: "Optional".to_string(),
                    description: "-".to_string(),
                    is_installed: true,
                });
            }
        }
    }

    result.sort_by(|a, b| a.package.cmp(&b.package));
    Ok(result)
}

#[tauri::command]
pub fn uninstall_package(serial: Option<String>, package: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["shell", "pm", "uninstall", "-k", "--user", "0", &package]);

    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 && stdout.contains("Success") {
        Ok(format!("Successfully uninstalled {}", package))
    } else {
        Err(if stdout.trim().is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn restore_package(serial: Option<String>, package: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["shell", "cmd", "package", "install-existing", &package]);

    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 && (stdout.to_lowercase().contains("installed") || stdout.to_lowercase().contains("success")) {
        Ok(format!("Successfully restored {}", package))
    } else {
        Err(if stdout.trim().is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn disable_package(serial: Option<String>, package: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["shell", "pm", "disable-user", "--user", "0", &package]);

    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 && stdout.contains("new state") {
        Ok(format!("Successfully disabled {}", package))
    } else {
        Err(if stdout.trim().is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn enable_package(serial: Option<String>, package: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.extend_from_slice(&["shell", "pm", "enable", &package]);

    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 && stdout.contains("new state") {
        Ok(format!("Successfully enabled {}", package))
    } else {
        Err(if stdout.trim().is_empty() { stderr } else { stdout })
    }
}

#[tauri::command]
pub fn reboot_device(serial: Option<String>, mode: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }
    args.push("reboot");
    match mode.as_str() {
        "recovery" => args.push("recovery"),
        "bootloader" => args.push("bootloader"),
        "edl" => args.push("edl"),
        _ => {}
    }

    let (code, _stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 {
        Ok(format!("Device rebooting to {}", mode))
    } else {
        Err(stderr)
    }
}

#[tauri::command]
pub fn install_apk(serial: Option<String>, apk_path: String) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        if !s.trim().is_empty() {
            args.extend_from_slice(&["-s", s.trim()]);
        }
    }
    args.extend_from_slice(&["install", "-r", &apk_path]);

    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 && (stdout.to_lowercase().contains("success") || stderr.to_lowercase().contains("success")) {
        Ok("APK installed successfully!".to_string())
    } else {
        let msg = if !stdout.trim().is_empty() { stdout } else { stderr };
        Err(msg.trim().to_string())
    }
}

#[tauri::command]
pub fn take_screenshot(serial: Option<String>) -> Result<String, String> {
    let timestamp = chrono_like_timestamp();
    let local_name = format!("screenshot_{}.png", timestamp);
    let remote_path = format!("/sdcard/{}", local_name);

    let mut cap_args = Vec::new();
    if let Some(ref s) = serial {
        cap_args.extend_from_slice(&["-s", s]);
    }
    cap_args.extend_from_slice(&["shell", "screencap", "-p", &remote_path]);
    run_adb_cmd(&cap_args)?;

    let mut pull_args = Vec::new();
    if let Some(ref s) = serial {
        pull_args.extend_from_slice(&["-s", s]);
    }
    let local_dest = format!("./{}", local_name);
    pull_args.extend_from_slice(&["pull", &remote_path, &local_dest]);
    run_adb_cmd(&pull_args)?;

    let mut rm_args = Vec::new();
    if let Some(ref s) = serial {
        rm_args.extend_from_slice(&["-s", s]);
    }
    rm_args.extend_from_slice(&["shell", "rm", "-f", &remote_path]);
    let _ = run_adb_cmd(&rm_args);

    Ok(format!("Screenshot saved to {}", local_dest))
}

#[tauri::command]
pub fn screen_mirror(serial: Option<String>) -> Result<String, String> {
    if which::which("scrcpy").is_err() {
        return Err("scrcpy is not installed on this system. Please install scrcpy first.".to_string());
    }

    let mut cmd = Command::new("scrcpy");
    if let Some(ref s) = serial {
        cmd.args(&["-s", s]);
    }

    match cmd.spawn() {
        Ok(_) => Ok("scrcpy launched successfully".to_string()),
        Err(e) => Err(format!("Failed to launch scrcpy: {}", e)),
    }
}

#[tauri::command]
pub fn execute_shell(serial: Option<String>, command: String, root_mode: bool) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        args.extend_from_slice(&["-s", s]);
    }

    let escaped = command.replace('"', "\\\"");
    let full_cmd = if root_mode {
        format!("su -c \"{}\"", escaped)
    } else {
        command
    };

    args.extend_from_slice(&["shell", &full_cmd]);
    let (code, stdout, stderr) = run_adb_cmd(&args)?;
    if code == 0 {
        Ok(stdout)
    } else if !stdout.is_empty() {
        Ok(format!("{}\n{}", stdout, stderr))
    } else {
        Err(stderr)
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

static LOGCAT_CHILD: std::sync::Mutex<Option<std::process::Child>> = std::sync::Mutex::new(None);

#[tauri::command]
pub fn start_logcat_stream(
    app: tauri::AppHandle,
    serial: Option<String>,
    filter: Option<String>,
    level: Option<String>,
) -> Result<(), String> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command as StdCommand, Stdio};
    use tauri::Emitter;

    // Terminate existing child if any
    if let Ok(mut lock) = LOGCAT_CHILD.lock() {
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    let adb = detect_adb();
    let mut cmd = StdCommand::new(&adb);
    if let Some(ref s) = serial {
        if !s.trim().is_empty() {
            cmd.args(&["-s", s.trim()]);
        }
    }
    cmd.arg("logcat");
    cmd.args(&["-v", "time"]);

    if let Some(ref lvl) = level {
        let l = lvl.trim();
        if !l.is_empty() && l != "V" && l != "All" {
            cmd.arg(format!("*:{}", l));
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn adb logcat: {}", e))?;
    let stdout = child.stdout.take().ok_or_else(|| "Failed to capture logcat stdout".to_string())?;

    if let Ok(mut lock) = LOGCAT_CHILD.lock() {
        *lock = Some(child);
    }

    let filter_kw = filter
        .map(|f| f.trim().to_lowercase())
        .filter(|f| !f.is_empty());

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let matches = if let Some(ref kw) = filter_kw {
                        l.to_lowercase().contains(kw)
                    } else {
                        true
                    };
                    if matches {
                        if app.emit("logcat-line", l).is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_logcat_stream() -> Result<(), String> {
    if let Ok(mut lock) = LOGCAT_CHILD.lock() {
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn clear_logcat(serial: Option<String>) -> Result<String, String> {
    let mut args = Vec::new();
    if let Some(ref s) = serial {
        if !s.trim().is_empty() {
            args.extend_from_slice(&["-s", s.trim()]);
        }
    }
    args.extend_from_slice(&["logcat", "-c"]);
    let (code, _, err) = run_adb_cmd(&args)?;
    if code == 0 {
        Ok("Logcat buffer cleared".to_string())
    } else {
        Err(err)
    }
}

