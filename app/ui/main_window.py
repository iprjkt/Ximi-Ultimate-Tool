"""
Main Window for Ximi Ultimate Tool.
Integrates:
- Modern MIUIX / HyperOS liquid glass theme
- Floating Bottom Navigation Bar (iOS 26 / HyperOS pill style)
- Device status monitor (ADB & Fastboot)
- Dynamic view switching (ADB, Fastboot, Terminal, Explorer, Settings)
- Live language and theme switching
"""

import os
from typing import Optional
from PyQt6.QtCore import QPoint, QSize, Qt, QTimer
from PyQt6.QtGui import QColor, QFont, QIcon, QPainter, QPixmap
from PyQt6.QtWidgets import (
    QApplication,
    QFrame,
    QHBoxLayout,
    QLabel,
    QMainWindow,
    QMenu,
    QPushButton,
    QStackedWidget,
    QVBoxLayout,
    QWidget
)
from app.core.adb_manager import ADBManager
from app.core.explorer_manager import ExplorerManager
from app.core.fastboot_manager import FastbootManager
from app.core.settings_manager import SettingsManager
from app.ui.floating_bar import FloatingBottomBar
from app.ui.i18n import I18nManager, tr
from app.ui.styles import get_theme_qss
from app.ui.views.adb_view import ADBView
from app.ui.views.explorer_view import ExplorerView
from app.ui.views.fastboot_view import FastbootView
from app.ui.views.settings_view import SettingsView
from app.ui.views.terminal_view import TerminalView

