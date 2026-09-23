"""
MIUIX and HyperOS modern styling sheets for Ximi Ultimate Tool.
Supports Dark Mode (HyperOS Midnight) and Light Mode (MIUIX Clean).
Engineered for perfect rendering in Hyprland / Wayland and X11 without opacity glitches.
"""

DARK_THEME_QSS = """
/* ===== HyperOS Midnight Theme (Dark) ===== */
* {
    font-family: 'Roboto', 'Inter', 'Segoe UI', sans-serif;
}

QMainWindow, QDialog {
    background-color: #0E0E18;
    color: #F8FAFC;
}

#CentralBackgroundWidget {
    background-color: #0E0E18;
    color: #F8FAFC;
}

/* Glass Card Containers (Dark) */
#HyperOSCard1,
#HyperOSCard2,
#SettingsCardPref,
#SettingsCardAbout,
#HeaderFrame,
#ExplorerBridgeCard,
#PCPanelFrame,
#AndroidPanelFrame,
#FlashModeCard {
    background-color: #171628;
    border: 1px solid rgba(139, 92, 246, 0.25);
    border-radius: 18px;
}

#HyperOSCard1:hover,
#HyperOSCard2:hover,
#SettingsCardPref:hover,
#SettingsCardAbout:hover {
    border: 1px solid rgba(139, 92, 246, 0.45);
}

/* Typography Headings */
#HyperOSBrandLabel {
    font-size: 32px;
    font-weight: 800;
    color: #C084FC;
    letter-spacing: 0.5px;
}

#HyperOSVersionLabel {
    font-size: 15px;
    font-weight: 600;
    color: #A5B4FC;
}

#CardRowTitle {
    font-size: 15px;
    font-weight: 600;
    color: #FFFFFF;
}

#CardRowValue {
    font-size: 14px;
    font-weight: 500;
    color: #9CA3AF;
}

#SpecsItemValue {
    font-size: 16px;
    font-weight: 700;
    color: #FFFFFF;
}

#SpecsItemLabel {
    font-size: 12px;
    color: #9CA3AF;
}

#SettingsPrefTitle, #SettingsAboutTitle {
    font-size: 17px;
    font-weight: 700;
    color: #FFFFFF;
}

#SettingsPrefLabel {
    font-size: 14px;
    font-weight: 600;
    color: #E2E8F0;
}

QLabel {
    color: #E2E8F0;
}

/* PushButtons (Dark) */
QPushButton {
    background-color: #24223A;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    color: #F1F5F9;
}

QPushButton:hover {
    background-color: #332F52;
    border-color: #8B5CF6;
    color: #FFFFFF;
}

QPushButton:pressed {
    background-color: #1A182C;
}

QPushButton:disabled {
    background-color: #161524;
    border-color: rgba(255, 255, 255, 0.05);
    color: #475569;
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
    border: 1px solid rgba(239, 68, 68, 0.6);
    color: #FCA5A5;
}

QPushButton.btn-danger:hover {
    background-color: rgba(239, 68, 68, 0.4);
    color: #FFFFFF;
}

QPushButton.btn-success {
    background-color: rgba(34, 197, 94, 0.2);
    border: 1px solid rgba(34, 197, 94, 0.6);
    color: #86EFAC;
}

QPushButton.btn-success:hover {
    background-color: rgba(34, 197, 94, 0.4);
    color: #FFFFFF;
}

/* Text Inputs (Dark) */
QLineEdit, QTextEdit, QPlainTextEdit {
    background-color: #141322;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 8px 12px;
    color: #F8FAFC;
    font-size: 13px;
}

QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus {
    border: 1px solid #8B5CF6;
    background-color: #1A192E;
}

/* ComboBox & Dropdown Popup (Dark) - Specially tuned for Hyprland */
QComboBox {
    background-color: #211F38;
    border: 1px solid rgba(139, 92, 246, 0.4);
    border-radius: 11px;
    padding: 6px 14px;
    min-width: 170px;
    min-height: 24px;
    color: #FFFFFF;
    font-size: 13px;
    font-weight: 600;
}

QComboBox:hover {
    border-color: #8B5CF6;
    background-color: #2D2A4C;
}

QComboBox::drop-down {
    subcontrol-origin: padding;
    subcontrol-position: top right;
    width: 26px;
    border-left: none;
}

QComboBox::down-arrow {
    image: none;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-top: 5px solid #C084FC;
    width: 0px;
    height: 0px;
    margin-right: 6px;
}

/* Dropdown list popup */
QComboBox QAbstractItemView {
    background-color: #17152B;
    color: #F8FAFC;
    border: 1px solid #7C3AED;
    border-radius: 10px;
    selection-background-color: #7C3AED;
    selection-color: #FFFFFF;
    padding: 6px;
    outline: none;
}

QComboBox QAbstractItemView::item {
    min-height: 32px;
    padding: 6px 12px;
    border-radius: 6px;
    color: #F8FAFC;
    background-color: transparent;
}

QComboBox QAbstractItemView::item:hover,
QComboBox QAbstractItemView::item:selected {
    background-color: #6D28D9;
    color: #FFFFFF;
}

/* Table View (Dark) */
QTableWidget, QTableView {
    background-color: #131220;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    gridline-color: rgba(255, 255, 255, 0.04);
    color: #E2E8F0;
}

QHeaderView::section {
    background-color: #1B1A2C;
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    padding: 8px 12px;
    font-weight: 700;
    color: #94A3B8;
    font-size: 12px;
}

QTableWidget::item {
    padding: 6px 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

QTableWidget::item:selected {
    background-color: rgba(99, 102, 241, 0.3);
    color: #FFFFFF;
}

/* ScrollBars */
QScrollBar:vertical {
    border: none;
    background: #11101D;
    width: 8px;
    border-radius: 4px;
    margin: 2px;
}

QScrollBar::handle:vertical {
    background: rgba(255, 255, 255, 0.2);
    min-height: 25px;
    border-radius: 4px;
}

QScrollBar::handle:vertical:hover {
    background: #8B5CF6;
}

QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
    height: 0px;
}

QScrollBar:horizontal {
    border: none;
    background: #11101D;
    height: 8px;
    border-radius: 4px;
}

QScrollBar::handle:horizontal {
    background: rgba(255, 255, 255, 0.2);
    min-width: 25px;
    border-radius: 4px;
}

/* CheckBox (Dark) */
QCheckBox {
    spacing: 8px;
    color: #E2E8F0;
    font-size: 13px;
    font-weight: 500;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    background-color: #1A192E;
}

QCheckBox::indicator:hover {
    border-color: #8B5CF6;
}

QCheckBox::indicator:checked {
    background-color: #3B82F6;
    border-color: #3B82F6;
}

/* RadioButton (Dark) */
QRadioButton {
    spacing: 8px;
    color: #E2E8F0;
    font-size: 13px;
    font-weight: 500;
}

QRadioButton::indicator {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    background-color: #1A192E;
}

QRadioButton::indicator:checked {
    background-color: #3B82F6;
    border: 4px solid #141322;
}

/* Progress Bar */
QProgressBar {
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background-color: #141322;
    text-align: center;
    color: #FFFFFF;
    font-size: 11px;
    font-weight: bold;
}

QProgressBar::chunk {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #3B82F6, stop:1 #8B5CF6);
    border-radius: 7px;
}

/* TabWidget (Dark) */
QTabWidget::pane {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    background-color: #151424;
    padding: 8px;
}

QTabBar::tab {
    background-color: #1E1D30;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-bottom: none;
    border-top-left-radius: 10px;
    border-top-right-radius: 10px;
    padding: 8px 18px;
    margin-right: 4px;
    color: #94A3B8;
    font-weight: 600;
}

QTabBar::tab:selected {
    background-color: #27253F;
    color: #FFFFFF;
    border-color: rgba(139, 92, 246, 0.5);
}

QTabBar::tab:hover:!selected {
    background-color: #232238;
    color: #E2E8F0;
}

/* Floating Bottom Bar (Dark Liquid Glass) */
#FloatingBottomBar {
    background-color: rgba(23, 21, 38, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 28px;
}

#FloatingNavItem {
    background-color: transparent;
    border: none;
    border-radius: 22px;
    color: #94A3B8;
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
* {
    font-family: 'Roboto', 'Inter', 'Segoe UI', sans-serif;
}

QMainWindow, QDialog {
    background-color: #F1F3F7;
    color: #0F172A;
}

#CentralBackgroundWidget {
    background-color: #F1F3F7;
    color: #0F172A;
}

/* Glass Card Containers (Light) */
#HyperOSCard1,
#HyperOSCard2,
#SettingsCardPref,
#SettingsCardAbout,
#HeaderFrame,
#ExplorerBridgeCard,
#PCPanelFrame,
#AndroidPanelFrame,
#FlashModeCard {
    background-color: #FFFFFF;
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 18px;
}

#HyperOSCard1:hover,
#HyperOSCard2:hover,
#SettingsCardPref:hover,
#SettingsCardAbout:hover {
    border: 1px solid rgba(59, 130, 246, 0.35);
}

/* Typography Headings (Light) */
#HyperOSBrandLabel {
    font-size: 32px;
    font-weight: 800;
    color: #4F46E5;
    letter-spacing: 0.5px;
}

#HyperOSVersionLabel {
    font-size: 15px;
    font-weight: 600;
    color: #6366F1;
}

#CardRowTitle {
    font-size: 15px;
    font-weight: 600;
    color: #0F172A;
}

#CardRowValue {
    font-size: 14px;
    font-weight: 500;
    color: #475569;
}

#SpecsItemValue {
    font-size: 16px;
    font-weight: 700;
    color: #0F172A;
}

#SpecsItemLabel {
    font-size: 12px;
    color: #64748B;
}

#SettingsPrefTitle, #SettingsAboutTitle {
    font-size: 17px;
    font-weight: 700;
    color: #0F172A;
}

#SettingsPrefLabel {
    font-size: 14px;
    font-weight: 600;
    color: #1E293B;
}

QLabel {
    color: #1E293B;
}

/* PushButtons (Light) */
QPushButton {
    background-color: #F8FAFC;
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 12px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    color: #1E293B;
}

QPushButton:hover {
    background-color: #FFFFFF;
    border-color: #2563EB;
    color: #1D4ED8;
}

QPushButton:pressed {
    background-color: #E2E8F0;
}

QPushButton:disabled {
    background-color: #F1F5F9;
    border-color: rgba(0, 0, 0, 0.05);
    color: #94A3B8;
}

/* Primary Accent Button (Light) */
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
    background-color: #FEE2E2;
    border: 1px solid rgba(239, 68, 68, 0.5);
    color: #B91C1C;
}

QPushButton.btn-danger:hover {
    background-color: #FECACA;
    color: #991B1B;
}

QPushButton.btn-success {
    background-color: #DCFCE7;
    border: 1px solid rgba(34, 197, 94, 0.5);
    color: #15803D;
}

QPushButton.btn-success:hover {
    background-color: #BBF7D0;
    color: #166534;
}

/* Text Inputs (Light) */
QLineEdit, QTextEdit, QPlainTextEdit {
    background-color: #FFFFFF;
    border: 1px solid rgba(0, 0, 0, 0.14);
    border-radius: 12px;
    padding: 8px 12px;
    color: #0F172A;
    font-size: 13px;
}

QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus {
    border: 1px solid #2563EB;
    background-color: #FFFFFF;
}

/* ComboBox & Dropdown Popup (Light) */
QComboBox {
    background-color: #FFFFFF;
    border: 1px solid rgba(0, 0, 0, 0.18);
    border-radius: 11px;
    padding: 6px 14px;
    min-width: 170px;
    min-height: 24px;
    color: #0F172A;
    font-size: 13px;
    font-weight: 600;
}

QComboBox:hover {
    border-color: #2563EB;
    background-color: #F8FAFC;
}

QComboBox::drop-down {
    subcontrol-origin: padding;
    subcontrol-position: top right;
    width: 26px;
    border-left: none;
}

QComboBox::down-arrow {
    image: none;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-top: 5px solid #2563EB;
    width: 0px;
    height: 0px;
    margin-right: 6px;
}

/* Dropdown list popup */
QComboBox QAbstractItemView {
    background-color: #FFFFFF;
    color: #0F172A;
    border: 1px solid #2563EB;
    border-radius: 10px;
    selection-background-color: #2563EB;
    selection-color: #FFFFFF;
    padding: 6px;
    outline: none;
}

QComboBox QAbstractItemView::item {
    min-height: 32px;
    padding: 6px 12px;
    border-radius: 6px;
    color: #0F172A;
    background-color: transparent;
}

QComboBox QAbstractItemView::item:hover,
QComboBox QAbstractItemView::item:selected {
    background-color: #2563EB;
    color: #FFFFFF;
}

/* Table View (Light) */
QTableWidget, QTableView {
    background-color: #FFFFFF;
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 14px;
    gridline-color: rgba(0, 0, 0, 0.04);
    color: #0F172A;
}

QHeaderView::section {
    background-color: #F8FAFC;
    border: none;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    padding: 8px 12px;
    font-weight: 700;
    color: #64748B;
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

/* CheckBox (Light) */
QCheckBox {
    spacing: 8px;
    color: #1E293B;
    font-size: 13px;
    font-weight: 500;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.25);
    background-color: #FFFFFF;
}

QCheckBox::indicator:hover {
    border-color: #2563EB;
}

QCheckBox::indicator:checked {
    background-color: #2563EB;
    border-color: #2563EB;
}

/* RadioButton (Light) */
QRadioButton {
    spacing: 8px;
    color: #1E293B;
    font-size: 13px;
    font-weight: 500;
}

QRadioButton::indicator {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    border: 1px solid rgba(0, 0, 0, 0.25);
    background-color: #FFFFFF;
}

QRadioButton::indicator:checked {
    background-color: #2563EB;
    border: 4px solid #F1F3F7;
}

/* Progress Bar (Light) */
QProgressBar {
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 8px;
    background-color: #E2E8F0;
    text-align: center;
    color: #0F172A;
    font-size: 11px;
    font-weight: bold;
}

QProgressBar::chunk {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #2563EB, stop:1 #6366F1);
    border-radius: 7px;
}

/* TabWidget (Light) */
QTabWidget::pane {
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 14px;
    background-color: #FFFFFF;
    padding: 8px;
}

QTabBar::tab {
    background-color: #E2E8F0;
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-bottom: none;
    border-top-left-radius: 10px;
    border-top-right-radius: 10px;
    padding: 8px 18px;
    margin-right: 4px;
    color: #64748B;
    font-weight: 600;
}

QTabBar::tab:selected {
    background-color: #FFFFFF;
    color: #1D4ED8;
    border-color: rgba(37, 99, 235, 0.4);
}

/* Floating Bottom Bar (Light Liquid Glass) */
#FloatingBottomBar {
    background-color: rgba(255, 255, 255, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 28px;
}

#FloatingNavItem {
    background-color: transparent;
    border: none;
    border-radius: 22px;
    color: #64748B;
    font-weight: 600;
    font-size: 13px;
    padding: 6px 18px;
}

#FloatingNavItem:hover {
    background-color: rgba(0, 0, 0, 0.06);
    color: #0F172A;
}

#FloatingNavItem[selected="true"] {
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #2563EB, stop:1 #6366F1);
    color: #FFFFFF;
    font-weight: 700;
}
"""

def get_theme_qss(theme: str = "dark") -> str:
    return DARK_THEME_QSS if theme == "dark" else LIGHT_THEME_QSS
