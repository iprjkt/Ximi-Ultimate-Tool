"""
Android & PC Dual-Panel Explorer (MT Manager Style) for Ximi Ultimate Tool.
Features:
- Left Panel: Local PC File System
- Right Panel: Android Device File System (with Root 'su' mode)
- Seamless Transfer: PC ➜ Android (Push) and Android ➜ PC (Pull)
- In-App text editor for both PC and Android files
- Complete file operations on both panels (create, rename, delete, extract ZIP)
"""

import os
from pathlib import Path
from typing import List, Optional
from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QCheckBox,
    QDialog,
    QDialogButtonBox,
    QFileDialog,
    QFrame,
    QHBoxLayout,
    QHeaderView,
    QInputDialog,
    QLabel,
    QLineEdit,
    QMessageBox,
    QPlainTextEdit,
    QPushButton,
    QSplitter,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget
)
from app.core.explorer_manager import ExplorerManager, FileItem
from app.ui.i18n import tr

class FileEditorDialog(QDialog):
    """Text editor dialog for editing PC or Android files."""
    def __init__(self, file_path: str, initial_content: str, title_prefix: str = "Edit", parent=None):
        super().__init__(parent)
        self.file_path = file_path
        self.setWindowTitle(f"{title_prefix} - {os.path.basename(file_path)}")
        self.resize(780, 520)

        layout = QVBoxLayout(self)
        self.lbl_path = QLabel(f"Path: {file_path}")
        self.lbl_path.setStyleSheet("color: #9CA3AF; font-size: 12px;")
        layout.addWidget(self.lbl_path)

        self.txt_editor = QPlainTextEdit()
        self.txt_editor.setPlainText(initial_content)
        self.txt_editor.setStyleSheet("""
            background-color: #0A0A10;
            color: #F8FAFC;
            font-family: 'Fira Code', 'Roboto Mono', 'Courier New', monospace;
            font-size: 12px;
            border-radius: 8px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            padding: 8px;
        """)
        layout.addWidget(self.txt_editor)

        btn_box = QDialogButtonBox(QDialogButtonBox.StandardButton.Save | QDialogButtonBox.StandardButton.Cancel)
        btn_box.accepted.connect(self.accept)
        btn_box.rejected.connect(self.reject)
        layout.addWidget(btn_box)

    def get_content(self) -> str:
        return self.txt_editor.toPlainText()


