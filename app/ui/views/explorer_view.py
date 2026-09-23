"""
Android Root Explorer View for Ximi Ultimate Tool.
Supports browsing directories in root or standard mode, file creation,
editing, moving, copying, zip extraction, and pushing/pulling files.
"""

import os
from typing import List, Optional
from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QCheckBox,
    QDialog,
    QDialogButtonBox,
    QFileDialog,
    QHBoxLayout,
    QHeaderView,
    QInputDialog,
    QLabel,
    QLineEdit,
    QMessageBox,
    QPlainTextEdit,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget
)
from app.core.explorer_manager import ExplorerManager, FileItem
from app.ui.i18n import tr

class FileEditorDialog(QDialog):
    """Simple in-app text editor for build.prop, hosts, config files."""
    def __init__(self, file_path: str, initial_content: str, parent=None):
        super().__init__(parent)
        self.file_path = file_path
        self.setWindowTitle(f"Edit - {os.path.basename(file_path)}")
        self.resize(750, 500)
        self.saved_content = initial_content

        layout = QVBoxLayout(self)
        self.lbl_path = QLabel(f"File: {file_path}")
        self.lbl_path.setStyleSheet("color: #9CA3AF; font-size: 12px;")
        layout.addWidget(self.lbl_path)

        self.txt_editor = QPlainTextEdit()
        self.txt_editor.setPlainText(initial_content)
        self.txt_editor.setStyleSheet("""
            background-color: #0A0A10;
            color: #F8FAFC;
            font-family: monospace;
            font-size: 12px;
            border-radius: 8px;
            border: 1px solid rgba(255, 255, 255, 0.1);
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
        self.current_path = "/sdcard"
        self.file_items: List[FileItem] = []
        self.init_ui()

    def set_serial(self, serial: Optional[str]):
        self.current_serial = serial

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(10)

        # 1. Navigation Top Bar
        nav_box = QHBoxLayout()
        self.btn_up = QPushButton(f"⬆ {tr('btn_parent_dir')}")
        self.btn_up.clicked.connect(self.navigate_up)

        self.btn_home = QPushButton(f"🏠 {tr('btn_home_dir')}")
        self.btn_home.clicked.connect(lambda: self.navigate_to("/sdcard"))

        self.btn_root_dir = QPushButton("⚡ / (Root)")
        self.btn_root_dir.clicked.connect(lambda: self.navigate_to("/"))

        self.txt_path = QLineEdit(self.current_path)
        self.txt_path.returnPressed.connect(lambda: self.navigate_to(self.txt_path.text().strip()))

        self.btn_refresh = QPushButton("🔄")
        self.btn_refresh.clicked.connect(self.refresh_current_dir)

        self.chk_root_mode = QCheckBox(tr("explorer_root_mode"))
        self.chk_root_mode.setChecked(True)
        self.chk_root_mode.stateChanged.connect(self.refresh_current_dir)

        nav_box.addWidget(self.btn_up)
        nav_box.addWidget(self.btn_home)
        nav_box.addWidget(self.btn_root_dir)
        nav_box.addWidget(self.txt_path, 1)
        nav_box.addWidget(self.btn_refresh)
        nav_box.addWidget(self.chk_root_mode)
        layout.addLayout(nav_box)

        # 2. File Action Toolbar
        action_box = QHBoxLayout()
        self.btn_new_folder = QPushButton(f"📁 {tr('btn_new_folder')}")
        self.btn_new_folder.clicked.connect(self.on_new_folder)

        self.btn_new_file = QPushButton(f"📄 {tr('btn_new_file')}")
        self.btn_new_file.clicked.connect(self.on_new_file)

        self.btn_push = QPushButton(f"⬆ {tr('btn_push_file')}")
        self.btn_push.setProperty("class", "btn-primary")
        self.btn_push.clicked.connect(self.on_push_file)

        self.btn_pull = QPushButton(f"⬇ {tr('btn_pull_file')}")
        self.btn_pull.clicked.connect(self.on_pull_file)

        self.btn_extract = QPushButton(f"📦 {tr('btn_extract_zip')}")
        self.btn_extract.clicked.connect(self.on_extract_zip)

        self.btn_edit = QPushButton(f"✏️ {tr('btn_edit_file')}")
        self.btn_edit.clicked.connect(self.on_edit_file)

        self.btn_rename = QPushButton(tr("btn_rename"))
        self.btn_rename.clicked.connect(self.on_rename_item)

        self.btn_delete = QPushButton(f"🗑 {tr('btn_delete')}")
        self.btn_delete.setProperty("class", "btn-danger")
        self.btn_delete.clicked.connect(self.on_delete_item)

        action_box.addWidget(self.btn_new_folder)
        action_box.addWidget(self.btn_new_file)
        action_box.addWidget(self.btn_push)
        action_box.addWidget(self.btn_pull)
        action_box.addWidget(self.btn_extract)
        action_box.addWidget(self.btn_edit)
        action_box.addWidget(self.btn_rename)
        action_box.addWidget(self.btn_delete)
        action_box.addStretch()
        layout.addLayout(action_box)

        # 3. File Table
        self.table = QTableWidget()
        self.table.setColumnCount(4)
        self.table.setHorizontalHeaderLabels([
            tr("col_file_name"),
            tr("col_file_size"),
            tr("col_file_perms"),
            tr("col_file_date")
        ])
        self.table.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.Stretch)
        self.table.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents)
        self.table.verticalHeader().setVisible(False)
        self.table.cellDoubleClicked.connect(self.on_cell_double_clicked)
        layout.addWidget(self.table)

    def navigate_to(self, path: str):
        if not path:
            path = "/"
        self.current_path = path
        self.txt_path.setText(path)
        self.refresh_current_dir()

    def navigate_up(self):
        parent = os.path.dirname(self.current_path.rstrip("/"))
        if not parent:
            parent = "/"
        self.navigate_to(parent)

    def refresh_current_dir(self):
        is_root = self.chk_root_mode.isChecked()
        items = self.explorer.list_dir(self.current_path, root_mode=is_root, serial=self.current_serial)
        self.file_items = items
        self.table.setRowCount(len(items))

        for row, item in enumerate(items):
            icon = "📁 " if item.is_dir else ("🔗 " if item.is_link else "📄 ")
            display_name = f"{icon}{item.name}"
            if item.link_target:
                display_name += f" -> {item.link_target}"

            name_widget = QTableWidgetItem(display_name)
            if item.is_dir:
                name_widget.setForeground(Qt.GlobalColor.cyan)
            self.table.setItem(row, 0, name_widget)
            self.table.setItem(row, 1, QTableWidgetItem(item.size_formatted))
            self.table.setItem(row, 2, QTableWidgetItem(item.permissions))
            self.table.setItem(row, 3, QTableWidgetItem(item.date))

    def on_cell_double_clicked(self, row: int, col: int):
        if row < len(self.file_items):
            item = self.file_items[row]
            if item.is_dir:
                self.navigate_to(item.path)
            else:
                self.on_edit_file()

    def get_selected_item(self) -> Optional[FileItem]:
        row = self.table.currentRow()
        if 0 <= row < len(self.file_items):
            return self.file_items[row]
        return None

    def on_new_folder(self):
        name, ok = QInputDialog.getText(self, tr("dialog_new_folder_title"), tr("dialog_new_folder_msg"))
        if ok and name:
            new_path = f"{self.current_path}/{name}".replace("//", "/")
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.create_folder(new_path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_current_dir()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_new_file(self):
        name, ok = QInputDialog.getText(self, tr("dialog_new_file_title"), tr("dialog_new_file_msg"))
        if ok and name:
            new_path = f"{self.current_path}/{name}".replace("//", "/")
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.create_file(new_path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_current_dir()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_rename_item(self):
        item = self.get_selected_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Select an item to rename.")
            return

        new_name, ok = QInputDialog.getText(self, tr("dialog_rename_title"), tr("dialog_rename_msg"), text=item.name)
        if ok and new_name and new_name != item.name:
            new_path = f"{os.path.dirname(item.path)}/{new_name}".replace("//", "/")
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.rename_item(item.path, new_path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_current_dir()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_delete_item(self):
        item = self.get_selected_item()
        if not item:
            QMessageBox.warning(self, tr("warning"), "Select an item to delete.")
            return

        res = QMessageBox.question(
            self,
            tr("confirm"),
            tr("confirm_delete_msg", item=item.name),
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if res == QMessageBox.StandardButton.Yes:
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.delete_item(item.path, root_mode=is_root, serial=self.current_serial)
            if succ:
                self.refresh_current_dir()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_push_file(self):
        local_file, _ = QFileDialog.getOpenFileName(self, "Select File to Push to Phone")
        if local_file:
            is_root = self.chk_root_mode.isChecked()
            target_remote = f"{self.current_path}/{os.path.basename(local_file)}".replace("//", "/")
            succ, msg = self.explorer.push_file(local_file, target_remote, root_mode=is_root, serial=self.current_serial)
            if succ:
                QMessageBox.information(self, tr("success"), f"File pushed to {target_remote}")
                self.refresh_current_dir()
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_pull_file(self):
        item = self.get_selected_item()
        if not item or item.is_dir:
            QMessageBox.warning(self, tr("warning"), "Select a file to pull.")
            return

        local_save, _ = QFileDialog.getSaveFileName(self, "Save File to PC", item.name)
        if local_save:
            is_root = self.chk_root_mode.isChecked()
            succ, msg = self.explorer.pull_file(item.path, local_save, root_mode=is_root, serial=self.current_serial)
            if succ:
                QMessageBox.information(self, tr("success"), f"File saved to: {local_save}")
            else:
                QMessageBox.critical(self, tr("error"), msg)

    def on_extract_zip(self):
        item = self.get_selected_item()
        if not item or not item.name.lower().endswith(".zip"):
            QMessageBox.warning(self, tr("warning"), "Please select a .zip file.")
            return

        dest_folder = self.current_path
        is_root = self.chk_root_mode.isChecked()
        succ, msg = self.explorer.extract_zip(item.path, dest_folder, root_mode=is_root, serial=self.current_serial)
        if succ:
            QMessageBox.information(self, tr("success"), "ZIP extracted successfully.")
            self.refresh_current_dir()
        else:
            QMessageBox.critical(self, tr("error"), msg)

    def on_edit_file(self):
        item = self.get_selected_item()
        if not item or item.is_dir:
            return

        is_root = self.chk_root_mode.isChecked()
        content = self.explorer.read_text_file(item.path, root_mode=is_root, serial=self.current_serial)
        
        dlg = FileEditorDialog(item.path, content, self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            new_content = dlg.get_content()
            succ, msg = self.explorer.write_text_file(item.path, new_content, root_mode=is_root, serial=self.current_serial)
            if succ:
                QMessageBox.information(self, tr("success"), "File saved.")
            else:
                QMessageBox.critical(self, tr("error"), f"Failed to save file: {msg}")

    def update_translations(self):
        self.btn_up.setText(f"⬆ {tr('btn_parent_dir')}")
        self.btn_home.setText(f"🏠 {tr('btn_home_dir')}")
        self.chk_root_mode.setText(tr("explorer_root_mode"))
        self.btn_new_folder.setText(f"📁 {tr('btn_new_folder')}")
        self.btn_new_file.setText(f"📄 {tr('btn_new_file')}")
        self.btn_push.setText(f"⬆ {tr('btn_push_file')}")
        self.btn_pull.setText(f"⬇ {tr('btn_pull_file')}")
        self.btn_extract.setText(f"📦 {tr('btn_extract_zip')}")
        self.btn_edit.setText(f"✏️ {tr('btn_edit_file')}")
        self.btn_rename.setText(tr("btn_rename"))
        self.btn_delete.setText(f"🗑 {tr('btn_delete')}")
