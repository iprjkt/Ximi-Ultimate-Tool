"""
Fastboot Manager for Ximi Ultimate Tool.
Handles fastboot devices, single partition flashing, full ROM flashing (Mi Flash alternative),
advance mode partition unchecking, and real-time live terminal output.
"""

import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from PyQt6.QtCore import QObject, QThread, pyqtSignal

@dataclass
class FlashCommandItem:
    index: int
    raw_command: str
    partition: str
    image_file: str
    is_dangerous: bool
    enabled: bool = True

class FastbootExecutionWorker(QThread):
    log_line = pyqtSignal(str)
    progress_changed = pyqtSignal(int, int)  # current, total
    finished_status = pyqtSignal(bool, str)

    def __init__(self, commands: List[str], cwd: Optional[str] = None):
        super().__init__()
        self.commands = commands
        self.cwd = cwd
        self._is_aborted = False
        self.process: Optional[subprocess.Popen] = None

    def run(self):
        total = len(self.commands)
        self.log_line.emit(f"[INFO] Starting Fastboot execution sequence ({total} commands)...")
        
        for idx, cmd_str in enumerate(self.commands):
            if self._is_aborted:
                self.log_line.emit("[ABORT] Flashing process aborted by user.")
                self.finished_status.emit(False, "Aborted by user")
                return

            self.progress_changed.emit(idx + 1, total)
            self.log_line.emit(f"\n[EXEC] ({idx + 1}/{total}) > {cmd_str}")
            
            try:
                # Run command
                shell_mode = True if os.name == "nt" else False
                args = cmd_str if shell_mode else cmd_str.split()

                self.process = subprocess.Popen(
                    args,
                    cwd=self.cwd,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    bufsize=1,
                    shell=shell_mode,
                    creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
                )

                while self.process and self.process.stdout:
                    line = self.process.stdout.readline()
                    if not line:
                        break
                    self.log_line.emit(line.rstrip("\r\n"))

                self.process.wait()
                if self.process.returncode != 0:
                    err_msg = f"[FAILED] Command exited with code {self.process.returncode}"
                    self.log_line.emit(err_msg)
                    # Non-zero on flash command might mean critical error
                    if "flash" in cmd_str:
                        self.log_line.emit("[ERROR] Critical partition flash failed! Halting.")
                        self.finished_status.emit(False, err_msg)
                        return

            except Exception as e:
                self.log_line.emit(f"[EXCEPTION] {e}")
                self.finished_status.emit(False, str(e))
                return

        self.log_line.emit("\n==========================================")
        self.log_line.emit("[SUCCESS] All flashing tasks finished successfully!")
        self.log_line.emit("==========================================")
        self.finished_status.emit(True, "Flashing complete")

    def abort(self):
        self._is_aborted = True
        if self.process:
            try:
                self.process.terminate()
                self.process.kill()
            except Exception:
                pass
            self.process = None


