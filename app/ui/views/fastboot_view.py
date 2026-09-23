"""
Fastboot Flasher View for Ximi Ultimate Tool.
Features:
- Single partition flasher (boot, recovery, vendor_boot, vbmeta flags, etc.)
- Fastboot ROM Flasher (Mi Flash alternative preventing accidental bootloader locking)
- Advance Mode partition selector (checklist to uncheck preloader/cust and comment out script)
- Live real-time flashing log output terminal.
"""

import os
from typing import List, Optional
from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QButtonGroup,
    QCheckBox,
    QComboBox,
    QFileDialog,
    QFrame,
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QMessageBox,
    QPlainTextEdit,
    QProgressBar,
    QPushButton,
    QRadioButton,
    QTabWidget,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget
)
from app.core.fastboot_manager import FastbootExecutionWorker, FastbootManager, FlashCommandItem
from app.ui.i18n import tr

class FastbootView(QWidget):
    def __init__(self, fastboot_manager: FastbootManager, parent=None):
        super().__init__(parent)
        self.fastboot = fastboot_manager
        self.current_serial: Optional[str] = None
        self.advance_items: List[FlashCommandItem] = []
        self.init_ui()

    def set_serial(self, serial: Optional[str]):
        self.current_serial = serial

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(16, 16, 16, 16)
        main_layout.setSpacing(14)

        # Tabs for Flasher modes
        self.tabs = QTabWidget()
        main_layout.addWidget(self.tabs)

        # 1. Single Partition Flasher
        self.tab_single = QWidget()
        self.setup_single_partition_ui()
        self.tabs.addTab(self.tab_single, tr("tab_single_partition"))

        # 2. Fastboot ROM Flasher (Mi Flash Alternative)
        self.tab_rom = QWidget()
        self.setup_full_rom_ui()
        self.tabs.addTab(self.tab_rom, tr("tab_full_rom"))

        # 3. Advance Mode
        self.tab_advance = QWidget()
        self.setup_advance_mode_ui()
        self.tabs.addTab(self.tab_advance, tr("tab_advance_mode"))

        # Live Real-time Terminal Log Section
        log_header = QHBoxLayout()
        self.lbl_realtime_log = QLabel(f"📟 {tr('realtime_log')}")
        self.lbl_realtime_log.setStyleSheet("font-size: 14px; font-weight: 700; color: #FFFFFF;")
        
        self.btn_clear_log = QPushButton(tr("clear"))
        self.btn_clear_log.clicked.connect(lambda: self.txt_log.clear())

        self.btn_abort_flash = QPushButton(f"🛑 {tr('btn_stop_flash')}")
        self.btn_abort_flash.setProperty("class", "btn-danger")
        self.btn_abort_flash.setEnabled(False)
        self.btn_abort_flash.clicked.connect(self.abort_flashing)

        log_header.addWidget(self.lbl_realtime_log)
        log_header.addStretch()
        log_header.addWidget(self.btn_clear_log)
        log_header.addWidget(self.btn_abort_flash)
        main_layout.addLayout(log_header)

        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        main_layout.addWidget(self.progress_bar)

        # Log text area
        self.txt_log = QPlainTextEdit()
        self.txt_log.setReadOnly(True)
        self.txt_log.setMaximumBlockCount(4000)
        self.txt_log.setStyleSheet("""
            background-color: #0A0A10;
            color: #38BDF8;
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            font-family: monospace;
            font-size: 11px;
        """)
        main_layout.addWidget(self.txt_log, 1)

    def setup_single_partition_ui(self):
        vbox = QVBoxLayout(self.tab_single)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(12)

        # File Chooser
        file_box = QHBoxLayout()
        self.lbl_select_img = QLabel(tr("select_image_file"))
        self.lbl_select_img.setStyleSheet("font-weight: 600;")
        self.txt_single_img = QLineEdit()
        self.txt_single_img.setPlaceholderText("/path/to/boot.img")
        self.btn_browse_img = QPushButton(tr("browse"))
        self.btn_browse_img.clicked.connect(self.browse_single_image)

        file_box.addWidget(self.lbl_select_img)
        file_box.addWidget(self.txt_single_img, 1)
        file_box.addWidget(self.btn_browse_img)
        vbox.addLayout(file_box)

        # Target Partition selection
        part_box = QHBoxLayout()
        self.lbl_target_part = QLabel(tr("target_partition"))
        self.lbl_target_part.setStyleSheet("font-weight: 600;")
        self.cmb_single_part = QComboBox()
        self.cmb_single_part.addItems(FastbootManager.COMMON_PARTITIONS)
        self.cmb_single_part.setEditable(True)

        self.chk_vbmeta_flags = QCheckBox(tr("vbmeta_disable_flags"))
        self.chk_vbmeta_flags.setChecked(True)

        part_box.addWidget(self.lbl_target_part)
        part_box.addWidget(self.cmb_single_part, 1)
        part_box.addWidget(self.chk_vbmeta_flags)
        vbox.addLayout(part_box)

        # Flash Action Button
        action_box = QHBoxLayout()
        self.btn_flash_single = QPushButton(f"⚡ {tr('btn_flash_partition')}")
        self.btn_flash_single.setProperty("class", "btn-primary")
        self.btn_flash_single.clicked.connect(self.start_single_partition_flash)
        action_box.addStretch()
        action_box.addWidget(self.btn_flash_single)
        vbox.addLayout(action_box)
        vbox.addStretch()

    def browse_single_image(self):
        path, _ = QFileDialog.getOpenFileName(self, tr("select_image_file"), "", "Image Files (*.img *.bin);;All Files (*)")
        if path:
            self.txt_single_img.setText(path)
            # Auto detect partition from filename
            base = os.path.basename(path).lower()
            for part in FastbootManager.COMMON_PARTITIONS:
                if part in base:
                    idx = self.cmb_single_part.findText(part)
                    if idx >= 0:
                        self.cmb_single_part.setCurrentIndex(idx)
                    break

    def setup_full_rom_ui(self):
        vbox = QVBoxLayout(self.tab_rom)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(12)

        # ROM Directory Chooser
        rom_box = QHBoxLayout()
        self.lbl_rom_dir = QLabel(tr("rom_directory"))
        self.lbl_rom_dir.setStyleSheet("font-weight: 600;")
        self.txt_rom_dir = QLineEdit()
        self.txt_rom_dir.setPlaceholderText("/path/to/extracted_fastboot_rom/")
        self.btn_browse_rom = QPushButton(tr("browse"))
        self.btn_browse_rom.clicked.connect(self.browse_rom_dir)

        rom_box.addWidget(self.lbl_rom_dir)
        rom_box.addWidget(self.txt_rom_dir, 1)
        rom_box.addWidget(self.btn_browse_rom)
        vbox.addLayout(rom_box)

        # Flashing Modes (Mi Flash alternative choices)
        mode_card = QFrame()
        mode_card.setObjectName("FlashModeCard")
        mode_card.setStyleSheet("background-color: rgba(30, 30, 48, 0.6); border-radius: 14px; padding: 10px;")
        mode_layout = QVBoxLayout(mode_card)

        self.lbl_mode_title = QLabel(f"🔒 {tr('flash_mode')}")
        self.lbl_mode_title.setStyleSheet("font-size: 14px; font-weight: 700; color: #FFFFFF;")
        mode_layout.addWidget(self.lbl_mode_title)

        self.radio_group = QButtonGroup(self)

        self.rb_clean_all = QRadioButton(tr("mode_clean_all"))
        self.rb_clean_all.setChecked(True)
        self.rb_clean_all.setStyleSheet("color: #60A5FA; font-weight: 600;")
        self.radio_group.addButton(self.rb_clean_all, 1)
        mode_layout.addWidget(self.rb_clean_all)

        self.rb_clean_keep_data = QRadioButton(tr("mode_clean_keep_data"))
        self.rb_clean_keep_data.setStyleSheet("color: #34D399; font-weight: 600;")
        self.radio_group.addButton(self.rb_clean_keep_data, 2)
        mode_layout.addWidget(self.rb_clean_keep_data)

        self.rb_clean_lock = QRadioButton(tr("mode_clean_lock"))
        self.rb_clean_lock.setStyleSheet("color: #F87171; font-weight: 600;")
        self.radio_group.addButton(self.rb_clean_lock, 3)
        mode_layout.addWidget(self.rb_clean_lock)

        vbox.addWidget(mode_card)

        # Start Button
        btn_bar = QHBoxLayout()
        self.btn_start_rom_flash = QPushButton(f"🚀 {tr('btn_start_flash')}")
        self.btn_start_rom_flash.setProperty("class", "btn-primary")
        self.btn_start_rom_flash.clicked.connect(self.start_full_rom_flash)
        btn_bar.addStretch()
        btn_bar.addWidget(self.btn_start_rom_flash)
        vbox.addLayout(btn_bar)
        vbox.addStretch()

    def browse_rom_dir(self):
        d = QFileDialog.getExistingDirectory(self, tr("rom_directory"))
        if d:
            self.txt_rom_dir.setText(d)
            # Try parsing scripts immediately for advance mode
            self.auto_load_advance_mode(d)

    def setup_advance_mode_ui(self):
        vbox = QVBoxLayout(self.tab_advance)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(10)

        self.lbl_adv_note = QLabel(tr("advance_mode_note"))
        self.lbl_adv_note.setWordWrap(True)
        self.lbl_adv_note.setStyleSheet("color: #FBBF24; font-size: 13px; font-weight: 500;")
        vbox.addWidget(self.lbl_adv_note)

        # Toolbar
        tool_box = QHBoxLayout()
        self.btn_parse_script = QPushButton(f"🔍 {tr('btn_parse_script')}")
        self.btn_parse_script.clicked.connect(lambda: self.auto_load_advance_mode(self.txt_rom_dir.text().strip()))

        self.btn_check_all = QPushButton("Select All")
        self.btn_check_all.clicked.connect(lambda: self.set_all_advance_checks(True))

        self.btn_uncheck_dangerous = QPushButton("Uncheck Dangerous Partitions")
        self.btn_uncheck_dangerous.clicked.connect(self.uncheck_dangerous_partitions)

        tool_box.addWidget(self.btn_parse_script)
        tool_box.addWidget(self.btn_check_all)
        tool_box.addWidget(self.btn_uncheck_dangerous)
        tool_box.addStretch()
        vbox.addLayout(tool_box)

        # Checklist Table
        self.table_advance = QTableWidget()
        self.table_advance.setColumnCount(4)
        self.table_advance.setHorizontalHeaderLabels([
            tr("col_select"),
            tr("target_partition"),
            "Image File",
            "Command Preview"
        ])
        self.table_advance.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        self.table_advance.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        self.table_advance.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.ResizeToContents)
        self.table_advance.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeMode.Stretch)
        self.table_advance.verticalHeader().setVisible(False)
        self.table_advance.itemChanged.connect(self.on_advance_item_checked)
        vbox.addWidget(self.table_advance)

    def auto_load_advance_mode(self, rom_dir: str):
        if not rom_dir or not os.path.isdir(rom_dir):
            return

        is_win = (os.name == "nt")
        script = os.path.join(rom_dir, "flash_all.bat" if is_win else "flash_all.sh")
        if not os.path.exists(script):
            # Check without extension
            candidates = ["flash_all.sh", "flash_all.bat"]
            for c in candidates:
                cand = os.path.join(rom_dir, c)
                if os.path.exists(cand):
                    script = cand
                    break

        items = self.fastboot.parse_rom_script(script)
        self.advance_items = items
        self.populate_advance_table(items)

    def populate_advance_table(self, items: List[FlashCommandItem]):
        self.table_advance.blockSignals(True)
        self.table_advance.setRowCount(len(items))

        for row, item in enumerate(items):
            chk = QTableWidgetItem()
            chk.setFlags(Qt.ItemFlag.ItemIsUserCheckable | Qt.ItemFlag.ItemIsEnabled)
            chk.setCheckState(Qt.CheckState.Checked if item.enabled else Qt.CheckState.Unchecked)
            self.table_advance.setItem(row, 0, chk)

            part_item = QTableWidgetItem(item.partition)
            if item.is_dangerous:
                part_item.setForeground(Qt.GlobalColor.red)
                part_item.setText(f"⚠️ {item.partition} (Dangerous)")
            self.table_advance.setItem(row, 1, part_item)

            self.table_advance.setItem(row, 2, QTableWidgetItem(item.image_file))
            self.table_advance.setItem(row, 3, QTableWidgetItem(item.raw_command))

        self.table_advance.blockSignals(False)

    def on_advance_item_checked(self, item: QTableWidgetItem):
        if item.column() == 0 and item.row() < len(self.advance_items):
            self.advance_items[item.row()].enabled = (item.checkState() == Qt.CheckState.Checked)

    def set_all_advance_checks(self, checked: bool):
        for row in range(self.table_advance.rowCount()):
            chk = self.table_advance.item(row, 0)
            if chk:
                chk.setCheckState(Qt.CheckState.Checked if checked else Qt.CheckState.Unchecked)

    def uncheck_dangerous_partitions(self):
        for row, item in enumerate(self.advance_items):
            if item.is_dangerous:
                chk = self.table_advance.item(row, 0)
                if chk:
                    chk.setCheckState(Qt.CheckState.Unchecked)

    def start_single_partition_flash(self):
        img = self.txt_single_img.text().strip()
        part = self.cmb_single_part.currentText().strip()
        if not img or not os.path.exists(img):
            QMessageBox.warning(self, tr("warning"), "Please choose a valid image file (.img).")
            return
        if not part:
            QMessageBox.warning(self, tr("warning"), "Please enter target partition.")
            return

        cmds = self.fastboot.flash_single_partition(
            partition=part,
            image_path=img,
            disable_vbmeta_verity=self.chk_vbmeta_flags.isChecked(),
            serial=self.current_serial
        )
        self.execute_commands_worker(cmds)

    def start_full_rom_flash(self):
        rom_dir = self.txt_rom_dir.text().strip()
        if not rom_dir or not os.path.isdir(rom_dir):
            QMessageBox.warning(self, tr("warning"), "Please select a valid Fastboot ROM directory.")
            return

        mode_id = self.radio_group.checkedId()
        mode_str = "clean_all"
        if mode_id == 2: mode_str = "clean_keep_data"
        elif mode_id == 3: mode_str = "clean_lock"

        # Lock Bootloader Warning Check
        if mode_str == "clean_lock":
            res = QMessageBox.critical(
                self,
                tr("warning"),
                tr("flash_lock_warning_msg"),
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                QMessageBox.StandardButton.No
            )
            if res != QMessageBox.StandardButton.Yes:
                return

        # General confirmation
        res = QMessageBox.question(
            self,
            tr("flash_warning_title"),
            tr("flash_warning_msg", mode=mode_str),
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if res != QMessageBox.StandardButton.Yes:
            return

        # Check if advance items should be used
        advance_used = self.advance_items if (self.tabs.currentIndex() == 2 and self.advance_items) else None

        cmds = self.fastboot.generate_flasher_commands(
            rom_dir=rom_dir,
            mode=mode_str,
            advance_items=advance_used,
            serial=self.current_serial
        )

        if not cmds:
            QMessageBox.critical(self, tr("error"), "No valid flashing commands generated for this directory.")
            return

        self.execute_commands_worker(cmds, cwd=rom_dir)

    def execute_commands_worker(self, commands: List[str], cwd: Optional[str] = None):
        self.txt_log.clear()
        self.progress_bar.setVisible(True)
        self.progress_bar.setRange(0, len(commands))
        self.progress_bar.setValue(0)
        self.btn_abort_flash.setEnabled(True)
        self.btn_start_rom_flash.setEnabled(False)
        self.btn_flash_single.setEnabled(False)

        self.worker = FastbootExecutionWorker(commands, cwd)
        self.worker.log_line.connect(self.append_log)
        self.worker.progress_changed.connect(lambda cur, total: self.progress_bar.setValue(cur))
        self.worker.finished_status.connect(self.on_flash_completed)
        self.worker.start()

    def append_log(self, text: str):
        self.txt_log.appendPlainText(text)

    def abort_flashing(self):
        if getattr(self, "worker", None):
            self.worker.abort()
            self.btn_abort_flash.setEnabled(False)

    def on_flash_completed(self, success: bool, msg: str):
        self.btn_abort_flash.setEnabled(False)
        self.btn_start_rom_flash.setEnabled(True)
        self.btn_flash_single.setEnabled(True)
        self.progress_bar.setVisible(False)

        if success:
            QMessageBox.information(self, tr("success"), f"Flashing finished successfully!\n{msg}")
        else:
            QMessageBox.critical(self, tr("error"), f"Flashing halted or failed:\n{msg}")

    def update_translations(self):
        self.tabs.setTabText(0, tr("tab_single_partition"))
        self.tabs.setTabText(1, tr("tab_full_rom"))
        self.tabs.setTabText(2, tr("tab_advance_mode"))
        self.lbl_realtime_log.setText(f"📟 {tr('realtime_log')}")
        self.btn_clear_log.setText(tr("clear"))
        self.btn_abort_flash.setText(f"🛑 {tr('btn_stop_flash')}")
        self.lbl_select_img.setText(tr("select_image_file"))
        self.btn_browse_img.setText(tr("browse"))
        self.lbl_target_part.setText(tr("target_partition"))
        self.chk_vbmeta_flags.setText(tr("vbmeta_disable_flags"))
        self.btn_flash_single.setText(f"⚡ {tr('btn_flash_partition')}")
        self.lbl_rom_dir.setText(tr("rom_directory"))
        self.btn_browse_rom.setText(tr("browse"))
        self.lbl_mode_title.setText(f"🔒 {tr('flash_mode')}")
        self.rb_clean_all.setText(tr("mode_clean_all"))
        self.rb_clean_keep_data.setText(tr("mode_clean_keep_data"))
        self.rb_clean_lock.setText(tr("mode_clean_lock"))
        self.btn_start_rom_flash.setText(f"🚀 {tr('btn_start_flash')}")
        self.lbl_adv_note.setText(tr("advance_mode_note"))
        self.btn_parse_script.setText(f"🔍 {tr('btn_parse_script')}")
