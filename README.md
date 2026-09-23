# Ximi Ultimate Tool 🚀

<div align="center">

![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB?style=for-the-badge&logo=python&logoColor=white)
![PyQt6](https://img.shields.io/badge/GUI-PyQt6-41CD52?style=for-the-badge&logo=qt&logoColor=white)
![HyperOS](https://img.shields.io/badge/Xiaomi-HyperOS%20%7C%20MIUI-FF6900?style=for-the-badge&logo=xiaomi&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-0078D6?style=for-the-badge&logo=linux&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

**Ximi Ultimate Tool** adalah utility tool Android komprehensif berbasis Python & PyQt6 dengan desain modern **MIUIX / Xiaomi HyperOS Liquid Glass**, dirancang khusus untuk pengoperasian ADB dan Fastboot yang aman, cepat, dan intuitif.

[Fitur Utama](#-fitur-utama) • [Instalasi](#-instalasi) • [Tampilan UI](#-konsep-interface-ui) • [Dokumentasi Fitur](#-panduan-fitur) • [Komunitas](#-komunitas)

</div>

---

## 🌟 Fitur Utama

### 1. ⚡ Debloater (ADB)
- **Debloat Otomatis**: Dilengkapi database kurasi bloatware Xiaomi, HyperOS, MIUI, Google, dan Facebook. Pengguna dapat memilih/menghilangkan centang (*unchecklist*) aplikasi yang masih dibutuhkan sebelum proses eksekusi.
- **Debloat Manual**: Input langsung nama paket (*package name*) atau telusuri seluruh aplikasi sistem/pengguna yang terpasang dengan filter pencarian instan.
- **Aksi Fleksibel**: Uninstall (`--user 0`), Restore/Reinstall (`install-existing`), Disable (`disable-user`), dan Enable.
- **Logcat & Dmesg Viewer**: Pemantau log kernel (*dmesg*) dan log sistem (*logcat*) secara *real-time* dengan opsi filter dan ekspor ke file `.txt`/`.log`.

### 2. 📁 Dual-Panel Explorer (Gaya MT Manager)
- **Konsep Dua Panel (Dual-Panel)**:
  - **Panel Kiri (PC Storage)**: Menjelajahi berkas dan folder di komputer lokal pengguna.
  - **Panel Kanan (Android Storage)**: Menjelajahi direktori ponsel Android dengan dukungan mode **Root (`su`)** penuh serta fallback non-root.
- **Transfer Dua Arah Cepat**:
  - Tombol **`Salin ke Android ➜`**: Menyalin file/folder yang dipilih dari PC langsung ke direktori Android yang sedang dibuka di panel kanan.
  - Tombol **`⬅ Salin ke PC`**: Menarik file/folder yang dipilih dari Android langsung ke direktori PC yang sedang dibuka di panel kiri.
- **Operasi Berkas Lengkap pada Kedua Panel**:
  - Buat folder baru (*mkdir*) & berkas baru (*touch*).
  - Ganti nama (*rename*), salin (*copy*), dan hapus (*delete*).
  - Ekstrak arsip ZIP langsung di PC maupun di perangkat Android.
  - **In-App Text Editor**: Membuka dan mengedit berkas teks sistem (seperti `build.prop`, `hosts`, dll.) atau berkas lokal langsung di aplikasi.
  - Tampilan izin berkas (*permissions*) & tanggal modifikasi.

### 3. 💻 Terminal Shell Interaktif
- Terminal interaktif Android dengan tombol toggle cepat antara **Root Shell (`su`)** dan **Standard Shell (`sh`)**.
- Mendukung riwayat perintah (*command history* menggunakan tombol panah atas/bawah).
- Dilengkapi custom command bawaan **`fastfetch`** ala Linux dengan logo ASCII Xiaomi HyperOS yang menampilkan spesifikasi lengkap perangkat (OS, Kernel, Uptime, Resolusi Layar, CPU, RAM, Storage, Baterai, dan Security Patch).

### 4. 🚀 Fastboot Flasher (Solusi Alternatif Mi Flash)
- **Single Partition Flasher**: Flash partisi spesifik (`boot`, `init_boot`, `vendor_boot`, `recovery`, `vbmeta`, `vbmeta_system`, `vbmeta_vendor`, `dtbo`, `super`, `cust`, dll.) dilengkapi tombol aktivasi flag *disable verity & verification*.
- **Pencegah Anti-Brick Bootloader**: Mengatasi kelemahan fatal Mi Flash yang kerap mengunci bootloader secara tidak sengaja:
  1. *Clean Flash (Format Data)*: Menghapus data/userdata tanpa mengunci bootloader (**Keep Unlocked**).
  2. *Clean Flash without Format Data (Dirty Flash)*: Memperbarui ROM tanpa menghapus data pengguna.
  3. *Clean Flash + Lock Bootloader*: Opsi penguncian dengan sistem **konfirmasi ganda** guna mencegah *hard brick* akibat ROM beda wilayah (*cross-region*).
- **Advance Mode (Partition Selector)**:
  - Memindai skrip resmi fastboot (`flash_all.bat` / `flash_all.sh`).
  - Pengguna dapat menghilangkan centang (*unchecklist*) partisi berisiko tinggi seperti `preloader`, `cust`, atau `persist`.
  - Partisi yang tidak dicentang otomatis dilewati/dikomentari saat proses flashing.
  - Output log terminal interaktif secara *real-time* dengan bilah progres dan tombol pembatalan (*abort*).

### 5. 🎨 Konsep Interface UI (MIUIX Liquid Glass)
- **Floating Bottom Bar**: Navigasi pil mengambang di bagian bawah layar bergaya **iOS 26 / HyperOS Liquid Glass** dengan efek transparan *frosted glass* dan aksen glow halus.
- **Menu Settings & About Phone**:
  - Mengadopsi tata letak kartu *"About Phone"* HyperOS resmi (sesuai referensi [example.png](file:///run/media/fxxyz73/sigeonpex/Ximi-Ultimate-Tool/example.png)).
  - Membaca versi HyperOS langsung dari properti `ro.mi.os.version.incremental`, nama model/pasar, CPU/chipset, kapasitas RAM, penyimpanan, dan baterai.
- **Multi-Bahasa (Bilingual)**: Tersedia pilihan Bahasa Indonesia dan Bahasa Inggris dengan pergantian bahasa secara dinamis tanpa perlu restart aplikasi.
- **Kustomisasi Tema**:
  - Mode Gelap (*HyperOS Midnight*).
  - Mode Terang (*MIUIX Clean*).
  - Mendukung kustomisasi wallpaper latar belakang (*custom background image*) dengan lapisan overlay cerdas.

---

## 💻 Persyaratan Sistem

- **Sistem Operasi**: Linux (Arch, Ubuntu, Debian, Fedora, dll.) atau Windows 10/11.
- **Python**: Versi 3.10 ke atas.
- **Android Platform Tools**: `adb` dan `fastboot` terpasang di sistem.

---

## 🚀 Panduan Instalasi & Menjalankan

### Linux (Arch Linux / Ubuntu / Debian / Fedora)

```bash
# 1. Clone repositori
git clone https://github.com/iprjkt/Ximi-Ultimate-Tool.git
cd Ximi-Ultimate-Tool

# 2. Pasang dependensi
pip install -r requirements.txt
# Atau melalui package manager sistem:
# Arch: sudo pacman -S python-pyqt6
# Ubuntu/Debian: sudo apt install python3-pyqt6

# 3. Jalankan aplikasi
python3 main.py
```

### Windows

1. Unduh atau clone repositori ini.
2. Pastikan Python 3 dan Platform Tools (ADB/Fastboot) sudah terpasang dan terdaftar di PATH.
3. Buka Command Prompt / PowerShell di folder proyek:
   ```cmd
   pip install -r requirements.txt
   python main.py
   ```

---

## 📁 Struktur Berkas Proyek

```
Ximi-Ultimate-Tool/
├── main.py                     # Entry point aplikasi
├── requirements.txt            # Dependensi Python (PyQt6)
├── Roboto-Regular.ttf          # Font tipografi bawaan
├── example.png                 # Referensi desain kartu About Phone HyperOS
├── app/
│   ├── core/
│   │   ├── adb_manager.py      # Manajemen perangkat ADB, debloater, logcat, dmesg
│   │   ├── fastboot_manager.py # Flasher fastboot, parser skrip ROM, advance mode
│   │   ├── explorer_manager.py # File manager root & non-root Android
│   │   ├── fastfetch.py        # Mesin fastfetch logo HyperOS & spesifikasi
│   │   └── settings_manager.py # Manajemen persistensi konfigurasi
│   └── ui/
│       ├── floating_bar.py     # Floating Bottom Bar pill liquid glass
│       ├── i18n.py             # Modul multi-bahasa (ID & EN)
│       ├── styles.py           # MIUIX QSS stylesheets (Dark & Light)
│       ├── main_window.py      # Window utama aplikasi
│       ├── components/
│       │   └── hyperos_card.py # Kartu About Phone identik example.png
│       └── views/
│           ├── adb_view.py       # Tampilan Debloater & Logs
│           ├── fastboot_view.py  # Tampilan Fastboot Flasher
│           ├── terminal_view.py  # Tampilan Terminal Shell & Fastfetch
│           ├── explorer_view.py  # Tampilan Root Explorer
│           └── settings_view.py  # Tampilan Pengaturan & Info Perangkat
```

---

## 🌐 Komunitas & Pengembang

- **Pengembang**: [iprjkt](https://github.com/iprjkt)
- **Repositori GitHub**: [https://github.com/iprjkt/Ximi-Ultimate-Tool](https://github.com/iprjkt)
- **Saluran Telegram**: [@anotherside551](https://t.me/anotherside551)

---

## ⚠️ Disclaimer
*Ximi Ultimate Tool disediakan untuk tujuan perbaikan, kustomisasi, dan pemeliharaan perangkat. Pengembang tidak bertanggung jawab atas kerusakan perangkat akibat kesalahan pemilihan berkas partisi atau penguncian bootloader pada ROM yang tidak sesuai.*
