"""
Core ADB Manager for Ximi Ultimate Tool.
Handles ADB connectivity, device properties, debloater operations, and log streaming.
"""

import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple
from PyQt6.QtCore import QObject, QThread, pyqtSignal

@dataclass
class BloatwareItem:
    package: str
    name: str
    category: str
    risk: str  # "Safe", "Caution", "Optional"
    description: str

# Curated Xiaomi / HyperOS / MIUI bloatware database
BLOATWARE_DATABASE: List[BloatwareItem] = [
    # Analytics & Tracking
    BloatwareItem("com.miui.analytics", "Xiaomi Analytics", "Analytics", "Safe", "Telemetry, logs, and tracking system."),
    BloatwareItem("com.xiaomi.joyose", "Joyose", "Analytics", "Safe", "Game throttle, analytics, and telemetry service."),
    BloatwareItem("com.miui.daemon", "MIUI Daemon", "Analytics", "Safe", "Background diagnostic & monitoring service."),
    BloatwareItem("com.miui.msa.global", "MIUI System Ads (Global)", "Ads", "Safe", "Service responsible for push ads in MIUI apps."),
    BloatwareItem("com.miui.bugreport", "Mi Bug Report", "Analytics", "Safe", "Crash report and feedback uploader."),
    BloatwareItem("com.miui.miservice", "Services & Feedback", "Analytics", "Safe", "User feedback and log collection service."),
    BloatwareItem("com.xiaomi.xmsf", "Xiaomi Service Framework", "Analytics", "Caution", "Framework for Xiaomi cloud & push notifications."),
    BloatwareItem("com.xiaomi.finddevice", "Find Device", "Analytics", "Caution", "Xiaomi Cloud Find Device feature."),
    
    # Ads & Commercial Apps
    BloatwareItem("com.xiaomi.mipicks", "GetApps", "Ads", "Safe", "Xiaomi App Store with frequent promotional ads."),
    BloatwareItem("com.miui.hybrid", "Quick Apps", "Ads", "Safe", "Mini apps service running ads in background."),
    BloatwareItem("com.miui.hybrid.accessory", "Quick Apps Accessory", "Ads", "Safe", "Companion service for Quick Apps."),
    BloatwareItem("com.mipay.wallet.id", "Mi Pay ID", "Ads", "Safe", "Payment wallet service."),
    BloatwareItem("com.mipay.wallet.in", "Mi Pay IN", "Ads", "Safe", "Payment wallet service."),
    BloatwareItem("com.miui.micredit", "Mi Credit", "Ads", "Safe", "Loan and financial marketing app."),
    BloatwareItem("com.android.browser", "Mi Browser", "Ads", "Safe", "Xiaomi browser preloaded with promotional feed."),
    BloatwareItem("com.miui.videoplayer", "Mi Video", "Ads", "Safe", "Video player bundled with online trending feed."),
    BloatwareItem("com.miui.player", "Mi Music", "Ads", "Safe", "Music player bundled with promotional music feed."),
    BloatwareItem("com.miui.yellowpage", "Yellow Pages", "Ads", "Safe", "Caller ID and business directory lookup."),
    BloatwareItem("com.miui.android.fashiongallery", "Wallpaper Carousel", "Ads", "Safe", "Lock screen ads and news carousel."),
    BloatwareItem("com.mfashiongallery.emag", "Glance Carousel", "Ads", "Safe", "Glance lockscreen news and ads."),

    # Preloaded & Games
    BloatwareItem("com.xiaomi.glgm", "Xiaomi Game Center", "Games", "Safe", "Promotional games marketplace."),
    BloatwareItem("com.xiaomi.midrop", "ShareMe / Mi Drop", "System", "Optional", "File sharing tool."),
    BloatwareItem("com.miui.cleanmaster", "Clean Master Cleaner", "System", "Safe", "MIUI security cleaner database/engine."),
    BloatwareItem("com.miui.compass", "Compass", "System", "Safe", "Default compass app."),
    BloatwareItem("com.miui.notes", "Mi Notes", "System", "Optional", "Stock notes app."),
    BloatwareItem("com.miui.weather2", "Mi Weather", "System", "Optional", "Stock weather application."),
    BloatwareItem("com.miui.calculator", "Mi Calculator", "System", "Optional", "Stock calculator app."),
    BloatwareItem("com.miui.screenrecorder", "Screen Recorder", "System", "Optional", "Built-in screen recording utility."),
    BloatwareItem("com.miui.voiceassist", "Mi AI Voice", "System", "Safe", "Voice assistant (often China/Asian variants)."),
    
    # Facebook Services
    BloatwareItem("com.facebook.katana", "Facebook", "Facebook", "Safe", "Preinstalled Facebook main app."),
    BloatwareItem("com.facebook.system", "Facebook App Installer", "Facebook", "Safe", "Silent background APK installer for FB."),
    BloatwareItem("com.facebook.appmanager", "Facebook App Manager", "Facebook", "Safe", "Background updater for Facebook products."),
    BloatwareItem("com.facebook.services", "Facebook Services", "Facebook", "Safe", "Background sync and tracking service."),
    
    # Google Preinstalled
    BloatwareItem("com.google.android.apps.tachyon", "Google Meet / Duo", "Google", "Safe", "Video calling service."),
    BloatwareItem("com.google.android.videos", "Google TV / Play Movies", "Google", "Safe", "Movie rental and purchase store."),
    BloatwareItem("com.google.android.music", "Google Play Music", "Google", "Safe", "Legacy Google Music app."),
    BloatwareItem("com.google.android.apps.photos", "Google Photos", "Google", "Optional", "Cloud photo backup and gallery."),
    BloatwareItem("com.google.android.apps.docs", "Google Drive / Docs", "Google", "Optional", "Google cloud docs utility."),
    BloatwareItem("com.google.android.youtube", "YouTube", "Google", "Optional", "Stock YouTube video client."),
    BloatwareItem("com.google.android.apps.youtube.music", "YouTube Music", "Google", "Safe", "Streaming music service."),
    BloatwareItem("com.google.android.feedback", "Google Feedback", "Google", "Safe", "Google bug and feedback uploader.")
]


