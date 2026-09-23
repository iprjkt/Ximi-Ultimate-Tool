"""
Terminal Shell View for Ximi Ultimate Tool.
Supports running shell commands as root (su) or standard (sh),
command history, and custom 'fastfetch' with HyperOS logo.
"""

from typing import List, Optional
from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QCheckBox,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QPlainTextEdit,
    QPushButton,
    QVBoxLayout,
    QWidget
)
from app.core.adb_manager import ADBManager
from app.core.fastfetch import FastfetchEngine
from app.ui.i18n import tr

class TerminalLineEdit(QLineEdit):
    """Custom QLineEdit that captures Up/Down arrows for command history."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.history: List[str] = []
        self.history_idx: int = -1

    def add_to_history(self, cmd: str):
        if cmd and (not self.history or self.history[-1] != cmd):
            self.history.append(cmd)
        self.history_idx = len(self.history)

    def keyPressEvent(self, event):
        if event.key() == Qt.Key.Key_Up:
            if self.history and self.history_idx > 0:
                self.history_idx -= 1
                self.setText(self.history[self.history_idx])
            return
        elif event.key() == Qt.Key.Key_Down:
            if self.history and self.history_idx < len(self.history) - 1:
                self.history_idx += 1
                self.setText(self.history[self.history_idx])
            elif self.history_idx >= len(self.history) - 1:
                self.history_idx = len(self.history)
                self.clear()
            return
        super().keyPressEvent(event)


class TerminalView(QWidget):
    def __init__(self, adb_manager: ADBManager, parent=None):
        super().__init__(parent)
        self.adb = adb_manager
        self.current_serial: Optional[str] = None
        self.init_ui()

    def set_serial(self, serial: Optional[str]):
        self.current_serial = serial

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(12)

        # Toolbar
        bar = QHBoxLayout()
        self.lbl_title = QLabel(f"💻 {tr('terminal_title')}")
        self.lbl_title.setStyleSheet("font-size: 16px; font-weight: 700; color: #FFFFFF;")

        self.chk_root_mode = QCheckBox(tr("terminal_su_mode"))
        self.chk_root_mode.setChecked(True)

        self.btn_fastfetch = QPushButton(f"✨ {tr('btn_run_fastfetch')}")
        self.btn_fastfetch.setProperty("class", "btn-primary")
        self.btn_fastfetch.clicked.connect(self.run_fastfetch)

        self.btn_clear = QPushButton(f"🗑 {tr('btn_clear_terminal')}")
        self.btn_clear.clicked.connect(self.clear_terminal)

        bar.addWidget(self.lbl_title)
        bar.addSpacing(16)
        bar.addWidget(self.chk_root_mode)
        bar.addStretch()
        bar.addWidget(self.btn_fastfetch)
        bar.addWidget(self.btn_clear)
        layout.addLayout(bar)

        # Terminal Screen (QPlainTextEdit)
        self.txt_terminal = QPlainTextEdit()
        self.txt_terminal.setReadOnly(True)
        self.txt_terminal.setStyleSheet("""
            background-color: #0A0A10;
            color: #E2E8F0;
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 14px;
            font-family: 'Fira Code', 'Roboto Mono', 'Courier New', monospace;
            font-size: 12px;
            line-height: 1.4;
            padding: 10px;
        """)
        layout.addWidget(self.txt_terminal, 1)

        # Prompt & Input Box
        input_box = QHBoxLayout()
        self.lbl_prompt = QLabel("$")
        self.lbl_prompt.setStyleSheet("font-size: 14px; font-weight: bold; color: #8B5CF6; padding-left: 4px;")

        self.txt_input = TerminalLineEdit()
        self.txt_input.setPlaceholderText(tr("terminal_placeholder"))
        self.txt_input.returnPressed.connect(self.on_enter_command)

        self.btn_send = QPushButton("Send")
        self.btn_send.clicked.connect(self.on_enter_command)

        input_box.addWidget(self.lbl_prompt)
        input_box.addWidget(self.txt_input, 1)
        input_box.addWidget(self.btn_send)
        layout.addLayout(input_box)

        # Welcome message
        self.txt_terminal.appendPlainText("╔════════════════════════════════════════════════════════════════════════╗")
        self.txt_terminal.appendPlainText("║  Ximi Ultimate Tool - Android Interactive Shell                       ║")
        self.txt_terminal.appendPlainText("║  Type 'fastfetch' for system specs or standard Linux/Android commands ║")
        self.txt_terminal.appendPlainText("╚════════════════════════════════════════════════════════════════════════╝\n")

    def on_enter_command(self):
        cmd = self.txt_input.text().strip()
        if not cmd:
            return

        self.txt_input.add_to_history(cmd)
        self.txt_input.clear()

        # Intercept fastfetch
        if cmd.lower() == "fastfetch":
            self.run_fastfetch()
            return

        if cmd.lower() == "clear":
            self.clear_terminal()
            return

        if cmd.lower() == "help":
            self.txt_terminal.appendPlainText("[HELP] Built-in Commands:\n  fastfetch  - Display Xiaomi HyperOS specifications & ASCII logo\n  clear      - Clear terminal screen\n  su / exit  - Toggle root mode\n")
            return

        # Execute on Android Device
        is_root = self.chk_root_mode.isChecked()
        prompt_char = "#" if is_root else "$"
        self.txt_terminal.appendPlainText(f"\n{prompt_char} {cmd}")

        if is_root:
            escaped = cmd.replace('"', '\\"')
            args = ["shell", f'su -c "{escaped}"']
        else:
            args = ["shell", cmd]

        if self.current_serial:
            args = ["-s", self.current_serial] + args

        code, out, err = self.adb.run_cmd(args, timeout=20)
        output_text = out if out.strip() else err
        if output_text:
            self.txt_terminal.appendPlainText(output_text.rstrip("\r\n"))
        elif code != 0:
            self.txt_terminal.appendPlainText(f"[Exit code: {code}]")

    def run_fastfetch(self):
        self.txt_terminal.appendPlainText("\n$ fastfetch")
        result = FastfetchEngine.generate(self.adb, self.current_serial)
        # Strip ANSI colors for plain text widget while preserving text layout
        clean = result
        import re
        ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
        clean = ansi_escape.sub('', result)
        self.txt_terminal.appendPlainText(clean)

    def clear_terminal(self):
        self.txt_terminal.clear()

    def update_translations(self):
        self.lbl_title.setText(f"💻 {tr('terminal_title')}")
        self.chk_root_mode.setText(tr("terminal_su_mode"))
        self.btn_fastfetch.setText(f"✨ {tr('btn_run_fastfetch')}")
        self.btn_clear.setText(f"🗑 {tr('btn_clear_terminal')}")
        self.txt_input.setPlaceholderText(tr("terminal_placeholder"))
