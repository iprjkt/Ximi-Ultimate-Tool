"""
Floating Bottom Navigation Bar for Ximi Ultimate Tool.
Features:
- Liquid glass aesthetic (iOS 26 / HyperOS style)
- Fluid spring sliding indicator pill (QEasingCurve.OutBack)
- Responsive hover and press micro-interactions
- Active tab text glow and smooth transitions
"""

from typing import List, Tuple
from PyQt6.QtCore import (
    QEasingCurve,
    QPoint,
    QPropertyAnimation,
    QRect,
    Qt,
    pyqtProperty,
    pyqtSignal
)
from PyQt6.QtGui import QColor, QFont
from PyQt6.QtWidgets import (
    QFrame,
    QGraphicsDropShadowEffect,
    QHBoxLayout,
    QPushButton,
    QWidget
)
from app.ui.i18n import tr

class SlidingIndicatorPill(QFrame):
    """Smooth pill indicator that slides behind the active navigation item."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("SlidingIndicatorPill")
        self.setAttribute(Qt.WidgetAttribute.WA_TransparentForMouseEvents, True)
        self.setStyleSheet("""
            #SlidingIndicatorPill {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #3B82F6, stop:1 #8B5CF6);
                border-radius: 20px;
                border: 1px solid rgba(255, 255, 255, 0.25);
            }
        """)
        # Drop glow effect
        glow = QGraphicsDropShadowEffect(self)
        glow.setBlurRadius(18)
        glow.setColor(QColor(139, 92, 246, 120))
        glow.setOffset(0, 2)
        self.setGraphicsEffect(glow)


class FloatingNavItem(QPushButton):
    def __init__(self, icon_str: str, text_key: str, index: int, parent=None):
        super().__init__(parent)
        self.index = index
        self.icon_str = icon_str
        self.text_key = text_key
        self.setObjectName("FloatingNavItem")
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setStyleSheet("""
            #FloatingNavItem {
                background-color: transparent;
                border: none;
                border-radius: 20px;
                color: #94A3B8;
                font-weight: 600;
                font-size: 13px;
                padding: 8px 18px;
            }
            #FloatingNavItem:hover {
                color: #FFFFFF;
                background-color: rgba(255, 255, 255, 0.06);
            }
            #FloatingNavItem[selected="true"] {
                color: #FFFFFF;
                font-weight: 700;
                background-color: transparent;
            }
        """)
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
        self._initial_positioned = False

        # Drop shadow on the overall pill bar
        shadow = QGraphicsDropShadowEffect(self)
        shadow.setBlurRadius(28)
        shadow.setColor(QColor(0, 0, 0, 100))
        shadow.setOffset(0, 8)
        self.setGraphicsEffect(shadow)

        # Sliding Pill Indicator (behind buttons)
        self.pill = SlidingIndicatorPill(self)
        self.pill_anim = QPropertyAnimation(self.pill, b"geometry")
        self.pill_anim.setDuration(300)
        # OutBack gives the snappy spring bounce of iOS 26 / HyperOS
        self.pill_anim.setEasingCurve(QEasingCurve.Type.OutBack)

        # Setup Layout
        self.layout = QHBoxLayout(self)
        self.layout.setContentsMargins(10, 6, 10, 6)
        self.layout.setSpacing(6)

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

        # Ensure pill is behind buttons
        self.pill.lower()

    def select_tab(self, index: int, emit_signal: bool = True):
        self.current_index = index
        for idx, item in enumerate(self.items):
            item.set_selected(idx == index)

        self.animate_pill_to_index(index)

        if emit_signal:
            self.tab_changed.emit(index)

    def animate_pill_to_index(self, index: int):
        if not (0 <= index < len(self.items)):
            return

        target_item = self.items[index]
        target_rect = target_item.geometry()

        if target_rect.width() <= 0 or not self.isVisible():
            return

        # Slight vertical adjustment for snug pill alignment
        adjusted_rect = QRect(
            target_rect.x(),
            target_rect.y(),
            target_rect.width(),
            target_rect.height()
        )

        if not self._initial_positioned:
            self.pill.setGeometry(adjusted_rect)
            self._initial_positioned = True
        else:
            self.pill_anim.stop()
            self.pill_anim.setStartValue(self.pill.geometry())
            self.pill_anim.setEndValue(adjusted_rect)
            self.pill_anim.start()

    def resizeEvent(self, event):
        super().resizeEvent(event)
        # Snap pill geometry on resize without animation
        if 0 <= self.current_index < len(self.items):
            target_rect = self.items[self.current_index].geometry()
            if target_rect.width() > 0:
                self.pill.setGeometry(target_rect)
                self._initial_positioned = True

    def showEvent(self, event):
        super().showEvent(event)
        # Position pill when first displayed
        if 0 <= self.current_index < len(self.items):
            self.items[self.current_index].set_selected(True)
            target_rect = self.items[self.current_index].geometry()
            if target_rect.width() > 0:
                self.pill.setGeometry(target_rect)
                self._initial_positioned = True

    def update_translations(self):
        for item in self.items:
            item.update_label()
        self.adjustSize()
        self.animate_pill_to_index(self.current_index)
