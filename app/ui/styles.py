"""
MIUIX and HyperOS modern styling sheets for Ximi Ultimate Tool.
Supports Dark Mode (HyperOS Midnight) and Light Mode (MIUIX Clean),
along with liquid glass / frosted glass effects.
"""

DARK_THEME_QSS = """
/* ===== HyperOS Midnight Theme (Dark) ===== */
QWidget {
    font-family: 'Roboto', 'Inter', 'Segoe UI', sans-serif;
    color: #F3F4F6;
    background-color: transparent;
    selection-background-color: #3B82F6;
    selection-color: #FFFFFF;
}

QMainWindow {
    background-color: #0F0F18;
}

/* Background overlay container */
#CentralBackgroundWidget {
    background-color: #0F0F18;
}

/* Glass Card Containers */
QFrame.miuix-card, QWidget.miuix-card, #HyperOSCard, #AboutCard {
    background-color: rgba(26, 26, 40, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 18px;
}

QFrame.miuix-card:hover {
    border: 1px solid rgba(139, 92, 246, 0.35);
}

/* Typography Headings */
QLabel.miuix-title {
    font-size: 26px;
    font-weight: 700;
    color: #FFFFFF;
}

QLabel.miuix-subtitle {
    font-size: 14px;
    font-weight: 500;
    color: #9CA3AF;
}

QLabel.miuix-caption {
    font-size: 12px;
    color: #6B7280;
}

/* PushButtons */
QPushButton {
    background-color: rgba(45, 45, 68, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    color: #E5E7EB;
}

QPushButton:hover {
    background-color: rgba(65, 65, 95, 0.9);
    border-color: rgba(139, 92, 246, 0.5);
    color: #FFFFFF;
}

QPushButton:pressed {
    background-color: rgba(35, 35, 55, 1.0);
}

QPushButton:disabled {
    background-color: rgba(30, 30, 45, 0.4);
    border-color: rgba(255, 255, 255, 0.04);
    color: #4B5563;
}

/* Primary Accent Button */
QPushButton.btn-primary {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #3B82F6, stop:1 #8B5CF6);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #FFFFFF;
    font-weight: 700;
}

QPushButton.btn-primary:hover {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #60A5FA, stop:1 #A78BFA);
    border-color: rgba(255, 255, 255, 0.4);
}

QPushButton.btn-danger {
    background-color: rgba(239, 68, 68, 0.2);
    border: 1px solid rgba(239, 68, 68, 0.5);
    color: #FCA5A5;
}

QPushButton.btn-danger:hover {
    background-color: rgba(239, 68, 68, 0.35);
    color: #FFFFFF;
}

QPushButton.btn-success {
    background-color: rgba(34, 197, 94, 0.2);
    border: 1px solid rgba(34, 197, 94, 0.5);
    color: #86EFAC;
}

QPushButton.btn-success:hover {
    background-color: rgba(34, 197, 94, 0.35);
    color: #FFFFFF;
}

/* Text Inputs */
QLineEdit, QTextEdit, QPlainTextEdit {
    background-color: rgba(20, 20, 32, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 8px 12px;
    color: #F3F4F6;
    font-size: 13px;
}

QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus {
    border: 1px solid #6366F1;
    background-color: rgba(25, 25, 40, 0.95);
}

/* ComboBox */
QComboBox {
    background-color: rgba(30, 30, 48, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 6px 12px;
    color: #F3F4F6;
    font-size: 13px;
}

QComboBox:hover {
    border-color: #6366F1;
}

QComboBox::drop-down {
    subcontrol-origin: padding;
    subcontrol-position: top right;
    width: 25px;
    border-left: none;
}

QComboBox QAbstractItemView {
    background-color: #1A1A28;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    selection-background-color: #3B82F6;
    color: #F3F4F6;
    padding: 4px;
}

/* Table View */
QTableWidget, QTableView {
    background-color: rgba(20, 20, 32, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    gridline-color: rgba(255, 255, 255, 0.04);
    color: #E5E7EB;
}

QHeaderView::section {
    background-color: rgba(30, 30, 48, 0.9);
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    padding: 8px 12px;
    font-weight: 700;
    color: #9CA3AF;
    font-size: 12px;
}

QTableWidget::item {
    padding: 6px 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

QTableWidget::item:selected {
    background-color: rgba(59, 130, 246, 0.25);
    color: #FFFFFF;
}

/* ScrollBars */
QScrollBar:vertical {
    border: none;
    background: rgba(0, 0, 0, 0.15);
    width: 8px;
    border-radius: 4px;
    margin: 2px 2px 2px 2px;
}

QScrollBar::handle:vertical {
    background: rgba(255, 255, 255, 0.2);
    min-height: 25px;
    border-radius: 4px;
}

QScrollBar::handle:vertical:hover {
    background: rgba(255, 255, 255, 0.4);
}

QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
    height: 0px;
}

QScrollBar:horizontal {
    border: none;
    background: rgba(0, 0, 0, 0.15);
    height: 8px;
    border-radius: 4px;
}

QScrollBar::handle:horizontal {
    background: rgba(255, 255, 255, 0.2);
    min-width: 25px;
    border-radius: 4px;
}

/* CheckBox */
QCheckBox {
    spacing: 8px;
    color: #E5E7EB;
    font-size: 13px;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    background-color: rgba(30, 30, 48, 0.6);
}

QCheckBox::indicator:hover {
    border-color: #6366F1;
}

QCheckBox::indicator:checked {
    background-color: #3B82F6;
    border-color: #3B82F6;
    image: none;
}

/* RadioButton */
QRadioButton {
    spacing: 8px;
    color: #E5E7EB;
    font-size: 13px;
}

QRadioButton::indicator {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    background-color: rgba(30, 30, 48, 0.6);
}

QRadioButton::indicator:checked {
    background-color: #3B82F6;
    border: 4px solid #1E1E2E;
}

/* Progress Bar */
QProgressBar {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    background-color: rgba(20, 20, 32, 0.8);
    text-align: center;
    color: #FFFFFF;
    font-size: 11px;
    font-weight: bold;
}

QProgressBar::chunk {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #3B82F6, stop:1 #8B5CF6);
    border-radius: 7px;
}

/* TabWidget */
QTabWidget::pane {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    background-color: rgba(22, 22, 34, 0.7);
    padding: 8px;
}

QTabBar::tab {
    background-color: rgba(30, 30, 48, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-bottom: none;
    border-top-left-radius: 10px;
    border-top-right-radius: 10px;
    padding: 8px 18px;
    margin-right: 4px;
    color: #9CA3AF;
    font-weight: 600;
}

QTabBar::tab:selected {
    background-color: rgba(45, 45, 70, 0.85);
    color: #FFFFFF;
    border-color: rgba(139, 92, 246, 0.4);
}

QTabBar::tab:hover:!selected {
    background-color: rgba(35, 35, 55, 0.7);
    color: #E5E7EB;
}

/* Floating Bottom Bar (Liquid Glass) */
#FloatingBottomBar {
    background-color: rgba(24, 24, 38, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 28px;
}

#FloatingNavItem {
    background-color: transparent;
    border: none;
    border-radius: 22px;
    color: #9CA3AF;
    font-weight: 600;
    font-size: 13px;
    padding: 6px 18px;
}

#FloatingNavItem:hover {
    background-color: rgba(255, 255, 255, 0.08);
    color: #FFFFFF;
}

#FloatingNavItem[selected="true"] {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #3B82F6, stop:1 #8B5CF6);
    color: #FFFFFF;
    font-weight: 700;
}
"""

