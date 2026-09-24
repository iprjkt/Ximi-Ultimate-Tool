# Ximi Ultimate Tool 🚀 (Rust & Tauri 2.0 Edition)

<div align="center">

![Rust](https://img.shields.io/badge/Backend-Rust%201.77%2B-DEA584?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/GUI-Tauri%202.0-24C8D8?style=for-the-badge&logo=tauri&logoColor=white)
![HyperOS](https://img.shields.io/badge/Xiaomi-HyperOS%20%7C%20MIUI-FF6900?style=for-the-badge&logo=xiaomi&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-0078D6?style=for-the-badge&logo=linux&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

**Ximi Ultimate Tool** is a high-performance, all-in-one Android utility toolkit re-engineered with **Rust + Tauri 2.0** featuring an ultra-fluid **MIUIX / Xiaomi HyperOS Liquid Glass** interface. Designed for blazing fast startup (<200ms), minimal RAM footprint (~35MB vs ~180MB Python), rock-solid Wayland/Hyprland rendering, and secure ADB/Fastboot operations.

[Key Features](#-key-features) • [Installation & Build](#-installation--build) • [MT Manager Explorer](#-mt-manager-dual-panel-file-explorer) • [Project Architecture](#-project-architecture) • [Community](#-community--developer)

</div>

---

## ⚡ Why Rust & Tauri 2.0?

| Metric | Python + PyQt6 | Rust + Tauri 2.0 | Improvement |
| :--- | :--- | :--- | :--- |
| **Startup Time** | ~1.8s – 2.5s | **~150ms – 250ms** | **~10x Faster** ⚡ |
| **Idle Memory (RAM)** | ~180MB – 240MB | **~35MB – 55MB** | **~75% Less RAM** 📉 |
| **File Transfer Overhead** | Python GIL / subprocess buffer | Direct async Tokio & zero-copy stream | **Native Wire Speed** 🚀 |
| **Wayland/Hyprland Compatibility** | QSS opacity glitches on composite | 100% Solid Opaque Base + Glass CSS | **Flawless Rendering** 🖥️ |
| **Binary Size & Portability** | Requires Python runtime & bulky PyQt libs | Single standalone native executable | **Zero Setup Dependency** 📦 |

---

## 🌟 Key Features

### 1. ⚡ Debloater (ADB)
- **Curated Xiaomi Bloatware Database**: One-click automatic debloat covering Xiaomi Analytics, Joyose telemetry, MIUI System Ads (`msa`), GetApps, Quick Apps, Wallpaper Carousel, Facebook services, and Google preloads.
- **Manual Debloat & App Filter**: Real-time filtering by *Curated Bloatware*, *All Packages*, *3rd Party Apps*, *System Apps*, or *Disabled Apps*.
- **Flexible Package Operations**: Uninstall (`pm uninstall -k --user 0`), Restore/Reinstall (`cmd package install-existing`), Disable (`pm disable-user`), and Enable.
- **Device Quick Tools**: Screenshot grabber, One-click APK installer, and Screen Mirroring via `scrcpy`.

### 2. 📁 MT Manager Dual-Panel File Explorer
- **MT Manager Dual-Panel Workflow**:
  - **Left Panel (PC Storage)**: Browse and navigate local folders and files on your computer.
  - **Right Panel (Android Storage)**: Browse Android device internal storage (`/sdcard`) or system root (`/`) with full **Root (`su`)** permissions.
- **1-Click Bidirectional Transfer**:
  - **`PC ➔ Android`**: Push selected files or directories from your PC into the active Android directory.
  - **`Android ➔ PC`**: Pull selected files or directories from Android into your active PC folder.
- **Root-Safe Transfer Engine**: Pushing or pulling to protected system partitions (`/system`, `/data`) automatically stages through `/data/local/tmp/` with root ownership and permissions (`chmod 644`).
- **File Management & In-App Editor**:
  - Create folders (*mkdir*), create files (*touch*), rename (*mv*), and delete (*rm*).
  - ZIP archive extraction on both PC and Android.
  - In-app text editor for modifying system scripts and configuration files (`build.prop`, `hosts`, etc.).

### 3. 🚀 Fastboot Flasher (Mi Flash Alternative)
- **Single Partition Flasher**: Flash individual partitions (`boot`, `init_boot`, `recovery`, `vbmeta`, `dtbo`, `vendor_boot`, `super`) with optional `--disable-verity --disable-verification` flags.
- **Full ROM Flasher**:
  - Select and parse official Fastboot ROM directories (`flash_all.sh` / `flash_all.bat`).
  - **Advance Mode**: Table with individual checkboxes to selectively uncheck sensitive partitions (e.g. `preloader`, `nvram`, `persist`) to prevent catastrophic hard-bricks.
  - Real-time terminal log viewer with color output and progress bar.

### 4. 💻 Terminal Shell & Fastfetch
- Full interactive shell connected via ADB with instant **Root Shell (`su`)** toggle.
- Built-in **HyperOS `fastfetch`**: Custom ASCII HyperOS logo with system diagnostics (Kernel uname, Uptime, Screen resolution, SoC, RAM, Storage, Battery, and Security Patch date).

### 5. 🎨 MIUIX / HyperOS Liquid Glass Interface
- **iOS 26 / HyperOS Floating Dock**: Frosted glass bottom bar with smooth spring transitions (`cubic-bezier(0.34, 1.56, 0.64, 1)`).
- **Settings & "About Phone" Card**: Replicating the authentic Xiaomi HyperOS *About Phone* card layout, displaying incremental version, CPU, RAM, and storage meter.
- **Bilingual Support**: Dynamic live switching between **English** and **Bahasa Indonesia** without restarting.
- **Dark & Light Themes**: Solid opaque base colors compatible with Hyprland and Wayland compositors.

---

## 🚀 Installation & Build

### Prerequisites
- **Rust Toolchain**: `rustc` and `cargo` 1.77+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Node.js**: Node 18+ and npm
- **System Libraries (Linux)**:
  - Arch Linux: `sudo pacman -S webkit2gtk-4.1 gtk3 libsoup-3.0 pkg-config`
  - Ubuntu / Debian: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev build-essential curl wget file libssl-dev`
  - Fedora: `sudo dnf install webkit2gtk4.1-devel gtk3-devel libsoup3-devel`
- **Android Platform Tools**: `adb` and `fastboot` in system PATH.

### 1. Development Mode (Live Reload)
```bash
git clone https://github.com/iprjkt/Ximi-Ultimate-Tool.git
cd Ximi-Ultimate-Tool

# Install frontend tools
npm install

# Run in live development mode
npx tauri dev
```

### 2. Compile Release Binary
```bash
npx tauri build
```
The optimized native executable will be generated at:
`src-tauri/target/release/ximi-ultimate-tool`

### 3. Quick Run (Pre-built Debug Binary)
```bash
./src-tauri/target/debug/ximi-ultimate-tool
```

---

## 📁 Project Architecture

```
Ximi-Ultimate-Tool/
├── package.json               # NPM manifest for Tauri 2.0 CLI
├── ui/                        # High-Performance Liquid Glass Frontend
│   ├── index.html             # Responsive layout with floating dock & dual-panel view
│   ├── style.css              # MIUIX glassmorphism & solid Hyprland base colors
│   ├── app.js                 # Tauri IPC bridge, i18n dictionary, MT Manager controller
│   └── assets/                # Icons, logos, and MiSans/Roboto typography
├── src-tauri/                 # Pure Rust Native Backend
│   ├── Cargo.toml             # Rust dependencies (Tauri 2, Tokio, Regex, Zip, Which)
│   ├── tauri.conf.json        # Window dimensions, title, and capability mapping
│   ├── build.rs               # Tauri build script
│   ├── src/
│   │   ├── main.rs            # Application entry point
│   │   ├── lib.rs             # Tauri command handlers & plugin registration
│   │   ├── adb.rs             # ADB device query, package manager, debloater, shell
│   │   ├── fastboot.rs        # Fastboot partition flasher & ROM script parser
│   │   ├── explorer.rs        # MT Manager dual-panel file manager (PC & Android root)
│   │   ├── fastfetch.rs       # Custom ASCII HyperOS fastfetch engine
│   │   └── utils.rs           # Tool detection (adb/fastboot) and URL browser opener
└── app/                       # (Legacy) Python + PyQt6 Reference Implementation
```

---

## 🌐 Community & Developer

- **Developer**: [iprjkt](https://github.com/iprjkt)
- **GitHub Repository**: [https://github.com/iprjkt/Ximi-Ultimate-Tool](https://github.com/iprjkt)
- **Telegram Channel**: [@iprjkt](https://t.me/iprjkt)

---

## ⚠️ Disclaimer
*Ximi Ultimate Tool is provided for device maintenance, recovery, and customization purposes. The developer is not responsible for device soft-bricks or hard-bricks resulting from flashing incompatible firmware, improper partition selection, or flashing cross-region ROMs with bootloader locking.*
