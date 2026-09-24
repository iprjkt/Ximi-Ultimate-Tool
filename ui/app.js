/**
 * Ximi Ultimate Tool - Frontend Controller
 * Interfacing with Tauri 2.0 Rust Backend
 */

// ==========================================================
// 1. TAURI IPC HELPER
// ==========================================================
const invoke = async (cmd, args = {}) => {
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
    return await window.__TAURI__.core.invoke(cmd, args);
  }
  console.warn(`[DEV MOCK] Invoking ${cmd} with`, args);
  return Promise.reject(`Tauri IPC not available for ${cmd}`);
};

// ==========================================================
// 2. I18N MULTI-LANGUAGE SYSTEM
// ==========================================================
const I18N = {
  en: {
    no_device: "No Device Connected",
    select_device: "Select Device...",
    reboot: "Reboot",
    reboot_system: "System",
    reboot_recovery: "Recovery",
    reboot_bootloader: "Fastboot / Bootloader",
    reboot_edl: "EDL (Emergency Download)",
    nav_adb: "ADB & Debloat",
    nav_fastboot: "Fastboot Flasher",
    nav_explorer: "MT Explorer",
    nav_terminal: "Terminal Shell",
    nav_settings: "Settings",
    adb_desc: "Manage installed packages, debloat Xiaomi telemetry and apps, reboot modes.",
    fastboot_desc: "Flash single partition images or full Xiaomi Fastboot ROM (Mi Flash alternative).",
    explorer_desc: "Bidirectional PC & Android file manager with Root Mode and 1-Click transfer.",
    terminal_desc: "Interactive ADB shell with HyperOS fastfetch and root permissions.",
    settings_desc: "Inspect device hardware specifications, change application theme and language.",
    filter_recommended: "Curated Bloatware",
    filter_all: "All Packages",
    filter_3rd: "3rd Party",
    filter_system: "System",
    filter_disabled: "Disabled",
    search_placeholder: "Search package or app...",
    col_name: "App / Package",
    col_category: "Category",
    col_risk: "Risk",
    col_actions: "Actions",
    connect_device_prompt: "Connect a device with USB Debugging enabled",
    uninstall_selected: "Uninstall Selected",
    disable_selected: "Disable Selected",
    enable_selected: "Enable Selected",
    restore_pkg: "Restore Package",
    quick_tools: "Device Utilities",
    take_screenshot: "Take Screenshot",
    screen_mirror: "Screen Mirror (scrcpy)",
    install_apk: "Install APK",
    live_logcat: "Live Logcat",
    drag_apk: "Drag & Drop APK here to install",
    device_info_title: "Device Info",
    spec_model: "Model:",
    spec_codename: "Codename:",
    spec_hyperos: "HyperOS:",
    spec_android: "Android:",
    spec_battery: "Battery:",
    no_fastboot_dev: "No Fastboot Device",
    refresh: "Refresh",
    single_partition: "Single Partition",
    full_rom: "Full Fastboot ROM",
    target_partition: "Target Partition:",
    image_file: "Image File (.img):",
    browse: "Browse...",
    disable_verity: "Disable Verity & Verification (--disable-verity --disable-verification)",
    flash_partition_btn: "⚡ Flash Partition",
    boot_image_btn: "Temporarily Boot Image",
    rom_folder: "Fastboot ROM Directory:",
    flash_script: "Flash Script:",
    advance_partitions: "Advanced Partition Control",
    advance_tip: "Uncheck dangerous partitions (preloader, nvram) to avoid hard brick",
    select_rom_prompt: "Select ROM folder to load partitions",
    flash_rom_btn: "🔥 Flash Full ROM",
    abort: "Abort",
    root_mode: "Root (su)",
    push_to_android: "PC ➔ Android",
    pull_to_pc: "Android ➔ PC",
    root_shell: "Root Shell (su)",
    run_fastfetch: "✨ HyperOS Fastfetch",
    clear: "Clear",
    device_model: "Device Model",
    tile_cpu: "Processor",
    tile_ram: "RAM",
    tile_storage: "Storage",
    tile_battery: "Battery Capacity",
    tile_android: "Android Version",
    tile_security: "Security Patch",
    app_preferences: "App Preferences",
    theme_mode: "Theme Mode",
    theme_desc: "Switch between Dark and Light HyperOS styles",
    language_label: "Language",
    lang_desc: "Pilih Bahasa / Select Language",
    community_title: "Ximi Ultimate Tool v2.0 (Rust Edition)",
    community_desc: "High-performance Xiaomi HyperOS & MIUI ADB/Fastboot toolkit rewritten in Rust & Tauri 2.0 with liquid glass design.",
    ctx_open: "Open",
    ctx_edit: "Edit as Text",
    ctx_extract: "Extract ZIP",
    ctx_rename: "Rename",
    ctx_delete: "Delete",
    cancel: "Cancel",
    save: "Save Changes",
    restore_pkg_title: "Restore Uninstalled Package",
    restore_pkg_desc: "Enter the package name of the previously uninstalled system app:",
    restore: "Restore"
  },
  id: {
    no_device: "Tidak Ada Perangkat Terhubung",
    select_device: "Pilih Perangkat...",
    reboot: "Mulai Ulang",
    reboot_system: "Sistem",
    reboot_recovery: "Recovery",
    reboot_bootloader: "Fastboot / Bootloader",
    reboot_edl: "EDL (Mode Darurat)",
    nav_adb: "ADB & Debloat",
    nav_fastboot: "Flasher Fastboot",
    nav_explorer: "MT Explorer",
    nav_terminal: "Terminal Shell",
    nav_settings: "Pengaturan",
    adb_desc: "Kelola aplikasi terinstal, hapus bloatware & analitik Xiaomi, dan reboot mode.",
    fastboot_desc: "Flash image partisi tunggal atau Full ROM Fastboot Xiaomi (Alternatif Mi Flash).",
    explorer_desc: "Manajer file dua panel (PC & Android) dengan Mode Root dan transfer 1-Klik.",
    terminal_desc: "Shell ADB interaktif dengan fastfetch Xiaomi HyperOS dan hak akses root.",
    settings_desc: "Cek spesifikasi hardware perangkat, ganti tema tampilan, dan bahasa.",
    filter_recommended: "Rekomendasi Bloatware",
    filter_all: "Semua Paket",
    filter_3rd: "Aplikasi Pengguna",
    filter_system: "Sistem",
    filter_disabled: "Dinonaktifkan",
    search_placeholder: "Cari nama aplikasi atau paket...",
    col_name: "Aplikasi / Paket",
    col_category: "Kategori",
    col_risk: "Risiko",
    col_actions: "Aksi",
    connect_device_prompt: "Hubungkan perangkat dengan USB Debugging aktif",
    uninstall_selected: "Copot Pilihan",
    disable_selected: "Nonaktifkan",
    enable_selected: "Aktifkan",
    restore_pkg: "Pulihkan Aplikasi",
    quick_tools: "Utilitas Perangkat",
    take_screenshot: "Ambil Tangkapan Layar",
    screen_mirror: "Cermin Layar (scrcpy)",
    install_apk: "Pasang APK",
    live_logcat: "Logcat Langsung",
    drag_apk: "Tarik & Lepas APK ke sini untuk memasang",
    device_info_title: "Info Perangkat",
    spec_model: "Model:",
    spec_codename: "Codename:",
    spec_hyperos: "HyperOS:",
    spec_android: "Android:",
    spec_battery: "Baterai:",
    no_fastboot_dev: "Tidak Ada Perangkat Fastboot",
    refresh: "Segarkan",
    single_partition: "Partisi Tunggal",
    full_rom: "Full Fastboot ROM",
    target_partition: "Partisi Target:",
    image_file: "File Image (.img):",
    browse: "Pilih...",
    disable_verity: "Nonaktifkan Verity & Verification (--disable-verity --disable-verification)",
    flash_partition_btn: "⚡ Flash Partisi",
    boot_image_btn: "Boot Image Sementara",
    rom_folder: "Direktori ROM Fastboot:",
    flash_script: "Skrip Flash:",
    advance_partitions: "Kontrol Partisi Lanjutan",
    advance_tip: "Hapus centang partisi berbahaya (preloader, nvram) untuk mencegah hard brick",
    select_rom_prompt: "Pilih folder ROM untuk memuat daftar partisi",
    flash_rom_btn: "🔥 Flash Full ROM",
    abort: "Batalkan",
    root_mode: "Root (su)",
    push_to_android: "PC ➔ Android",
    pull_to_pc: "Android ➔ PC",
    root_shell: "Root Shell (su)",
    run_fastfetch: "✨ HyperOS Fastfetch",
    clear: "Bersihkan",
    device_model: "Model Perangkat",
    tile_cpu: "Prosesor",
    tile_ram: "RAM",
    tile_storage: "Penyimpanan",
    tile_battery: "Kapasitas Baterai",
    tile_android: "Versi Android",
    tile_security: "Patch Keamanan",
    app_preferences: "Preferensi Aplikasi",
    theme_mode: "Mode Tema",
    theme_desc: "Ganti tampilan antara gaya HyperOS Gelap dan Terang",
    language_label: "Bahasa",
    lang_desc: "Pilih Bahasa / Select Language",
    community_title: "Ximi Ultimate Tool v2.0 (Edisi Rust)",
    community_desc: "Perangkat toolkit ADB/Fastboot Xiaomi HyperOS & MIUI berkinerja tinggi ditulis ulang dalam Rust & Tauri 2.0.",
    ctx_open: "Buka",
    ctx_edit: "Edit sebagai Teks",
    ctx_extract: "Ekstrak ZIP",
    ctx_rename: "Ganti Nama",
    ctx_delete: "Hapus",
    cancel: "Batal",
    save: "Simpan Perubahan",
    restore_pkg_title: "Pulihkan Aplikasi Terhapus",
    restore_pkg_desc: "Masukkan nama paket aplikasi sistem yang sebelumnya dicopot:",
    restore: "Pulihkan"
  }
};

