#!/usr/bin/env python3
"""
Ximi Ultimate Tool
Xiaomi HyperOS & MIUI ADB and Fastboot Swiss Army Knife.
Author: iprjkt
Telegram: https://t.me/anotherside551
GitHub: https://github.com/iprjkt
"""

import os
import sys
from pathlib import Path
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QFont, QFontDatabase, QIcon
from PyQt6.QtWidgets import QApplication

# Ensure project root is in sys.path
BASE_DIR = Path(__file__).resolve().parent
if str(BASE_DIR) not in sys.path:
    sys.path.insert(0, str(BASE_DIR))

from app.ui.main_window import MainWindow

def main():
    # Enable high DPI scaling
    os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"

    app = QApplication(sys.argv)
    app.setApplicationName("Ximi Ultimate Tool")
    app.setOrganizationName("iprjkt")

    # Load custom Roboto font if present
    font_path = BASE_DIR / "Roboto-Regular.ttf"
    if font_path.exists():
        font_id = QFontDatabase.addApplicationFont(str(font_path))
        if font_id != -1:
            families = QFontDatabase.applicationFontFamilies(font_id)
            if families:
                app.setFont(QFont(families[0], 10))
    else:
        app.setFont(QFont("Segoe UI" if os.name == "nt" else "Inter", 10))

    window = MainWindow()
    window.show()

    sys.exit(app.exec())

if __name__ == "__main__":
    main()