class ExplorerView(QWidget):
    def __init__(self, explorer_manager: ExplorerManager, parent=None):
        super().__init__(parent)
        self.explorer = explorer_manager
        self.current_serial: Optional[str] = None
        
        # State paths
        self.pc_path = str(Path.home().resolve())
        self.android_path = "/sdcard"
        
        # Cached items
        self.pc_items: List[FileItem] = []
        self.android_items: List[FileItem] = []

        self.init_ui()

    def set_serial(self, serial: Optional[str]):
        self.current_serial = serial
        self.refresh_android_panel()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(12, 12, 12, 12)
        main_layout.setSpacing(10)

        # 1. Top Global Actions & Transfer Bar (MT Manager Bridge)
        bridge_card = QFrame()
        bridge_card.setObjectName("ExplorerBridgeCard")
        bridge_card.setStyleSheet("""
            #ExplorerBridgeCard {
                background-color: rgba(30, 27, 60, 0.7);
                border: 1px solid rgba(139, 92, 246, 0.25);
                border-radius: 14px;
                padding: 4px;
            }
        """)
        bridge_layout = QHBoxLayout(bridge_card)
        bridge_layout.setContentsMargins(12, 6, 12, 6)

        self.lbl_pc_title = QLabel(f"🖥️ {tr('panel_pc')}")
        self.lbl_pc_title.setStyleSheet("font-size: 14px; font-weight: 700; color: #60A5FA;")

        # Transfer Buttons
        self.btn_copy_to_android = QPushButton(f" {tr('btn_transfer_to_android')} ")
        self.btn_copy_to_android.setProperty("class", "btn-primary")
        self.btn_copy_to_android.setToolTip("Copy selected item from PC into current Android directory")
        self.btn_copy_to_android.clicked.connect(self.transfer_pc_to_android)

        self.btn_copy_to_pc = QPushButton(f" {tr('btn_transfer_to_pc')} ")
        self.btn_copy_to_pc.setProperty("class", "btn-success")
        self.btn_copy_to_pc.setToolTip("Copy selected item from Android into current PC directory")
        self.btn_copy_to_pc.clicked.connect(self.transfer_android_to_pc)

        self.lbl_android_title = QLabel(f"📱 {tr('panel_android')}")
        self.lbl_android_title.setStyleSheet("font-size: 14px; font-weight: 700; color: #34D399;")

        bridge_layout.addWidget(self.lbl_pc_title)
        bridge_layout.addStretch()
        bridge_layout.addWidget(self.btn_copy_to_android)
        bridge_layout.addSpacing(10)
        bridge_layout.addWidget(self.btn_copy_to_pc)
        bridge_layout.addStretch()
        bridge_layout.addWidget(self.lbl_android_title)

        main_layout.addWidget(bridge_card)

        # 2. Dual Panel Splitter (PC on Left, Android on Right)
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        self.splitter.setStyleSheet("""
            QSplitter::handle {
                background-color: rgba(139, 92, 246, 0.3);
                width: 6px;
                border-radius: 3px;
                margin: 4px;
            }
            QSplitter::handle:hover {
                background-color: #8B5CF6;
            }
        """)

        # Left Panel (PC)
        self.left_panel = self.create_pc_panel()
        # Right Panel (Android)
        self.right_panel = self.create_android_panel()

        self.splitter.addWidget(self.left_panel)
        self.splitter.addWidget(self.right_panel)
        self.splitter.setSizes([550, 550])

        main_layout.addWidget(self.splitter, 1)

        # Initial Refresh
        self.refresh_pc_panel()
        self.refresh_android_panel()

    # ==========================================
    # LEFT PANEL: PC FILE EXPLORER
    # ==========================================
    def create_pc_panel(self) -> QWidget:
        panel = QFrame()
        panel.setObjectName("PCPanelFrame")
        panel.setStyleSheet("""
            #PCPanelFrame {
                background-color: rgba(22, 22, 35, 0.65);
                border: 1px solid rgba(255, 255, 255, 0.08);
                border-radius: 16px;
            }
        """)
        vbox = QVBoxLayout(panel)
        vbox.setContentsMargins(10, 10, 10, 10)
        vbox.setSpacing(8)

        # PC Navigation Bar
        nav_box = QHBoxLayout()
        self.btn_pc_up = QPushButton(f"⬆ {tr('btn_parent_dir')}")
        self.btn_pc_up.clicked.connect(self.pc_navigate_up)

        self.btn_pc_home = QPushButton(f"🏠 {tr('btn_home_dir')}")
        self.btn_pc_home.clicked.connect(lambda: self.pc_navigate_to(str(Path.home().resolve())))

        self.txt_pc_path = QLineEdit(self.pc_path)
        self.txt_pc_path.returnPressed.connect(lambda: self.pc_navigate_to(self.txt_pc_path.text().strip()))

        self.btn_pc_refresh = QPushButton("🔄")
        self.btn_pc_refresh.clicked.connect(self.refresh_pc_panel)

        nav_box.addWidget(self.btn_pc_up)
        nav_box.addWidget(self.btn_pc_home)
        nav_box.addWidget(self.txt_pc_path, 1)
        nav_box.addWidget(self.btn_pc_refresh)
        vbox.addLayout(nav_box)

        # PC Action Toolbar
        action_box = QHBoxLayout()
        action_box.setSpacing(6)
        btn_style = "padding: 5px 10px; font-size: 12px; font-weight: 600;"

        self.btn_pc_new_folder = QPushButton("📁 Folder")
        self.btn_pc_new_folder.setStyleSheet(btn_style)
        self.btn_pc_new_folder.clicked.connect(self.on_pc_new_folder)

        self.btn_pc_new_file = QPushButton("📄 File")
        self.btn_pc_new_file.setStyleSheet(btn_style)
        self.btn_pc_new_file.clicked.connect(self.on_pc_new_file)

        self.btn_pc_edit = QPushButton("✏️ Edit")
        self.btn_pc_edit.setStyleSheet(btn_style)
        self.btn_pc_edit.clicked.connect(self.on_pc_edit_file)

        self.btn_pc_zip = QPushButton("📦 ZIP")
        self.btn_pc_zip.setStyleSheet(btn_style)
        self.btn_pc_zip.clicked.connect(self.on_pc_extract_zip)

        self.btn_pc_rename = QPushButton("🏷️ Rename")
        self.btn_pc_rename.setStyleSheet(btn_style)
        self.btn_pc_rename.clicked.connect(self.on_pc_rename)

        self.btn_pc_delete = QPushButton("🗑️ Delete")
        self.btn_pc_delete.setStyleSheet(btn_style)
        self.btn_pc_delete.setProperty("class", "btn-danger")
        self.btn_pc_delete.clicked.connect(self.on_pc_delete)

        action_box.addWidget(self.btn_pc_new_folder)
        action_box.addWidget(self.btn_pc_new_file)
        action_box.addWidget(self.btn_pc_edit)
        action_box.addWidget(self.btn_pc_zip)
        action_box.addWidget(self.btn_pc_rename)
        action_box.addWidget(self.btn_pc_delete)
        action_box.addStretch()
        vbox.addLayout(action_box)

        # PC Table Widget
        self.table_pc = QTableWidget()
        self.table_pc.setColumnCount(3)
        self.table_pc.setHorizontalHeaderLabels([
            tr("col_file_name"),
            tr("col_file_size"),
            tr("col_file_date")
        ])
        self.table_pc.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.Stretch)
        self.table_pc.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        self.table_pc.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.ResizeToContents)
        self.table_pc.verticalHeader().setVisible(False)
        self.table_pc.cellDoubleClicked.connect(self.on_pc_cell_double_clicked)
        vbox.addWidget(self.table_pc, 1)

        return panel

    def pc_navigate_to(self, path: str):
        if not path:
            path = str(Path.home())
        p = Path(path).expanduser().resolve()
        if p.exists() and p.is_dir():
            self.pc_path = str(p)
            self.txt_pc_path.setText(self.pc_path)
            self.refresh_pc_panel()

    def pc_navigate_up(self):
        parent = Path(self.pc_path).parent
        self.pc_navigate_to(str(parent))

    def refresh_pc_panel(self):
        items = self.explorer.list_local_dir(self.pc_path)
        self.pc_items = items
        self.table_pc.setRowCount(len(items))

        for row, item in enumerate(items):
            icon = "📁 " if item.is_dir else ("🔗 " if item.is_link else "📄 ")
            display_name = f"{icon}{item.name}"
            if item.link_target:
                display_name += f" -> {item.link_target}"

            name_widget = QTableWidgetItem(display_name)
            if item.is_dir:
                name_widget.setForeground(Qt.GlobalColor.cyan)
            self.table_pc.setItem(row, 0, name_widget)
            self.table_pc.setItem(row, 1, QTableWidgetItem(item.size_formatted))
            self.table_pc.setItem(row, 2, QTableWidgetItem(item.date))

    def on_pc_cell_double_clicked(self, row: int, col: int):
        if row < len(self.pc_items):
            item = self.pc_items[row]
            if item.is_dir:
                self.pc_navigate_to(item.path)
            else:
                self.on_pc_edit_file()

    def get_selected_pc_item(self) -> Optional[FileItem]:
        row = self.table_pc.currentRow()
        if 0 <= row < len(self.pc_items):
            return self.pc_items[row]
        return None

    def on_pc_new_folder(self):
        name, ok = QInputDialog.getText(self, tr("dialog_new_folder_title"), tr("dialog_new_folder_msg"))
        if ok and name:
            new_path = os.path.join(self.pc_path, name)
            succ, msg = self.explorer.create_local_folder(new_path)
            if succ:
                self.refresh_pc_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_pc_new_file(self):
        name, ok = QInputDialog.getText(self, tr("dialog_new_file_title"), tr("dialog_new_file_msg"))
        if ok and name:
            new_path = os.path.join(self.pc_path, name)
            succ, msg = self.explorer.create_local_file(new_path)
            if succ:
                self.refresh_pc_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_pc_rename(self):
        item = self.get_selected_pc_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Please select a PC file/folder to rename.")
            return
        new_name, ok = QInputDialog.getText(self, tr("dialog_rename_title"), tr("dialog_rename_msg"), text=item.name)
        if ok and new_name and new_name != item.name:
            new_path = os.path.join(os.path.dirname(item.path), new_name)
            succ, msg = self.explorer.rename_local_item(item.path, new_path)
            if succ:
                self.refresh_pc_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_pc_delete(self):
        item = self.get_selected_pc_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Please select a PC file/folder to delete.")
            return
        res = QMessageBox.question(self, tr("confirm"), tr("confirm_delete_msg", item=item.name), QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
        if res == QMessageBox.StandardButton.Yes:
            succ, msg = self.explorer.delete_local_item(item.path)
            if succ:
                self.refresh_pc_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_pc_extract_zip(self):
        item = self.get_selected_pc_item()
        if not item or not item.name.lower().endswith(".zip"):
            QMessageBox.warning(self, tr("warning"), "Please select a .zip file on the PC.")
            return
        succ, msg = self.explorer.extract_local_zip(item.path, self.pc_path)
        if succ:
            QMessageBox.information(self, tr("success"), "ZIP extracted successfully.")
            self.refresh_pc_panel()
        else:
            QMessageBox.critical(self, tr("error"), msg)

    def on_pc_edit_file(self):
        item = self.get_selected_pc_item()
        if not item or item.is_dir:
            return
        content = self.explorer.read_local_text_file(item.path)
        dlg = FileEditorDialog(item.path, content, title_prefix="PC Edit", parent=self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            succ, msg = self.explorer.write_local_text_file(item.path, dlg.get_content())
            if succ:
                self.refresh_pc_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    # ==========================================
    # RIGHT PANEL: ANDROID FILE EXPLORER
    # ==========================================
    def create_android_panel(self) -> QWidget:
        panel = QFrame()
        panel.setObjectName("AndroidPanelFrame")
        panel.setStyleSheet("""
            #AndroidPanelFrame {
                background-color: rgba(22, 22, 35, 0.65);
                border: 1px solid rgba(255, 255, 255, 0.08);
                border-radius: 16px;
            }
        """)
        vbox = QVBoxLayout(panel)
        vbox.setContentsMargins(10, 10, 10, 10)
        vbox.setSpacing(8)

        # Android Navigation Bar
        nav_box = QHBoxLayout()
        self.btn_android_up = QPushButton(f"⬆ {tr('btn_parent_dir')}")
        self.btn_android_up.clicked.connect(self.android_navigate_up)

        self.btn_android_home = QPushButton(f"🏠 {tr('btn_home_dir')}")
        self.btn_android_home.clicked.connect(lambda: self.android_navigate_to("/sdcard"))

        self.btn_android_root_dir = QPushButton("⚡ / (Root)")
        self.btn_android_root_dir.clicked.connect(lambda: self.android_navigate_to("/"))

        self.txt_android_path = QLineEdit(self.android_path)
        self.txt_android_path.returnPressed.connect(lambda: self.android_navigate_to(self.txt_android_path.text().strip()))

        self.btn_android_refresh = QPushButton("🔄")
        self.btn_android_refresh.clicked.connect(self.refresh_android_panel)

        self.chk_root_mode = QCheckBox(tr("explorer_root_mode"))
        self.chk_root_mode.setChecked(True)
        self.chk_root_mode.stateChanged.connect(self.refresh_android_panel)

        nav_box.addWidget(self.btn_android_up)
        nav_box.addWidget(self.btn_android_home)
        nav_box.addWidget(self.btn_android_root_dir)
        nav_box.addWidget(self.txt_android_path, 1)
        nav_box.addWidget(self.btn_android_refresh)
        nav_box.addWidget(self.chk_root_mode)
        vbox.addLayout(nav_box)

        # Android Action Toolbar
        action_box = QHBoxLayout()
        action_box.setSpacing(6)
        btn_style = "padding: 5px 10px; font-size: 12px; font-weight: 600;"

        self.btn_and_new_folder = QPushButton("📁 Folder")
        self.btn_and_new_folder.setStyleSheet(btn_style)
        self.btn_and_new_folder.clicked.connect(self.on_android_new_folder)

        self.btn_and_new_file = QPushButton("📄 File")
        self.btn_and_new_file.setStyleSheet(btn_style)
        self.btn_and_new_file.clicked.connect(self.on_android_new_file)

        self.btn_and_edit = QPushButton("✏️ Edit")
        self.btn_and_edit.setStyleSheet(btn_style)
        self.btn_and_edit.clicked.connect(self.on_android_edit_file)

        self.btn_and_zip = QPushButton("📦 ZIP")
        self.btn_and_zip.setStyleSheet(btn_style)
        self.btn_and_zip.clicked.connect(self.on_android_extract_zip)

        self.btn_and_rename = QPushButton("🏷️ Rename")
        self.btn_and_rename.setStyleSheet(btn_style)
        self.btn_and_rename.clicked.connect(self.on_android_rename)

        self.btn_and_delete = QPushButton("🗑️ Delete")
        self.btn_and_delete.setStyleSheet(btn_style)
        self.btn_and_delete.setProperty("class", "btn-danger")
        self.btn_and_delete.clicked.connect(self.on_android_delete)

        action_box.addWidget(self.btn_and_new_folder)
        action_box.addWidget(self.btn_and_new_file)
        action_box.addWidget(self.btn_and_edit)
        action_box.addWidget(self.btn_and_zip)
        action_box.addWidget(self.btn_and_rename)
        action_box.addWidget(self.btn_and_delete)
        action_box.addStretch()
        vbox.addLayout(action_box)

        # Android Table Widget
        self.table_android = QTableWidget()
        self.table_android.setColumnCount(4)
        self.table_android.setHorizontalHeaderLabels([
            tr("col_file_name"),
            tr("col_file_size"),
            tr("col_file_perms"),
            tr("col_file_date")
        ])
        self.table_android.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.Stretch)
        self.table_android.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        self.table_android.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.ResizeToContents)
        self.table_android.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents)
        self.table_android.verticalHeader().setVisible(False)
        self.table_android.cellDoubleClicked.connect(self.on_android_cell_double_clicked)
        vbox.addWidget(self.table_android, 1)

        return panel

    def android_navigate_to(self, path: str):
        if not path:
            path = "/"
        self.android_path = path
        self.txt_android_path.setText(path)
        self.refresh_android_panel()

    def android_navigate_up(self):
        parent = os.path.dirname(self.android_path.rstrip("/"))
        if not parent:
            parent = "/"
        self.android_navigate_to(parent)

    def refresh_android_panel(self):
        is_root = self.chk_root_mode.isChecked()
        items = self.explorer.list_dir(self.android_path, root_mode=is_root, serial=self.current_serial)
        self.android_items = items
        self.table_android.setRowCount(len(items))

        for row, item in enumerate(items):
            icon = "📁 " if item.is_dir else ("🔗 " if item.is_link else "📄 ")
            display_name = f"{icon}{item.name}"
            if item.link_target:
                display_name += f" -> {item.link_target}"

            name_widget = QTableWidgetItem(display_name)
            if item.is_dir:
                name_widget.setForeground(Qt.GlobalColor.cyan)
            self.table_android.setItem(row, 0, name_widget)
            self.table_android.setItem(row, 1, QTableWidgetItem(item.size_formatted))
            self.table_android.setItem(row, 2, QTableWidgetItem(item.permissions))
            self.table_android.setItem(row, 3, QTableWidgetItem(item.date))

    def on_android_cell_double_clicked(self, row: int, col: int):
        if row < len(self.android_items):
            item = self.android_items[row]
            if item.is_dir:
                self.android_navigate_to(item.path)
            else:
                self.on_android_edit_file()

    def get_selected_android_item(self) -> Optional[FileItem]:
        row = self.table_android.currentRow()
        if 0 <= row < len(self.android_items):
            return self.android_items[row]
        return None

    def on_android_new_folder(self):
        name, ok = QInputDialog.getText(self, tr("dialog_new_folder_title"), tr("dialog_new_folder_msg"))
        if ok and name:
            new_path = f"{self.android_path}/{name}".replace("//", "/")
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.create_folder(new_path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_android_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_android_new_file(self):
        name, ok = QInputDialog.getText(self, tr("dialog_new_file_title"), tr("dialog_new_file_msg"))
        if ok and name:
            new_path = f"{self.android_path}/{name}".replace("//", "/")
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.create_file(new_path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_android_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_android_rename(self):
        item = self.get_selected_android_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Select an Android item to rename.")
            return
        new_name, ok = QInputDialog.getText(self, tr("dialog_rename_title"), tr("dialog_rename_msg"), text=item.name)
        if ok and new_name and new_name != item.name:
            new_path = f"{os.path.dirname(item.path)}/{new_name}".replace("//", "/")
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.rename_item(item.path, new_path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_android_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_android_delete(self):
        item = self.get_selected_android_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Select an Android item to delete.")
            return
        res = QMessageBox.question(self, tr("confirm"), tr("confirm_delete_msg", item=item.name), QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
        if res == QMessageBox.StandardButton.Yes:
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.delete_item(item.path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_android_panel()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_android_extract_zip(self):
        item = self.get_selected_android_item()
        if not item or not item.name.lower().endswith(".zip"):
            QMessageBox.warning(self, tr("warning"), "Please select a .zip file on Android.")
            return
        is_root = self.chk_root_mode.isChecked()
        succ, msg = self.explorer.extract_zip(item.path, self.android_path, root_mode=is_root, serial=self.current_serial)
        if succ:
            QMessageBox.information(self, tr("success"), "ZIP extracted successfully on device.")
            self.refresh_android_panel()
        else:
            QMessageBox.critical(self, tr("error"), msg)

    def on_android_edit_file(self):
        item = self.get_selected_android_item()
        if not item or item.is_dir:
            return
        is_root = self.chk_root_mode.isChecked()
        content = self.explorer.read_text_file(item.path, root_mode=is_root, serial=self.current_serial)
        dlg = FileEditorDialog(item.path, content, title_prefix="Android Edit", parent=self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            succ, msg = self.explorer.write_text_file(item.path, dlg.get_content(), root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_android_panel()
            else:
                QMessageBox.critical(self, tr("error"), f"Failed to save file: {msg}")

    # ==========================================
    # BRIDGE: MT MANAGER STYLE TRANSFERS
    # ==========================================
    def transfer_pc_to_android(self):
        """Copies selected item on PC side into currently opened folder on Android side."""
        item = self.get_selected_pc_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Please select a file or folder from the PC panel to copy.")
            return

        target_dest = self.android_path
        res = QMessageBox.question(
            self,
            tr("confirm"),
            tr("confirm_transfer_to_android", item=item.name, dst=target_dest),
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if res != QMessageBox.StandardButton.Yes:
            return

        is_root = self.chk_root_mode.isChecked()
        succ, msg = self.explorer.push_file(item.path, target_dest, root_mode=is_root, serial=self.current_serial)
        if succ:
            QMessageBox.information(self, tr("success"), f"Copied '{item.name}' to Android ({target_dest})")
            self.refresh_android_panel()
        else:
            QMessageBox.critical(self, tr("error"), f"Copy failed:\n{msg}")

    def transfer_android_to_pc(self):
        """Copies selected item on Android side into currently opened folder on PC side."""
        item = self.get_selected_android_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Please select a file or folder from the Android panel to copy.")
            return

        target_local_path = os.path.join(self.pc_path, item.name)
        res = QMessageBox.question(
            self,
            tr("confirm"),
            tr("confirm_transfer_to_pc", item=item.name, dst=self.pc_path),
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if res != QMessageBox.StandardButton.Yes:
            return

        is_root = self.chk_root_mode.isChecked()
        succ, msg = self.explorer.pull_file(item.path, target_local_path, root_mode=is_root, serial=self.current_serial)
        if succ:
            QMessageBox.information(self, tr("success"), f"Copied '{item.name}' to PC ({self.pc_path})")
            self.refresh_pc_panel()
        else:
            QMessageBox.critical(self, tr("error"), f"Copy failed:\n{msg}")

    def update_translations(self):
        self.lbl_pc_title.setText(f"🖥️ {tr('panel_pc')}")
        self.lbl_android_title.setText(f"📱 {tr('panel_android')}")
        self.btn_copy_to_android.setText(f" {tr('btn_transfer_to_android')} ")
        self.btn_copy_to_pc.setText(f" {tr('btn_transfer_to_pc')} ")

        # PC Controls
        self.btn_pc_up.setText(f"⬆ {tr('btn_parent_dir')}")
        self.btn_pc_home.setText(f"🏠 {tr('btn_home_dir')}")
        self.btn_pc_new_folder.setText(f"📁 {tr('btn_new_folder')}")
        self.btn_pc_new_file.setText(f"📄 {tr('btn_new_file')}")
        self.btn_pc_edit.setText(f"✏️ {tr('btn_edit_file')}")
        self.btn_pc_zip.setText(f"📦 {tr('btn_extract_zip')}")
        self.btn_pc_rename.setText(tr("btn_rename"))
        self.btn_pc_delete.setText(f"🗑 {tr('btn_delete')}")

        # Android Controls
        self.btn_android_up.setText(f"⬆ {tr('btn_parent_dir')}")
        self.btn_android_home.setText(f"🏠 {tr('btn_home_dir')}")
        self.chk_root_mode.setText(tr("explorer_root_mode"))
        self.btn_and_new_folder.setText(f"📁 {tr('btn_new_folder')}")
        self.btn_and_new_file.setText(f"📄 {tr('btn_new_file')}")
        self.btn_and_edit.setText(f"✏️ {tr('btn_edit_file')}")
        self.btn_and_zip.setText(f"📦 {tr('btn_extract_zip')}")
        self.btn_and_rename.setText(tr("btn_rename"))
        self.btn_and_delete.setText(f"🗑 {tr('btn_delete')}")
