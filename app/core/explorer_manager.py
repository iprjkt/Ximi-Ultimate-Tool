"""
Root and Standard Android Explorer Manager for Ximi Ultimate Tool.
Handles file browsing, directory navigation, file creation, editing, moving,
copying, zip extraction, and pushing/pulling files between PC and Android.
"""

import os
import re
import shlex
import tempfile
from dataclasses import dataclass
from typing import List, Optional, Tuple
from app.core.adb_manager import ADBManager

@dataclass
class FileItem:
    name: str
    path: str
    is_dir: bool
    is_link: bool
    link_target: str
    size: int
    size_formatted: str
    permissions: str
    owner: str
    group: str
    date: str

class ExplorerManager:
    def __init__(self, adb_manager: Optional[ADBManager] = None):
        self.adb = adb_manager or ADBManager()

    def check_root_access(self, serial: Optional[str] = None) -> bool:
        """Check if root (su) is available and granted on the device."""
        args = ["shell", "su", "-c", "id"]
        if serial:
            args = ["-s", serial] + args
        code, out, _ = self.adb.run_cmd(args, timeout=5)
        return (code == 0 and "uid=0(root)" in out)

    def _exec_shell(self, cmd: str, root_mode: bool, serial: Optional[str] = None, timeout: int = 15) -> Tuple[int, str, str]:
        if root_mode:
            # Escape double quotes inside cmd
            escaped_cmd = cmd.replace('"', '\\"')
            args = ["shell", f'su -c "{escaped_cmd}"']
        else:
            args = ["shell", cmd]

        if serial:
            args = ["-s", serial] + args
        return self.adb.run_cmd(args, timeout=timeout)

    @staticmethod
    def format_size(bytes_val: int) -> str:
        for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
            if bytes_val < 1024.0:
                return f"{bytes_val:.1f} {unit}" if unit != 'B' else f"{bytes_val} B"
            bytes_val /= 1024.0
        return f"{bytes_val:.1f} PB"

    def list_dir(self, dir_path: str, root_mode: bool = True, serial: Optional[str] = None) -> List[FileItem]:
        """
        Lists directory contents using toybox / ls parser.
        """
        if not dir_path.endswith("/"):
            dir_path += "/"

        # Normalize directory path
        clean_path = dir_path.rstrip("/")
        if not clean_path:
            clean_path = "/"

        # Command with detailed listing
        cmd = f"ls -la '{clean_path}'"
        code, out, err = self._exec_shell(cmd, root_mode, serial)
        
        items: List[FileItem] = []
        if code != 0 and not out:
            # Try non-root fallback if root command failed
            if root_mode:
                return self.list_dir(dir_path, root_mode=False, serial=serial)
            return items

        lines = out.strip().splitlines()
        for line in lines:
            line = line.strip()
            if not line or line.startswith("total "):
                continue

            # Standard toybox / android ls -la regex:
            # drwxr-xr-x 2 root root 4096 2024-01-01 12:00 name -> target
            parts = line.split(maxsplit=7)
            if len(parts) < 7:
                # Alternate format with no group or different columns
                parts = line.split(maxsplit=6)
                if len(parts) < 6:
                    continue

            perms = parts[0]
            is_dir = perms.startswith("d")
            is_link = perms.startswith("l")

            # Try to identify size and name
            try:
                # Check for toybox format: perms, links, owner, group, size, date, time, name
                if len(parts) >= 8:
                    owner = parts[2]
                    group = parts[3]
                    size_val = int(parts[4]) if parts[4].isdigit() else 0
                    date_val = f"{parts[5]} {parts[6]}"
                    raw_name = parts[7]
                else:
                    owner = parts[1]
                    group = parts[2]
                    size_val = int(parts[3]) if parts[3].isdigit() else 0
                    date_val = parts[4]
                    raw_name = parts[5]
            except Exception:
                owner = "-"
                group = "-"
                size_val = 0
                date_val = "-"
                raw_name = parts[-1]

            # Handle symlinks: "link_name -> target"
            link_target = ""
            name = raw_name
            if " -> " in raw_name:
                name_parts = raw_name.split(" -> ")
                name = name_parts[0]
                link_target = name_parts[1]

            if name in [".", ".."]:
                continue

            full_path = f"{clean_path}/{name}".replace("//", "/")
            items.append(FileItem(
                name=name,
                path=full_path,
                is_dir=is_dir,
                is_link=is_link,
                link_target=link_target,
                size=size_val,
                size_formatted=self.format_size(size_val) if not is_dir else "--",
                permissions=perms,
                owner=owner,
                group=group,
                date=date_val
            ))

        # Sort: directories first, then alphabetically
        items.sort(key=lambda x: (not x.is_dir, x.name.lower()))
        return items

    def create_folder(self, path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"mkdir -p '{path}'", root_mode, serial)
        return code == 0, out or err

    def create_file(self, path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"touch '{path}'", root_mode, serial)
        return code == 0, out or err

    def rename_item(self, old_path: str, new_path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"mv '{old_path}' '{new_path}'", root_mode, serial)
        return code == 0, out or err

    def copy_item(self, src: str, dst: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"cp -r '{src}' '{dst}'", root_mode, serial)
        return code == 0, out or err

    def move_item(self, src: str, dst: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"mv '{src}' '{dst}'", root_mode, serial)
        return code == 0, out or err

    def delete_item(self, path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"rm -rf '{path}'", root_mode, serial)
        return code == 0, out or err

    def chmod(self, path: str, perms: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        code, out, err = self._exec_shell(f"chmod {perms} '{path}'", root_mode, serial)
        return code == 0, out or err

    def extract_zip(self, zip_path: str, dst_folder: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        # Try native unzip on device
        code, out, err = self._exec_shell(f"unzip -o '{zip_path}' -d '{dst_folder}'", root_mode, serial, timeout=60)
        if code == 0:
            return True, "Extracted successfully"
        # Try toybox unzip
        code2, out2, err2 = self._exec_shell(f"toybox unzip -o '{zip_path}' -d '{dst_folder}'", root_mode, serial, timeout=60)
        if code2 == 0:
            return True, "Extracted successfully"
        return False, err or err2 or "Device unzip utility failed"

    def push_file(self, local_path: str, remote_path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        """
        Push file from PC to Android. If root_mode, pushes to /data/local/tmp/
        first, then moves with root permissions to handle restricted paths.
        """
        file_name = os.path.basename(local_path)
        ser_args = ["-s", serial] if serial else []

        if not root_mode or remote_path.startswith("/sdcard") or remote_path.startswith("/storage"):
            # Direct push
            code, out, err = self.adb.run_cmd(ser_args + ["push", local_path, remote_path], timeout=60)
            return code == 0, out or err

        # Root push via /data/local/tmp/
        tmp_remote = f"/data/local/tmp/{file_name}"
        code, out, err = self.adb.run_cmd(ser_args + ["push", local_path, tmp_remote], timeout=60)
        if code != 0:
            return False, err or "Failed to push to temporary directory"

        # Move to target destination with root
        code2, out2, err2 = self._exec_shell(f"mv '{tmp_remote}' '{remote_path}'", root_mode=True, serial=serial)
        if code2 == 0:
            # Grant standard read perms
            self._exec_shell(f"chmod 644 '{remote_path}'", root_mode=True, serial=serial)
            return True, "File pushed successfully with root access"
        return False, err2 or "Failed to move file to final destination"

    def pull_file(self, remote_path: str, local_path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        """
        Pull file from Android to PC. If root_mode on restricted path,
        copies to /data/local/tmp/ first.
        """
        ser_args = ["-s", serial] if serial else []

        # Try direct pull first
        code, out, err = self.adb.run_cmd(ser_args + ["pull", remote_path, local_path], timeout=60)
        if code == 0:
            return True, out.strip() or "File pulled successfully"

        if root_mode:
            # Copy to temp directory with open permissions
            file_name = os.path.basename(remote_path.rstrip("/"))
            tmp_remote = f"/data/local/tmp/_ximi_pull_{file_name}"
            self._exec_shell(f"cp -r '{remote_path}' '{tmp_remote}' && chmod -R 777 '{tmp_remote}'", root_mode=True, serial=serial)
            code2, out2, err2 = self.adb.run_cmd(ser_args + ["pull", tmp_remote, local_path], timeout=60)
            # Cleanup temp
            self._exec_shell(f"rm -rf '{tmp_remote}'", root_mode=True, serial=serial)
            if code2 == 0:
                return True, "File pulled successfully via root"

        return False, err or "Failed to pull file"

    def read_text_file(self, path: str, root_mode: bool = True, serial: Optional[str] = None) -> str:
        code, out, _ = self._exec_shell(f"cat '{path}'", root_mode, serial, timeout=10)
        return out if code == 0 else ""

    def write_text_file(self, path: str, content: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as tf:
            tf.write(content)
            tf_path = tf.name

        try:
            ok, msg = self.push_file(tf_path, path, root_mode, serial)
            return ok, msg
        finally:
            if os.path.exists(tf_path):
                os.remove(tf_path)