class BackgroundContainer(QWidget):
    """Container widget supporting custom image background with dark/light tint overlay."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WidgetAttribute.WA_OpaquePaintEvent, True)
        self.bg_pixmap: Optional[QPixmap] = None
        self.solid_bg = QColor("#0E0E18")
        self.overlay_color = QColor(14, 14, 24, 210)

    def set_background_image(self, img_path: str):
        if img_path and os.path.exists(img_path):
            self.bg_pixmap = QPixmap(img_path)
        else:
            self.bg_pixmap = None
        self.update()

    def set_theme_colors(self, solid_bg: QColor, overlay_color: QColor):
        self.solid_bg = solid_bg
        self.overlay_color = overlay_color
        self.update()

    def paintEvent(self, event):
        painter = QPainter(self)
        # Always fill the entire canvas with 100% opaque solid color first!
        painter.fillRect(self.rect(), self.solid_bg)

        if self.bg_pixmap and not self.bg_pixmap.isNull():
            scaled = self.bg_pixmap.scaled(self.size(), Qt.AspectRatioMode.KeepAspectRatioByExpanding, Qt.TransformationMode.SmoothTransformation)
            x = (self.width() - scaled.width()) // 2
            y = (self.height() - scaled.height()) // 2
            painter.drawPixmap(x, y, scaled)
            painter.fillRect(self.rect(), self.overlay_color)
        super().paintEvent(event)


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Ximi Ultimate Tool - Xiaomi HyperOS & MIUI Utility")
        self.resize(1150, 780)
        self.setMinimumSize(950, 650)

        # Core Managers
        self.settings = SettingsManager()
        self.adb = ADBManager()
        self.fastboot = FastbootManager()
        self.explorer = ExplorerManager(self.adb)

        # Active Device Tracking
        self.current_serial: Optional[str] = None
        self.current_mode: str = "none"  # "adb", "fastboot", "none"

        # Apply Saved Language & Theme
        saved_lang = self.settings.get("language", "id")
        I18nManager().set_language(saved_lang)
        self.current_theme = self.settings.get("theme", "dark")

        self.init_ui()
        self.apply_theme(self.current_theme)

        # Custom Background if set
        custom_bg = self.settings.get("custom_background", "")
        if custom_bg:
            self.central_bg.set_background_image(custom_bg)

        # Start periodic device detection timer
        self.device_timer = QTimer(self)
        self.device_timer.timeout.connect(self.check_device_status)
        self.device_timer.start(2500)
        self.check_device_status()

    def init_ui(self):
        # Master Background Widget
        self.central_bg = BackgroundContainer(self)
        self.central_bg.setObjectName("CentralBackgroundWidget")
        self.setCentralWidget(self.central_bg)

        # Master Layout (with bottom margin for floating bar clearance)
        self.master_layout = QVBoxLayout(self.central_bg)
        self.master_layout.setContentsMargins(16, 12, 16, 68)
        self.master_layout.setSpacing(8)

        # 1. Top Header Bar (Device status, Reboot buttons)
        self.setup_header_bar()

        # 2. Main Stacked Views
        self.stack = QStackedWidget()
        
        self.view_adb = ADBView(self.adb, self)
        self.view_fastboot = FastbootView(self.fastboot, self)
        self.view_terminal = TerminalView(self.adb, self)
        self.view_explorer = ExplorerView(self.explorer, self)
        self.view_settings = SettingsView(self.adb, self.settings, self)

        # Wire settings signals
        self.view_settings.language_changed.connect(self.on_language_changed)
        self.view_settings.theme_changed.connect(self.apply_theme)
        self.view_settings.background_changed.connect(self.on_background_changed)

        self.stack.addWidget(self.view_adb)        # Index 0
        self.stack.addWidget(self.view_fastboot)   # Index 1
        self.stack.addWidget(self.view_terminal)   # Index 2
        self.stack.addWidget(self.view_explorer)   # Index 3
        self.stack.addWidget(self.view_settings)   # Index 4

        self.master_layout.addWidget(self.stack, 1)

        # 3. Floating Bottom Navigation Bar (iOS 26 / HyperOS pill liquid glass)
        self.floating_bar = FloatingBottomBar(self.central_bg)
        self.floating_bar.tab_changed.connect(self.on_tab_changed)
        self.floating_bar.show()

        # Reposition floating bar to center bottom
        self.position_floating_bar()

    def setup_header_bar(self):
        self.header_frame = QFrame()
        self.header_frame.setObjectName("HeaderFrame")
        self.header_frame.setStyleSheet("""
            #HeaderFrame {
                background-color: rgba(28, 28, 42, 0.7);
                border: 1px solid rgba(255, 255, 255, 0.08);
                border-radius: 16px;
                padding: 4px 12px;
            }
        """)
        h_layout = QHBoxLayout(self.header_frame)
        h_layout.setContentsMargins(12, 6, 12, 6)
        h_layout.setSpacing(10)

        # App Brand & Logo
        self.lbl_logo = QLabel("XIMI")
        self.lbl_logo.setStyleSheet("""
            font-size: 16px;
            font-weight: 900;
            color: #C084FC;
            letter-spacing: 1.5px;
        """)

        # Device connection status badge
        self.lbl_status = QLabel(tr("status_no_device"))
        self.lbl_status.setStyleSheet("""
            font-size: 13px;
            font-weight: 600;
            color: #9CA3AF;
            padding: 4px 10px;
            background-color: rgba(255, 255, 255, 0.05);
            border-radius: 8px;
        """)

        # Reboot Quick Menu
        self.btn_reboot_menu = QPushButton(f"🔄 {tr('reboot')}")
        self.reboot_menu = QMenu(self)
        
        self.act_reboot_sys = self.reboot_menu.addAction(tr("reboot_system"))
        self.act_reboot_sys.triggered.connect(lambda: self.execute_reboot("system"))
        
        self.act_reboot_rec = self.reboot_menu.addAction(tr("reboot_recovery"))
        self.act_reboot_rec.triggered.connect(lambda: self.execute_reboot("recovery"))
        
        self.act_reboot_fb = self.reboot_menu.addAction(tr("reboot_bootloader"))
        self.act_reboot_fb.triggered.connect(lambda: self.execute_reboot("bootloader"))
        
        self.act_reboot_edl = self.reboot_menu.addAction(tr("reboot_edl"))
        self.act_reboot_edl.triggered.connect(lambda: self.execute_reboot("edl"))

        self.btn_reboot_menu.setMenu(self.reboot_menu)

        self.btn_refresh_devices = QPushButton(f"⚡ {tr('refresh')}")
        self.btn_refresh_devices.clicked.connect(self.check_device_status)

        h_layout.addWidget(self.lbl_logo)
        h_layout.addWidget(self.lbl_status)
        h_layout.addStretch()
        h_layout.addWidget(self.btn_reboot_menu)
        h_layout.addWidget(self.btn_refresh_devices)

        self.master_layout.addWidget(self.header_frame)

    def position_floating_bar(self):
        if not hasattr(self, "floating_bar") or not self.floating_bar:
            return
        bar_width = self.floating_bar.sizeHint().width()
        bar_height = self.floating_bar.sizeHint().height()
        # Keep centered at bottom with 20px margin
        x = (self.width() - bar_width) // 2
        y = self.height() - bar_height - 20
        self.floating_bar.setGeometry(x, y, bar_width, bar_height)
        self.floating_bar.raise_()

    def resizeEvent(self, event):
        super().resizeEvent(event)
        self.position_floating_bar()

    def on_tab_changed(self, index: int):
        self.stack.setCurrentIndex(index)
        # If navigating to settings, refresh device card
        if index == 4:
            self.view_settings.refresh_device_card()

    def check_device_status(self):
        # 1. Check ADB Devices
        adb_devs = self.adb.get_connected_devices()
        if adb_devs:
            dev = adb_devs[0]
            self.current_serial = dev["serial"]
            self.current_mode = "adb"
            state = dev["state"]
            if state == "device":
                model = self.adb.get_prop("ro.product.marketname", self.current_serial) or dev["serial"]
                self.lbl_status.setText(f"🟢 {tr('status_adb_connected', device=model)}")
                self.lbl_status.setStyleSheet("color: #34D399; font-weight: 600; padding: 4px 10px; background-color: rgba(52, 211, 153, 0.12); border-radius: 8px;")
            elif state == "unauthorized":
                self.lbl_status.setText(f"🟡 {tr('status_unauthorized')}")
                self.lbl_status.setStyleSheet("color: #FBBF24; font-weight: 600; padding: 4px 10px; background-color: rgba(251, 191, 36, 0.12); border-radius: 8px;")
            else:
                self.lbl_status.setText(f"🔵 ADB: {dev['serial']} ({state})")
            self.propagate_serial_to_views()
            return

        # 2. Check Fastboot Devices
        fb_devs = self.fastboot.get_connected_devices()
        if fb_devs:
            dev = fb_devs[0]
            self.current_serial = dev["serial"]
            self.current_mode = "fastboot"
            self.lbl_status.setText(f"🚀 {tr('status_fastboot_connected', device=dev['serial'])}")
            self.lbl_status.setStyleSheet("color: #60A5FA; font-weight: 600; padding: 4px 10px; background-color: rgba(96, 165, 250, 0.12); border-radius: 8px;")
            self.propagate_serial_to_views()
            return

        # No device connected
        self.current_serial = None
        self.current_mode = "none"
        self.lbl_status.setText(f"⚪ {tr('status_no_device')}")
        self.lbl_status.setStyleSheet("color: #9CA3AF; font-weight: 600; padding: 4px 10px; background-color: rgba(255, 255, 255, 0.05); border-radius: 8px;")
        self.propagate_serial_to_views()

    def propagate_serial_to_views(self):
        self.view_adb.set_serial(self.current_serial)
        self.view_fastboot.set_serial(self.current_serial)
        self.view_terminal.set_serial(self.current_serial)
        self.view_explorer.set_serial(self.current_serial)
        self.view_settings.set_serial(self.current_serial)

    def execute_reboot(self, mode: str):
        if self.current_mode == "fastboot":
            if mode == "recovery":
                self.fastboot.run_cmd(["reboot", "recovery"])
            elif mode == "bootloader":
                self.fastboot.run_cmd(["reboot-bootloader"])
            elif mode == "edl":
                self.fastboot.run_cmd(["oem", "edl"])
            else:
                self.fastboot.run_cmd(["reboot"])
        else:
            self.adb.reboot(mode, self.current_serial)
        # Schedule status check
        QTimer.singleShot(1500, self.check_device_status)

    def on_language_changed(self, lang_code: str):
        # Update labels everywhere
        self.floating_bar.update_translations()
        self.view_adb.update_translations()
        self.view_fastboot.update_translations()
        self.view_terminal.update_translations()
        self.view_explorer.update_translations()
        self.btn_reboot_menu.setText(f"🔄 {tr('reboot')}")
        self.act_reboot_sys.setText(tr("reboot_system"))
        self.act_reboot_rec.setText(tr("reboot_recovery"))
        self.act_reboot_fb.setText(tr("reboot_bootloader"))
        self.act_reboot_edl.setText(tr("reboot_edl"))
        self.btn_refresh_devices.setText(f"⚡ {tr('refresh')}")
        self.check_device_status()
        self.position_floating_bar()

    def apply_theme(self, theme_mode: str):
        self.current_theme = theme_mode
        qss = get_theme_qss(theme_mode)
        app = QApplication.instance()
        if app:
            app.setStyleSheet(qss)
        
        # Solid backgrounds for Wayland / Hyprland
        if theme_mode == "dark":
            self.central_bg.set_theme_colors(QColor("#0E0E18"), QColor(14, 14, 24, 210))
        else:
            self.central_bg.set_theme_colors(QColor("#F1F3F7"), QColor(241, 243, 247, 210))

    def on_background_changed(self, img_path: str):
        self.central_bg.set_background_image(img_path)
