"""
Fastfetch implementation for Android Shell with Xiaomi HyperOS logo and device specs.
"""

from typing import Optional
from app.core.adb_manager import ADBManager

# ANSI Color codes
C_RESET = "\033[0m"
C_BOLD = "\033[1m"
C_BLUE = "\033[38;2;66;133;244m"
C_PURPLE = "\033[38;2;168;85;247m"
C_CYAN = "\033[38;2;6;182;212m"
C_GREEN = "\033[38;2;34;197;94m"
C_YELLOW = "\033[38;2;234;179;8m"
C_RED = "\033[38;2;239;68;68m"
C_WHITE = "\033[38;2;248;250;252m"
C_GRAY = "\033[38;2;148;163;184m"

# HyperOS stylized ASCII logo (interlocking rounded squares / star)
HYPEROS_LOGO = [
    f"{C_PURPLE}       .---.       {C_RESET}",
    f"{C_PURPLE}    .-'     '-.    {C_RESET}",
    f"{C_PURPLE}  .'   {C_BLUE}.-.   {C_PURPLE}'.  {C_RESET}",
    f"{C_BLUE} /    /   \\    \\ {C_RESET}",
    f"{C_BLUE}|    |  {C_WHITE}●{C_BLUE}  |    |{C_RESET}",
    f"{C_CYAN} \\    \\   /    / {C_RESET}",
    f"{C_CYAN}  '.   {C_CYAN}'-'   {C_CYAN}.'  {C_RESET}",
    f"{C_CYAN}    '-.     .-'    {C_RESET}",
    f"{C_CYAN}       '---'       {C_RESET}",
    f"                   ",
    f"                   ",
]

class FastfetchEngine:
    @staticmethod
    def generate(adb: ADBManager, serial: Optional[str] = None) -> str:
        specs = adb.get_device_full_specs(serial)
        
        # Kernel uname
        code, uname, _ = adb.run_cmd(["shell", "uname", "-r"], timeout=5)
        kernel_str = uname.strip() if code == 0 else "Linux 5.15.x"

        # Uptime
        code, uptime_out, _ = adb.run_cmd(["shell", "cat", "/proc/uptime"], timeout=5)
        uptime_str = "1 day, 4 hours"
        if code == 0 and uptime_out:
            try:
                sec = float(uptime_out.split()[0])
                days = int(sec // 86400)
                hours = int((sec % 86400) // 3600)
                mins = int((sec % 3600) // 60)
                parts = []
                if days: parts.append(f"{days} days")
                if hours: parts.append(f"{hours} hours")
                parts.append(f"{mins} mins")
                uptime_str = ", ".join(parts)
            except Exception:
                pass

        # Screen resolution
        code, wm_size, _ = adb.run_cmd(["shell", "wm", "size"], timeout=5)
        res_str = wm_size.split(":")[-1].strip() if code == 0 and ":" in wm_size else "1080x2400"

        # Build Title
        title_user = "root" if adb.run_cmd(["shell", "su", "-c", "whoami"], timeout=3)[0] == 0 else "shell"
        device_host = specs.get("device", "hyperos")
        title_line = f"{C_BOLD}{C_PURPLE}{title_user}{C_RESET}@{C_BOLD}{C_BLUE}{device_host}{C_RESET}"
        separator = f"{C_GRAY}" + "─" * len(f"{title_user}@{device_host}") + f"{C_RESET}"

        info_rows = [
            title_line,
            separator,
            f"{C_BOLD}{C_PURPLE}OS{C_RESET}: Xiaomi HyperOS {specs.get('hyperos_version', '1.0')}",
            f"{C_BOLD}{C_PURPLE}Host{C_RESET}: {specs.get('market_name', 'Xiaomi')} ({specs.get('model', '')})",
            f"{C_BOLD}{C_PURPLE}Kernel{C_RESET}: {kernel_str}",
            f"{C_BOLD}{C_PURPLE}Android{C_RESET}: {specs.get('android_ver', '14')}",
            f"{C_BOLD}{C_PURPLE}Uptime{C_RESET}: {uptime_str}",
            f"{C_BOLD}{C_PURPLE}Display{C_RESET}: {res_str}",
            f"{C_BOLD}{C_PURPLE}CPU{C_RESET}: {specs.get('cpu', 'Octa-core')}",
            f"{C_BOLD}{C_PURPLE}Memory{C_RESET}: {specs.get('ram', '8.0GB')}",
            f"{C_BOLD}{C_PURPLE}Storage{C_RESET}: {specs.get('storage', '72.5GB/256GB')}",
            f"{C_BOLD}{C_PURPLE}Battery{C_RESET}: {specs.get('battery', '5000mAh')}",
            f"{C_BOLD}{C_PURPLE}Security{C_RESET}: {specs.get('security_patch', '-')}",
            "",
            # Color palette dots
            f"{C_RED}● {C_YELLOW}● {C_GREEN}● {C_CYAN}● {C_BLUE}● {C_PURPLE}● {C_WHITE}● {C_GRAY}●{C_RESET}"
        ]

        # Combine Logo on left, Info on right
        lines = []
        max_rows = max(len(HYPEROS_LOGO), len(info_rows))
        for i in range(max_rows):
            logo_part = HYPEROS_LOGO[i] if i < len(HYPEROS_LOGO) else " " * 19
            info_part = info_rows[i] if i < len(info_rows) else ""
            lines.append(f"  {logo_part}   {info_part}")

        return "\n" + "\n".join(lines) + "\n"
