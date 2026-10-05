use crate::utils::{
    blocking, detect_adb, new_task_id, run_adb, run_adb_streaming, sh_quote, std_command,
    tokio_command, SerialGuard,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

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
    pub rom_type: String,
    pub rom_name: String,
    pub rom_version: String,
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

fn curated_db() -> &'static Vec<BloatwareItem> {
    static DB: OnceLock<Vec<BloatwareItem>> = OnceLock::new();
    DB.get_or_init(get_curated_bloatware)
}

/// Only plain Android package names may reach `adb shell`, which re-parses its arguments.
fn validate_package(pkg: &str) -> Result<(), String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9_.]*$").unwrap());
    if re.is_match(pkg) {
        Ok(())
    } else {
        Err(format!("Invalid package name: {}", pkg))
    }
}

fn non_empty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t) }
}

// ---------------------------------------------------------------------------
// Devices
// ---------------------------------------------------------------------------

fn parse_devices(stdout: &str) -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            devices.push(DeviceInfo {
                serial: parts[0].to_string(),
                state: parts[1].to_string(),
                info: parts[2..].join(" "),
            });
        }
    }
    devices
}

#[tauri::command]
pub async fn get_adb_devices() -> Result<Vec<DeviceInfo>, String> {
    blocking(|| {
        let (code, stdout, stderr) = run_adb(None, &["devices", "-l"])?;
        if code != 0 {
            return Err(format!("adb devices failed: {}", stderr.trim()));
        }
        Ok(parse_devices(&stdout))
    })
    .await
}

// ---------------------------------------------------------------------------
// Device specs (one getprop dump + one batched shell call)
// ---------------------------------------------------------------------------

/// Parse `adb shell getprop` output: `[ro.product.model]: [23049PCD8G]`.
pub fn parse_getprop(out: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in out.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once("]: [") {
            let key = k.trim_start_matches('[');
            let val = v.trim_end_matches(']');
            if !val.is_empty() {
                map.insert(key.to_string(), val.to_string());
            }
        }
    }
    map
}

fn prop<'a>(p: &'a HashMap<String, String>, key: &str) -> &'a str {
    p.get(key).map(String::as_str).unwrap_or("")
}

fn first_prop<'a>(p: &'a HashMap<String, String>, keys: &[&str]) -> &'a str {
    for k in keys {
        let v = prop(p, k);
        if !v.is_empty() {
            return v;
        }
    }
    ""
}

#[derive(Debug, Clone)]
pub struct RomDetails {
    pub rom_type: String,
    pub rom_name: String,
    pub rom_version: String,
    pub short_version: String,
}

/// (rom type, display name, props holding the version, separator that cuts the short version)
const CUSTOM_ROMS: &[(&str, &str, &[&str], Option<char>)] = &[
    ("LineageOS", "LineageOS", &["ro.lineage.display.version", "ro.lineage.version", "ro.lineage.build.version"], Some('-')),
    ("Pixel Experience", "Pixel Experience", &["ro.pixelexperience.version", "ro.pe.version"], Some('-')),
    ("Evolution X", "Evolution X", &["ro.evolution.version", "ro.evo.version"], Some('_')),
    ("crDroid", "crDroid Android", &["ro.crdroid.version", "ro.cr.version"], None),
    ("ArrowOS", "ArrowOS", &["ro.arrow.version"], Some('_')),
    ("Paranoid Android", "Paranoid Android", &["ro.aospa.version", "ro.pa.version"], None),
    ("RisingOS", "RisingOS", &["ro.rising.version"], Some('-')),
    ("DerpFest", "DerpFest", &["ro.derp.version"], None),
    ("Project Elixir", "Project Elixir", &["ro.elixir.version"], None),
    ("PixelOS", "PixelOS", &["ro.pixelos.version"], None),
    ("BlissROM", "BlissROM", &["ro.bliss.version"], None),
    ("Havoc-OS", "Havoc-OS", &["ro.havoc.version"], None),
    ("SparkOS", "SparkOS", &["ro.spark.version"], None),
    ("CherishOS", "CherishOS", &["ro.cherish.version"], None),
    ("CalyxOS", "CalyxOS", &["ro.calyxos.version"], None),
];

/// `OS2.0.214.0.VOGEUXM` -> `OS2.0.214.0`, `V14.0.5.0.TKXMIXM` -> `V14.0.5.0`.
fn short_xiaomi_version(full: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^(OS|V)(\d+(?:\.\d+){1,3})").unwrap());
    match re.captures(full) {
        Some(c) => format!("{}{}", &c[1], &c[2]),
        None => full.to_string(),
    }
}

