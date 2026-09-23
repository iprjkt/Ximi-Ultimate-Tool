"""
ADB View for Ximi Ultimate Tool.
Features:
- Auto Debloater (checklist with preset database)
- Manual Debloater (package search and individual operations)
- Live Logcat streaming & search
- Kernel dmesg viewer
"""

import os
from typing import Dict, List, Optional
from PyQt6.QtCore import Qt, QThread, pyqtSignal
from PyQt6.QtWidgets import (
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
    QTabWidget,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget
)
from app.core.adb_manager import ADBManager, BLOATWARE_DATABASE, BloatwareItem
from app.ui.i18n import tr

class DebloatWorker(QThread):
    progress = pyqtSignal(int, int, str)
    finished = pyqtSignal(list)

    def __init__(self, adb: ADBManager, action: str, packages: List[str], serial: Optional[str] = None):
        super().__init__()
        self.adb = adb
        self.action = action  # "uninstall", "restore", "disable", "enable"
        self.packages = packages
        self.serial = serial

    def run(self):
        results = []
        total = len(self.packages)
        for idx, pkg in enumerate(self.packages):
            if self.action == "uninstall":
                ok, msg = self.adb.uninstall_package(pkg, self.serial)
            elif self.action == "restore":
                ok, msg = self.adb.restore_package(pkg, self.serial)
            elif self.action == "disable":
                ok, msg = self.adb.disable_package(pkg, self.serial)
            elif self.action == "enable":
                ok, msg = self.adb.enable_package(pkg, self.serial)
            else:
                ok, msg = False, "Unknown action"

            results.append((pkg, ok, msg))
            self.progress.emit(idx + 1, total, f"{self.action.capitalize()}: {pkg} ({msg})")

        self.finished.emit(results)