let currentLang = localStorage.getItem('ximi_lang') || 'en';

function applyLanguage(lang) {
  currentLang = lang;
  localStorage.setItem('ximi_lang', lang);
  const dict = I18N[lang] || I18N.en;

  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (dict[key]) {
      el.textContent = dict[key];
    }
  });

  document.querySelectorAll('[data-i18n-ph]').forEach(el => {
    const key = el.getAttribute('data-i18n-ph');
    if (dict[key]) {
      el.setAttribute('placeholder', dict[key]);
    }
  });

  const langSelect = document.getElementById('lang-select');
  if (langSelect) langSelect.value = lang;
}

// ==========================================================
// 3. THEME SYSTEM (Dark / Light)
// ==========================================================
let currentTheme = localStorage.getItem('ximi_theme') || 'dark';

function applyTheme(theme) {
  currentTheme = theme;
  localStorage.setItem('ximi_theme', theme);
  document.body.className = `theme-${theme}`;

  const darkBtn = document.getElementById('btn-theme-dark');
  const lightBtn = document.getElementById('btn-theme-light');
  if (darkBtn && lightBtn) {
    if (theme === 'dark') {
      darkBtn.classList.add('active');
      lightBtn.classList.remove('active');
    } else {
      lightBtn.classList.add('active');
      darkBtn.classList.remove('active');
    }
  }
}

// ==========================================================
// 4. FLOATING DOCK NAVIGATION
// ==========================================================
function setupDockNavigation() {
  const tabs = document.querySelectorAll('.dock-tab');
  const indicator = document.getElementById('dock-indicator');
  const views = document.querySelectorAll('.view');

  function updateIndicator(activeTab) {
    if (!indicator || !activeTab) return;
    const tabRect = activeTab.getBoundingClientRect();
    const dockRect = activeTab.parentElement.getBoundingClientRect();
    const leftOffset = tabRect.left - dockRect.left;

    indicator.style.transform = `translateX(${leftOffset}px)`;
    indicator.style.width = `${tabRect.width}px`;
  }

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const targetId = tab.getAttribute('data-target');
      
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      updateIndicator(tab);

      views.forEach(v => {
        if (v.id === targetId) {
          v.classList.add('active');
        } else {
          v.classList.remove('active');
        }
      });
    });
  });

  // Initial indicator position
  setTimeout(() => {
    const activeTab = document.querySelector('.dock-tab.active');
    if (activeTab) updateIndicator(activeTab);
  }, 100);

  window.addEventListener('resize', () => {
    const activeTab = document.querySelector('.dock-tab.active');
    if (activeTab) updateIndicator(activeTab);
  });
}

// ==========================================================
// 5. APPLICATION STATE
// ==========================================================
const AppState = {
  currentAdbDevice: null,
  adbDevices: [],
  fastbootDevices: [],
  currentFastbootDevice: null,
  packages: [],
  selectedPackages: new Set(),
  currentFilter: 'recommended',
  currentPartition: 'boot',
  pcCurrentPath: '/home',
  androidCurrentPath: '/sdcard',
  selectedPcItem: null,
  selectedAndroidItem: null,
};