pub fn detect_rom(p: &HashMap<String, String>, android_ver: &str) -> RomDetails {
    let hyperos_inc = prop(p, "ro.mi.os.version.incremental");
    let hyperos_name = prop(p, "ro.mi.os.version.name");
    let miui_ui = prop(p, "ro.miui.ui.version.name");
    let miui_code = prop(p, "ro.miui.version.code_time");

    // Xiaomi HyperOS
    if !hyperos_inc.is_empty() || hyperos_name.starts_with("OS") || miui_ui == "V816" {
        let full = if !hyperos_inc.is_empty() {
            hyperos_inc.to_string()
        } else if !hyperos_name.is_empty() {
            hyperos_name.to_string()
        } else {
            "OS1.0".to_string()
        };
        return RomDetails {
            rom_type: "HyperOS".into(),
            rom_name: "Xiaomi HyperOS".into(),
            short_version: short_xiaomi_version(&full),
            rom_version: full,
        };
    }

    // Xiaomi MIUI
    if !miui_ui.is_empty() || !miui_code.is_empty() {
        let inc = prop(p, "ro.build.version.incremental");
        let full = non_empty(inc).or(non_empty(miui_ui)).unwrap_or("MIUI").to_string();
        let name = match miui_ui.strip_prefix('V') {
            // "V140" -> MIUI 14, "V125" -> MIUI 12.5
            Some(n) if n.len() >= 3 && n.chars().all(|c| c.is_ascii_digit()) => {
                let (major, minor) = n.split_at(n.len() - 1);
                if minor == "0" {
                    format!("Xiaomi MIUI {}", major)
                } else {
                    format!("Xiaomi MIUI {}.{}", major, minor)
                }
            }
            _ if !miui_ui.is_empty() => format!("Xiaomi MIUI {}", miui_ui),
            _ => "Xiaomi MIUI".to_string(),
        };
        return RomDetails {
            rom_type: "MIUI".into(),
            rom_name: name,
            short_version: short_xiaomi_version(&full),
            rom_version: full,
        };
    }

    // Known custom ROM props
    for (rtype, rname, keys, sep) in CUSTOM_ROMS {
        let v = first_prop(p, keys);
        if !v.is_empty() {
            let short = match sep {
                Some(c) => v.split(*c).next().unwrap_or(v).to_string(),
                None => v.to_string(),
            };
            return RomDetails {
                rom_type: rtype.to_string(),
                rom_name: rname.to_string(),
                short_version: short,
                rom_version: v.to_string(),
            };
        }
    }

    // Generic inspection of build strings
    let display_id = prop(p, "ro.build.display.id");
    let flavor = prop(p, "ro.build.flavor");
    let combined = format!(
        "{} {} {} {}",
        display_id,
        flavor,
        prop(p, "ro.modversion"),
        prop(p, "ro.rom.version")
    )
    .to_lowercase();
    let build = non_empty(display_id).or(non_empty(flavor)).unwrap_or("-").to_string();

    for (needle, rtype, rname) in [
        ("lineage", "LineageOS", "LineageOS"),
        ("evolution", "Evolution X", "Evolution X"),
        ("crdroid", "crDroid", "crDroid Android"),
        ("graphene", "GrapheneOS", "GrapheneOS"),
    ] {
        if combined.contains(needle) {
            return RomDetails {
                rom_type: rtype.into(),
                rom_name: rname.into(),
                short_version: rtype.into(),
                rom_version: build,
            };
        }
    }

    let android_short = if android_ver.is_empty() { "-".to_string() } else { format!("Android {}", android_ver) };

    if combined.contains("aosp") || flavor.starts_with("aosp_") {
        return RomDetails {
            rom_type: "AOSP".into(),
            rom_name: "AOSP Pure".into(),
            short_version: android_short,
            rom_version: build,
        };
    }
    if !display_id.is_empty() && !display_id.contains("MIUI") {
        return RomDetails {
            rom_type: "AOSP / Custom".into(),
            rom_name: "AOSP Custom ROM".into(),
            short_version: android_short,
            rom_version: display_id.to_string(),
        };
    }
    RomDetails {
        rom_type: "AOSP".into(),
        rom_name: "AOSP Android".into(),
        short_version: android_short,
        rom_version: build,
    }
}

