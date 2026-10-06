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
        // Samsung Services & Bloatware
        BloatwareItem {
            package: "com.samsung.android.bixby.agent".into(),
            name: "Bixby Voice".into(),
            category: "Bixby".into(),
            risk: "Safe".into(),
            description: "Samsung Bixby voice assistant.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.bixby.service".into(),
            name: "Bixby Service".into(),
            category: "Bixby".into(),
            risk: "Safe".into(),
            description: "Bixby core background service.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.bixby.wakeup".into(),
            name: "Bixby Wakeup".into(),
            category: "Bixby".into(),
            risk: "Safe".into(),
            description: "Bixby hotword voice wakeup listener.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.app.spage".into(),
            name: "Samsung Free / Daily".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Samsung lockscreen/homescreen media and news feed.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.game.gamehome".into(),
            name: "Samsung Gaming Hub".into(),
            category: "Games".into(),
            risk: "Safe".into(),
            description: "Samsung Game Launcher / Gaming Hub with promo games.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.ardrawing".into(),
            name: "AR Doodle".into(),
            category: "Samsung".into(),
            risk: "Safe".into(),
            description: "Samsung AR camera doodle feature.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.arzone".into(),
            name: "AR Zone".into(),
            category: "Samsung".into(),
            risk: "Safe".into(),
            description: "Samsung AR emoji and stickers suite.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.kidsinstaller".into(),
            name: "Samsung Kids Installer".into(),
            category: "Samsung".into(),
            risk: "Safe".into(),
            description: "Installer for Samsung Kids Mode sandbox.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.app.tips".into(),
            name: "Samsung Tips".into(),
            category: "Samsung".into(),
            risk: "Safe".into(),
            description: "Device usage tips and feature promotional popups.".into(),
        },
        BloatwareItem {
            package: "com.samsung.android.rubin.app".into(),
            name: "Customization Service (Rubin)".into(),
            category: "Analytics".into(),
            risk: "Safe".into(),
            description: "Samsung personalized analytics and ad targeting engine.".into(),
        },
        // Vivo / iQOO Services & Bloatware
        BloatwareItem {
            package: "com.vivo.appstore".into(),
            name: "V-Appstore".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Vivo official app marketplace with push ads.".into(),
        },
        BloatwareItem {
            package: "com.bbk.appstore".into(),
            name: "V-Appstore Engine".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Core download engine for BBK/Vivo App Store.".into(),
        },
        BloatwareItem {
            package: "com.vivo.browser".into(),
            name: "Vivo Browser".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Stock Vivo browser bundled with news and ad recommendations.".into(),
        },
        BloatwareItem {
            package: "com.vivo.game".into(),
            name: "Vivo Game Center".into(),
            category: "Games".into(),
            risk: "Safe".into(),
            description: "Vivo gaming store and promotion service.".into(),
        },
        BloatwareItem {
            package: "com.vivo.globalsearch".into(),
            name: "Jovi Search".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Global search with online trending feeds and ads.".into(),
        },
        BloatwareItem {
            package: "com.vivo.hiboard".into(),
            name: "Jovi Smart Scene (HiBoard)".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Negative-one screen with news feeds and sponsored cards.".into(),
        },
        BloatwareItem {
            package: "com.vivo.magazine".into(),
            name: "Lockscreen Magazine".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Lock screen rotating wallpapers with sponsored stories.".into(),
        },
        BloatwareItem {
            package: "com.bbk.theme".into(),
            name: "i Theme".into(),
            category: "System".into(),
            risk: "Optional".into(),
            description: "Vivo theme and wallpaper store.".into(),
        },
        // OPPO / Realme / OnePlus (ColorOS / OxygenOS)
        BloatwareItem {
            package: "com.heytap.browser".into(),
            name: "HeyTap Browser".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "OPPO/Realme stock browser with promotional feed.".into(),
        },
        BloatwareItem {
            package: "com.heytap.market".into(),
            name: "App Market".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "OPPO/Realme official application marketplace.".into(),
        },
        BloatwareItem {
            package: "com.heytap.pictorial".into(),
            name: "Lock Screen Magazine".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "OPPO/Realme lock screen wallpapers with ads.".into(),
        },
        BloatwareItem {
            package: "com.heytap.themestore".into(),
            name: "Theme Store".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Theme marketplace with promotional push banners.".into(),
        },
        BloatwareItem {
            package: "com.coloros.gamespace".into(),
            name: "Game Space".into(),
            category: "Games".into(),
            risk: "Optional".into(),
            description: "Gaming assistant and accelerator.".into(),
        },
        BloatwareItem {
            package: "com.oppo.quicksearchbox".into(),
            name: "Global Search".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "OPPO global search bar with trending ad tags.".into(),
        },
        BloatwareItem {
            package: "com.oneplus.mall".into(),
            name: "OnePlus Store".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "OnePlus hardware and accessory store.".into(),
        },
        BloatwareItem {
            package: "com.oneplus.membership".into(),
            name: "Red Cable Club".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "OnePlus membership and promotional rewards.".into(),
        },
        // Transsion (Infinix XOS / Tecno HiOS / itel)
        BloatwareItem {
            package: "com.transsion.palmpay".into(),
            name: "PalmPay".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Preloaded mobile payment and promotional wallet.".into(),
        },
        BloatwareItem {
            package: "com.transsion.phoenix".into(),
            name: "Phoenix Browser".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Aggressive third-party browser bundled on Transsion devices.".into(),
        },
        BloatwareItem {
            package: "com.transsnet.boomplayer".into(),
            name: "Boomplay".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Preloaded music streaming client with banner ads.".into(),
        },
        BloatwareItem {
            package: "com.talpa.hibrowser".into(),
            name: "Hi Browser".into(),
            category: "Ads".into(),
            risk: "Safe".into(),
            description: "Tecno HiOS default browser with sponsored feeds.".into(),
        },
        // Microsoft & Partner Preloads
        BloatwareItem {
            package: "com.microsoft.skydrive".into(),
            name: "Microsoft OneDrive".into(),
            category: "Microsoft".into(),
            risk: "Safe".into(),
            description: "Microsoft cloud storage client preloaded on Samsung/OEMs.".into(),
        },
        BloatwareItem {
            package: "com.microsoft.office.officehubrow".into(),
            name: "Microsoft 365 (Office)".into(),
            category: "Microsoft".into(),
            risk: "Safe".into(),
            description: "Microsoft Office productivity suite.".into(),
        },
        BloatwareItem {
            package: "com.microsoft.office.outlook".into(),
            name: "Microsoft Outlook".into(),
            category: "Microsoft".into(),
            risk: "Safe".into(),
            description: "Microsoft Outlook email client.".into(),
        },
        BloatwareItem {
            package: "com.linkedin.android".into(),
            name: "LinkedIn".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "LinkedIn professional network app.".into(),
        },
        BloatwareItem {
            package: "com.amazon.mShop.android.shopping".into(),
            name: "Amazon Shopping".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Amazon e-commerce store client.".into(),
        },
        BloatwareItem {
            package: "com.amazon.appmanager".into(),
            name: "Amazon App Manager".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Silent Amazon installer service.".into(),
        },
        BloatwareItem {
            package: "com.netflix.mediaclient".into(),
            name: "Netflix".into(),
            category: "Bloatware".into(),
            risk: "Optional".into(),
            description: "Netflix streaming service client.".into(),
        },
        BloatwareItem {
            package: "com.netflix.partner.activation".into(),
            name: "Netflix Partner Activation".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Background stub for activating bundled Netflix offers.".into(),
        },
        BloatwareItem {
            package: "com.spotify.music".into(),
            name: "Spotify".into(),
            category: "Bloatware".into(),
            risk: "Optional".into(),
            description: "Preloaded Spotify music streaming app.".into(),
        },
        BloatwareItem {
            package: "com.bytedance.tiktok".into(),
            name: "TikTok".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Preloaded TikTok short-video app.".into(),
        },
        BloatwareItem {
            package: "com.zhiliaoapp.musically".into(),
            name: "TikTok Global".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Preloaded TikTok regional/global package.".into(),
        },
        BloatwareItem {
            package: "com.shopee.id".into(),
            name: "Shopee".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Shopee e-commerce client preloaded on SE Asian OEM builds.".into(),
        },
        BloatwareItem {
            package: "com.lazada.android".into(),
            name: "Lazada".into(),
            category: "Bloatware".into(),
            risk: "Safe".into(),
            description: "Lazada shopping client preloaded on OEM devices.".into(),
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
    ("Corvus OS", "Corvus OS", &["ro.corvus.version"], None),
    ("SuperiorOS", "SuperiorOS", &["ro.superior.version"], None),
    ("AncientOS", "AncientOS", &["ro.ancient.version"], None),
    ("Nusantara Project", "Nusantara Project", &["ro.nusantara.version"], None),
    ("Resurrection Remix", "Resurrection Remix", &["ro.rr.version"], None),
    ("GrapheneOS", "GrapheneOS", &["ro.build.version.graphene"], None),
    ("/e/OS", "/e/OS", &["ro.e.version"], None),
    ("iodéOS", "iodéOS", &["ro.iode.version"], None),
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

fn parse_vivo_rom(p: &HashMap<String, String>) -> Option<RomDetails> {
    let vivo_name = prop(p, "ro.vivo.os.name");
    let vivo_ver = prop(p, "ro.vivo.os.version");
    let vivo_display = prop(p, "ro.vivo.os.build.display.id");
    let vivo_prod_ver = prop(p, "ro.vivo.product.version");
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();

    if !vivo_name.is_empty() || !vivo_display.is_empty() || brand == "vivo" || brand == "iqoo" {
        let is_origin = vivo_name.to_lowercase().contains("origin")
            || vivo_display.to_lowercase().contains("origin");
        let rom_type = if is_origin { "OriginOS" } else { "Funtouch OS" };
        let rom_name = if is_origin { "Vivo OriginOS" } else { "Vivo Funtouch OS" };

        let short_ver = if !vivo_ver.is_empty() {
            vivo_ver.to_string()
        } else if !vivo_display.is_empty() {
            vivo_display
                .replace("Funtouch OS_", "")
                .replace("Funtouch OS ", "")
                .replace("OriginOS ", "")
        } else {
            "-".to_string()
        };

        let full_ver = if !vivo_prod_ver.is_empty() && !vivo_display.is_empty() {
            format!("{} ({})", vivo_prod_ver, vivo_display)
        } else if !vivo_display.is_empty() {
            vivo_display.to_string()
        } else if !vivo_prod_ver.is_empty() {
            vivo_prod_ver.to_string()
        } else {
            short_ver.clone()
        };

        return Some(RomDetails {
            rom_type: rom_type.into(),
            rom_name: rom_name.into(),
            short_version: short_ver,
            rom_version: full_ver,
        });
    }
    None
}

fn parse_samsung_oneui(p: &HashMap<String, String>, android_ver: &str) -> Option<RomDetails> {
    let oneui_prop = first_prop(p, &["ro.build.version.oneui", "ro.build.version.one_ui"]);
    let sep_prop = prop(p, "ro.build.version.sep");
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();

    if !oneui_prop.is_empty() || !sep_prop.is_empty() || brand == "samsung" {
        let mut short = String::new();
        if !oneui_prop.is_empty() {
            if oneui_prop.contains('.') {
                short = oneui_prop.to_string();
            } else if let Ok(num) = oneui_prop.parse::<u32>() {
                let major = num / 10000;
                let minor = (num % 10000) / 100;
                short = if minor > 0 { format!("{}.{}", major, minor) } else { format!("{}", major) };
            }
        }
        if short.is_empty() && !sep_prop.is_empty() {
            if let Ok(num) = sep_prop.parse::<u32>() {
                let sep_major = num / 10000;
                let sep_minor = (num % 10000) / 100;
                if sep_major >= 10 {
                    let oneui_major = sep_major - 9;
                    short = if sep_minor > 0 { format!("{}.{}", oneui_major, sep_minor) } else { format!("{}", oneui_major) };
                }
            }
        }
        if short.is_empty() && brand == "samsung" {
            short = match android_ver.trim() {
                "15" => "7.0",
                "14" => "6.0",
                "13" => "5.0",
                "12" => "4.0",
                "11" => "3.0",
                "10" => "2.0",
                "9" => "1.0",
                _ => "",
            }
            .to_string();
        }

        if !short.is_empty() || brand == "samsung" {
            let display_id = prop(p, "ro.build.display.id");
            let build = if !display_id.is_empty() {
                display_id.to_string()
            } else if !short.is_empty() {
                format!("One UI {}", short)
            } else {
                "One UI".to_string()
            };
            return Some(RomDetails {
                rom_type: "One UI".into(),
                rom_name: "Samsung One UI".into(),
                short_version: if short.is_empty() { "-".into() } else { short },
                rom_version: build,
            });
        }
    }
    None
}

fn parse_oppo_oneplus_realme(p: &HashMap<String, String>) -> Option<RomDetails> {
    // 1. Realme UI
    let realme_ui = first_prop(p, &["ro.build.version.realmeui", "ro.realme.version"]);
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();
    if !realme_ui.is_empty() || brand == "realme" {
        let v = if !realme_ui.is_empty() { realme_ui } else { prop(p, "ro.build.display.id") };
        let short = v.trim_start_matches(['V', 'v']).to_string();
        return Some(RomDetails {
            rom_type: "Realme UI".into(),
            rom_name: "Realme UI".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    // 2. OxygenOS / HydrogenOS (OnePlus)
    let oxygen = first_prop(p, &["ro.oxygen.version", "ro.build.ota.versionname"]);
    if (!oxygen.is_empty() && oxygen.to_lowercase().contains("oxygen")) || prop(p, "ro.oxygen.version") != "" || brand == "oneplus" {
        let v = first_prop(p, &["ro.oxygen.version", "ro.build.ota.versionname", "ro.build.display.id"]);
        let short = v.replace("OxygenOS_", "").replace("Oxygen OS_", "").replace("OxygenOS ", "");
        return Some(RomDetails {
            rom_type: "OxygenOS".into(),
            rom_name: "OnePlus OxygenOS".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    // 3. ColorOS (OPPO)
    let coloros = first_prop(p, &["ro.build.version.opporom", "ro.coloros.version", "ro.build.version.oplusrom"]);
    if !coloros.is_empty() || brand == "oppo" {
        let v = if !coloros.is_empty() { coloros } else { prop(p, "ro.build.display.id") };
        let short = v.trim_start_matches(['V', 'v']).to_string();
        return Some(RomDetails {
            rom_type: "ColorOS".into(),
            rom_name: "OPPO ColorOS".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    None
}

fn parse_huawei_honor(p: &HashMap<String, String>) -> Option<RomDetails> {
    let magic = first_prop(p, &["ro.build.version.magic", "ro.honor.build.version.incremental"]);
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();
    if !magic.is_empty() || brand == "honor" {
        let v = if !magic.is_empty() { magic } else { prop(p, "ro.build.display.id") };
        let short = v.replace("MagicOS ", "").replace("MagicUI ", "").replace("MagicOS_", "");
        return Some(RomDetails {
            rom_type: "MagicOS".into(),
            rom_name: "Honor MagicOS".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    let harmony = first_prop(p, &["hw_sc.build.platform.version", "ro.harmony.version"]);
    if !harmony.is_empty() {
        return Some(RomDetails {
            rom_type: "HarmonyOS".into(),
            rom_name: "Huawei HarmonyOS".into(),
            short_version: harmony.replace("HarmonyOS ", "").replace("HarmonyOS_", ""),
            rom_version: harmony.to_string(),
        });
    }

    let emui = first_prop(p, &["ro.build.version.emui", "ro.build.hw_emui_api_level"]);
    if !emui.is_empty() || brand == "huawei" {
        let v = if !emui.is_empty() { emui } else { prop(p, "ro.build.display.id") };
        let short = v.replace("EmotionUI_", "").replace("EMUI ", "").replace("EMUI_", "");
        return Some(RomDetails {
            rom_type: "EMUI".into(),
            rom_name: "Huawei EMUI".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    None
}

fn parse_transsion(p: &HashMap<String, String>) -> Option<RomDetails> {
    let xos = first_prop(p, &["ro.xos.version", "ro.transtek.xos.version"]);
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();
    if !xos.is_empty() || brand == "infinix" {
        let v = if !xos.is_empty() { xos } else { prop(p, "ro.build.display.id") };
        let short = v.replace("XOS ", "").replace('v', "").replace('V', "");
        return Some(RomDetails {
            rom_type: "XOS".into(),
            rom_name: "Infinix XOS".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    let hios = first_prop(p, &["ro.hios.version", "ro.transtek.hios.version"]);
    if !hios.is_empty() || brand == "tecno" {
        let v = if !hios.is_empty() { hios } else { prop(p, "ro.build.display.id") };
        let short = v.replace("HiOS ", "").replace('v', "").replace('V', "");
        return Some(RomDetails {
            rom_type: "HiOS".into(),
            rom_name: "Tecno HiOS".into(),
            short_version: if short.is_empty() { "-".into() } else { short },
            rom_version: v.to_string(),
        });
    }

    let itel = first_prop(p, &["ro.itel.version", "ro.transtek.itel.version"]);
    if !itel.is_empty() || brand == "itel" {
        let v = if !itel.is_empty() { itel } else { prop(p, "ro.build.display.id") };
        return Some(RomDetails {
            rom_type: "itelOS".into(),
            rom_name: "itelOS".into(),
            short_version: v.replace("itelOS ", "").to_string(),
            rom_version: v.to_string(),
        });
    }

    None
}

fn parse_other_oems(p: &HashMap<String, String>, android_ver: &str) -> Option<RomDetails> {
    let asus = first_prop(p, &["ro.asus.uiversion", "ro.build.asus.version"]);
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();
    if !asus.is_empty() || brand == "asus" {
        let is_rog = asus.to_lowercase().contains("rog") || prop(p, "ro.product.model").to_lowercase().contains("rog");
        let rom_type = if is_rog { "ROG UI" } else { "ZenUI" };
        let rom_name = if is_rog { "ASUS ROG UI" } else { "ASUS ZenUI" };
        let v = if !asus.is_empty() { asus } else { prop(p, "ro.build.display.id") };
        return Some(RomDetails {
            rom_type: rom_type.into(),
            rom_name: rom_name.into(),
            short_version: v.replace("ZenUI ", "").replace("ROG UI ", ""),
            rom_version: v.to_string(),
        });
    }

    let nothing = first_prop(p, &["ro.nothing.version", "ro.build.version.nos"]);
    if !nothing.is_empty() || brand == "nothing" {
        let v = if !nothing.is_empty() { nothing } else { prop(p, "ro.build.display.id") };
        return Some(RomDetails {
            rom_type: "Nothing OS".into(),
            rom_name: "Nothing OS".into(),
            short_version: v.replace("Nothing OS ", "").replace("NOS ", ""),
            rom_version: v.to_string(),
        });
    }

    let moto_blur = first_prop(p, &["ro.mot.build.customerid", "ro.motorola.build.version"]);
    if !moto_blur.is_empty() || brand == "motorola" {
        let v = first_prop(p, &["ro.build.version.full", "ro.build.display.id"]);
        let is_hello = android_ver.parse::<u32>().unwrap_or(0) >= 14;
        let rom_type = if is_hello { "Hello UI" } else { "My UX" };
        let rom_name = format!("Motorola {}", rom_type);
        return Some(RomDetails {
            rom_type: rom_type.to_string(),
            rom_name,
            short_version: if android_ver.is_empty() { "-".to_string() } else { format!("Android {}", android_ver) },
            rom_version: if !v.is_empty() { v.to_string() } else { moto_blur.to_string() },
        });
    }

    None
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

    // OEM ROMs: Vivo / iQOO
    if let Some(rom) = parse_vivo_rom(p) {
        return rom;
    }

    // OEM ROMs: Samsung One UI
    if let Some(rom) = parse_samsung_oneui(p, android_ver) {
        return rom;
    }

    // OEM ROMs: OPPO / OnePlus / Realme
    if let Some(rom) = parse_oppo_oneplus_realme(p) {
        return rom;
    }

    // OEM ROMs: Huawei / Honor
    if let Some(rom) = parse_huawei_honor(p) {
        return rom;
    }

    // OEM ROMs: Transsion (Infinix / Tecno / itel)
    if let Some(rom) = parse_transsion(p) {
        return rom;
    }

    // OEM ROMs: Asus / Nothing / Motorola
    if let Some(rom) = parse_other_oems(p, android_ver) {
        return rom;
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
        ("pixelos", "PixelOS", "PixelOS"),
        ("corvus", "Corvus OS", "Corvus OS"),
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

    // Google Pixel Stock
    let brand = first_prop(p, &["ro.product.brand", "ro.product.manufacturer"]).to_lowercase();
    if brand == "google" && !combined.contains("aosp") {
        return RomDetails {
            rom_type: "Pixel".into(),
            rom_name: "Google Pixel".into(),
            short_version: android_short,
            rom_version: build,
        };
    }

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
// Packages & Label Extraction
// ---------------------------------------------------------------------------

const LABEL_HELPER_BYTES: &[u8] = include_bytes!("../resources/ximi-pkglabels.jar");
const LABEL_HELPER_DEVICE_PATH: &str = "/data/local/tmp/ximi-pkglabels.jar";

static PROVISIONED_DEVICES: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();

fn is_helper_provisioned(serial: &str) -> bool {
    let map = PROVISIONED_DEVICES.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = map.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(instant) = guard.get(serial) {
        if instant.elapsed() < Duration::from_secs(3600) {
            return true;
        }
    }
    false
}

fn mark_helper_provisioned(serial: &str) {
    let map = PROVISIONED_DEVICES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap_or_else(|p| p.into_inner());
    guard.insert(serial.to_string(), Instant::now());
}

fn invalidate_helper_provisioned(serial: &str) {
    let map = PROVISIONED_DEVICES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap_or_else(|p| p.into_inner());
    guard.remove(serial);
}

fn deploy_helper_if_needed(serial: Option<&str>) -> Result<(), String> {
    let key = serial.unwrap_or("default");
    if is_helper_provisioned(key) {
        return Ok(());
    }
    let temp_jar = std::env::temp_dir().join("ximi-pkglabels.jar");
    if !temp_jar.exists() || std::fs::metadata(&temp_jar).map(|m| m.len()).unwrap_or(0) != LABEL_HELPER_BYTES.len() as u64 {
        std::fs::write(&temp_jar, LABEL_HELPER_BYTES)
            .map_err(|e| format!("Failed to write helper jar locally: {}", e))?;
    }
    let (code, _out, err) = run_adb(serial, &["push", temp_jar.to_str().unwrap_or(""), LABEL_HELPER_DEVICE_PATH])?;
    if code != 0 {
        return Err(format!("Failed to push helper to device: {}", err.trim()));
    }
    mark_helper_provisioned(key);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct RawPkgInfo {
    pub package: String,
    pub is_system: bool,
    pub is_enabled: bool,
    pub is_installed: bool,
    pub label: String,
}

pub fn parse_pkg_labels_output(stdout: &str) -> Vec<RawPkgInfo> {
    let mut list = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let pkg = parts[0].trim().to_string();
            if pkg.is_empty() {
                continue;
            }
            let is_system = parts[1].trim() == "1";
            let is_enabled = parts[2].trim() == "1";
            let is_installed = parts[3].trim() == "1";
            let label = if parts.len() >= 5 {
                parts[4].trim().to_string()
            } else {
                String::new()
            };
            list.push(RawPkgInfo {
                package: pkg,
                is_system,
                is_enabled,
                is_installed,
                label,
            });
        }
    }
    list
}

fn run_pkg_labels_helper(serial: Option<&str>) -> Result<Vec<RawPkgInfo>, String> {
    let key = serial.unwrap_or("default");
    deploy_helper_if_needed(serial)?;

    let cmd_str = format!("CLASSPATH={} app_process /system/bin XimiPkgLabels", LABEL_HELPER_DEVICE_PATH);
    let (code, stdout, _stderr) = run_adb(serial, &["shell", &cmd_str])?;

    if code != 0 || stdout.trim().is_empty() {
        invalidate_helper_provisioned(key);
        deploy_helper_if_needed(serial)?;
        let (code2, stdout2, stderr2) = run_adb(serial, &["shell", &cmd_str])?;
        if code2 != 0 || stdout2.trim().is_empty() {
            return Err(format!("Helper execution failed: {}", stderr2.trim()));
        }
        return Ok(parse_pkg_labels_output(&stdout2));
    }

    Ok(parse_pkg_labels_output(&stdout))
}

fn fallback_pm_list_packages(serial: Option<&str>) -> Result<Vec<RawPkgInfo>, String> {
    let (code, stdout, stderr) = run_adb(
        serial,
        &["shell", "pm list packages; echo @@; pm list packages -s; echo @@; pm list packages -d"],
    )?;
    if code != 0 {
        return Err(format!("Failed to list packages: {}", explain_adb_error(&stderr)));
    }
    let mut parts = stdout.split("@@");
    let all_part = parts.next().unwrap_or("");
    let sys_part = parts.next().unwrap_or("");
    let dis_part = parts.next().unwrap_or("");

    let sys_set: std::collections::HashSet<&str> =
        sys_part.lines().filter_map(|l| l.trim().strip_prefix("package:")).map(str::trim).collect();
    let dis_set: std::collections::HashSet<&str> =
        dis_part.lines().filter_map(|l| l.trim().strip_prefix("package:")).map(str::trim).collect();

    let mut list = Vec::new();
    for line in all_part.lines() {
        if let Some(pkg) = line.trim().strip_prefix("package:").map(str::trim) {
            if !pkg.is_empty() {
                list.push(RawPkgInfo {
                    package: pkg.to_string(),
                    is_system: sys_set.contains(pkg),
                    is_enabled: !dis_set.contains(pkg),
                    is_installed: true,
                    label: String::new(),
                });
            }
        }
    }
    Ok(list)
}

fn humanize_package(pkg: &str) -> String {
    let last = pkg.rsplit('.').next().unwrap_or(pkg);
    let mut chars = last.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => pkg.to_string(),
    }
}

type PkgCache = Mutex<HashMap<String, (Instant, Vec<RawPkgInfo>)>>;

fn pkg_cache() -> &'static PkgCache {
    static C: OnceLock<PkgCache> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn invalidate_pkg_cache(serial: Option<&str>) {
    let key = serial.unwrap_or("default");
    if let Ok(mut lock) = pkg_cache().lock() {
        lock.remove(key);
    }
}

#[tauri::command]
pub async fn get_packages(serial: Option<String>, filter_mode: String) -> Result<Vec<PackageItem>, String> {
    blocking(move || {
        let s_ref = serial.as_deref();
        let s_key = s_ref.unwrap_or("default");

        let cached = {
            let guard = pkg_cache().lock().unwrap_or_else(|p| p.into_inner());
            guard.get(s_key).and_then(|(at, list)| {
                if at.elapsed() < Duration::from_secs(8) {
                    Some(list.clone())
                } else {
                    None
                }
            })
        };

        let raw_list = match cached {
            Some(list) => list,
            None => {
                let list = match run_pkg_labels_helper(s_ref) {
                    Ok(l) if !l.is_empty() => l,
                    Ok(_) | Err(_) => fallback_pm_list_packages(s_ref)?,
                };
                pkg_cache()
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .insert(s_key.to_string(), (Instant::now(), list.clone()));
                list
            }
        };

        let db = curated_db();
        let bloat_map: HashMap<&str, &BloatwareItem> = db.iter().map(|b| (b.package.as_str(), b)).collect();

        let mut result: Vec<PackageItem> = Vec::new();
        for item in raw_list {
            let matches_filter = match filter_mode.as_str() {
                "recommended" => item.is_installed && bloat_map.contains_key(item.package.as_str()),
                "3rd" | "user" => item.is_installed && !item.is_system,
                "system" => item.is_installed && item.is_system,
                "disabled" => item.is_installed && !item.is_enabled,
                "all" => item.is_installed,
                "uninstalled" => !item.is_installed,
                _ => item.is_installed,
            };

            if !matches_filter {
                continue;
            }

            let curated = bloat_map.get(item.package.as_str()).copied();
            let name = if !item.label.is_empty() && item.label != item.package {
                item.label
            } else if let Some(b) = curated {
                b.name.clone()
            } else {
                humanize_package(&item.package)
            };

            let (category, risk, description) = if let Some(b) = curated {
                (b.category.clone(), b.risk.clone(), b.description.clone())
            } else if item.is_system {
                ("System".to_string(), "Optional".to_string(), "-".to_string())
            } else {
                ("3rd Party".to_string(), "Optional".to_string(), "-".to_string())
            };

            result.push(PackageItem {
                package: item.package,
                name,
                category,
                risk,
                description,
                is_installed: item.is_installed,
            });
        }

        result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
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
        invalidate_pkg_cache(serial.as_deref());
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

    #[test]
    fn vivo_funtouch_detection() {
        let p = props(&[
            ("ro.vivo.os.name", "Funtouch"),
            ("ro.vivo.os.version", "4.5"),
            ("ro.vivo.os.build.display.id", "Funtouch OS_4.5"),
            ("ro.vivo.product.version", "PD1818EF_EX_A_1.12.2"),
        ]);
        let r = detect_rom(&p, "8.1.0");
        assert_eq!(r.rom_type, "Funtouch OS");
        assert_eq!(r.rom_name, "Vivo Funtouch OS");
        assert_eq!(r.short_version, "4.5");
        assert!(r.rom_version.contains("PD1818EF_EX_A_1.12.2"));
    }

    #[test]
    fn samsung_oneui_detection() {
        let p = props(&[
            ("ro.build.version.oneui", "60100"),
            ("ro.build.display.id", "UP1A.231005.007.S918BXXU3BWJM"),
            ("ro.product.brand", "samsung"),
        ]);
        let r = detect_rom(&p, "14");
        assert_eq!(r.rom_type, "One UI");
        assert_eq!(r.rom_name, "Samsung One UI");
        assert_eq!(r.short_version, "6.1");
    }

    #[test]
    fn oppo_coloros_detection() {
        let p = props(&[
            ("ro.build.version.opporom", "V14.0.0"),
            ("ro.build.display.id", "CPH2451_14.0.0.300(EX01)"),
            ("ro.product.brand", "OPPO"),
        ]);
        let r = detect_rom(&p, "14");
        assert_eq!(r.rom_type, "ColorOS");
        assert_eq!(r.rom_name, "OPPO ColorOS");
        assert_eq!(r.short_version, "14.0.0");
    }

    #[test]
    fn transsion_xos_detection() {
        let p = props(&[
            ("ro.xos.version", "XOS 13.0.0"),
            ("ro.product.brand", "Infinix"),
        ]);
        let r = detect_rom(&p, "13");
        assert_eq!(r.rom_type, "XOS");
        assert_eq!(r.rom_name, "Infinix XOS");
        assert_eq!(r.short_version, "13.0.0");
    }

    #[test]
    fn package_labels_parsing() {
        let sample = "com.google.android.youtube\t1\t1\t1\tYouTube\ncom.vivo.easyshare\t0\t1\t1\tEasyShare\n";
        let parsed = parse_pkg_labels_output(sample);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].package, "com.google.android.youtube");
        assert_eq!(parsed[1].package, "com.vivo.easyshare");
        assert_eq!(parsed[1].label, "EasyShare");
        assert!(!parsed[1].is_system);
    }

    #[test]
    fn live_connected_device_check() {
        if let Ok((code, stdout, _)) = run_adb(None, &["devices"]) {
            if code == 0 && stdout.contains("\tdevice") {
                let devices = parse_devices(&stdout);
                if let Some(dev) = devices.first() {
                    let specs = collect_device_specs(&dev.serial).unwrap();
                    println!("Live detected device: {} (Brand: {})", dev.serial, specs.brand);
                    println!("ROM Type: {}, Name: {}, Short: {}, Full: {}", specs.rom_type, specs.rom_name, specs.hyperos_short, specs.rom_version);
                    assert!(!specs.rom_type.is_empty());
                    assert_ne!(specs.rom_type, "-");

                    let raw_pkgs = run_pkg_labels_helper(Some(&dev.serial)).unwrap();
                    println!("Total live packages extracted with labels: {}", raw_pkgs.len());
                    assert!(raw_pkgs.len() > 50);
                    let labeled = raw_pkgs.iter().filter(|p| !p.label.is_empty() && p.label != p.package).count();
                    println!("Packages with real UI display label string: {}", labeled);
                    for p in raw_pkgs.iter().filter(|p| !p.label.is_empty() && p.label != p.package).take(5) {
                        println!("  Sample app: {} -> '{}'", p.package, p.label);
                    }
                    assert!(labeled > 20);
                }
            }
        }
    }
}