class LogStreamWorker(QThread):
    new_line = pyqtSignal(str)

    def __init__(self, command: List[str]):
        super().__init__()
        self.command = command
        self._is_running = True
        self.process: Optional[subprocess.Popen] = None

    def run(self):
        try:
            self.process = subprocess.Popen(
                self.command,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
            )
            while self._is_running and self.process and self.process.stdout:
                line = self.process.stdout.readline()
                if not line:
                    break
                self.new_line.emit(line.rstrip("\r\n"))
        except Exception as e:
            self.new_line.emit(f"Logcat error: {e}")
        finally:
            self.stop()

    def stop(self):
        self._is_running = False
        if self.process:
            try:
                self.process.terminate()
                self.process.kill()
            except Exception:
                pass
            self.process = None


class ADBManager:
    """Core manager executing ADB commands and querying phone state."""
    
    def __init__(self):
        self._adb_path = self._detect_adb()
        self.log_worker: Optional[LogStreamWorker] = None

    def _detect_adb(self) -> str:
        # Check system PATH
        path = shutil.which("adb")
        if path:
            return path
        # Check standard Windows paths
        if os.name == "nt":
            candidates = [
                r"C:\platform-tools\adb.exe",
                os.path.expandvars(r"%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe"),
                os.path.join(os.path.dirname(__file__), "..", "..", "bin", "adb.exe")
            ]
            for c in candidates:
                if os.path.exists(c):
                    return c
        return "adb"

    @property
    def adb_path(self) -> str:
        return self._adb_path

    def set_adb_path(self, path: str):
        if path and os.path.exists(path):
            self._adb_path = path

    def run_cmd(self, args: List[str], timeout: int = 15) -> Tuple[int, str, str]:
        """Run an ADB command synchronously."""
        cmd = [self._adb_path] + args
        try:
            res = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
            )
            return res.returncode, res.stdout, res.stderr
        except subprocess.TimeoutExpired:
            return -1, "", "Command timed out"
        except Exception as e:
            return -1, "", str(e)

    def get_connected_devices(self) -> List[Dict[str, str]]:
        """List attached devices with state (device, unauthorized, recovery, sideload)."""
        code, out, _ = self.run_cmd(["devices", "-l"])
        devices = []
        if code == 0:
            lines = out.strip().splitlines()
            for line in lines[1:]:  # skip 'List of devices attached'
                line = line.strip()
                if not line:
                    continue
                parts = line.split()
                if len(parts) >= 2:
                    serial = parts[0]
                    state = parts[1]
                    info = " ".join(parts[2:]) if len(parts) > 2 else ""
                    devices.append({
                        "serial": serial,
                        "state": state,
                        "info": info
                    })
        return devices

    def get_prop(self, prop_name: str, serial: Optional[str] = None) -> str:
        args = []
        if serial:
            args.extend(["-s", serial])
        args.extend(["shell", "getprop", prop_name])
        code, out, _ = self.run_cmd(args, timeout=5)
        return out.strip() if code == 0 else ""

    def get_device_full_specs(self, serial: Optional[str] = None) -> Dict[str, str]:
        """Fetch specs including ro.mi.os.version.incremental, model, CPU, RAM, storage."""
        specs = {}
        
        # HyperOS & Android Versions
        hyperos_incremental = self.get_prop("ro.mi.os.version.incremental", serial)
        hyperos_name = self.get_prop("ro.miui.ui.version.name", serial)
        android_ver = self.get_prop("ro.build.version.release", serial)
        security_patch = self.get_prop("ro.build.version.security_patch", serial)
        
        # Device & Model
        market_name = self.get_prop("ro.product.marketname", serial)
        model = self.get_prop("ro.product.model", serial)
        device = self.get_prop("ro.product.device", serial)
        brand = self.get_prop("ro.product.brand", serial)
        
        specs["market_name"] = market_name or model or device or "Xiaomi Device"
        specs["model"] = model or device or ""
        specs["device"] = device or ""
        specs["brand"] = brand or "Xiaomi"
        specs["android_ver"] = android_ver or "14"
        specs["security_patch"] = security_patch or "-"
        
        # HyperOS version display logic
        if hyperos_incremental:
            specs["hyperos_version"] = hyperos_incremental
            # Try parsing leading numeric version (e.g. 3.0.303.0 from 3.0.303.0.WPJCNXM.C06)
            match = re.match(r"^(\d+\.\d+\.\d+\.\d+|\d+\.\d+\.\d+)", hyperos_incremental)
            specs["hyperos_short"] = match.group(1) if match else hyperos_incremental
        else:
            specs["hyperos_version"] = hyperos_name or "HyperOS 1.0"
            specs["hyperos_short"] = "1.0.0.0"

        # Processor / CPU
        soc = self.get_prop("ro.soc.model", serial) or self.get_prop("ro.board.platform", serial) or self.get_prop("ro.hardware", serial)
        if not soc:
            # Query /proc/cpuinfo
            code, out, _ = self.run_cmd(["shell", "cat /proc/cpuinfo | grep 'Hardware' | head -n 1"], timeout=5)
            if code == 0 and "Hardware" in out:
                soc = out.split(":")[-1].strip()
        specs["cpu"] = soc or "Octa-core Processor"

        # RAM Memory
        code, mem_out, _ = self.run_cmd(["shell", "cat", "/proc/meminfo"], timeout=5)
        ram_str = "8.0GB"
        if code == 0:
            total_kb = 0
            for line in mem_out.splitlines():
                if "MemTotal:" in line:
                    parts = line.split()
                    if len(parts) >= 2:
                        total_kb = int(parts[1])
                        gb = round(total_kb / (1024 * 1024), 1)
                        # Normalize to standard values: 4.0GB, 6.0GB, 8.0GB, 12.0GB, 16.0GB
                        if gb <= 4.3: ram_str = "4.0GB"
                        elif gb <= 6.3: ram_str = "6.0GB"
                        elif gb <= 8.5: ram_str = "8.0GB"
                        elif gb <= 12.5: ram_str = "12.0GB"
                        elif gb <= 16.5: ram_str = "16.0GB"
                        else: ram_str = f"{gb}GB"
                        break
        specs["ram"] = ram_str

        # Storage (df -h /data)
        code, df_out, _ = self.run_cmd(["shell", "df -h /data"], timeout=5)
        storage_str = "72.5GB/256GB"
        if code == 0:
            lines = df_out.strip().splitlines()
            if len(lines) >= 2:
                parts = lines[1].split()
                # Filesystem Size Used Available Use% Mounted on
                if len(parts) >= 4:
                    size = parts[1]
                    used = parts[2]
                    storage_str = f"{used}/{size}"
        specs["storage"] = storage_str

        # Battery capacity & status
        code, batt_out, _ = self.run_cmd(["shell", "dumpsys battery"], timeout=5)
        battery_str = "5000mAh(typ)"
        if code == 0:
            level = "100"
            for line in batt_out.splitlines():
                if "level:" in line:
                    level = line.split(":")[-1].strip()
            # If known model, provide typical mAh
            battery_str = f"5000mAh ({level}%)"
            if "14" in specs["market_name"]:
                battery_str = f"5500mAh(typ) ({level}%)"
        specs["battery"] = battery_str

        return specs

    def get_installed_packages(self, filter_mode: str = "all", serial: Optional[str] = None) -> List[str]:
        """Fetch list of package names based on filter (all, 3rd, system, disabled)."""
        args = ["shell", "pm", "list", "packages"]
        if filter_mode == "3rd":
            args.append("-3")
        elif filter_mode == "system":
            args.append("-s")
        elif filter_mode == "disabled":
            args.append("-d")
        
        if serial:
            args = ["-s", serial] + args

        code, out, _ = self.run_cmd(args, timeout=10)
        packages = []
        if code == 0:
            for line in out.strip().splitlines():
                line = line.strip()
                if line.startswith("package:"):
                    packages.append(line.replace("package:", "").strip())
        return sorted(packages)

    def uninstall_package(self, package: str, serial: Optional[str] = None) -> Tuple[bool, str]:
        """Uninstall package for user 0 (standard debloat)."""
        args = []
        if serial:
            args.extend(["-s", serial])
        args.extend(["shell", "pm", "uninstall", "-k", "--user", "0", package])
        code, out, err = self.run_cmd(args)
        success = (code == 0 and "Success" in out)
        msg = out.strip() if out.strip() else err.strip()
        return success, msg

    def restore_package(self, package: str, serial: Optional[str] = None) -> Tuple[bool, str]:
        """Reinstall an uninstalled existing system package for user 0."""
        args = []
        if serial:
            args.extend(["-s", serial])
        args.extend(["shell", "cmd", "package", "install-existing", package])
        code, out, err = self.run_cmd(args)
        success = (code == 0 and ("installed" in out.lower() or "success" in out.lower()))
        msg = out.strip() if out.strip() else err.strip()
        return success, msg

    def disable_package(self, package: str, serial: Optional[str] = None) -> Tuple[bool, str]:
        args = []
        if serial:
            args.extend(["-s", serial])
        args.extend(["shell", "pm", "disable-user", "--user", "0", package])
        code, out, err = self.run_cmd(args)
        success = (code == 0 and "new state" in out)
        msg = out.strip() if out.strip() else err.strip()
        return success, msg

    def enable_package(self, package: str, serial: Optional[str] = None) -> Tuple[bool, str]:
        args = []
        if serial:
            args.extend(["-s", serial])
        args.extend(["shell", "pm", "enable", package])
        code, out, err = self.run_cmd(args)
        success = (code == 0 and "new state" in out)
        msg = out.strip() if out.strip() else err.strip()
        return success, msg

    def reboot(self, mode: str = "system", serial: Optional[str] = None) -> Tuple[bool, str]:
        """Reboot device to system, recovery, bootloader, or edl."""
        args = []
        if serial:
            args.extend(["-s", serial])
        if mode == "recovery":
            args.extend(["reboot", "recovery"])
        elif mode == "bootloader":
            args.extend(["reboot", "bootloader"])
        elif mode == "edl":
            args.extend(["reboot", "edl"])
        else:
            args.extend(["reboot"])
        code, out, err = self.run_cmd(args)
        return code == 0, out or err

    def start_log_stream(self, log_type: str = "logcat", serial: Optional[str] = None, filter_str: str = "") -> LogStreamWorker:
        """Start a background streaming thread for logcat or dmesg."""
        if self.log_worker:
            self.log_worker.stop()
            self.log_worker.wait(1000)

        cmd = [self._adb_path]
        if serial:
            cmd.extend(["-s", serial])
        
        if log_type == "dmesg":
            cmd.extend(["shell", "dmesg", "-w"])
        else:
            cmd.extend(["logcat", "-v", "time"])
            if filter_str:
                cmd.append(filter_str)

        self.log_worker = LogStreamWorker(cmd)
        self.log_worker.start()
        return self.log_worker

    def stop_log_stream(self):
        if self.log_worker:
            self.log_worker.stop()
            self.log_worker = None