/// Marketing label for installed RAM. `MemTotal` is always a little below the physical size.
pub fn ram_label(mem_total_kb: u64) -> String {
    let gib = mem_total_kb as f64 / 1_048_576.0;
    const SIZES: [f64; 12] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 12.0, 16.0, 18.0, 24.0, 32.0];
    for s in SIZES {
        if gib <= s * 1.02 {
            return format!("{} GB", s);
        }
    }
    format!("{:.0} GB", gib.ceil())
}

/// "used / advertised" storage from `df -k /data` numbers.
pub fn storage_label(total_kb: u64, used_kb: u64) -> String {
    let total_gib = total_kb as f64 / 1_048_576.0;
    let used_gib = used_kb as f64 / 1_048_576.0;
    const SIZES: [u32; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
    let advertised = SIZES
        .iter()
        .find(|s| total_gib <= **s as f64)
        .map(|s| *s as f64)
        .unwrap_or(total_gib.ceil());
    let adv = if advertised >= 1024.0 {
        format!("{} TB", advertised / 1024.0)
    } else {
        format!("{} GB", advertised)
    };
    format!("{:.1} GB / {}", used_gib, adv)
}

/// Parse `df -k /data` (tolerates wrapped lines): returns (total_kb, used_kb).
pub fn parse_df(out: &str) -> Option<(u64, u64)> {
    let mut lines = out.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next()?;
    if !header.to_lowercase().contains("filesystem") && !header.to_lowercase().contains("size") {
        return None;
    }
    let tokens: Vec<&str> = lines.flat_map(|l| l.split_whitespace()).collect();
    if tokens.len() < 3 {
        return None;
    }
    let total = tokens[1].parse::<u64>().ok()?;
    let used = tokens[2].parse::<u64>().ok()?;
    Some((total, used))
}

const SOC_NAMES: &[(&str, &str)] = &[
    ("sm8750", "Snapdragon 8 Elite"),
    ("sun", "Snapdragon 8 Elite"),
    ("sm8650", "Snapdragon 8 Gen 3"),
    ("pineapple", "Snapdragon 8 Gen 3"),
    ("sm8550", "Snapdragon 8 Gen 2"),
    ("kalama", "Snapdragon 8 Gen 2"),
    ("sm8475", "Snapdragon 8+ Gen 1"),
    ("cape", "Snapdragon 8+ Gen 1"),
    ("sm8450", "Snapdragon 8 Gen 1"),
    ("taro", "Snapdragon 8 Gen 1"),
    ("sm8350", "Snapdragon 888"),
    ("lahaina", "Snapdragon 888"),
    ("sm8250", "Snapdragon 865"),
    ("kona", "Snapdragon 865"),
    ("sm8150", "Snapdragon 855"),
    ("msmnile", "Snapdragon 855"),
    ("sm7325", "Snapdragon 778G"),
    ("sm7475", "Snapdragon 7+ Gen 2"),
    ("sm6375", "Snapdragon 695"),
    ("sm6225", "Snapdragon 680"),
    ("sm6115", "Snapdragon 662"),
    ("mt6989", "Dimensity 9300"),
    ("mt6985", "Dimensity 9200"),
    ("mt6983", "Dimensity 9000"),
    ("mt6895", "Dimensity 8100"),
    ("mt6893", "Dimensity 1200"),
    ("mt6891", "Dimensity 1100"),
    ("mt6877", "Dimensity 900"),
];

pub fn soc_label(p: &HashMap<String, String>) -> String {
    let model = first_prop(p, &["ro.soc.model", "ro.board.platform", "ro.hardware.chipname", "ro.hardware"]);
    if model.is_empty() {
        return "-".to_string();
    }
    let key = model.to_lowercase();
    match SOC_NAMES.iter().find(|(k, _)| *k == key) {
        Some((_, name)) => format!("{} ({})", name, model.to_uppercase()),
        None => {
            let maker = prop(p, "ro.soc.manufacturer");
            if maker.is_empty() || key == "qcom" {
                model.to_string()
            } else {
                format!("{} {}", maker, model)
            }
        }
    }
}

pub fn battery_label(out: &str) -> String {
    let mut level = None;
    let mut status = None;
    let mut temp = None;
    let mut ac = false;
    let mut usb = false;
    for line in out.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once(':') {
            let v = v.trim();
            match k.trim() {
                "level" => level = v.parse::<u32>().ok(),
                "status" => status = v.parse::<u32>().ok(),
                "temperature" => temp = v.parse::<i32>().ok(),
                "AC powered" => ac = v == "true",
                "USB powered" => usb = v == "true",
                _ => {}
            }
        }
    }
    let Some(level) = level else { return "-".to_string() };
    let st = match status {
        Some(2) => "Charging",
        Some(3) => "Discharging",
        Some(4) => "Not charging",
        Some(5) => "Full",
        _ if ac || usb => "Charging",
        _ => "",
    };
    let mut extra = Vec::new();
    if !st.is_empty() {
        extra.push(st.to_string());
    }
    if let Some(t) = temp {
        extra.push(format!("{:.1}°C", t as f64 / 10.0));
    }
    if extra.is_empty() {
        format!("{}%", level)
    } else {
        format!("{}% ({})", level, extra.join(", "))
    }
}

