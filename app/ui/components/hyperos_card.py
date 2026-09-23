"""
HyperOS About Phone card component directly reproducing the layout of example.png.
Theme-aware: adapts to both HyperOS Midnight (Dark) and MIUIX Clean (Light).
"""

from typing import Dict, Optional
from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QFrame,
    QHBoxLayout,
    QLabel,
    QVBoxLayout,
    QWidget
)
from app.ui.i18n import tr

class HyperOSCardRow(QWidget):
    """A row inside the top card with label, value, and chevron '>'"""
    def __init__(self, label_key: str, value_text: str = "--", show_chevron: bool = True, parent=None):
        super().__init__(parent)
        self.label_key = label_key
        layout = QHBoxLayout(self)
        layout.setContentsMargins(18, 14, 18, 14)

        self.lbl_title = QLabel(tr(label_key))
        self.lbl_title.setObjectName("CardRowTitle")

        self.lbl_val = QLabel(value_text)
        self.lbl_val.setObjectName("CardRowValue")

        layout.addWidget(self.lbl_title)
        layout.addStretch()
        layout.addWidget(self.lbl_val)

        if show_chevron:
            chevron = QLabel("›")
            chevron.setObjectName("CardRowValue")
            chevron.setStyleSheet("font-size: 18px; font-weight: 700; margin-left: 6px;")
            layout.addWidget(chevron)

    def set_value(self, val: str):
        self.lbl_val.setText(val)

    def update_label(self):
        self.lbl_title.setText(tr(self.label_key))


class HyperOSSpecsItem(QWidget):
    """Item inside specs card: Value on top, small label beneath."""
    def __init__(self, value_text: str, label_key: str, parent=None):
        super().__init__(parent)
        self.label_key = label_key
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 4, 0, 8)
        layout.setSpacing(2)

        self.lbl_value = QLabel(value_text)
        self.lbl_value.setObjectName("SpecsItemValue")

        self.lbl_label = QLabel(tr(label_key))
        self.lbl_label.setObjectName("SpecsItemLabel")

        layout.addWidget(self.lbl_value)
        layout.addWidget(self.lbl_label)

    def set_value(self, val: str):
        self.lbl_value.setText(val)

    def update_label(self):
        self.lbl_label.setText(tr(self.label_key))


class HyperOSAboutCard(QWidget):
    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(16)

        # 1. Header Banner: "Xiaomi HyperOS" and version code
        header_container = QWidget()
        header_layout = QVBoxLayout(header_container)
        header_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        header_layout.setSpacing(4)

        self.lbl_brand = QLabel("Xiaomi HyperOS")
        self.lbl_brand.setObjectName("HyperOSBrandLabel")
        self.lbl_brand.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self.lbl_version_short = QLabel("3.0.303.0")
        self.lbl_version_short.setObjectName("HyperOSVersionLabel")
        self.lbl_version_short.setAlignment(Qt.AlignmentFlag.AlignCenter)

        header_layout.addWidget(self.lbl_brand)
        header_layout.addWidget(self.lbl_version_short)
        main_layout.addWidget(header_container)

        # 2. Card 1: Device Name, Storage, OS Version
        self.card1 = QFrame()
        self.card1.setObjectName("HyperOSCard1")
        c1_layout = QVBoxLayout(self.card1)
        c1_layout.setContentsMargins(0, 4, 0, 4)
        c1_layout.setSpacing(0)

        self.row_device_name = HyperOSCardRow("device_name", "Redmi Note 14")
        self.row_storage = HyperOSCardRow("storage", "72.5GB/256GB")
        self.row_os_version = HyperOSCardRow("os_version", "3.0.303.0.WPJCNXM.C06", show_chevron=False)

        def make_divider():
            div = QFrame()
            div.setFrameShape(QFrame.Shape.HLine)
            div.setStyleSheet("background-color: rgba(128, 128, 128, 0.15); max-height: 1px; margin: 0 16px;")
            return div

        c1_layout.addWidget(self.row_device_name)
        c1_layout.addWidget(make_divider())
        c1_layout.addWidget(self.row_storage)
        c1_layout.addWidget(make_divider())
        c1_layout.addWidget(self.row_os_version)

        main_layout.addWidget(self.card1)

        # 3. Card 2: Model Name, CPU, RAM, Battery
        self.card2 = QFrame()
        self.card2.setObjectName("HyperOSCard2")
        c2_layout = QVBoxLayout(self.card2)
        c2_layout.setContentsMargins(20, 18, 20, 18)
        c2_layout.setSpacing(12)

        self.lbl_device_title = QLabel("Redmi Note 14")
        self.lbl_device_title.setObjectName("CardRowTitle")
        self.lbl_device_title.setStyleSheet("font-size: 20px; font-weight: 700;")
        c2_layout.addWidget(self.lbl_device_title)

        self.item_cpu = HyperOSSpecsItem("Mediatek Helio G99-Ultra", "cpu_processor")
        self.item_ram = HyperOSSpecsItem("8.0GB", "ram_memory")
        self.item_battery = HyperOSSpecsItem("5500mAh(typ)", "battery_capacity")

        c2_layout.addWidget(self.item_cpu)
        c2_layout.addWidget(self.item_ram)
        c2_layout.addWidget(self.item_battery)

        main_layout.addWidget(self.card2)

    def update_data(self, specs: Dict[str, str]):
        market = specs.get("market_name", "Redmi Note 14")
        short_ver = specs.get("hyperos_short", "3.0.303.0")
        full_ver = specs.get("hyperos_version", "3.0.303.0.WPJCNXM.C06")
        storage = specs.get("storage", "72.5GB/256GB")
        cpu = specs.get("cpu", "Mediatek Helio G99-Ultra")
        ram = specs.get("ram", "8.0GB")
        battery = specs.get("battery", "5500mAh(typ)")

        self.lbl_version_short.setText(short_ver)
        self.row_device_name.set_value(market)
        self.row_storage.set_value(storage)
        self.row_os_version.set_value(full_ver)

        self.lbl_device_title.setText(market)
        self.item_cpu.set_value(cpu)
        self.item_ram.set_value(ram)
        self.item_battery.set_value(battery)

    def update_translations(self):
        self.row_device_name.update_label()
        self.row_storage.update_label()
        self.row_os_version.update_label()
        self.item_cpu.update_label()
        self.item_ram.update_label()
        self.item_battery.update_label()