// ==========================================================
// 6. DEVICE POLLING & SPECS
// ==========================================================
async function refreshDevices() {
  try {
    const devices = await invoke('get_adb_devices');
    AppState.adbDevices = devices || [];

    const adbSelect = document.getElementById('adb-device-select');
    const headerDevName = document.getElementById('header-device-name');
    const deviceDot = document.getElementById('device-dot');

    if (adbSelect) {
      adbSelect.innerHTML = `<option value="">${I18N[currentLang].select_device}</option>`;
      AppState.adbDevices.forEach(d => {
        const opt = document.createElement('option');
        opt.value = d.serial;
        opt.textContent = `${d.serial} (${d.state})`;
        adbSelect.appendChild(opt);
      });
    }

    if (AppState.adbDevices.length > 0) {
      AppState.currentAdbDevice = AppState.adbDevices[0].serial;
      if (adbSelect) adbSelect.value = AppState.currentAdbDevice;
      
      deviceDot.className = 'status-dot connected';
      headerDevName.textContent = AppState.currentAdbDevice;

      // Load specs and packages
      loadDeviceSpecs(AppState.currentAdbDevice);
      loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
      loadAndroidDirectory(AppState.androidCurrentPath);
    } else {
      AppState.currentAdbDevice = null;
      deviceDot.className = 'status-dot disconnected';
      headerDevName.textContent = I18N[currentLang].no_device;
    }
  } catch (e) {
    console.error('Error refreshing ADB devices:', e);
  }

  // Refresh Fastboot devices
  try {
    const fbDevices = await invoke('get_fastboot_devices');
    AppState.fastbootDevices = fbDevices || [];
    const fbSelect = document.getElementById('fastboot-device-select');
    if (fbSelect) {
      fbSelect.innerHTML = `<option value="">${I18N[currentLang].no_fastboot_dev}</option>`;
      AppState.fastbootDevices.forEach(d => {
        const opt = document.createElement('option');
        opt.value = d.serial;
        opt.textContent = `${d.serial} (${d.state})`;
        fbSelect.appendChild(opt);
      });
      if (AppState.fastbootDevices.length > 0) {
        AppState.currentFastbootDevice = AppState.fastbootDevices[0].serial;
        fbSelect.value = AppState.currentFastbootDevice;
      }
    }
  } catch (e) {
    console.error('Error refreshing fastboot devices:', e);
  }
}

async function loadDeviceSpecs(serial) {
  try {
    const specs = await invoke('get_device_specs', { serial });
    if (!specs) return;

    // Quick specs
    document.getElementById('qs-model').textContent = specs.model || '-';
    document.getElementById('qs-codename').textContent = specs.device || '-';
    document.getElementById('qs-hyperos').textContent = specs.hyperos_version || '-';
    document.getElementById('qs-android').textContent = specs.android_ver || '-';
    document.getElementById('qs-battery').textContent = specs.battery || '-';

    // About Phone Card (example.png replica)
    document.getElementById('ap-brand-title').textContent = `${specs.brand || 'Xiaomi'} HyperOS`;
    document.getElementById('ap-version-incremental').textContent = specs.hyperos_version || 'OS4.0.0.3';
    document.getElementById('ap-market-name').textContent = specs.market_name || specs.model || 'Xiaomi Device';
    document.getElementById('ap-codename').textContent = specs.device || 'tanzanite';
    document.getElementById('ap-cpu').textContent = specs.cpu || 'Octa-core';
    document.getElementById('ap-ram').textContent = specs.ram || '8.0GB';
    document.getElementById('ap-storage').textContent = specs.storage || '72.5GB/256GB';
    document.getElementById('ap-battery').textContent = specs.battery || '5000mAh';
    document.getElementById('ap-android').textContent = specs.android_ver || '14';
    document.getElementById('ap-security').textContent = specs.security_patch || '-';
  } catch (e) {
    console.error('Failed to load specs:', e);
  }
}

// ==========================================================
// 7. ADB DEBLOATER & PACKAGE MANAGEMENT
// ==========================================================
async function loadPackages(serial, filterMode) {
  const tbody = document.getElementById('package-list-tbody');
  tbody.innerHTML = `<tr><td colspan="5" class="table-empty">Loading packages...</td></tr>`;

  try {
    const pkgs = await invoke('get_packages', {
      serial: serial || null,
      filterMode: filterMode || 'recommended'
    });

    AppState.packages = pkgs || [];
    renderPackagesTable();
  } catch (e) {
    tbody.innerHTML = `<tr><td colspan="5" class="table-empty text-danger">Failed to load packages: ${e}</td></tr>`;
  }
}

function renderPackagesTable() {
  const tbody = document.getElementById('package-list-tbody');
  const query = (document.getElementById('debloat-search').value || '').toLowerCase();
  tbody.innerHTML = '';

  const filtered = AppState.packages.filter(p => {
    return p.name.toLowerCase().includes(query) || p.package.toLowerCase().includes(query);
  });

  if (filtered.length === 0) {
    tbody.innerHTML = `<tr><td colspan="5" class="table-empty">No matching packages found</td></tr>`;
    return;
  }

  filtered.forEach(p => {
    const tr = document.createElement('tr');
    const isChecked = AppState.selectedPackages.has(p.package);

    tr.innerHTML = `
      <td><input type="checkbox" class="pkg-check" data-pkg="${p.package}" ${isChecked ? 'checked' : ''}></td>
      <td>
        <span class="pkg-title">${p.name}</span>
        <span class="pkg-id">${p.package}</span>
      </td>
      <td><span class="badge-category">${p.category}</span></td>
      <td><span class="badge-risk ${p.risk}">${p.risk}</span></td>
      <td>
        <button class="btn btn-secondary btn-icon-tiny btn-pkg-action" data-action="uninstall" data-pkg="${p.package}" title="Uninstall">🗑️</button>
        <button class="btn btn-secondary btn-icon-tiny btn-pkg-action" data-action="disable" data-pkg="${p.package}" title="Disable">⏸️</button>
      </td>
    `;
    tbody.appendChild(tr);
  });

  // Checkbox listeners
  tbody.querySelectorAll('.pkg-check').forEach(chk => {
    chk.addEventListener('change', (e) => {
      const pkg = e.target.getAttribute('data-pkg');
      if (e.target.checked) {
        AppState.selectedPackages.add(pkg);
      } else {
        AppState.selectedPackages.delete(pkg);
      }
      updateDebloatFooter();
    });
  });

  // Single action buttons
  tbody.querySelectorAll('.btn-pkg-action').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      const action = btn.getAttribute('data-action');
      const pkg = btn.getAttribute('data-pkg');
      if (action === 'uninstall') {
        if (confirm(`Uninstall ${pkg}?`)) {
          await invoke('uninstall_package', { serial: AppState.currentAdbDevice, package: pkg });
          loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
        }
      } else if (action === 'disable') {
        await invoke('disable_package', { serial: AppState.currentAdbDevice, package: pkg });
        loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
      }
    });
  });

  updateDebloatFooter();
}

