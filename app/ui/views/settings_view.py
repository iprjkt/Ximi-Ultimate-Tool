"""
Settings View for Ximi Ultimate Tool.
Features:
- Exact HyperOS 'About Phone' card matching example.png (ro.mi.os.version.incremental)
- Language configuration (Indonesian & English)
- Theme configuration (Dark Mode, Light Mode)
- Custom wallpaper background configuration
- Links to GitHub (https://github.com/iprjkt) and Telegram (https://t.me/anotherside551)
Theme-aware: adapts cleanly to both Dark Mode and Light Mode.
"""

from typing import Dict, Optional
from PyQt6.QtCore import QUrl, Qt, pyqtSignal
from PyQt6.QtGui import QDesktopServices
from PyQt6.QtWidgets import (
    QComboBox,
    QFileDialog,
    QFrame,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QScrollArea,
    QVBoxLayout,
    QWidget
)
from app.core.adb_manager import ADBManager
from app.core.settings_manager import SettingsManager
from app.ui.components.hyperos_card import HyperOSAboutCard
from app.ui.i18n import I18nManager, tr

class SettingsView(QWidget):
    language_changed = pyqtSignal(str)
    theme_changed = pyqtSignal(str)
    background_changed = pyqtSignal(str)

    def __init__(self, adb_manager: ADBManager, settings: SettingsManager, parent=None):
        super().__init__(parent)
        self.adb = adb_manager
        self.settings = settings
        self.current_serial: Optional[str] = None
        self.init_ui()

    def set_serial(self, serial: Optional[str]):
        self.current_serial = serial
        self.refresh_device_card()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)

        # Scroll area for clean scrolling on smaller screens
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll.setStyleSheet("background: transparent;")
        main_layout.addWidget(self.scroll)

        content = QWidget()
        self.scroll.setWidget(content)
        content_layout = QVBoxLayout(content)
        content_layout.setContentsMargins(24, 14, 24, 80)
        content_layout.setSpacing(16)

        # 1. HyperOS About Phone Card Component (matches example.png)
        self.card_about = HyperOSAboutCard()
        content_layout.addWidget(self.card_about)

        # Refresh device specs button
        sync_box = QHBoxLayout()
        self.btn_refresh_specs = QPushButton("🔄 Refresh Device Specs")
        self.btn_refresh_specs.clicked.connect(self.refresh_device_card)
        sync_box.addStretch()
        sync_box.addWidget(self.btn_refresh_specs)
        sync_box.addStretch()
        content_layout.addLayout(sync_box)

        # 2. Preferences & Appearance Card
        self.card_pref = QFrame()
        self.card_pref.setObjectName("SettingsCardPref")
        pref_layout = QVBoxLayout(self.card_pref)
        pref_layout.setContentsMargins(18, 16, 18, 16)
        pref_layout.setSpacing(14)

        self.lbl_pref_title = QLabel(f"🎨 {tr('section_preferences')}")
        self.lbl_pref_title.setObjectName("SettingsPrefTitle")
        pref_layout.addWidget(self.lbl_pref_title)

        # Language row
        lang_row = QHBoxLayout()
        self.lbl_lang = QLabel(tr("pref_language"))
        self.lbl_lang.setObjectName("SettingsPrefLabel")
        self.cmb_lang = QComboBox()
        self.cmb_lang.setMinimumWidth(190)
        self.cmb_lang.setFixedHeight(36)
        self.cmb_lang.addItem("Bahasa Indonesia", "id")
        self.cmb_lang.addItem("English", "en")
        
        current_lang = self.settings.get("language", "id")
        lang_idx = self.cmb_lang.findData(current_lang)
        if lang_idx >= 0:
            self.cmb_lang.setCurrentIndex(lang_idx)
        self.cmb_lang.currentIndexChanged.connect(self.on_language_changed)

        lang_row.addWidget(self.lbl_lang)
        lang_row.addStretch()
        lang_row.addWidget(self.cmb_lang)
        pref_layout.addLayout(lang_row)

        # Theme row
        theme_row = QHBoxLayout()
        self.lbl_theme = QLabel(tr("pref_theme"))
        self.lbl_theme.setObjectName("SettingsPrefLabel")
        self.cmb_theme = QComboBox()
        self.cmb_theme.setMinimumWidth(190)
        self.cmb_theme.setFixedHeight(36)
        self.cmb_theme.addItem(tr("theme_dark"), "dark")
        self.cmb_theme.addItem(tr("theme_light"), "light")
        
        current_theme = self.settings.get("theme", "dark")
        theme_idx = self.cmb_theme.findData(current_theme)
        if theme_idx >= 0:
            self.cmb_theme.setCurrentIndex(theme_idx)
        self.cmb_theme.currentIndexChanged.connect(self.on_theme_changed)

        theme_row.addWidget(self.lbl_theme)
        theme_row.addStretch()
        theme_row.addWidget(self.cmb_theme)
        pref_layout.addLayout(theme_row)

        # Custom Background row
        bg_row = QHBoxLayout()
        self.lbl_bg = QLabel(tr("pref_custom_bg"))
        self.lbl_bg.setObjectName("SettingsPrefLabel")
        self.btn_select_bg = QPushButton(tr("btn_select_bg"))
        self.btn_select_bg.clicked.connect(self.choose_custom_background)
        self.btn_reset_bg = QPushButton(tr("btn_reset_bg"))
        self.btn_reset_bg.clicked.connect(self.reset_custom_background)

        bg_row.addWidget(self.lbl_bg)
        bg_row.addStretch()
        bg_row.addWidget(self.btn_select_bg)
        bg_row.addWidget(self.btn_reset_bg)
        pref_layout.addLayout(bg_row)

        content_layout.addWidget(self.card_pref)

        # 3. Community & Developer Links Card
        self.card_about_app = QFrame()
        self.card_about_app.setObjectName("SettingsCardAbout")
        about_layout = QVBoxLayout(self.card_about_app)
        about_layout.setContentsMargins(18, 16, 18, 16)
        about_layout.setSpacing(14)

        self.lbl_about_title = QLabel(f"🌐 {tr('section_about')}")
        self.lbl_about_title.setObjectName("SettingsAboutTitle")
        about_layout.addWidget(self.lbl_about_title)

        self.lbl_desc = QLabel(tr("app_description"))
        self.lbl_desc.setWordWrap(True)
        self.lbl_desc.setStyleSheet("font-size: 13px;")
        about_layout.addWidget(self.lbl_desc)

        # Links Buttons
        links_box = QHBoxLayout()
        self.btn_github = QPushButton("🐙 GitHub: https://github.com/iprjkt")
        self.btn_github.setStyleSheet("""
            QPushButton {
                background-color: #24292E;
                color: #FFFFFF;
                font-weight: 600;
                padding: 10px 16px;
                border-radius: 12px;
            }
            QPushButton:hover {
                background-color: #2F363D;
                border: 1px solid #6E7681;
            }
        """)
        self.btn_github.setCursor(Qt.CursorShape.PointingHandCursor)
        self.btn_github.clicked.connect(lambda: QDesktopServices.openUrl(QUrl("https://github.com/iprjkt")))

        self.btn_telegram = QPushButton("✈️ Telegram: https://t.me/anotherside551")
        self.btn_telegram.setStyleSheet("""
            QPushButton {
                background-color: #229ED9;
                color: #FFFFFF;
                font-weight: 600;
                padding: 10px 16px;
                border-radius: 12px;
            }
            QPushButton:hover {
                background-color: #28A8EA;
                border: 1px solid #70C5F5;
            }
        """)
        self.btn_telegram.setCursor(Qt.CursorShape.PointingHandCursor)
        self.btn_telegram.clicked.connect(lambda: QDesktopServices.openUrl(QUrl("https://t.me/anotherside551")))

        links_box.addWidget(self.btn_github)
        links_box.addWidget(self.btn_telegram)
        about_layout.addLayout(links_box)

        # Version & Credits
        self.lbl_credits = QLabel(tr("developer_label"))
        self.lbl_credits.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.lbl_credits.setStyleSheet("font-size: 12px; margin-top: 6px;")
        about_layout.addWidget(self.lbl_credits)

        content_layout.addWidget(self.card_about_app)

    def refresh_device_card(self):
        specs = self.adb.get_device_full_specs(self.current_serial)
        self.card_about.update_data(specs)

    def on_language_changed(self, index: int):
        lang_code = self.cmb_lang.itemData(index)
        if lang_code:
            self.settings.set("language", lang_code)
            I18nManager().set_language(lang_code)
            self.update_translations()
            self.language_changed.emit(lang_code)

    def on_theme_changed(self, index: int):
        theme_mode = self.cmb_theme.itemData(index)
        if theme_mode:
            self.settings.set("theme", theme_mode)
            self.theme_changed.emit(theme_mode)

    def choose_custom_background(self):
        path, _ = QFileDialog.getOpenFileName(self, "Choose Background Image", "", "Images (*.png *.jpg *.jpeg *.svg *.webp)")
        if path:
            self.settings.set("custom_background", path)
            self.background_changed.emit(path)

    def reset_custom_background(self):
        self.settings.set("custom_background", "")
        self.background_changed.emit("")

    def update_translations(self):
        self.card_about.update_translations()
        self.lbl_pref_title.setText(f"🎨 {tr('section_preferences')}")
        self.lbl_lang.setText(tr("pref_language"))
        self.lbl_theme.setText(tr("pref_theme"))
        self.cmb_theme.setItemText(0, tr("theme_dark"))
        self.cmb_theme.setItemText(1, tr("theme_light"))
        self.lbl_bg.setText(tr("pref_custom_bg"))
        self.btn_select_bg.setText(tr("btn_select_bg"))
        self.btn_reset_bg.setText(tr("btn_reset_bg"))
        self.lbl_about_title.setText(f"🌐 {tr('section_about')}")
        self.lbl_desc.setText(tr("app_description"))
        self.lbl_credits.setText(tr("developer_label"))