fn dash(s: &str) -> String {
    if s.trim().is_empty() { "-".to_string() } else { s.trim().to_string() }
}

/// Turn adb stderr into something the user can act on.
fn explain_adb_error(stderr: &str) -> String {
    let low = stderr.to_lowercase();
    if low.contains("unauthorized") {
        "Device is unauthorized — unlock the phone and accept the USB debugging prompt.".to_string()
    } else if low.contains("offline") {
        "Device is offline — re-plug the cable or toggle USB debugging.".to_string()
    } else if low.contains("no devices") || low.contains("not found") {
        "Device not found — it may have been disconnected.".to_string()
    } else if stderr.trim().is_empty() {
        "Device did not respond.".to_string()
    } else {
        stderr.trim().to_string()
    }
}

const SPEC_BATCH: &str = "grep -m1 MemTotal /proc/meminfo; echo @@; df -k /data; echo @@; dumpsys battery";

fn collect_device_specs(serial: &str) -> Result<DeviceSpecs, String> {
    let s = Some(serial);
    let (code, out, err) = run_adb(s, &["shell", "getprop"])?;
    if code != 0 {
        return Err(explain_adb_error(&err));
    }
    let props = parse_getprop(&out);
    if props.is_empty() {
        return Err(explain_adb_error(&err));
    }

    let android_ver = prop(&props, "ro.build.version.release");
    let model = prop(&props, "ro.product.model");
    let device = first_prop(&props, &["ro.product.device", "ro.build.product", "ro.product.name"]);
    let market = first_prop(
        &props,
        &[
            "ro.product.marketname",
            "ro.product.vendor.marketname",
            "ro.product.odm.marketname",
            "ro.product.system.marketname",
            "ro.product.model",
            "ro.product.device",
        ],
    );
    let brand = first_prop(&props, &["ro.product.brand", "ro.product.manufacturer"]);
    let patch = prop(&props, "ro.build.version.security_patch");

    let rom = detect_rom(&props, android_ver);

    // RAM / storage / battery in a single round trip
    let mut ram = "-".to_string();
    let mut storage = "-".to_string();
    let mut battery = "-".to_string();
    if let Ok((_, out, _)) = run_adb(s, &["shell", SPEC_BATCH]) {
        let mut parts = out.splitn(3, "@@");
        if let Some(mem) = parts.next() {
            if let Some(kb) = mem.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok()) {
                ram = ram_label(kb);
            }
        }
        if let Some(df) = parts.next() {
            if let Some((total, used)) = parse_df(df) {
                storage = storage_label(total, used);
            }
        }
        if let Some(b) = parts.next() {
            battery = battery_label(b);
        }
    }

    Ok(DeviceSpecs {
        market_name: dash(market),
        model: dash(model),
        device: dash(device),
        brand: dash(brand),
        android_ver: dash(android_ver),
        security_patch: dash(patch),
        hyperos_version: rom.rom_version.clone(),
        hyperos_short: rom.short_version.clone(),
        rom_type: rom.rom_type,
        rom_name: rom.rom_name,
        rom_version: rom.rom_version,
        cpu: soc_label(&props),
        ram,
        storage,
        battery,
    })
}

type SpecCache = Mutex<HashMap<String, (Instant, DeviceSpecs)>>;

fn spec_cache() -> &'static SpecCache {
    static C: OnceLock<SpecCache> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Cached for a few seconds so the UI, fastfetch and refresh clicks don't re-query the phone.
pub fn device_specs_cached(serial: &str, max_age: Duration) -> Result<DeviceSpecs, String> {
    if let Some((at, specs)) = spec_cache().lock().unwrap_or_else(|p| p.into_inner()).get(serial) {
        if at.elapsed() < max_age {
            return Ok(specs.clone());
        }
    }
    let specs = collect_device_specs(serial)?;
    spec_cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(serial.to_string(), (Instant::now(), specs.clone()));
    Ok(specs)
}

