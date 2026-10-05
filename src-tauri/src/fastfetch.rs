use crate::adb::device_specs_cached;
use crate::utils::{blocking, run_adb};
use std::time::Duration;

const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_BLUE: &str = "\x1b[38;2;66;133;244m";
const C_PURPLE: &str = "\x1b[38;2;168;85;247m";
const C_CYAN: &str = "\x1b[38;2;6;182;212m";
const C_GREEN: &str = "\x1b[38;2;34;197;94m";
const C_YELLOW: &str = "\x1b[38;2;234;179;8m";
const C_RED: &str = "\x1b[38;2;239;68;68m";
const C_WHITE: &str = "\x1b[38;2;248;250;252m";
const C_GRAY: &str = "\x1b[38;2;148;163;184m";

#[tauri::command]
pub async fn run_fastfetch(serial: Option<String>, root_mode: Option<bool>) -> Result<String, String> {
    let serial = serial
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "No ADB device connected. Please connect a device first.".to_string())?;
    let is_root_req = root_mode.unwrap_or(false);
    blocking(move || build_fastfetch(&serial, is_root_req)).await
}

fn build_fastfetch(serial: &str, root_mode: bool) -> Result<String, String> {
    let specs = device_specs_cached(serial, Duration::from_secs(30))?;
    let s_ref = Some(serial);

    // kernel, uptime, display and uid in one round trip
    let (_, out, _) = run_adb(s_ref, &["shell", "uname -r; echo @@; cat /proc/uptime; echo @@; wm size; echo @@; id -u"])?;
    let mut parts = out.splitn(4, "@@");
    let kernel_str = parts.next().map(str::trim).filter(|s| !s.is_empty()).unwrap_or("-").to_string();

    let mut uptime_str = "-".to_string();
    if let Some(sec) = parts.next().and_then(|p| p.split_whitespace().next()).and_then(|f| f.parse::<f64>().ok()) {
        let s_int = sec as u64;
        let (days, hours, mins) = (s_int / 86400, (s_int % 86400) / 3600, (s_int % 3600) / 60);
        let mut v = Vec::new();
        if days > 0 { v.push(format!("{} days", days)); }
        if hours > 0 { v.push(format!("{} hours", hours)); }
        v.push(format!("{} mins", mins));
        uptime_str = v.join(", ");
    }

    // "Physical size: 1080x2400" and optionally "Override size: ..." (the override is what is shown)
    let res_str = parts
        .next()
        .and_then(|w| {
            let line = w.lines().rev().find(|l| l.contains("size:"))?;
            line.split(':').nth(1).map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "-".into());

    let uid = parts.next().map(str::trim).unwrap_or("2000").to_string();
    let is_root = root_mode || uid == "0";
    let title_user = if is_root { "root" } else { "shell" };
    let device_host = &specs.device;
    let title_line = format!("{C_BOLD}{C_PURPLE}{title_user}{C_RESET}@{C_BOLD}{C_BLUE}{device_host}{C_RESET}");
    let sep_len = format!("{}@{}", title_user, device_host).len();
    let separator = format!("{C_GRAY}{}{C_RESET}", "─".repeat(sep_len));

    let logo = [
        format!("{C_PURPLE}       .---.       {C_RESET}"),
        format!("{C_PURPLE}    .-'     '-.    {C_RESET}"),
        format!("{C_PURPLE}  .'   {C_BLUE}.-.   {C_PURPLE}'.  {C_RESET}"),
        format!("{C_BLUE} /    /   \\    \\ {C_RESET}"),
        format!("{C_BLUE}|    |  {C_WHITE}●{C_BLUE}  |    |{C_RESET}"),
        format!("{C_CYAN} \\    \\   /    / {C_RESET}"),
        format!("{C_CYAN}  '.   {C_CYAN}'-'   {C_CYAN}.'  {C_RESET}"),
        format!("{C_CYAN}    '-.     .-'    {C_RESET}"),
        format!("{C_CYAN}       '---'       {C_RESET}"),
        "                   ".to_string(),
        "                   ".to_string(),
    ];

    let os_display = if specs.rom_type == "HyperOS" {
        format!("Xiaomi HyperOS {}", specs.hyperos_version)
    } else if specs.rom_type == "MIUI" {
        format!("{} {}", specs.rom_name, specs.hyperos_version)
    } else {
        format!("{} ({})", specs.rom_name, specs.rom_version)
    };

    let info_rows = [
        title_line,
        separator,
        format!("{C_BOLD}{C_PURPLE}OS{C_RESET}: {}", os_display),
        format!("{C_BOLD}{C_PURPLE}ROM Type{C_RESET}: {}", specs.rom_type),
        format!("{C_BOLD}{C_PURPLE}Host{C_RESET}: {} ({})", specs.market_name, specs.model),
        format!("{C_BOLD}{C_PURPLE}Kernel{C_RESET}: {}", kernel_str),
        format!("{C_BOLD}{C_PURPLE}User{C_RESET}: {} ({})", title_user, if is_root { "Privileged UID 0".to_string() } else { format!("Unprivileged UID {}", uid) }),
        format!("{C_BOLD}{C_PURPLE}Android{C_RESET}: {}", specs.android_ver),
        format!("{C_BOLD}{C_PURPLE}Uptime{C_RESET}: {}", uptime_str),
        format!("{C_BOLD}{C_PURPLE}Display{C_RESET}: {}", res_str),
        format!("{C_BOLD}{C_PURPLE}CPU{C_RESET}: {}", specs.cpu),
        format!("{C_BOLD}{C_PURPLE}Memory{C_RESET}: {}", specs.ram),
        format!("{C_BOLD}{C_PURPLE}Storage{C_RESET}: {}", specs.storage),
        format!("{C_BOLD}{C_PURPLE}Battery{C_RESET}: {}", specs.battery),
        format!("{C_BOLD}{C_PURPLE}Security{C_RESET}: {}", specs.security_patch),
        "".to_string(),
        format!("{C_RED}● {C_YELLOW}● {C_GREEN}● {C_CYAN}● {C_BLUE}● {C_PURPLE}● {C_WHITE}● {C_GRAY}●{C_RESET}"),
    ];

    let max_len = std::cmp::max(logo.len(), info_rows.len());
    let mut out_lines = Vec::new();
    out_lines.push("".to_string());

    for i in 0..max_len {
        let l = if i < logo.len() { &logo[i] } else { "                   " };
        let r = if i < info_rows.len() { &info_rows[i] } else { "" };
        out_lines.push(format!("  {}   {}", l, r));
    }
    out_lines.push("".to_string());

    Ok(out_lines.join("\n"))
}