LIGHT_THEME_QSS = """
/* ===== MIUIX Clean Theme (Light) ===== */
QWidget {
    font-family: 'Roboto', 'Inter', 'Segoe UI', sans-serif;
    color: #1F2937;
    background-color: transparent;
    selection-background-color: #2563EB;
    selection-color: #FFFFFF;
}

QMainWindow {
    background-color: #F3F4F6;
}

#CentralBackgroundWidget {
    background-color: #F3F4F6;
}

/* Glass Card Containers */
QFrame.miuix-card, QWidget.miuix-card, #HyperOSCard, #AboutCard {
    background-color: rgba(255, 255, 255, 0.85);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 18px;
}

QFrame.miuix-card:hover {
    border: 1px solid rgba(59, 130, 246, 0.35);
}

/* Typography Headings */
QLabel.miuix-title {
    font-size: 26px;
    font-weight: 700;
    color: #111827;
}

QLabel.miuix-subtitle {
    font-size: 14px;
    font-weight: 500;
    color: #4B5563;
}

QLabel.miuix-caption {
    font-size: 12px;
    color: #9CA3AF;
}

/* PushButtons */
QPushButton {
    background-color: rgba(240, 242, 245, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 12px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    color: #374151;
}

QPushButton:hover {
    background-color: #FFFFFF;
    border-color: #3B82F6;
    color: #1D4ED8;
}

QPushButton:pressed {
    background-color: #E5E7EB;
}

QPushButton:disabled {
    background-color: rgba(229, 231, 235, 0.5);
    border-color: rgba(0, 0, 0, 0.05);
    color: #9CA3AF;
}

/* Primary Accent Button */
QPushButton.btn-primary {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #2563EB, stop:1 #6366F1);
    border: 1px solid rgba(0, 0, 0, 0.1);
    color: #FFFFFF;
    font-weight: 700;
}

QPushButton.btn-primary:hover {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #3B82F6, stop:1 #818CF8);
}

QPushButton.btn-danger {
    background-color: rgba(254, 226, 226, 0.9);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #B91C1C;
}

QPushButton.btn-danger:hover {
    background-color: #FEE2E2;
    color: #991B1B;
}

QPushButton.btn-success {
    background-color: rgba(220, 252, 231, 0.9);
    border: 1px solid rgba(34, 197, 94, 0.4);
    color: #15803D;
}

QPushButton.btn-success:hover {
    background-color: #DCFCE7;
    color: #166534;
}

/* Text Inputs */
QLineEdit, QTextEdit, QPlainTextEdit {
    background-color: rgba(255, 255, 255, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 12px;
    padding: 8px 12px;
    color: #111827;
    font-size: 13px;
}

QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus {
    border: 1px solid #2563EB;
    background-color: #FFFFFF;
}

/* ComboBox */
QComboBox {
    background-color: rgba(255, 255, 255, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 12px;
    padding: 6px 12px;
    color: #1F2937;
    font-size: 13px;
}

QComboBox:hover {
    border-color: #2563EB;
}

QComboBox QAbstractItemView {
    background-color: #FFFFFF;
    border: 1px solid rgba(0, 0, 0, 0.15);
    border-radius: 10px;
    selection-background-color: #2563EB;
    color: #1F2937;
    padding: 4px;
}

/* Table View */
QTableWidget, QTableView {
    background-color: rgba(255, 255, 255, 0.85);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 14px;
    gridline-color: rgba(0, 0, 0, 0.04);
    color: #1F2937;
}

QHeaderView::section {
    background-color: rgba(243, 244, 246, 0.95);
    border: none;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    padding: 8px 12px;
    font-weight: 700;
    color: #6B7280;
    font-size: 12px;
}

QTableWidget::item {
    padding: 6px 10px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
}

QTableWidget::item:selected {
    background-color: rgba(37, 99, 235, 0.15);
    color: #1D4ED8;
}

/* CheckBox */
QCheckBox {
    spacing: 8px;
    color: #374151;
    font-size: 13px;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.25);
    background-color: rgba(255, 255, 255, 0.9);
}

QCheckBox::indicator:hover {
    border-color: #2563EB;
}

QCheckBox::indicator:checked {
    background-color: #2563EB;
    border-color: #2563EB;
}

/* Progress Bar */
QProgressBar {
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 8px;
    background-color: rgba(229, 231, 235, 0.8);
    text-align: center;
    color: #1F2937;
    font-size: 11px;
    font-weight: bold;
}

QProgressBar::chunk {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #2563EB, stop:1 #6366F1);
    border-radius: 7px;
}

/* TabWidget */
QTabWidget::pane {
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 14px;
    background-color: rgba(255, 255, 255, 0.8);
    padding: 8px;
}

QTabBar::tab {
    background-color: rgba(229, 231, 235, 0.6);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-bottom: none;
    border-top-left-radius: 10px;
    border-top-right-radius: 10px;
    padding: 8px 18px;
    margin-right: 4px;
    color: #6B7280;
    font-weight: 600;
}

QTabBar::tab:selected {
    background-color: #FFFFFF;
    color: #1D4ED8;
    border-color: rgba(59, 130, 246, 0.4);
}

/* Floating Bottom Bar (Liquid Glass Light) */
#FloatingBottomBar {
    background-color: rgba(255, 255, 255, 0.88);
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 28px;
}

#FloatingNavItem {
    background-color: transparent;
    border: none;
    border-radius: 22px;
    color: #6B7280;
    font-weight: 600;
    font-size: 13px;
    padding: 6px 18px;
}

#FloatingNavItem:hover {
    background-color: rgba(0, 0, 0, 0.05);
    color: #111827;
}

#FloatingNavItem[selected="true"] {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #2563EB, stop:1 #6366F1);
    color: #FFFFFF;
    font-weight: 700;
}
"""

def get_theme_qss(theme: str = "dark") -> str:
    return DARK_THEME_QSS if theme == "dark" else LIGHT_THEME_QSS