class ADBView(QWidget):
    def __init__(self, adb_manager: ADBManager, parent=None):
        super().__init__(parent)
        self.adb = adb_manager
        self.current_serial: Optional[str] = None
        self.debloat_worker: Optional[DebloatWorker] = None
        self.init_ui()

    def set_serial(self, serial: Optional[str]):
        self.current_serial = serial

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(14)

        # Tab Widget
        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        # 1. Auto Debloater Tab
        self.tab_auto = QWidget()
        self.setup_auto_debloater()
        self.tabs.addTab(self.tab_auto, tr("tab_debloater_auto"))

        # 2. Manual Debloater Tab
        self.tab_manual = QWidget()
        self.setup_manual_debloater()
        self.tabs.addTab(self.tab_manual, tr("tab_debloater_manual"))

        # 3. Logcat Tab
        self.tab_logcat = QWidget()
        self.setup_logcat()
        self.tabs.addTab(self.tab_logcat, tr("tab_logcat"))

        # 4. dmesg Tab
        self.tab_dmesg = QWidget()
        self.setup_dmesg()
        self.tabs.addTab(self.tab_dmesg, tr("tab_dmesg"))

    def setup_auto_debloater(self):
        vbox = QVBoxLayout(self.tab_auto)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(10)

        # Description
        self.lbl_auto_desc = QLabel(tr("debloater_desc"))
        self.lbl_auto_desc.setWordWrap(True)
        self.lbl_auto_desc.setStyleSheet("color: #9CA3AF; font-size: 13px;")
        vbox.addWidget(self.lbl_auto_desc)

        # Action Buttons Toolbar
        btn_box = QHBoxLayout()
        self.btn_scan_auto = QPushButton(f"🔍 {tr('btn_scan_bloatware')}")
        self.btn_scan_auto.clicked.connect(self.scan_installed_bloatware)

        self.btn_uninstall_auto = QPushButton(f"🗑️ {tr('btn_uninstall_selected')}")
        self.btn_uninstall_auto.setProperty("class", "btn-danger")
        self.btn_uninstall_auto.clicked.connect(lambda: self.run_batch_debloat("uninstall"))

        self.btn_restore_auto = QPushButton(f"🔄 {tr('btn_restore_selected')}")
        self.btn_restore_auto.setProperty("class", "btn-success")
        self.btn_restore_auto.clicked.connect(lambda: self.run_batch_debloat("restore"))

        self.btn_disable_auto = QPushButton(tr("btn_disable_selected"))
        self.btn_disable_auto.clicked.connect(lambda: self.run_batch_debloat("disable"))

        self.btn_enable_auto = QPushButton(tr("btn_enable_selected"))
        self.btn_enable_auto.clicked.connect(lambda: self.run_batch_debloat("enable"))

        btn_box.addWidget(self.btn_scan_auto)
        btn_box.addWidget(self.btn_uninstall_auto)
        btn_box.addWidget(self.btn_restore_auto)
        btn_box.addWidget(self.btn_disable_auto)
        btn_box.addWidget(self.btn_enable_auto)
        btn_box.addStretch()
        vbox.addLayout(btn_box)

        # Progress Bar
        self.progress_auto = QProgressBar()
        self.progress_auto.setVisible(False)
        vbox.addWidget(self.progress_auto)

        # Table of Bloatware
        self.table_auto = QTableWidget()
        self.table_auto.setColumnCount(5)
        self.table_auto.setHorizontalHeaderLabels([
            tr("col_select"),
            tr("col_app_name"),
            tr("col_package"),
            tr("col_category"),
            tr("col_risk")
        ])
        self.table_auto.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        self.table_auto.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        self.table_auto.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)
        self.table_auto.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents)
        self.table_auto.horizontalHeader().setSectionResizeMode(4, QHeaderView.ResizeMode.ResizeToContents)
        self.table_auto.verticalHeader().setVisible(False)
        vbox.addWidget(self.table_auto)

        self.populate_auto_table(BLOATWARE_DATABASE)

    def populate_auto_table(self, items: List[BloatwareItem]):
        self.table_auto.setRowCount(len(items))
        for row, item in enumerate(items):
            # Checkbox item
            chk = QTableWidgetItem()
            chk.setFlags(Qt.ItemFlag.ItemIsUserCheckable | Qt.ItemFlag.ItemIsEnabled)
            chk.setCheckState(Qt.CheckState.Checked if item.risk == "Safe" else Qt.CheckState.Unchecked)
            self.table_auto.setItem(row, 0, chk)

            # Name
            self.table_auto.setItem(row, 1, QTableWidgetItem(item.name))

            # Package
            self.table_auto.setItem(row, 2, QTableWidgetItem(item.package))

            # Category
            self.table_auto.setItem(row, 3, QTableWidgetItem(item.category))

            # Risk
            risk_item = QTableWidgetItem(item.risk)
            if item.risk == "Safe":
                risk_item.setForeground(Qt.GlobalColor.green)
            elif item.risk == "Caution":
                risk_item.setForeground(Qt.GlobalColor.yellow)
            else:
                risk_item.setForeground(Qt.GlobalColor.cyan)
            self.table_auto.setItem(row, 4, risk_item)

    def scan_installed_bloatware(self):
        installed = set(self.adb.get_installed_packages("all", self.current_serial))
        filtered = [item for item in BLOATWARE_DATABASE if item.package in installed]
        if not filtered:
            # Device might not have scanned bloatware or disconnected
            QMessageBox.information(self, tr("info"), f"Installed packages scanned: {len(installed)}. No known bloatware left or device offline.")
            self.populate_auto_table(BLOATWARE_DATABASE)
        else:
            self.populate_auto_table(filtered)

    def run_batch_debloat(self, action: str):
        selected_pkgs = []
        for row in range(self.table_auto.rowCount()):
            chk = self.table_auto.item(row, 0)
            if chk and chk.checkState() == Qt.CheckState.Checked:
                pkg_item = self.table_auto.item(row, 2)
                if pkg_item:
                    selected_pkgs.append(pkg_item.text())

        if not selected_pkgs:
            QMessageBox.warning(self, tr("warning"), "No packages selected!")
            return

        self.progress_auto.setVisible(True)
        self.progress_auto.setRange(0, len(selected_pkgs))
        self.progress_auto.setValue(0)

        self.debloat_worker = DebloatWorker(self.adb, action, selected_pkgs, self.current_serial)
        self.debloat_worker.progress.connect(lambda cur, total, msg: self.progress_auto.setValue(cur))
        self.debloat_worker.finished.connect(self.on_debloat_finished)
        self.debloat_worker.start()

    def on_debloat_finished(self, results):
        self.progress_auto.setVisible(False)
        succ = sum(1 for _, ok, _ in results if ok)
        QMessageBox.information(self, tr("success"), f"Processed {len(results)} packages.\nSuccessful: {succ}\nFailed: {len(results) - succ}")

    def setup_manual_debloater(self):
        vbox = QVBoxLayout(self.tab_manual)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(10)

        # Single Package Direct Action
        direct_box = QHBoxLayout()
        self.txt_manual_pkg = QLineEdit()
        self.txt_manual_pkg.setPlaceholderText(tr("manual_pkg_input"))

        self.btn_man_uninstall = QPushButton(tr("btn_uninstall_pkg"))
        self.btn_man_uninstall.setProperty("class", "btn-danger")
        self.btn_man_uninstall.clicked.connect(lambda: self.exec_single_pkg("uninstall"))

        self.btn_man_restore = QPushButton(tr("btn_restore_pkg"))
        self.btn_man_restore.setProperty("class", "btn-success")
        self.btn_man_restore.clicked.connect(lambda: self.exec_single_pkg("restore"))

        self.btn_man_disable = QPushButton(tr("btn_disable_pkg"))
        self.btn_man_disable.clicked.connect(lambda: self.exec_single_pkg("disable"))

        self.btn_man_enable = QPushButton(tr("btn_enable_pkg"))
        self.btn_man_enable.clicked.connect(lambda: self.exec_single_pkg("enable"))

        direct_box.addWidget(self.txt_manual_pkg, 3)
        direct_box.addWidget(self.btn_man_uninstall)
        direct_box.addWidget(self.btn_man_restore)
        direct_box.addWidget(self.btn_man_disable)
        direct_box.addWidget(self.btn_man_enable)
        vbox.addLayout(direct_box)

        # Filter and Search
        filter_box = QHBoxLayout()
        self.cmb_filter = QComboBox()
        self.cmb_filter.addItems([
            tr("filter_all"),
            tr("filter_third_party"),
            tr("filter_system"),
            tr("filter_disabled")
        ])
        self.cmb_filter.currentIndexChanged.connect(self.fetch_packages_list)

        self.txt_search_pkg = QLineEdit()
        self.txt_search_pkg.setPlaceholderText(tr("search"))
        self.txt_search_pkg.textChanged.connect(self.filter_manual_table)

        self.btn_fetch_pkgs = QPushButton(f"🔄 {tr('btn_fetch_pkgs')}")
        self.btn_fetch_pkgs.clicked.connect(self.fetch_packages_list)

        filter_box.addWidget(self.cmb_filter)
        filter_box.addWidget(self.txt_search_pkg, 2)
        filter_box.addWidget(self.btn_fetch_pkgs)
        vbox.addLayout(filter_box)

        # Table of All Installed Packages
        self.table_manual = QTableWidget()
        self.table_manual.setColumnCount(2)
        self.table_manual.setHorizontalHeaderLabels(["No.", tr("col_package")])
        self.table_manual.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        self.table_manual.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.Stretch)
        self.table_manual.verticalHeader().setVisible(False)
        self.table_manual.itemClicked.connect(self.on_manual_item_clicked)
        vbox.addWidget(self.table_manual)

    def on_manual_item_clicked(self, item: QTableWidgetItem):
        if item.column() == 1:
            self.txt_manual_pkg.setText(item.text())

    def fetch_packages_list(self):
        idx = self.cmb_filter.currentIndex()
        mode = "all"
        if idx == 1: mode = "3rd"
        elif idx == 2: mode = "system"
        elif idx == 3: mode = "disabled"

        pkgs = self.adb.get_installed_packages(mode, self.current_serial)
        self.all_loaded_pkgs = pkgs
        self.filter_manual_table(self.txt_search_pkg.text())

    def filter_manual_table(self, query: str):
        query = query.lower().strip()
        matched = [p for p in getattr(self, "all_loaded_pkgs", []) if query in p.lower()]
        self.table_manual.setRowCount(len(matched))
        for row, pkg in enumerate(matched):
            self.table_manual.setItem(row, 0, QTableWidgetItem(str(row + 1)))
            self.table_manual.setItem(row, 1, QTableWidgetItem(pkg))

    def exec_single_pkg(self, action: str):
        pkg = self.txt_manual_pkg.text().strip()
        if not pkg:
            QMessageBox.warning(self, tr("warning"), "Please enter or select a package name.")
            return

        if action == "uninstall":
            ok, msg = self.adb.uninstall_package(pkg, self.current_serial)
        elif action == "restore":
            ok, msg = self.adb.restore_package(pkg, self.current_serial)
        elif action == "disable":
            ok, msg = self.adb.disable_package(pkg, self.current_serial)
        elif action == "enable":
            ok, msg = self.adb.enable_package(pkg, self.current_serial)
        else:
            ok, msg = False, "Unknown action"

        if ok:
            QMessageBox.information(self, tr("success"), f"{action.capitalize()} {pkg}: {msg}")
            self.fetch_packages_list()
        else:
            QMessageBox.critical(self, tr("error"), f"Failed to {action} {pkg}: {msg}")

    def setup_logcat(self):
        vbox = QVBoxLayout(self.tab_logcat)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(10)

        tool_box = QHBoxLayout()
        self.btn_log_start = QPushButton(f"▶ {tr('log_start')}")
        self.btn_log_start.setProperty("class", "btn-primary")
        self.btn_log_start.clicked.connect(self.toggle_logcat)

        self.btn_log_clear = QPushButton(f"🗑 {tr('log_clear')}")
        self.btn_log_clear.clicked.connect(lambda: self.txt_logcat.clear())

        self.btn_log_export = QPushButton(f"💾 {tr('log_export')}")
        self.btn_log_export.clicked.connect(lambda: self.export_log(self.txt_logcat.toPlainText(), "logcat.txt"))

        self.txt_log_filter = QLineEdit()
        self.txt_log_filter.setPlaceholderText(tr("log_search_placeholder"))

        tool_box.addWidget(self.btn_log_start)
        tool_box.addWidget(self.btn_log_clear)
        tool_box.addWidget(self.btn_log_export)
        tool_box.addWidget(self.txt_log_filter)
        vbox.addLayout(tool_box)

        self.txt_logcat = QPlainTextEdit()
        self.txt_logcat.setReadOnly(True)
        self.txt_logcat.setMaximumBlockCount(5000)
        self.txt_logcat.setStyleSheet("background-color: #0A0A10; color: #10B981; font-family: monospace; font-size: 11px;")
        vbox.addWidget(self.txt_logcat)

    def toggle_logcat(self):
        if getattr(self, "is_logcat_running", False):
            self.adb.stop_log_stream()
            self.is_logcat_running = False
            self.btn_log_start.setText(f"▶ {tr('log_start')}")
        else:
            filter_val = self.txt_log_filter.text().strip()
            worker = self.adb.start_log_stream("logcat", self.current_serial, filter_val)
            worker.new_line.connect(self.append_logcat_line)
            self.is_logcat_running = True
            self.btn_log_start.setText(f"⏹ {tr('log_stop')}")

    def append_logcat_line(self, line: str):
        self.txt_logcat.appendPlainText(line)

    def setup_dmesg(self):
        vbox = QVBoxLayout(self.tab_dmesg)
        vbox.setContentsMargins(12, 12, 12, 12)
        vbox.setSpacing(10)

        tool_box = QHBoxLayout()
        self.btn_dmesg_fetch = QPushButton(f"🔄 {tr('log_start')}")
        self.btn_dmesg_fetch.setProperty("class", "btn-primary")
        self.btn_dmesg_fetch.clicked.connect(self.fetch_dmesg)

        self.btn_dmesg_clear = QPushButton(f"🗑 {tr('log_clear')}")
        self.btn_dmesg_clear.clicked.connect(lambda: self.txt_dmesg.clear())

        self.btn_dmesg_export = QPushButton(f"💾 {tr('log_export')}")
        self.btn_dmesg_export.clicked.connect(lambda: self.export_log(self.txt_dmesg.toPlainText(), "dmesg.txt"))

        tool_box.addWidget(self.btn_dmesg_fetch)
        tool_box.addWidget(self.btn_dmesg_clear)
        tool_box.addWidget(self.btn_dmesg_export)
        tool_box.addStretch()
        vbox.addLayout(tool_box)

        self.txt_dmesg = QPlainTextEdit()
        self.txt_dmesg.setReadOnly(True)
        self.txt_dmesg.setStyleSheet("background-color: #0A0A10; color: #38BDF8; font-family: monospace; font-size: 11px;")
        vbox.addWidget(self.txt_dmesg)

    def fetch_dmesg(self):
        code, out, err = self.adb.run_cmd(["shell", "dmesg"], timeout=10)
        if code == 0:
            self.txt_dmesg.setPlainText(out)
        else:
            # Fallback to root dmesg
            code2, out2, err2 = self.adb.run_cmd(["shell", "su", "-c", "dmesg"], timeout=10)
            self.txt_dmesg.setPlainText(out2 if code2 == 0 else (err or err2 or "Failed to read dmesg"))

    def export_log(self, content: str, default_name: str):
        path, _ = QFileDialog.getSaveFileName(self, tr("log_export"), default_name, "Text Files (*.txt);;Log Files (*.log)")
        if path:
            try:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(content)
                QMessageBox.information(self, tr("success"), f"Log saved to: {path}")
            except Exception as e:
                QMessageBox.critical(self, tr("error"), str(e))

    def update_translations(self):
        self.tabs.setTabText(0, tr("tab_debloater_auto"))
        self.tabs.setTabText(1, tr("tab_debloater_manual"))
        self.tabs.setTabText(2, tr("tab_logcat"))
        self.tabs.setTabText(3, tr("tab_dmesg"))
        self.lbl_auto_desc.setText(tr("debloater_desc"))
        self.btn_scan_auto.setText(f"🔍 {tr('btn_scan_bloatware')}")
        self.btn_uninstall_auto.setText(f"🗑️ {tr('btn_uninstall_selected')}")
        self.btn_restore_auto.setText(f"🔄 {tr('btn_restore_selected')}")
        self.btn_disable_auto.setText(tr("btn_disable_selected"))
        self.btn_enable_auto.setText(tr("btn_enable_selected"))
        self.txt_manual_pkg.setPlaceholderText(tr("manual_pkg_input"))
        self.btn_man_uninstall.setText(tr("btn_uninstall_pkg"))
        self.btn_man_restore.setText(tr("btn_restore_pkg"))
        self.btn_man_disable.setText(tr("btn_disable_pkg"))
        self.btn_man_enable.setText(tr("btn_enable_pkg"))
        self.txt_search_pkg.setPlaceholderText(tr("search"))
        self.btn_fetch_pkgs.setText(f"🔄 {tr('btn_fetch_pkgs')}")
