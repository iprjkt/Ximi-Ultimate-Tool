"""
Settings and configuration persistence manager for Ximi Ultimate Tool.
"""

import json
import os
from pathlib import Path
from typing import Any, Dict

CONFIG_DIR = Path.home() / ".config" / "ximi_ultimate_tool"
CONFIG_FILE = CONFIG_DIR / "config.json"

DEFAULT_CONFIG: Dict[str, Any] = {
    "language": "id",
    "theme": "dark",  # "dark" or "light"
    "custom_background": "",
    "custom_adb_path": "",
    "custom_fastboot_path": "",
    "root_mode_default": True,
    "last_flasher_directory": "",
    "recent_packages": []
}

class SettingsManager:
    _instance = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(SettingsManager, cls).__new__(cls)
            cls._instance._config = DEFAULT_CONFIG.copy()
            cls._instance._load()
        return cls._instance

    def _load(self):
        try:
            if CONFIG_FILE.exists():
                with open(CONFIG_FILE, "r", encoding="utf-8") as f:
                    loaded = json.load(f)
                    self._config.update(loaded)
        except Exception as e:
            print(f"Error loading configuration: {e}")

    def save(self):
        try:
            CONFIG_DIR.mkdir(parents=True, exist_ok=True)
            with open(CONFIG_FILE, "w", encoding="utf-8") as f:
                json.dump(self._config, f, indent=4, ensure_ascii=False)
        except Exception as e:
            print(f"Error saving configuration: {e}")

    def get(self, key: str, default: Any = None) -> Any:
        return self._config.get(key, default)

    def set(self, key: str, value: Any):
        self._config[key] = value
        self.save()