#[tauri::command]
pub async fn get_device_specs(serial: Option<String>) -> Result<DeviceSpecs, String> {
    let serial = serial
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "No device specified".to_string())?;
    blocking(move || device_specs_cached(&serial, Duration::from_secs(5))).await
}

// ---------------------------------------------------------------------------
// Packages
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_packages(serial: Option<String>, filter_mode: String) -> Result<Vec<PackageItem>, String> {
    blocking(move || {
        let mut rest = vec!["shell", "pm", "list", "packages"];
        match filter_mode.as_str() {
            "3rd" => rest.push("-3"),
            "system" => rest.push("-s"),
            "disabled" => rest.push("-d"),
            _ => {}
        }
        let (code, stdout, stderr) = run_adb(serial.as_deref(), &rest)?;
        if code != 0 {
            return Err(format!("Failed to list packages: {}", explain_adb_error(&stderr)));
        }

        let installed: std::collections::HashSet<&str> = stdout
            .lines()
            .filter_map(|l| l.trim().strip_prefix("package:"))
            .map(str::trim)
            .collect();

        let db = curated_db();
        let bloat_map: HashMap<&str, &BloatwareItem> = db.iter().map(|b| (b.package.as_str(), b)).collect();

        let mut result: Vec<PackageItem> = Vec::new();
        if filter_mode == "recommended" {
            for b in db.iter().filter(|b| installed.contains(b.package.as_str())) {
                result.push(PackageItem {
                    package: b.package.clone(),
                    name: b.name.clone(),
                    category: b.category.clone(),
                    risk: b.risk.clone(),
                    description: b.description.clone(),
                    is_installed: true,
                });
            }
        } else {
            for pkg in &installed {
                if let Some(b) = bloat_map.get(pkg) {
                    result.push(PackageItem {
                        package: pkg.to_string(),
                        name: b.name.clone(),
                        category: b.category.clone(),
                        risk: b.risk.clone(),
                        description: b.description.clone(),
                        is_installed: true,
                    });
                } else {
                    result.push(PackageItem {
                        package: pkg.to_string(),
                        name: pkg.rsplit('.').next().unwrap_or(pkg).to_string(),
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
    })
    .await
}

fn pm_action(
    serial: Option<String>,
    package: String,
    rest: Vec<&'static str>,
    ok_marker: &'static [&'static str],
    ok_msg: &'static str,
) -> Result<String, String> {
    validate_package(&package)?;
    let mut args: Vec<&str> = rest;
    args.push(&package);
    let (code, stdout, stderr) = run_adb(serial.as_deref(), &args)?;
    let low = stdout.to_lowercase();
    if code == 0 && ok_marker.iter().any(|m| low.contains(m)) {
        Ok(format!("{} {}", ok_msg, package))
    } else {
        Err(if stdout.trim().is_empty() { stderr } else { stdout }.trim().to_string())
    }
}

#[tauri::command]
pub async fn uninstall_package(serial: Option<String>, package: String) -> Result<String, String> {
    blocking(move || {
        pm_action(serial, package, vec!["shell", "pm", "uninstall", "-k", "--user", "0"], &["success"], "Successfully uninstalled")
    })
    .await
}

#[tauri::command]
pub async fn restore_package(serial: Option<String>, package: String) -> Result<String, String> {
    blocking(move || {
        pm_action(
            serial,
            package,
            vec!["shell", "cmd", "package", "install-existing"],
            &["installed", "success"],
            "Successfully restored",
        )
    })
    .await
}

#[tauri::command]
pub async fn disable_package(serial: Option<String>, package: String) -> Result<String, String> {
    blocking(move || {
        pm_action(serial, package, vec!["shell", "pm", "disable-user", "--user", "0"], &["new state"], "Successfully disabled")
    })
    .await
}

#[tauri::command]
pub async fn enable_package(serial: Option<String>, package: String) -> Result<String, String> {
    blocking(move || pm_action(serial, package, vec!["shell", "pm", "enable"], &["new state"], "Successfully enabled")).await
}

#[tauri::command]
pub async fn reboot_device(serial: Option<String>, mode: String) -> Result<String, String> {
    blocking(move || {
        let mut rest = vec!["reboot"];
        match mode.as_str() {
            "recovery" | "bootloader" | "edl" => rest.push(mode.as_str()),
            "system" | "" => {}
            other => return Err(format!("Unknown reboot mode: {}", other)),
        }
        let (code, _out, stderr) = run_adb(serial.as_deref(), &rest)?;
        if code == 0 {
            Ok(format!("Device rebooting to {}", mode))
        } else {
            Err(explain_adb_error(&stderr))
        }
    })
    .await
}

// ---------------------------------------------------------------------------
// Install / screenshot / shell
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn install_apk(
    app: tauri::AppHandle,
    serial: Option<String>,
    apk_path: String,
    task_id: Option<String>,
) -> Result<String, String> {
    let _guard = SerialGuard::acquire(serial.as_deref())?;
    let id = new_task_id(task_id);
    let (code, out) = run_adb_streaming(app, serial.as_deref(), &["install", "-r", &apk_path], id).await?;
    if code == 0 && out.to_lowercase().contains("success") {
        Ok("APK installed successfully!".to_string())
    } else {
        Err(out.trim().to_string())
    }
}

fn screenshot_dir() -> std::path::PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let pictures = home.join("Pictures");
    if pictures.is_dir() { pictures.join("Ximi Screenshots") } else { home.join("Ximi Screenshots") }
}

#[tauri::command]
pub async fn take_screenshot(serial: Option<String>) -> Result<String, String> {
    blocking(move || {
        let mut cmd = std_command(&detect_adb());
        cmd.args(crate::utils::serial_args(serial.as_deref()));
        cmd.args(["exec-out", "screencap", "-p"]).stdin(Stdio::null());
        let out = cmd.output().map_err(|e| format!("Failed to run adb: {}", e))?;
        // PNG signature check: exec-out is binary-safe, anything else is an error message.
        if !out.status.success() || out.stdout.len() < 8 || &out.stdout[1..4] != b"PNG" {
            let msg = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(format!("Screenshot failed: {}", explain_adb_error(&msg)));
        }
        let dir = screenshot_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = dir.join(format!("screenshot_{}.png", ts));
        std::fs::write(&path, &out.stdout).map_err(|e| format!("Cannot save screenshot: {}", e))?;
        Ok(format!("Screenshot saved to {}", path.display()))
    })
    .await
}

#[tauri::command]
pub async fn execute_shell(serial: Option<String>, command: String, root_mode: bool) -> Result<String, String> {
    let full_cmd = if root_mode { format!("su -c {}", sh_quote(&command)) } else { command };
    let mut cmd = tokio_command(&detect_adb());
    cmd.args(crate::utils::serial_args(serial.as_deref()));
    cmd.arg("shell").arg(&full_cmd).stdin(Stdio::null());

    let out = match tokio::time::timeout(Duration::from_secs(60), cmd.output()).await {
        Ok(r) => r.map_err(|e| format!("Failed to run adb: {}", e))?,
        Err(_) => return Err("Command timed out after 60s (interactive/long-running commands are not supported).".to_string()),
    };
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (false, true) => Ok(stdout),
        (false, false) => Ok(format!("{}\n{}", stdout.trim_end(), stderr)),
        (true, false) if out.status.success() => Ok(stderr),
        (true, false) => Err(stderr),
        (true, true) if out.status.success() => Ok(String::new()),
        (true, true) => Err(format!("Command exited with status {:?}", out.status.code())),
    }
}

// ---------------------------------------------------------------------------
// Live logcat (no console window, batched events, live filter)
// ---------------------------------------------------------------------------

struct LogcatSession {
    child: std::process::Child,
    filter: Arc<Mutex<String>>,
}

fn logcat_slot() -> &'static Mutex<Option<LogcatSession>> {
    static S: OnceLock<Mutex<Option<LogcatSession>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

#[derive(Serialize, Clone)]
struct LogcatBatch {
    session: String,
    lines: Vec<String>,
}

#[derive(Serialize, Clone)]
struct LogcatEnded {
    session: String,
}

/// Kill the running logcat child (also used on app exit).
pub fn stop_logcat_blocking() {
    let session = logcat_slot().lock().unwrap_or_else(|p| p.into_inner()).take();
    if let Some(mut s) = session {
        let _ = s.child.kill();
        let _ = s.child.wait();
    }
}

fn read_lossy_lines<R: std::io::Read>(r: R, mut f: impl FnMut(String) -> bool) {
    let mut reader = BufReader::new(r);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let line = String::from_utf8_lossy(&buf).trim_end_matches(['\r', '\n']).to_string();
                if !line.is_empty() && !f(line) {
                    break;
                }
            }
        }
    }
}