class FastbootManager:
    """Manages fastboot operations, script parsing, and flashing."""

    COMMON_PARTITIONS = [
        "boot",
        "init_boot",
        "vendor_boot",
        "recovery",
        "vbmeta",
        "vbmeta_system",
        "vbmeta_vendor",
        "dtbo",
        "cust",
        "super",
        "system",
        "vendor",
        "product",
        "modem",
        "userdata"
    ]

    DANGEROUS_PARTITIONS = {
        "preloader", "preloader_a", "preloader_b",
        "persist", "devinfo", "misc", "nvram", "nvdata",
        "sec1", "proinfo", "protect1", "protect2"
    }

    def __init__(self):
        self._fastboot_path = self._detect_fastboot()
        self.worker: Optional[FastbootExecutionWorker] = None

    def _detect_fastboot(self) -> str:
        path = shutil.which("fastboot")
        if path:
            return path
        if os.name == "nt":
            candidates = [
                r"C:\platform-tools\fastboot.exe",
                os.path.expandvars(r"%LOCALAPPDATA%\Android\Sdk\platform-tools\fastboot.exe"),
                os.path.join(os.path.dirname(__file__), "..", "..", "bin", "fastboot.exe")
            ]
            for c in candidates:
                if os.path.exists(c):
                    return c
        return "fastboot"

    @property
    def fastboot_path(self) -> str:
        return self._fastboot_path

    def set_fastboot_path(self, path: str):
        if path and os.path.exists(path):
            self._fastboot_path = path

    def run_cmd(self, args: List[str], timeout: int = 15) -> Tuple[int, str, str]:
        cmd = [self._fastboot_path] + args
        try:
            res = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
            )
            # Fastboot often outputs status to stderr! Combine if needed
            return res.returncode, res.stdout, res.stderr
        except subprocess.TimeoutExpired:
            return -1, "", "Command timed out"
        except Exception as e:
            return -1, "", str(e)

    def get_connected_devices(self) -> List[Dict[str, str]]:
        code, out, err = self.run_cmd(["devices"])
        devices = []
        raw = out if out.strip() else err
        if raw:
            for line in raw.strip().splitlines():
                parts = line.strip().split()
                if len(parts) >= 2 and parts[1].lower() == "fastboot":
                    devices.append({
                        "serial": parts[0],
                        "state": "fastboot",
                        "info": "Fastboot Mode"
                    })
        return devices

    def get_var(self, var_name: str, serial: Optional[str] = None) -> str:
        args = []
        if serial:
            args.extend(["-s", serial])
        args.extend(["getvar", var_name])
        code, out, err = self.run_cmd(args)
        raw = out + "\n" + err
        for line in raw.splitlines():
            if var_name in line:
                parts = line.split(":")
                if len(parts) >= 2:
                    return parts[1].strip()
        return ""

    def flash_single_partition(
        self,
        partition: str,
        image_path: str,
        disable_vbmeta_verity: bool = False,
        serial: Optional[str] = None
    ) -> List[str]:
        """Generate command list for single partition flash."""
        cmds = []
        fb = self._fastboot_path
        ser_arg = f"-s {serial} " if serial else ""

        if disable_vbmeta_verity and "vbmeta" in partition:
            cmds.append(f"{fb} {ser_arg}flash --disable-verity --disable-verification {partition} \"{image_path}\"")
        else:
            cmds.append(f"{fb} {ser_arg}flash {partition} \"{image_path}\"")
        return cmds

    def parse_rom_script(self, script_path: str) -> List[FlashCommandItem]:
        """
        Parses a flash_all.sh / flash_all.bat or rom directory.
        Identifies all fastboot commands, partition targets, and images.
        """
        items: List[FlashCommandItem] = []
        if not os.path.exists(script_path):
            return items

        try:
            with open(script_path, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()

            cmd_idx = 0
            for line in lines:
                clean = line.strip()
                # Skip comments or empty lines in initial parse
                if not clean or clean.startswith("#") or clean.lower().startswith("rem"):
                    continue

                if "fastboot" in clean and "flash" in clean:
                    # Match fastboot flash <partition> <image>
                    parts = clean.split()
                    try:
                        flash_idx = parts.index("flash")
                        part = parts[flash_idx + 1]
                        img = parts[flash_idx + 2] if len(parts) > flash_idx + 2 else ""
                        is_dangerous = part.lower() in self.DANGEROUS_PARTITIONS
                        
                        items.append(FlashCommandItem(
                            index=cmd_idx,
                            raw_command=clean,
                            partition=part,
                            image_file=img,
                            is_dangerous=is_dangerous,
                            enabled=not is_dangerous  # Auto-uncheck dangerous by default in advance mode!
                        ))
                        cmd_idx += 1
                    except (ValueError, IndexError):
                        items.append(FlashCommandItem(
                            index=cmd_idx,
                            raw_command=clean,
                            partition="other",
                            image_file="",
                            is_dangerous=False,
                            enabled=True
                        ))
                        cmd_idx += 1
                elif "fastboot" in clean:
                    # Other fastboot commands like erase, reboot, format
                    items.append(FlashCommandItem(
                        index=cmd_idx,
                        raw_command=clean,
                        partition="cmd",
                        image_file="",
                        is_dangerous=False,
                        enabled=True
                    ))
                    cmd_idx += 1

        except Exception as e:
            print(f"Error parsing script {script_path}: {e}")

        return items

    def generate_flasher_commands(
        self,
        rom_dir: str,
        mode: str,  # "clean_all", "clean_keep_data", "clean_lock"
        advance_items: Optional[List[FlashCommandItem]] = None,
        serial: Optional[str] = None
    ) -> List[str]:
        """
        Builds the complete command sequence based on selected mode
        and advance checklist. Unchecked partitions are commented/skipped!
        """
        fb = self._fastboot_path
        ser = f"-s {serial} " if serial else ""
        commands: List[str] = []

        if advance_items:
            # Advance mode: use parsed list with enabled filter
            for item in advance_items:
                if item.enabled:
                    # Replace binary name with detected fastboot if needed
                    cmd = item.raw_command
                    if cmd.startswith("fastboot "):
                        cmd = f"{fb} {ser}" + cmd[len("fastboot "):]
                    commands.append(cmd)
                else:
                    # Unchecked partition: skip!
                    pass

            # Handle lock flag based on mode
            if mode == "clean_lock":
                commands.append(f"{fb} {ser}oem lock")
            return commands

        # Script-based / Standard ROM folder detection
        is_windows = (os.name == "nt")
        script_name = "flash_all.bat" if is_windows else "flash_all.sh"
        if mode == "clean_keep_data":
            script_name = "flash_all_except_data_storage.bat" if is_windows else "flash_all_except_data_storage.sh"
        elif mode == "clean_lock":
            script_name = "flash_all_lock.bat" if is_windows else "flash_all_lock.sh"

        target_script = os.path.join(rom_dir, script_name)
        if os.path.exists(target_script):
            # Parse the specific script
            parsed = self.parse_rom_script(target_script)
            for item in parsed:
                cmd = item.raw_command
                if cmd.startswith("fastboot "):
                    cmd = f"{fb} {ser}" + cmd[len("fastboot "):]
                commands.append(cmd)
        else:
            # Fallback: scan images directory and generate flash commands
            images_dir = os.path.join(rom_dir, "images")
            search_dir = images_dir if os.path.isdir(images_dir) else rom_dir

            # Flash boot, recovery, vbmeta first
            for part in ["vbmeta", "vbmeta_system", "boot", "vendor_boot", "recovery", "dtbo", "super", "cust"]:
                img = os.path.join(search_dir, f"{part}.img")
                if os.path.exists(img):
                    commands.append(f"{fb} {ser}flash {part} \"{img}\"")

            if mode in ["clean_all", "clean_lock"]:
                commands.append(f"{fb} {ser}erase userdata")
                commands.append(f"{fb} {ser}format:ext4 userdata")

            if mode == "clean_lock":
                commands.append(f"{fb} {ser}oem lock")

            commands.append(f"{fb} {ser}reboot")

        return commands