function updateDebloatFooter() {
  const count = AppState.selectedPackages.size;
  document.getElementById('selected-count').textContent = `${count} items selected`;
  const disableBtns = count === 0;

  document.getElementById('btn-uninstall-selected').disabled = disableBtns;
  document.getElementById('btn-disable-selected').disabled = disableBtns;
  document.getElementById('btn-enable-selected').disabled = disableBtns;
}

// ==========================================================
// 8. MT MANAGER DUAL-PANEL FILE EXPLORER
// ==========================================================
async function loadPcDirectory(dirPath) {
  const container = document.getElementById('pc-file-list');
  container.innerHTML = '<div class="table-empty">Loading PC files...</div>';

  try {
    const items = await invoke('list_pc_directory', { dirPath });
    AppState.pcCurrentPath = dirPath;
    document.getElementById('pc-current-path').value = dirPath;
    document.getElementById('pc-status').textContent = `${items.length} items`;

    renderFileList(container, items, 'pc');
  } catch (e) {
    container.innerHTML = `<div class="table-empty text-danger">Error: ${e}</div>`;
  }
}

async function loadAndroidDirectory(dirPath) {
  const container = document.getElementById('android-file-list');
  const rootMode = document.getElementById('explorer-root-toggle').checked;
  container.innerHTML = '<div class="table-empty">Loading Android files...</div>';

  try {
    const items = await invoke('list_android_directory', {
      serial: AppState.currentAdbDevice,
      dirPath,
      rootMode
    });
    AppState.androidCurrentPath = dirPath;
    document.getElementById('android-current-path').value = dirPath;
    document.getElementById('android-status').textContent = `${items.length} items`;

    renderFileList(container, items, 'android');
  } catch (e) {
    container.innerHTML = `<div class="table-empty text-danger">Error: ${e}</div>`;
  }
}

function renderFileList(container, items, panelType) {
  container.innerHTML = '';
  if (items.length === 0) {
    container.innerHTML = '<div class="table-empty">Empty directory</div>';
    return;
  }

  items.forEach(item => {
    const row = document.createElement('div');
    row.className = 'file-row';
    row.setAttribute('data-path', item.path);
    row.setAttribute('data-isdir', item.is_dir);

    const icon = item.is_dir ? '📁' : (item.name.endsWith('.apk') ? '📦' : (item.name.endsWith('.zip') ? '🗜️' : '📄'));

    row.innerHTML = `
      <div class="file-icon">${icon}</div>
      <div class="file-meta">
        <div class="file-name" title="${item.name}">${item.name}</div>
        <div class="file-sub">${item.date} • ${item.permissions}</div>
      </div>
      <div class="file-size">${item.size_formatted}</div>
    `;

    // Click select
    row.addEventListener('click', (e) => {
      container.querySelectorAll('.file-row').forEach(r => r.classList.remove('selected'));
      row.classList.add('selected');
      if (panelType === 'pc') {
        AppState.selectedPcItem = item;
      } else {
        AppState.selectedAndroidItem = item;
      }
    });

    // Double click navigate
    row.addEventListener('dblclick', () => {
      if (item.is_dir) {
        if (panelType === 'pc') {
          loadPcDirectory(item.path);
        } else {
          loadAndroidDirectory(item.path);
        }
      }
    });

    // Right-click context menu
    row.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      container.querySelectorAll('.file-row').forEach(r => r.classList.remove('selected'));
      row.classList.add('selected');
      if (panelType === 'pc') {
        AppState.selectedPcItem = item;
      } else {
        AppState.selectedAndroidItem = item;
      }
      showContextMenu(e.clientX, e.clientY, item, panelType);
    });

    container.appendChild(row);
  });
}

function showContextMenu(x, y, item, panelType) {
  const menu = document.getElementById('file-context-menu');
  menu.style.left = `${x}px`;
  menu.style.top = `${y}px`;
  menu.classList.remove('hidden');

  menu.setAttribute('data-path', item.path);
  menu.setAttribute('data-panel', panelType);
  menu.setAttribute('data-isdir', item.is_dir);
}

// ==========================================================
// 9. TERMINAL SHELL & FASTFETCH
// ==========================================================
function appendTerminal(text, isAnsi = false) {
  const screen = document.getElementById('terminal-screen');
  const div = document.createElement('div');
  
  if (isAnsi) {
    div.innerHTML = ansiToHtml(text);
  } else {
    div.textContent = text;
  }

  screen.appendChild(div);
  screen.scrollTop = screen.scrollHeight;
}

