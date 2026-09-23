use crate::adb::get_device_specs;
use crate::utils::run_adb_cmd;

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
pub fn run_fastfetch(serial: Option<String>) -> Result<String, String> {
    let specs = get_device_specs(serial.clone()).unwrap_or_else(|_| crate::adb::DeviceSpecs {
        market_name: "Xiaomi HyperOS Device".into(),
        model: "HyperOS".into(),
        device: "hyperos".into(),
        brand: "Xiaomi".into(),
        android_ver: "14".into(),
        security_patch: "2026-09-01".into(),
        hyperos_version: "OS4.0.0.3".into(),
        hyperos_short: "4.0.0.3".into(),
        cpu: "Octa-core Max 2.8GHz".into(),
        ram: "8.0GB".into(),
        storage: "72.5GB/256GB".into(),
        battery: "5000mAh (100%)".into(),
    });

    let s_ref = serial.as_deref();

    // Kernel uname
    let mut uname_args = Vec::new();
    if let Some(s) = s_ref {
        uname_args.extend_from_slice(&["-s", s]);
    }
    uname_args.extend_from_slice(&["shell", "uname", "-r"]);
    let kernel_str = if let Ok((0, out, _)) = run_adb_cmd(&uname_args) {
        out.trim().to_string()
    } else {
        "Linux 5.15.x-android".to_string()
    };

    // Uptime
    let mut uptime_args = Vec::new();
    if let Some(s) = s_ref {
        uptime_args.extend_from_slice(&["-s", s]);
    }
    uptime_args.extend_from_slice(&["shell", "cat", "/proc/uptime"]);
    let mut uptime_str = "1 day, 4 hours".to_string();
    if let Ok((0, out, _)) = run_adb_cmd(&uptime_args) {
        if let Some(first) = out.split_whitespace().next() {
            if let Ok(sec) = first.parse::<f64>() {
                let s_int = sec as u64;
                let days = s_int / 86400;
                let hours = (s_int % 86400) / 3600;
                let mins = (s_int % 3600) / 60;
                let mut parts = Vec::new();
                if days > 0 { parts.push(format!("{} days", days)); }
                if hours > 0 { parts.push(format!("{} hours", hours)); }
                parts.push(format!("{} mins", mins));
                uptime_str = parts.join(", ");
            }
        }
    }

    // Display resolution
    let mut wm_args = Vec::new();
    if let Some(s) = s_ref {
        wm_args.extend_from_slice(&["-s", s]);
    }
    wm_args.extend_from_slice(&["shell", "wm", "size"]);
    let res_str = if let Ok((0, out, _)) = run_adb_cmd(&wm_args) {
        out.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_else(|| "1080x2400".into())
    } else {
        "1080x2400".into()
    };

    let title_user = "root";
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

    let info_rows = [
        title_line,
        separator,
        format!("{C_BOLD}{C_PURPLE}OS{C_RESET}: Xiaomi HyperOS {}", specs.hyperos_version),
        format!("{C_BOLD}{C_PURPLE}Host{C_RESET}: {} ({})", specs.market_name, specs.model),
        format!("{C_BOLD}{C_PURPLE}Kernel{C_RESET}: {}", kernel_str),
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
