# Ximi Ultimate Tool 🚀

<div align="center">

![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB?style=for-the-badge&logo=python&logoColor=white)
![PyQt6](https://img.shields.io/badge/GUI-PyQt6-41CD52?style=for-the-badge&logo=qt&logoColor=white)
![HyperOS](https://img.shields.io/badge/Xiaomi-HyperOS%20%7C%20MIUI-FF6900?style=for-the-badge&logo=xiaomi&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-0078D6?style=for-the-badge&logo=linux&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

**Ximi Ultimate Tool** is a comprehensive, all-in-one Android utility tool built with Python & PyQt6 featuring a modern **MIUIX / Xiaomi HyperOS Liquid Glass** user interface. Engineered specifically for secure, high-speed, and intuitive ADB and Fastboot management.

[Key Features](#-key-features) • [Installation](#-installation--quick-start) • [UI Concept](#-interface-and-ui-concept) • [Project Structure](#-project-structure) • [Community](#-community--developer)

</div>

---

## 🌟 Key Features

### 1. ⚡ Debloater (ADB)
- **Automatic Debloat**: Powered by a curated database of Xiaomi, HyperOS, MIUI, Google, and Facebook bloatware. Users can easily review and uncheck any applications they still need before execution.
- **Manual Debloat**: Enter any package name directly or browse/filter through all installed user, system, or disabled packages with instant search.
- **Flexible Package Operations**: Uninstall (`--user 0`), Restore/Reinstall (`install-existing`), Disable (`disable-user`), and Enable.
- **Real-Time Logcat & Kernel Dmesg**: Integrated streaming monitors for system logs (*logcat*) and kernel logs (*dmesg*) with live filtering and export to `.txt`/`.log` files.

### 2. 📁 Dual-Panel Explorer (MT Manager Style)
- **Two-Panel Workflow**:
  - **Left Panel (PC Storage)**: Browse and manage local folders and files on your computer.
  - **Right Panel (Android Storage)**: Browse Android device directories with full **Root (`su`)** mode or non-root fallback.
- **Seamless Bidirectional File Transfer**:
  - **`Copy to Android ➜`**: Copies selected files or folders from your PC directly into the active Android destination folder.
  - **`⬅ Copy to PC`**: Pulls selected files or folders from Android directly into the active PC destination folder.
- **Complete File Management Across Both Panels**:
  - Create new folders (*mkdir*) and new files (*touch*).
  - Rename (*mv*), copy (*cp*), and delete (*rm*).
  - Direct ZIP archive extraction on both PC and Android.
  - **In-App Text Editor**: View and modify system text configuration files (e.g. `build.prop`, `hosts`, etc.) or local scripts directly within the app.
  - File permissions (*chmod*) inspection and timestamp display.

### 3. 💻 Interactive Shell Terminal
- Android shell terminal with a quick toggle between **Root Shell (`su`)** and **Standard Shell (`sh`)**.
- Command history navigation using Up/Down arrow keys.
- Custom built-in **`fastfetch`** command featuring an ASCII HyperOS logo and system diagnostic summary (OS version, Kernel, Uptime, Screen Resolution, CPU/SOC, RAM, Storage, Battery, and Security Patch date).

### 4. 🚀 Fastboot Flasher (Reliable Mi Flash Alternative)
- **Single Partition Flasher**: Flash individual partitions (`boot`, `init_boot`, `vendor_boot`, `recovery`, `vbmeta`, `vbmeta_system`, `vbmeta_vendor`, `dtbo`, `super`, `cust`, etc.) with optional *disable-verity & disable-verification* flags.
- **Accidental Bootloader Lock Prevention**: Eliminates the critical flaw of official Mi Flash by ensuring transparency and explicit consent:
  1. *Clean Flash (Format Data)*: Wipes userdata and caches while keeping the bootloader safely **UNLOCKED**.
  2. *Clean Flash without Format Data (Dirty Flash)*: Flashes system partitions while preserving user data and personal files.
  3. *Clean Flash + Lock Bootloader*: Flashes and locks the bootloader with **strict confirmation warnings** to prevent hard-brick risks from cross-region flashing.
- **Advance Mode (Partition Selector)**:
  - Parses official fastboot flash scripts (`flash_all.bat` / `flash_all.sh`).
  - Allows users to selectively uncheck sensitive or dangerous partitions such as `preloader`, `cust`, or `persist`.
  - Unchecked partitions are automatically skipped/commented out during flashing execution.
  - Live color-coded terminal log output with progress indicator and safe abort option.

### 5. 🎨 Interface and UI Concept (MIUIX Liquid Glass)
- **Floating Bottom Bar**: Docked pill-shaped bottom navigation inspired by **iOS 26 / HyperOS Liquid Glass** with frosted glass blur, translucent backdrop, and glowing indicators.
- **Settings & "About Phone" Card**:
  - Faithfully reproduces the authentic Xiaomi HyperOS *"About Phone"* card layout (based on [example.png](file:///run/media/fxxyz73/sigeonpex/Ximi-Ultimate-Tool/example.png)).
  - Dynamically reads `ro.mi.os.version.incremental`, market name, model, CPU chipset, RAM, storage, and battery capacity via ADB.
- **Bilingual Support**: Instant live switching between **English** and **Bahasa Indonesia** without needing to restart the application.
- **Theme Customization**:
  - Dark Mode (*HyperOS Midnight*).
  - Light Mode (*MIUIX Clean*).
  - Custom background wallpaper support with adaptive glass tint overlays.

---

## 💻 System Requirements

- **Operating System**: Linux (Arch, Ubuntu, Debian, Fedora, openSUSE, etc.) or Windows 10/11.
- **Python**: Version 3.10 or newer.
- **Android Platform Tools**: `adb` and `fastboot` installed and accessible via system PATH.

---

## 🚀 Installation & Quick Start

### Linux (Arch Linux / Ubuntu / Debian / Fedora)

```bash
# 1. Clone the repository
git clone https://github.com/iprjkt/Ximi-Ultimate-Tool.git
cd Ximi-Ultimate-Tool

# 2. Install dependencies
pip install -r requirements.txt
# Or via your system package manager:
# Arch Linux: sudo pacman -S python-pyqt6
# Ubuntu/Debian: sudo apt install python3-pyqt6

# 3. Launch the application
python3 main.py
```

### Windows

1. Download or clone this repository.
2. Ensure Python 3.10+ and Android Platform Tools (ADB/Fastboot) are installed and added to your system PATH.
3. Open Command Prompt or PowerShell in the project directory:
   ```cmd
   pip install -r requirements.txt
   python main.py
   ```

---

## 📁 Project Structure

```
Ximi-Ultimate-Tool/
├── main.py                     # Application entry point with typography loader
├── requirements.txt            # Python dependencies (PyQt6)
├── Roboto-Regular.ttf          # MIUIX typography font
├── example.png                 # Reference design for HyperOS About Phone card
├── app/
│   ├── core/
│   │   ├── adb_manager.py      # ADB device query, bloatware database, logcat/dmesg
│   │   ├── fastboot_manager.py # Fastboot flasher, script parser, advance mode
│   │   ├── explorer_manager.py # Dual-panel file manager (PC local & Android root)
│   │   ├── fastfetch.py        # Custom fastfetch engine with HyperOS ASCII logo
│   │   └── settings_manager.py # Settings configuration persistence
│   └── ui/
│       ├── floating_bar.py     # Liquid glass floating bottom pill bar
│       ├── i18n.py             # Internationalization module (EN & ID)
│       ├── styles.py           # Modern MIUIX QSS stylesheets (Dark & Light)
│       ├── main_window.py      # Master window and device connection listener
│       ├── components/
│       │   └── hyperos_card.py # HyperOS About Phone card matching example.png
│       └── views/
│           ├── adb_view.py       # Auto & Manual Debloater, Logcat, Dmesg
│           ├── fastboot_view.py  # Fastboot Partition & ROM Flasher
│           ├── terminal_view.py  # Interactive Shell & Fastfetch
│           ├── explorer_view.py  # MT Manager style Dual-Panel Explorer
│           └── settings_view.py  # Settings, Specs Card & Community Links
```

---

## 🌐 Community & Developer

- **Developer**: [iprjkt](https://github.com/iprjkt)
- **GitHub Repository**: [https://github.com/iprjkt/Ximi-Ultimate-Tool](https://github.com/iprjkt)
- **Telegram Channel**: [@anotherside551](https://t.me/anotherside551)

---

## ⚠️ Disclaimer
*Ximi Ultimate Tool is provided for device maintenance, recovery, and customization purposes. The developer is not responsible for device soft-bricks or hard-bricks resulting from flashing incompatible firmware, improper partition selection, or flashing cross-region ROMs with bootloader locking.*
