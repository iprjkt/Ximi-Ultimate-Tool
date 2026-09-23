"""
Floating Bottom Navigation Bar for Ximi Ultimate Tool.
Features liquid glass effect (frosted glass / translucent blur style),
pill shape, responsive item highlighting, and smooth interaction.
"""

from typing import List, Tuple
from PyQt6.QtCore import QPoint, Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QFrame,
    QGraphicsDropShadowEffect,
    QHBoxLayout,
    QPushButton,
    QWidget
)
from PyQt6.QtGui import QColor
from app.ui.i18n import tr

class FloatingNavItem(QPushButton):
    def __init__(self, icon_str: str, text_key: str, index: int, parent=None):
        super().__init__(parent)
        self.index = index
        self.icon_str = icon_str
        self.text_key = text_key
        self.setObjectName("FloatingNavItem")
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.update_label()

    def update_label(self):
        localized_text = tr(self.text_key)
        self.setText(f"{self.icon_str}  {localized_text}")

    def set_selected(self, selected: bool):
        self.setProperty("selected", "true" if selected else "false")
        self.style().unpolish(self)
        self.style().polish(self)


class FloatingBottomBar(QFrame):
    tab_changed = pyqtSignal(int)

    def __init__(self, parent: QWidget = None):
        super().__init__(parent)
        self.setObjectName("FloatingBottomBar")
        self.current_index = 0
        self.items: List[FloatingNavItem] = []

        # Setup Layout
        self.layout = QHBoxLayout(self)
        self.layout.setContentsMargins(12, 6, 12, 6)
        self.layout.setSpacing(8)

        # 5 Navigation Items from instruction: ADB, Fastboot, Terminal, Explorer, Settings
        nav_defs: List[Tuple[str, str]] = [
            ("⚡", "nav_adb"),
            ("🚀", "nav_fastboot"),
            ("💻", "nav_terminal"),
            ("📁", "nav_explorer"),
            ("⚙️", "nav_settings"),
        ]

        for idx, (icon, key) in enumerate(nav_defs):
            item = FloatingNavItem(icon, key, idx, self)
            item.clicked.connect(lambda checked=False, i=idx: self.select_tab(i))
            self.layout.addWidget(item)
            self.items.append(item)

        # Set default active tab
        self.select_tab(0, emit_signal=False)

        # Add Liquid Glass drop shadow
        shadow = QGraphicsDropShadowEffect(self)
        shadow.setBlurRadius(25)
        shadow.setColor(QColor(0, 0, 0, 80))
        shadow.setOffset(0, 6)
        self.setGraphicsEffect(shadow)

    def select_tab(self, index: int, emit_signal: bool = True):
        self.current_index = index
        for idx, item in enumerate(self.items):
            item.set_selected(idx == index)
        if emit_signal:
            self.tab_changed.emit(index)

    def update_translations(self):
        for item in self.items:
            item.update_label()
