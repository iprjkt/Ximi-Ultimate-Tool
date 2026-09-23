"""
Root and Standard Android Explorer Manager for Ximi Ultimate Tool.
Handles file browsing, directory navigation, file creation, editing, moving,
copying, zip extraction, and pushing/pulling files between PC and Android.
Supports MT Manager style dual-panel (Local PC panel & Remote Android panel).
"""

import os
import re
import shlex
import shutil
import tempfile
import time
import zipfile
from dataclasses import dataclass
from pathlib import Path
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

    # ==========================================
    # LOCAL PC FILE SYSTEM OPERATIONS
    # ==========================================
    def list_local_dir(self, dir_path: str) -> List[FileItem]:
        """Lists files and folders on the local PC."""
        items: List[FileItem] = []
        p = Path(dir_path).expanduser().resolve()
        if not p.exists() or not p.is_dir():
            return items

        try:
            with os.scandir(str(p)) as entries:
                for entry in entries:
                    try:
                        stat = entry.stat(follow_symlinks=False)
                        is_dir = entry.is_dir(follow_symlinks=False)
                        is_link = entry.is_symlink()
                        link_target = os.readlink(entry.path) if is_link else ""
                        size_val = stat.st_size if not is_dir else 0
                        date_str = time.strftime("%Y-%m-%d %H:%M", time.localtime(stat.st_mtime))
                        
                        # Permissions string (e.g. rw-r--r--)
                        mode = stat.st_mode
                        perms = ("d" if is_dir else ("l" if is_link else "-")) + \
                                ("r" if mode & 0o400 else "-") + \
                                ("w" if mode & 0o200 else "-") + \
                                ("x" if mode & 0o100 else "-") + \
                                ("r" if mode & 0o040 else "-") + \
                                ("w" if mode & 0o020 else "-") + \
                                ("x" if mode & 0o010 else "-") + \
                                ("r" if mode & 0o004 else "-") + \
                                ("w" if mode & 0o002 else "-") + \
                                ("x" if mode & 0o001 else "-")

                        items.append(FileItem(
                            name=entry.name,
                            path=str(Path(entry.path).resolve()),
                            is_dir=is_dir,
                            is_link=is_link,
                            link_target=link_target,
                            size=size_val,
                            size_formatted=self.format_size(size_val) if not is_dir else "--",
                            permissions=perms,
                            owner=str(stat.st_uid),
                            group=str(stat.st_gid),
                            date=date_str
                        ))
                    except (PermissionError, FileNotFoundError):
                        continue
        except PermissionError:
            pass

        # Sort: directories first, then alphabetical
        items.sort(key=lambda x: (not x.is_dir, x.name.lower()))
        return items

    def create_local_folder(self, path: str) -> Tuple[bool, str]:
        try:
            os.makedirs(path, exist_ok=True)
            return True, "Folder created"
        except Exception as e:
            return False, str(e)

    def create_local_file(self, path: str) -> Tuple[bool, str]:
        try:
            with open(path, "w", encoding="utf-8") as f:
                f.write("")
            return True, "File created"
        except Exception as e:
            return False, str(e)

    def rename_local_item(self, old_path: str, new_path: str) -> Tuple[bool, str]:
        try:
            os.rename(old_path, new_path)
            return True, "Renamed successfully"
        except Exception as e:
            return False, str(e)

    def delete_local_item(self, path: str) -> Tuple[bool, str]:
        try:
            if os.path.isdir(path) and not os.path.islink(path):
                shutil.rmtree(path)
            else:
                os.remove(path)
            return True, "Deleted successfully"
        except Exception as e:
            return False, str(e)

    def extract_local_zip(self, zip_path: str, dst_folder: str) -> Tuple[bool, str]:
        try:
            with zipfile.ZipFile(zip_path, 'r') as z:
                z.extractall(dst_folder)
            return True, "Extracted successfully"
        except Exception as e:
            return False, str(e)

    def read_local_text_file(self, path: str) -> str:
        try:
            with open(path, "r", encoding="utf-8", errors="ignore") as f:
                return f.read()
        except Exception:
            return ""

    def write_local_text_file(self, path: str, content: str) -> Tuple[bool, str]:
        try:
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
            return True, "Saved successfully"
        except Exception as e:
            return False, str(e)

    # ==========================================
    # ANDROID REMOTE FILE SYSTEM OPERATIONS
    # ==========================================
    def list_dir(self, dir_path: str, root_mode: bool = True, serial: Optional[str] = None) -> List[FileItem]:
        """
        Lists directory contents on Android device using toybox / ls parser.
        """
        # Normalize directory path with trailing slash so symlinked folders (like /sdcard) list contents
        clean_path = dir_path.rstrip("/")
        target_path = f"{clean_path}/" if clean_path else "/"

        cmd = f"ls -la '{target_path}'"
        code, out, err = self._exec_shell(cmd, root_mode, serial)
        
        items: List[FileItem] = []
        if code != 0 and not out:
            if root_mode:
                return self.list_dir(dir_path, root_mode=False, serial=serial)
            return items

        lines = out.strip().splitlines()
        for line in lines:
            line = line.strip()
            if not line or line.startswith("total "):
                continue

            parts = line.split(maxsplit=7)
            if len(parts) < 7:
                parts = line.split(maxsplit=6)
                if len(parts) < 6:
                    continue

            perms = parts[0]
            is_dir = perms.startswith("d")
            is_link = perms.startswith("l")

            try:
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
        code, out, err = self._exec_shell(f"unzip -o '{zip_path}' -d '{dst_folder}'", root_mode, serial, timeout=60)
        if code == 0:
            return True, "Extracted successfully"
        code2, out2, err2 = self._exec_shell(f"toybox unzip -o '{zip_path}' -d '{dst_folder}'", root_mode, serial, timeout=60)
        if code2 == 0:
            return True, "Extracted successfully"
        return False, err or err2 or "Device unzip utility failed"

    def push_file(self, local_path: str, remote_path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        """
        Push file/folder from PC to Android.
        If root_mode and writing to protected root directory, pushes to /data/local/tmp/
        first, then moves with root permissions.
        """
        file_name = os.path.basename(local_path.rstrip(r"\/"))
        ser_args = ["-s", serial] if serial else []

        # If remote path is directory, append file_name
        dest_is_dir = remote_path.endswith("/") or remote_path in ["/", "/sdcard", "/system", "/data"]
        target_dest = f"{remote_path.rstrip('/')}/{file_name}" if dest_is_dir else remote_path

        if not root_mode or remote_path.startswith("/sdcard") or remote_path.startswith("/storage"):
            code, out, err = self.adb.run_cmd(ser_args + ["push", local_path, target_dest], timeout=120)
            return code == 0, out or err

        tmp_remote = f"/data/local/tmp/{file_name}"
        code, out, err = self.adb.run_cmd(ser_args + ["push", local_path, tmp_remote], timeout=120)
        if code != 0:
            return False, err or "Failed to push to temporary directory"

        code2, out2, err2 = self._exec_shell(f"cp -r '{tmp_remote}' '{target_dest}' && rm -rf '{tmp_remote}'", root_mode=True, serial=serial)
        if code2 == 0:
            self._exec_shell(f"chmod -R 644 '{target_dest}'", root_mode=True, serial=serial)
            return True, "File pushed successfully with root access"
        return False, err2 or "Failed to move file to final destination"

    def pull_file(self, remote_path: str, local_path: str, root_mode: bool = True, serial: Optional[str] = None) -> Tuple[bool, str]:
        """
        Pull file/folder from Android to PC.
        If root_mode on protected path, copies to /data/local/tmp/ first.
        """
        ser_args = ["-s", serial] if serial else []

        code, out, err = self.adb.run_cmd(ser_args + ["pull", remote_path, local_path], timeout=120)
        if code == 0:
            return True, out.strip() or "File pulled successfully"

        if root_mode:
            file_name = os.path.basename(remote_path.rstrip("/"))
            tmp_remote = f"/data/local/tmp/_ximi_pull_{file_name}"
            self._exec_shell(f"cp -r '{remote_path}' '{tmp_remote}' && chmod -R 777 '{tmp_remote}'", root_mode=True, serial=serial)
            code2, out2, err2 = self.adb.run_cmd(ser_args + ["pull", tmp_remote, local_path], timeout=120)
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