const LOGCAT_MAX_PENDING: usize = 4000;

#[tauri::command]
pub async fn start_logcat_stream(
    app: tauri::AppHandle,
    serial: Option<String>,
    filter: Option<String>,
    level: Option<String>,
    session_id: Option<String>,
) -> Result<(), String> {
    use tauri::Emitter;

    blocking(move || {
        stop_logcat_blocking();

        let session = new_task_id(session_id);
        let mut cmd = std_command(&detect_adb());
        cmd.args(crate::utils::serial_args(serial.as_deref()));
        cmd.args(["logcat", "-v", "time"]);
        if let Some(l) = level.as_deref().map(str::trim) {
            if matches!(l, "D" | "I" | "W" | "E" | "F") {
                cmd.arg(format!("*:{}", l));
            }
        }
        cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn adb logcat: {}", e))?;
        let stdout = child.stdout.take().ok_or("Failed to capture logcat output")?;
        let stderr = child.stderr.take();

        let filter_shared = Arc::new(Mutex::new(filter.unwrap_or_default().trim().to_lowercase()));
        let pending: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let alive = Arc::new(AtomicBool::new(true));

        // stdout reader: filter + queue
        {
            let (pending, alive, filter) = (pending.clone(), alive.clone(), filter_shared.clone());
            std::thread::spawn(move || {
                read_lossy_lines(stdout, |line| {
                    let kw = filter.lock().unwrap_or_else(|p| p.into_inner()).clone();
                    if kw.is_empty() || line.to_lowercase().contains(&kw) {
                        let mut q = pending.lock().unwrap_or_else(|p| p.into_inner());
                        if q.len() >= LOGCAT_MAX_PENDING {
                            let drop_n = q.len() / 2; // backpressure: shed oldest lines
                            q.drain(..drop_n);
                        }
                        q.push(line);
                    }
                    true
                });
                alive.store(false, Ordering::SeqCst);
            });
        }
        // stderr reader: surface adb errors instead of letting the pipe fill up
        if let Some(err) = stderr {
            let pending = pending.clone();
            std::thread::spawn(move || {
                read_lossy_lines(err, |line| {
                    pending.lock().unwrap_or_else(|p| p.into_inner()).push(format!("[adb] {}", line));
                    true
                });
            });
        }
        // flusher: one IPC event per ~60ms instead of one per line
        {
            let (pending, alive, id) = (pending, alive, session);
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(60));
                let batch = std::mem::take(&mut *pending.lock().unwrap_or_else(|p| p.into_inner()));
                if !batch.is_empty()
                    && app.emit("logcat-batch", LogcatBatch { session: id.clone(), lines: batch }).is_err()
                {
                    break;
                }
                if !alive.load(Ordering::SeqCst) && pending.lock().unwrap_or_else(|p| p.into_inner()).is_empty() {
                    let _ = app.emit("logcat-ended", LogcatEnded { session: id.clone() });
                    break;
                }
            });
        }

        *logcat_slot().lock().unwrap_or_else(|p| p.into_inner()) = Some(LogcatSession { child, filter: filter_shared });
        Ok(())
    })
    .await
}