function ansiToHtml(str) {
  // Simple ANSI 256/RGB to HTML converter
  return str
    .replace(/\x1b\[38;2;(\d+);(\d+);(\d+)m/g, '<span style="color:rgb($1,$2,$3)">')
    .replace(/\x1b\[1m/g, '<b>')
    .replace(/\x1b\[0m/g, '</span></b>')
    .replace(/\n/g, '<br>');
}

async function runTerminalCommand(cmd) {
  if (!cmd.trim()) return;
  const rootMode = document.getElementById('terminal-root-toggle').checked;
  const prompt = rootMode ? 'root@hyperos:~#' : 'shell@hyperos:~$';
  appendTerminal(`${prompt} ${cmd}`);

  if (cmd === 'clear') {
    document.getElementById('terminal-screen').innerHTML = '';
    return;
  }

  if (cmd === 'fastfetch') {
    try {
      const out = await invoke('run_fastfetch', { serial: AppState.currentAdbDevice, rootMode });
      appendTerminal(out, true);
    } catch (e) {
      appendTerminal(`Error: ${e}`);
    }
    return;
  }

  try {
    const out = await invoke('execute_shell', {
      serial: AppState.currentAdbDevice,
      command: cmd,
      rootMode
    });
    appendTerminal(out);
  } catch (e) {
    appendTerminal(`Error: ${e}`);
  }
}

// ==========================================================
// 9.5. APK SIDELOAD & LIVE LOGCAT CONTROLLERS
// ==========================================================
async function installApkFile(filePath) {
  if (!filePath) return;
  if (!AppState.currentAdbDevice) {
    alert(I18N[currentLang].select_device || 'Please connect and select an ADB device first.');
    return;
  }
  const fileName = filePath.split('/').pop().split('\\').pop();
  const dropzone = document.getElementById('apk-dropzone');
  const icon = document.getElementById('drop-icon');
  const text = document.getElementById('drop-text');

  if (dropzone) dropzone.classList.add('installing');
  if (icon) {
    icon.innerHTML = '🔄';
    icon.classList.add('spin');
  }
  if (text) text.textContent = `Installing ${fileName}...`;

  try {
    const res = await invoke('install_apk', {
      serial: AppState.currentAdbDevice,
      apkPath: filePath
    });
    if (dropzone) {
      dropzone.classList.remove('installing');
      dropzone.classList.add('success');
    }
    if (icon) {
      icon.innerHTML = '✅';
      icon.classList.remove('spin');
    }
    if (text) text.textContent = `Installed: ${fileName}!`;
    setTimeout(() => {
      if (dropzone) dropzone.classList.remove('success');
      if (icon) icon.innerHTML = '📥';
      if (text) text.textContent = I18N[currentLang].drag_apk || 'Drag & Drop APK here to install';
    }, 3500);
    // Refresh package list
    if (AppState.currentAdbDevice) {
      loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
    }
  } catch (err) {
    if (dropzone) dropzone.classList.remove('installing');
    if (icon) {
      icon.innerHTML = '❌';
      icon.classList.remove('spin');
    }
    if (text) text.textContent = `Install Failed: ${err}`;
    setTimeout(() => {
      if (icon) icon.innerHTML = '📥';
      if (text) text.textContent = I18N[currentLang].drag_apk || 'Drag & Drop APK here to install';
    }, 4500);
  }
}

async function selectAndInstallApk() {
  if (!AppState.currentAdbDevice) {
    alert(I18N[currentLang].select_device || 'Please connect and select an ADB device first.');
    return;
  }
  try {
    const path = await invoke('pick_file', {
      title: 'Select APK Package to Install',
      filterName: 'Android Package (*.apk)',
      filterPattern: '*.apk'
    });
    if (path) {
      installApkFile(path);
    }
  } catch (e) {
    console.error('File picker error:', e);
  }
}

function setupApkDragDrop() {
  const dropzone = document.getElementById('apk-dropzone');
  if (!dropzone) return;

  // Click on dropzone or Install APK button
  dropzone.addEventListener('click', selectAndInstallApk);
  const btnInstall = document.getElementById('btn-install-apk');
  if (btnInstall) {
    btnInstall.addEventListener('click', selectAndInstallApk);
  }

  // HTML5 Drag & Drop
  dropzone.addEventListener('dragover', (e) => {
    e.preventDefault();
    e.stopPropagation();
    dropzone.classList.add('drag-active');
  });

  dropzone.addEventListener('dragleave', (e) => {
    e.preventDefault();
    e.stopPropagation();
    dropzone.classList.remove('drag-active');
  });

  dropzone.addEventListener('drop', (e) => {
    e.preventDefault();
    e.stopPropagation();
    dropzone.classList.remove('drag-active');
    if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      const p = file.path || file.name;
      if (p && p.toLowerCase().endsWith('.apk')) {
        installApkFile(p);
      }
    }
  });

  // Tauri v2 Native Window Drag & Drop
  if (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen) {
    window.__TAURI__.event.listen('tauri://drag-enter', () => {
      dropzone.classList.add('drag-active');
    });

    window.__TAURI__.event.listen('tauri://drag-leave', () => {
      dropzone.classList.remove('drag-active');
    });

    window.__TAURI__.event.listen('tauri://drag-drop', (event) => {
      dropzone.classList.remove('drag-active');
      const paths = event.payload && event.payload.paths ? event.payload.paths : [];
      if (paths.length > 0) {
        const apkFile = paths.find(p => p.toLowerCase().endsWith('.apk')) || paths[0];
        if (apkFile && apkFile.toLowerCase().endsWith('.apk')) {
          installApkFile(apkFile);
        } else {
          alert('Please drop an .apk package file.');
        }
      }
    });
  }
}

const LogcatState = {
  isStreaming: false,
  lines: [],
  maxLines: 2500,
  filterText: '',
  level: 'V',
  autoScroll: true
};

function formatLogcatLine(line) {
  let severityClass = 'log-v';
  if (/\bE\/|\sE\s|ERROR|FATAL/i.test(line)) {
    severityClass = 'log-e';
  } else if (/\bW\/|\sW\s|WARN/i.test(line)) {
    severityClass = 'log-w';
  } else if (/\bI\/|\sI\s|INFO/i.test(line)) {
    severityClass = 'log-i';
  } else if (/\bD\/|\sD\s|DEBUG/i.test(line)) {
    severityClass = 'log-d';
  }
  const div = document.createElement('div');
  div.className = `logcat-line ${severityClass}`;
  div.textContent = line;
  return div;
}

function appendLogcatLine(line) {
  LogcatState.lines.push(line);
  if (LogcatState.lines.length > LogcatState.maxLines) {
    LogcatState.lines.shift();
  }

  const search = LogcatState.filterText.toLowerCase();
  if (search && !line.toLowerCase().includes(search)) {
    return;
  }

  const viewer = document.getElementById('logcat-viewer');
  if (!viewer) return;

  const lineElem = formatLogcatLine(line);
  viewer.appendChild(lineElem);

  while (viewer.childElementCount > LogcatState.maxLines) {
    viewer.removeChild(viewer.firstElementChild);
  }

  const counter = document.getElementById('logcat-counter');
  if (counter) counter.textContent = `${viewer.childElementCount} lines`;

  if (LogcatState.autoScroll) {
    viewer.scrollTop = viewer.scrollHeight;
  }
}

async function startLogcatStream() {
  if (!AppState.currentAdbDevice) {
    alert(I18N[currentLang].select_device || 'Please connect an ADB device first.');
    return;
  }

  try {
    const level = document.getElementById('logcat-level-select').value;
    const filter = document.getElementById('logcat-search-filter').value.trim();

    await invoke('start_logcat_stream', {
      serial: AppState.currentAdbDevice,
      filter: filter || null,
      level: level || null
    });

    LogcatState.isStreaming = true;
    updateLogcatControls();
  } catch (err) {
    alert(`Failed to start logcat: ${err}`);
  }
}

async function stopLogcatStream() {
  try {
    await invoke('stop_logcat_stream');
  } catch (e) {
    console.error('Stop logcat error:', e);
  }
  LogcatState.isStreaming = false;
  updateLogcatControls();
}

function updateLogcatControls() {
  const toggleBtn = document.getElementById('btn-toggle-logcat');
  const badge = document.getElementById('logcat-status-badge');
  if (!toggleBtn || !badge) return;
  if (LogcatState.isStreaming) {
    toggleBtn.textContent = '⏹ Stop';
    toggleBtn.className = 'btn btn-danger btn-sm';
    badge.className = 'badge-live live';
    badge.textContent = '● LIVE';
  } else {
    toggleBtn.textContent = '▶ Start';
    toggleBtn.className = 'btn btn-primary btn-sm';
    badge.className = 'badge-live stopped';
    badge.textContent = '⏹ STOPPED';
  }
}

function setupLogcatModal() {
  const modal = document.getElementById('logcat-modal');
  const btnDumpLogcat = document.getElementById('btn-dump-logcat');
  const btnClose = document.getElementById('btn-close-logcat');
  const btnCloseFooter = document.getElementById('btn-close-logcat-footer');
  const toggleBtn = document.getElementById('btn-toggle-logcat');
  const clearBtn = document.getElementById('btn-clear-logcat');
  const exportBtn = document.getElementById('btn-export-logcat');
  const filterInput = document.getElementById('logcat-search-filter');
  const levelSelect = document.getElementById('logcat-level-select');
  const autoscrollCheck = document.getElementById('logcat-autoscroll');

  if (!btnDumpLogcat || !modal) return;

  btnDumpLogcat.addEventListener('click', async () => {
    if (!AppState.currentAdbDevice) {
      alert(I18N[currentLang].select_device || 'Please select an ADB device first.');
      return;
    }
    const tag = document.getElementById('logcat-device-tag');
    if (tag) tag.textContent = AppState.currentAdbDevice;
    modal.classList.remove('hidden');
    startLogcatStream();
  });

  const closeModal = async () => {
    modal.classList.add('hidden');
    if (LogcatState.isStreaming) {
      await stopLogcatStream();
    }
  };

  if (btnClose) btnClose.addEventListener('click', closeModal);
  if (btnCloseFooter) btnCloseFooter.addEventListener('click', closeModal);

  if (toggleBtn) {
    toggleBtn.addEventListener('click', () => {
      if (LogcatState.isStreaming) {
        stopLogcatStream();
      } else {
        startLogcatStream();
      }
    });
  }

  if (clearBtn) {
    clearBtn.addEventListener('click', async () => {
      const viewer = document.getElementById('logcat-viewer');
      if (viewer) viewer.innerHTML = '';
      LogcatState.lines = [];
      const counter = document.getElementById('logcat-counter');
      if (counter) counter.textContent = '0 lines';
      try {
        await invoke('clear_logcat', { serial: AppState.currentAdbDevice });
      } catch (e) {
        console.warn('Clear logcat buffer warning:', e);
      }
    });
  }

  if (exportBtn) {
    exportBtn.addEventListener('click', () => {
      if (LogcatState.lines.length === 0) {
        alert('Log buffer is empty.');
        return;
      }
      const blob = new Blob([LogcatState.lines.join('\n')], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `logcat_${AppState.currentAdbDevice || 'device'}_${Date.now()}.txt`;
      a.click();
      URL.revokeObjectURL(url);
    });
  }

  if (filterInput) {
    filterInput.addEventListener('input', (e) => {
      LogcatState.filterText = e.target.value.trim();
      const viewer = document.getElementById('logcat-viewer');
      if (!viewer) return;
      viewer.innerHTML = '';
      const kw = LogcatState.filterText.toLowerCase();
      for (const l of LogcatState.lines) {
        if (!kw || l.toLowerCase().includes(kw)) {
          viewer.appendChild(formatLogcatLine(l));
        }
      }
      const counter = document.getElementById('logcat-counter');
      if (counter) counter.textContent = `${viewer.childElementCount} lines`;
      if (LogcatState.autoScroll) viewer.scrollTop = viewer.scrollHeight;
    });
  }

  if (levelSelect) {
    levelSelect.addEventListener('change', () => {
      if (LogcatState.isStreaming) {
        startLogcatStream();
      }
    });
  }

  if (autoscrollCheck) {
    autoscrollCheck.addEventListener('change', (e) => {
      LogcatState.autoScroll = e.target.checked;
    });
  }

  if (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen) {
    window.__TAURI__.event.listen('logcat-line', (event) => {
      if (LogcatState.isStreaming && event.payload) {
        appendLogcatLine(event.payload);
      }
    });
  }
}

// ==========================================================
// 10. INITIALIZATION & EVENT LISTENERS
// ==========================================================
document.addEventListener('DOMContentLoaded', () => {
  // 1. Initialize Theme & Language
  applyTheme(currentTheme);
  applyLanguage(currentLang);

  // 2. Dock navigation
  setupDockNavigation();

  // 3. Theme toggles
  document.getElementById('btn-theme-dark').addEventListener('click', () => applyTheme('dark'));
  document.getElementById('btn-theme-light').addEventListener('click', () => applyTheme('light'));

  // 4. Language select
  document.getElementById('lang-select').addEventListener('change', (e) => applyLanguage(e.target.value));

  // 5. Social & Links
  document.getElementById('btn-link-github').addEventListener('click', () => {
    invoke('open_url', { url: 'https://github.com/iprjkt' });
  });
  document.getElementById('btn-link-telegram').addEventListener('click', () => {
    invoke('open_url', { url: 'https://t.me/anotherside551' });
  });

  // 6. Device Refresh Buttons
  document.getElementById('btn-refresh-devices').addEventListener('click', refreshDevices);
  document.getElementById('btn-refresh-fastboot').addEventListener('click', refreshDevices);

  // 7. Device Selection Change
  document.getElementById('adb-device-select').addEventListener('change', (e) => {
    AppState.currentAdbDevice = e.target.value;
    if (AppState.currentAdbDevice) {
      loadDeviceSpecs(AppState.currentAdbDevice);
      loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
      loadAndroidDirectory(AppState.androidCurrentPath);
    }
  });

  // 8. Reboot Menu Dropdown
  const rebootBtn = document.getElementById('btn-reboot-menu');
  const rebootMenu = document.getElementById('reboot-dropdown');
  rebootBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    rebootMenu.classList.toggle('hidden');
  });
  document.addEventListener('click', () => rebootMenu.classList.add('hidden'));

  document.querySelectorAll('.dropdown-item').forEach(item => {
    item.addEventListener('click', async () => {
      const action = item.getAttribute('data-action');
      const mode = action.replace('reboot-', '');
      if (confirm(`Reboot device to ${mode}?`)) {
        await invoke('reboot_device', { serial: AppState.currentAdbDevice, mode });
      }
    });
  });

  // 9. Debloater Filter Chips & Search
  document.querySelectorAll('.filter-chip').forEach(chip => {
    chip.addEventListener('click', () => {
      const filter = chip.getAttribute('data-filter');
      if (!filter) return;
      document.querySelectorAll('.filter-chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      AppState.currentFilter = filter;
      loadPackages(AppState.currentAdbDevice, filter);
    });
  });

  document.getElementById('debloat-search').addEventListener('input', renderPackagesTable);

  document.getElementById('check-all-packages').addEventListener('change', (e) => {
    if (e.target.checked) {
      AppState.packages.forEach(p => AppState.selectedPackages.add(p.package));
    } else {
      AppState.selectedPackages.clear();
    }
    renderPackagesTable();
  });

  // Batch actions
  document.getElementById('btn-uninstall-selected').addEventListener('click', async () => {
    if (confirm(`Uninstall ${AppState.selectedPackages.size} selected packages?`)) {
      for (const pkg of AppState.selectedPackages) {
        await invoke('uninstall_package', { serial: AppState.currentAdbDevice, package: pkg });
      }
      AppState.selectedPackages.clear();
      loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
    }
  });

  document.getElementById('btn-disable-selected').addEventListener('click', async () => {
    for (const pkg of AppState.selectedPackages) {
      await invoke('disable_package', { serial: AppState.currentAdbDevice, package: pkg });
    }
    AppState.selectedPackages.clear();
    loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
  });

  document.getElementById('btn-enable-selected').addEventListener('click', async () => {
    for (const pkg of AppState.selectedPackages) {
      await invoke('enable_package', { serial: AppState.currentAdbDevice, package: pkg });
    }
    AppState.selectedPackages.clear();
    loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
  });

  // Restore Modal
  const restoreModal = document.getElementById('restore-modal');
  document.getElementById('btn-restore-package').addEventListener('click', () => {
    restoreModal.classList.remove('hidden');
  });
  document.getElementById('btn-close-restore').addEventListener('click', () => restoreModal.classList.add('hidden'));
  document.getElementById('btn-cancel-restore').addEventListener('click', () => restoreModal.classList.add('hidden'));
  document.getElementById('btn-confirm-restore').addEventListener('click', async () => {
    const pkg = document.getElementById('restore-package-input').value.trim();
    if (pkg) {
      await invoke('restore_package', { serial: AppState.currentAdbDevice, package: pkg });
      restoreModal.classList.add('hidden');
      loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
    }
  });

  // Quick Tools
  document.getElementById('btn-take-screenshot').addEventListener('click', async () => {
    try {
      const res = await invoke('take_screenshot', { serial: AppState.currentAdbDevice });
      alert(res);
    } catch (e) {
      alert(`Error: ${e}`);
    }
  });

  document.getElementById('btn-screen-mirror').addEventListener('click', async () => {
    try {
      await invoke('screen_mirror', { serial: AppState.currentAdbDevice });
    } catch (e) {
      alert(`scrcpy error: ${e}`);
    }
  });

  // Setup Sideload Drag & Drop and Live Logcat Modal
  setupApkDragDrop();
  setupLogcatModal();

  // 10. MT Manager Transfer & Navigation
  document.getElementById('btn-pc-go').addEventListener('click', () => {
    loadPcDirectory(document.getElementById('pc-current-path').value.trim());
  });
  document.getElementById('btn-pc-home').addEventListener('click', () => loadPcDirectory('/home'));
  document.getElementById('btn-pc-up').addEventListener('click', () => {
    const parent = AppState.pcCurrentPath.substring(0, AppState.pcCurrentPath.lastIndexOf('/')) || '/';
    loadPcDirectory(parent);
  });
  document.getElementById('btn-pc-refresh').addEventListener('click', () => loadPcDirectory(AppState.pcCurrentPath));

  document.getElementById('btn-android-go').addEventListener('click', () => {
    loadAndroidDirectory(document.getElementById('android-current-path').value.trim());
  });
  document.getElementById('btn-android-sdcard').addEventListener('click', () => loadAndroidDirectory('/sdcard'));
  document.getElementById('btn-android-root').addEventListener('click', () => loadAndroidDirectory('/'));
  document.getElementById('btn-android-up').addEventListener('click', () => {
    const parent = AppState.androidCurrentPath.substring(0, AppState.androidCurrentPath.lastIndexOf('/')) || '/';
    loadAndroidDirectory(parent);
  });
  document.getElementById('btn-android-refresh').addEventListener('click', () => loadAndroidDirectory(AppState.androidCurrentPath));

  // Bidirectional Transfer Buttons
  document.getElementById('btn-push-to-android').addEventListener('click', async () => {
    if (!AppState.selectedPcItem) {
      alert('Please select a file or folder from the PC panel to push.');
      return;
    }
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    try {
      await invoke('transfer_pc_to_android', {
        serial: AppState.currentAdbDevice,
        localPath: AppState.selectedPcItem.path,
        remoteDir: AppState.androidCurrentPath,
        rootMode
      });
      loadAndroidDirectory(AppState.androidCurrentPath);
    } catch (e) {
      alert(`Transfer failed: ${e}`);
    }
  });

  document.getElementById('btn-pull-to-pc').addEventListener('click', async () => {
    if (!AppState.selectedAndroidItem) {
      alert('Please select a file or folder from the Android panel to pull.');
      return;
    }
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    try {
      await invoke('transfer_android_to_pc', {
        serial: AppState.currentAdbDevice,
        remotePath: AppState.selectedAndroidItem.path,
        localDir: AppState.pcCurrentPath,
        rootMode
      });
      loadPcDirectory(AppState.pcCurrentPath);
    } catch (e) {
      alert(`Transfer failed: ${e}`);
    }
  });

  // Context Menu Global Dismiss
  document.addEventListener('click', () => {
    document.getElementById('file-context-menu').classList.add('hidden');
  });

  // Context Menu Actions
  document.getElementById('ctx-delete').addEventListener('click', async () => {
    const menu = document.getElementById('file-context-menu');
    const path = menu.getAttribute('data-path');
    const panel = menu.getAttribute('data-panel');
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    if (confirm(`Delete ${path}?`)) {
      await invoke('delete_item', {
        serial: AppState.currentAdbDevice,
        path,
        isAndroid: panel === 'android',
        rootMode
      });
      if (panel === 'pc') loadPcDirectory(AppState.pcCurrentPath);
      else loadAndroidDirectory(AppState.androidCurrentPath);
    }
  });

  document.getElementById('ctx-rename').addEventListener('click', async () => {
    const menu = document.getElementById('file-context-menu');
    const path = menu.getAttribute('data-path');
    const panel = menu.getAttribute('data-panel');
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    const newName = prompt('Enter new name:', path.split('/').pop());
    if (newName) {
      const parent = path.substring(0, path.lastIndexOf('/'));
      const newPath = `${parent}/${newName}`;
      await invoke('rename_item', {
        serial: AppState.currentAdbDevice,
        oldPath: path,
        newPath,
        isAndroid: panel === 'android',
        rootMode
      });
      if (panel === 'pc') loadPcDirectory(AppState.pcCurrentPath);
      else loadAndroidDirectory(AppState.androidCurrentPath);
    }
  });

  // Text Editor Modal
  const editorModal = document.getElementById('editor-modal');
  let currentEditingPath = null;
  let currentEditingPanel = null;

  document.getElementById('ctx-edit').addEventListener('click', async () => {
    const menu = document.getElementById('file-context-menu');
    const path = menu.getAttribute('data-path');
    const panel = menu.getAttribute('data-panel');
    const rootMode = document.getElementById('explorer-root-toggle').checked;

    try {
      const content = await invoke('read_file_text', {
        serial: AppState.currentAdbDevice,
        path,
        isAndroid: panel === 'android',
        rootMode
      });
      currentEditingPath = path;
      currentEditingPanel = panel;
      document.getElementById('editor-modal-title').textContent = `Edit: ${path}`;
      document.getElementById('editor-content').value = content;
      editorModal.classList.remove('hidden');
    } catch (e) {
      alert(`Could not open file: ${e}`);
    }
  });

  document.getElementById('btn-close-editor').addEventListener('click', () => editorModal.classList.add('hidden'));
  document.getElementById('btn-cancel-editor').addEventListener('click', () => editorModal.classList.add('hidden'));
  document.getElementById('btn-save-editor').addEventListener('click', async () => {
    const content = document.getElementById('editor-content').value;
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    try {
      await invoke('write_file_text', {
        serial: AppState.currentAdbDevice,
        path: currentEditingPath,
        content,
        isAndroid: currentEditingPanel === 'android',
        rootMode
      });
      editorModal.classList.add('hidden');
      alert('File saved successfully!');
    } catch (e) {
      alert(`Save failed: ${e}`);
    }
  });

  // 11. Terminal Shell Events
  document.getElementById('btn-run-fastfetch').addEventListener('click', () => runTerminalCommand('fastfetch'));
  document.getElementById('btn-clear-terminal').addEventListener('click', () => {
    document.getElementById('terminal-screen').innerHTML = '';
  });

  document.querySelectorAll('.chip-cmd').forEach(chip => {
    chip.addEventListener('click', () => {
      const cmd = chip.getAttribute('data-cmd');
      runTerminalCommand(cmd);
    });
  });

  const termInput = document.getElementById('terminal-input');
  termInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      const cmd = termInput.value.trim();
      termInput.value = '';
      runTerminalCommand(cmd);
    }
  });
  document.getElementById('btn-send-cmd').addEventListener('click', () => {
    const cmd = termInput.value.trim();
    termInput.value = '';
    runTerminalCommand(cmd);
  });

  // 12. Fastboot Form Tabs & Chips
  const tabFlashSingle = document.getElementById('tab-flash-single');
  const tabFlashRom = document.getElementById('tab-flash-rom');
  const formSingle = document.getElementById('form-single-partition');
  const formRom = document.getElementById('form-full-rom');

  tabFlashSingle.addEventListener('click', () => {
    tabFlashSingle.classList.add('active');
    tabFlashRom.classList.remove('active');
    formSingle.classList.remove('hidden');
    formRom.classList.add('hidden');
  });

  tabFlashRom.addEventListener('click', () => {
    tabFlashRom.classList.add('active');
    tabFlashSingle.classList.remove('active');
    formRom.classList.remove('hidden');
    formSingle.classList.add('hidden');
  });

  document.querySelectorAll('.p-chip:not(.custom)').forEach(chip => {
    chip.addEventListener('click', () => {
      document.querySelectorAll('.p-chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      AppState.currentPartition = chip.getAttribute('data-p');
    });
  });

  document.getElementById('chip-custom-partition').addEventListener('click', () => {
    const custom = prompt('Enter custom partition name:');
    if (custom) {
      AppState.currentPartition = custom;
      const chip = document.getElementById('chip-custom-partition');
      chip.textContent = custom;
      chip.classList.add('active');
    }
  });

  document.getElementById('btn-clear-fastboot-log').addEventListener('click', () => {
    document.getElementById('fastboot-terminal-log').textContent = '[IDLE] Waiting for fastboot commands...';
    document.getElementById('fastboot-progress-bar').style.width = '0%';
  });

  // Fastboot file & folder pickers
  const btnBrowseImg = document.getElementById('btn-browse-image');
  if (btnBrowseImg) {
    btnBrowseImg.addEventListener('click', async () => {
      try {
        const path = await invoke('pick_file', {
          title: 'Select Partition Image',
          filterName: 'Disk Image (*.img)',
          filterPattern: '*.img'
        });
        if (path) {
          document.getElementById('single-image-path').value = path;
        }
      } catch (e) {
        console.error('Pick image error:', e);
      }
    });
  }

  const btnBrowseRom = document.getElementById('btn-browse-rom');
  if (btnBrowseRom) {
    btnBrowseRom.addEventListener('click', async () => {
      try {
        const path = await invoke('pick_folder', {
          title: 'Select Extracted Fastboot ROM Directory'
        });
        if (path) {
          document.getElementById('rom-folder-path').value = path;
          try {
            const romInfo = await invoke('parse_rom_directory', { folderPath: path });
            const scriptSelect = document.getElementById('rom-script-select');
            if (scriptSelect && romInfo && romInfo.scripts) {
              scriptSelect.innerHTML = '';
              romInfo.scripts.forEach(s => {
                const opt = document.createElement('option');
                opt.value = s;
                opt.textContent = s;
                scriptSelect.appendChild(opt);
              });
            }
          } catch (pe) {
            console.warn('ROM parse warning:', pe);
          }
        }
      } catch (e) {
        console.error('Pick ROM folder error:', e);
      }
    });
  }

  const btnFastbootReboot = document.getElementById('btn-fastboot-reboot');
  if (btnFastbootReboot) {
    btnFastbootReboot.addEventListener('click', async () => {
      if (confirm('Reboot Fastboot device to system?')) {
        try {
          await invoke('reboot_fastboot', {
            serial: AppState.currentFastbootDevice,
            target: 'system'
          });
        } catch (e) {
          alert(`Reboot failed: ${e}`);
        }
      }
    });
  }

  // Initial Data Fetch
  loadPcDirectory('/home');
  refreshDevices();
});