#[tauri::command]
pub fn set_logcat_filter(filter: String) {
    if let Some(s) = logcat_slot().lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
        *s.filter.lock().unwrap_or_else(|p| p.into_inner()) = filter.trim().to_lowercase();
    }
}

#[tauri::command]
pub async fn stop_logcat_stream() -> Result<(), String> {
    blocking(|| {
        stop_logcat_blocking();
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clear_logcat(serial: Option<String>) -> Result<String, String> {
    blocking(move || {
        let (code, _, err) = run_adb(serial.as_deref(), &["logcat", "-c"])?;
        if code == 0 { Ok("Logcat buffer cleared".to_string()) } else { Err(err) }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn getprop_dump_is_parsed() {
        let out = "[ro.product.model]: [23049PCD8G]\n[ro.empty]: []\n[ro.build.version.release]: [15]\n";
        let p = parse_getprop(out);
        assert_eq!(p["ro.product.model"], "23049PCD8G");
        assert_eq!(p["ro.build.version.release"], "15");
        assert!(!p.contains_key("ro.empty"));
    }

    #[test]
    fn hyperos2_version_is_short() {
        let p = props(&[
            ("ro.mi.os.version.incremental", "OS2.0.214.0.VOGEUXM"),
            ("ro.mi.os.version.name", "OS2.0"),
        ]);
        let r = detect_rom(&p, "15");
        assert_eq!(r.rom_type, "HyperOS");
        assert_eq!(r.short_version, "OS2.0.214.0");
        assert_eq!(r.rom_version, "OS2.0.214.0.VOGEUXM");
    }

    #[test]
    fn miui14_is_detected() {
        let p = props(&[
            ("ro.miui.ui.version.name", "V140"),
            ("ro.build.version.incremental", "V14.0.5.0.TKXMIXM"),
        ]);
        let r = detect_rom(&p, "13");
        assert_eq!(r.rom_type, "MIUI");
        assert_eq!(r.rom_name, "Xiaomi MIUI 14");
        assert_eq!(r.short_version, "V14.0.5.0");
    }

    #[test]
    fn custom_rom_and_aosp_fallback() {
        let r = detect_rom(&props(&[("ro.lineage.version", "21.0-20250101-NIGHTLY-tanzanite")]), "15");
        assert_eq!(r.rom_type, "LineageOS");
        assert_eq!(r.short_version, "21.0");
        // a flavor that merely contains "pixel" must not become a "Pixel AOSP" ROM
        let r = detect_rom(&props(&[("ro.build.flavor", "somepixelthing-userdebug")]), "14");
        assert_ne!(r.rom_name, "Pixel AOSP ROM");
        assert_eq!(detect_rom(&HashMap::new(), "14").rom_type, "AOSP");
    }

    #[test]
    fn ram_buckets() {
        assert_eq!(ram_label(1_900_000), "2 GB");
        assert_eq!(ram_label(2_800_000), "3 GB");
        assert_eq!(ram_label(3_700_000), "4 GB");
        assert_eq!(ram_label(5_600_000), "6 GB");
        assert_eq!(ram_label(7_500_000), "8 GB");
        assert_eq!(ram_label(11_400_000), "12 GB");
        assert_eq!(ram_label(15_300_000), "16 GB");
        assert_eq!(ram_label(23_000_000), "24 GB");
    }

    #[test]
    fn storage_is_advertised_size() {
        // ~235 GiB /data on a 256 GB phone, 40 GiB used
        let s = storage_label(235 * 1_048_576, 40 * 1_048_576);
        assert_eq!(s, "40.0 GB / 256 GB");
        assert_eq!(storage_label(110 * 1_048_576, 10 * 1_048_576), "10.0 GB / 128 GB");
    }

    #[test]
    fn df_parsing_handles_wrapped_lines() {
        let one = "Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/block/dm-9 246000000 41000000 205000000 17% /data\n";
        assert_eq!(parse_df(one), Some((246000000, 41000000)));
        let wrapped = "Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/block/mapper/very_long_name\n 246000000 41000000 205000000 17% /data\n";
        assert_eq!(parse_df(wrapped), Some((246000000, 41000000)));
    }

    #[test]
    fn soc_names() {
        let p = props(&[("ro.soc.manufacturer", "QTI"), ("ro.soc.model", "SM8550")]);
        assert_eq!(soc_label(&p), "Snapdragon 8 Gen 2 (SM8550)");
        let p = props(&[("ro.board.platform", "kalama")]);
        assert_eq!(soc_label(&p), "Snapdragon 8 Gen 2 (KALAMA)");
        let p = props(&[("ro.board.platform", "unknownchip")]);
        assert_eq!(soc_label(&p), "unknownchip");
        assert_eq!(soc_label(&HashMap::new()), "-");
    }

    #[test]
    fn battery_summary() {
        let out = "Current Battery Service state:\n  AC powered: false\n  USB powered: true\n  status: 2\n  level: 85\n  temperature: 315\n";
        assert_eq!(battery_label(out), "85% (Charging, 31.5°C)");
        assert_eq!(battery_label("nothing"), "-");
    }

    #[test]
    fn package_validation() {
        assert!(validate_package("com.miui.analytics").is_ok());
        assert!(validate_package("com.x; rm -rf /").is_err());
        assert!(validate_package("").is_err());
    }

    #[test]
    fn device_list_parsing() {
        let out = "List of devices attached\nabc123\tdevice product:x model:Y\nxyz\tunauthorized\n\n";
        let d = parse_devices(out);
        assert_eq!(d.len(), 2);
        assert_eq!(d[1].state, "unauthorized");
    }
}
