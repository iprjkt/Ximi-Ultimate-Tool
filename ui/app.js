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

// Small shared helpers ------------------------------------------------
const esc = (v) => String(v ?? '').replace(/[&<>"']/g, (c) => (
  { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]
));

const debounce = (fn, ms = 150) => {
  let t;
  return (...a) => { clearTimeout(t); t = setTimeout(() => fn(...a), ms); };
};

const basename = (p) => String(p).split(/[\\/]/).pop();

const listen = (evt, cb) => (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen)
  ? window.__TAURI__.event.listen(evt, cb)
  : Promise.resolve(() => {});

let taskCounter = 0;
const newTaskId = (prefix = 'task') => `${prefix}-${Date.now()}-${++taskCounter}`;

const parentOf = (p) => {
  const trimmed = String(p).replace(/[\\/]+$/, '');
  const i = Math.max(trimmed.lastIndexOf('/'), trimmed.lastIndexOf('\\'));
  if (i < 0) return trimmed;
  if (i === 0) return '/';
  if (i === 2 && trimmed[1] === ':') return trimmed.slice(0, 3); // C:\
  return trimmed.slice(0, i);
};

// Per-task progress callbacks fed by the backend `task-progress` event
const taskHandlers = new Map();
async function runTask(prefix, onProgress, run) {
  const id = newTaskId(prefix);
  taskHandlers.set(id, onProgress);
  try {
    return await run(id);
  } finally {
    taskHandlers.delete(id);
  }
}
listen('task-progress', (e) => {
  const p = e.payload;
  const h = p && taskHandlers.get(p.id);
  if (h) h(p);
});

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
    screen_mirror: "Screen Mirror",
    install_apk: "Install APK",
    live_logcat: "Live Logcat",
    drag_apk: "Drag & Drop APK here to install",
    device_info_title: "Device Info",
    spec_model: "Model:",
    spec_codename: "Codename:",
    spec_hyperos: "ROM / OS:",
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
    select_rom_first: "Select Fastboot ROM directory first...",
    script_clean_all: "Clean All (Recommended)",
    script_save_data: "Save User Data",
    script_lock_bl: "Clean All & Lock Bootloader (CAUTION)",
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
    screen_mirror: "Cermin Layar",
    install_apk: "Pasang APK",
    live_logcat: "Logcat Langsung",
    drag_apk: "Tarik & Lepas APK ke sini untuk memasang",
    device_info_title: "Info Perangkat",
    spec_model: "Model:",
    spec_codename: "Codename:",
    spec_hyperos: "ROM / OS:",
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
    select_rom_first: "Pilih direktori ROM Fastboot terlebih dahulu...",
    script_clean_all: "Bersihkan Semua (Rekomendasi)",
    script_save_data: "Simpan Data Pengguna",
    script_lock_bl: "Bersihkan Semua & Kunci Bootloader (PERINGATAN)",
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
    if (el.id === 'ap-market-name' && AppState.currentAdbDevice) {
      return;
    }
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
  pcCurrentPath: '',
  androidCurrentPath: '/sdcard',
  selectedPcItem: null,
  selectedAndroidItem: null,
};

// ==========================================================
// 6. DEVICE POLLING & SPECS
// ==========================================================
const sameDevices = (a, b) =>
  a.length === b.length && a.every((d, i) => d.serial === b[i].serial && d.state === b[i].state);

let deviceRefreshBusy = false;

/**
 * Cheap device check. Only re-renders / reloads heavy data when the device list
 * changed (or `force` is set), and keeps the user's selected device.
 */
async function refreshDevices(force = false) {
  if (deviceRefreshBusy) return;
  deviceRefreshBusy = true;
  try {
    const [adb, fb] = await Promise.allSettled([invoke('get_adb_devices'), invoke('get_fastboot_devices')]);
    if (adb.status === 'fulfilled') applyAdbDevices(adb.value || [], force);
    else console.error('Error refreshing ADB devices:', adb.reason);
    if (fb.status === 'fulfilled') applyFastbootDevices(fb.value || [], force);
  } finally {
    deviceRefreshBusy = false;
  }
}

function applyAdbDevices(devices, force) {
  if (!force && sameDevices(devices, AppState.adbDevices)) return;
  AppState.adbDevices = devices;

  const adbSelect = document.getElementById('adb-device-select');
  const headerDevName = document.getElementById('header-device-name');
  const deviceDot = document.getElementById('device-dot');
  const previous = AppState.currentAdbDevice;

  if (adbSelect) {
    adbSelect.innerHTML = `<option value="">${esc(I18N[currentLang].select_device)}</option>` +
      devices.map((d) => `<option value="${esc(d.serial)}">${esc(d.serial)} (${esc(d.state)})</option>`).join('');
  }

  if (devices.length === 0) {
    AppState.currentAdbDevice = null;
    deviceDot.className = 'status-dot disconnected';
    headerDevName.textContent = I18N[currentLang].no_device;
    if (adbSelect) adbSelect.value = '';
    clearDeviceSpecs();
    return;
  }

  // keep the selection if that device is still attached, otherwise pick the first
  const keep = devices.find((d) => d.serial === previous);
  const chosen = keep || devices[0];
  AppState.currentAdbDevice = chosen.serial;
  if (adbSelect) adbSelect.value = chosen.serial;

  const ready = chosen.state === 'device';
  deviceDot.className = ready ? 'status-dot connected' : 'status-dot disconnected';
  headerDevName.textContent = ready ? chosen.serial : `${chosen.serial} (${chosen.state})`;

  if (force || AppState.specsLoadedFor !== `${chosen.serial}:${chosen.state}`) {
    onAdbDeviceSelected(chosen);
  }
}

function onAdbDeviceSelected(device) {
  AppState.specsLoadedFor = `${device.serial}:${device.state}`;
  if (device.state !== 'device') {
    clearDeviceSpecs();
    const msg = device.state === 'unauthorized'
      ? 'Unauthorized: accept the USB debugging prompt on the phone'
      : `Device is ${device.state}`;
    const apMarket = document.getElementById('ap-market-name');
    if (apMarket) apMarket.textContent = msg;
    return;
  }
  loadDeviceSpecs(device.serial);
  loadPackages(device.serial, AppState.currentFilter);
  loadAndroidDirectory(AppState.androidCurrentPath);
}

function applyFastbootDevices(devices, force) {
  if (!force && sameDevices(devices, AppState.fastbootDevices)) return;
  AppState.fastbootDevices = devices;
  const fbSelect = document.getElementById('fastboot-device-select');
  if (!fbSelect) return;
  const previous = AppState.currentFastbootDevice;
  fbSelect.innerHTML = `<option value="">${esc(I18N[currentLang].no_fastboot_dev)}</option>` +
    devices.map((d) => `<option value="${esc(d.serial)}">${esc(d.serial)} (${esc(d.state)})</option>`).join('');
  const keep = devices.find((d) => d.serial === previous);
  AppState.currentFastbootDevice = keep ? keep.serial : (devices[0] ? devices[0].serial : null);
  fbSelect.value = AppState.currentFastbootDevice || '';
}

let devicePollTimer = null;
function startDevicePolling(ms = 3000) {
  if (devicePollTimer) return;
  devicePollTimer = setInterval(() => {
    if (!document.hidden) refreshDevices();
  }, ms);
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) refreshDevices();
  });
}

function clearDeviceSpecs() {
  const dict = I18N[currentLang] || I18N.en;
  const noDevText = dict.no_device || 'No Device Connected';

  // Quick specs (ADB View)
  ['qs-model', 'qs-codename', 'qs-hyperos', 'qs-android', 'qs-battery'].forEach(id => {
    const el = document.getElementById(id);
    if (el) el.textContent = '-';
  });

  // About Phone Card (Settings View)
  const apBrand = document.getElementById('ap-brand-title');
  if (apBrand) apBrand.textContent = 'Xiaomi HyperOS';
  const apVer = document.getElementById('ap-version-incremental');
  if (apVer) apVer.textContent = '-';
  const apMarket = document.getElementById('ap-market-name');
  if (apMarket) apMarket.textContent = noDevText;
  const apCode = document.getElementById('ap-codename');
  if (apCode) apCode.textContent = '-';
  const apRomBadge = document.getElementById('ap-rom-badge');
  if (apRomBadge) {
    apRomBadge.textContent = '-';
    apRomBadge.className = 'device-rom-badge';
  }
  const apCpu = document.getElementById('ap-cpu');
  if (apCpu) apCpu.textContent = '-';
  const apRam = document.getElementById('ap-ram');
  if (apRam) apRam.textContent = '-';
  const apStorage = document.getElementById('ap-storage');
  if (apStorage) apStorage.textContent = '-';
  const apBattery = document.getElementById('ap-battery');
  if (apBattery) apBattery.textContent = '-';
  const apAndroid = document.getElementById('ap-android');
  if (apAndroid) apAndroid.textContent = '-';
  const apSecurity = document.getElementById('ap-security');
  if (apSecurity) apSecurity.textContent = '-';
}

async function loadDeviceSpecs(serial) {
  if (!serial) {
    clearDeviceSpecs();
    return;
  }
  try {
    const specs = await invoke('get_device_specs', { serial });
    if (AppState.currentAdbDevice !== serial) return; // user switched device meanwhile
    if (!specs) {
      clearDeviceSpecs();
      return;
    }

    const romType = specs.rom_type || 'AOSP';
    const romName = specs.rom_name || 'AOSP';
    const romVer = specs.rom_version && specs.rom_version !== '-' ? specs.rom_version : (specs.hyperos_version || '-');

    // Quick specs (ADB View)
    document.getElementById('qs-model').textContent = specs.model && specs.model !== '-' ? specs.model : '-';
    document.getElementById('qs-codename').textContent = specs.device && specs.device !== '-' ? specs.device : '-';
    
    // Quick specs ROM
    const qsRom = document.getElementById('qs-hyperos');
    if (qsRom) {
      if (romType === 'HyperOS' || romType === 'MIUI') {
        qsRom.textContent = `${romType} ${specs.hyperos_short || romVer}`;
      } else {
        qsRom.textContent = `${romName} (${specs.hyperos_short || romVer})`;
      }
    }

    document.getElementById('qs-android').textContent = specs.android_ver && specs.android_ver !== '-' ? specs.android_ver : '-';
    document.getElementById('qs-battery').textContent = specs.battery && specs.battery !== '-' ? specs.battery : '-';

    // About Phone Card (Settings View)
    let bannerTitle = romName;
    if (romType === 'HyperOS' || romType === 'MIUI') {
      const brand = specs.brand && specs.brand !== '-' ? specs.brand : 'Xiaomi';
      bannerTitle = `${brand} ${romType}`;
    } else {
      bannerTitle = `${romName} (AOSP)`;
    }

    document.getElementById('ap-brand-title').textContent = bannerTitle;
    document.getElementById('ap-version-incremental').textContent = romVer;
    document.getElementById('ap-market-name').textContent = specs.market_name && specs.market_name !== '-' ? specs.market_name : (specs.model && specs.model !== '-' ? specs.model : '-');
    document.getElementById('ap-codename').textContent = specs.device && specs.device !== '-' ? specs.device : '-';

    // ROM Type Badge
    const apRomBadge = document.getElementById('ap-rom-badge');
    if (apRomBadge) {
      apRomBadge.textContent = romType;
      const badgeCls = romType.toLowerCase().replace(/[^a-z0-9]/g, '');
      apRomBadge.className = `device-rom-badge badge-${badgeCls}`;
    }

    document.getElementById('ap-cpu').textContent = specs.cpu && specs.cpu !== '-' ? specs.cpu : '-';
    document.getElementById('ap-ram').textContent = specs.ram && specs.ram !== '-' ? specs.ram : '-';
    document.getElementById('ap-storage').textContent = specs.storage && specs.storage !== '-' ? specs.storage : '-';
    document.getElementById('ap-battery').textContent = specs.battery && specs.battery !== '-' ? specs.battery : '-';
    document.getElementById('ap-android').textContent = specs.android_ver && specs.android_ver !== '-' ? specs.android_ver : '-';
    document.getElementById('ap-security').textContent = specs.security_patch && specs.security_patch !== '-' ? specs.security_patch : '-';

    // header shows the friendly name once known
    const headerName = document.getElementById('header-device-name');
    if (headerName && specs.market_name && specs.market_name !== '-') {
      headerName.textContent = `${specs.market_name} · ${serial}`;
    }
  } catch (e) {
    if (AppState.currentAdbDevice !== serial) return; // stale response
    console.warn('Failed to load specs:', e);
    clearDeviceSpecs();
    const apMarket = document.getElementById('ap-market-name');
    if (apMarket) apMarket.textContent = String(e);
  }
}

// ==========================================================
// 7. ADB DEBLOATER & PACKAGE MANAGEMENT
// ==========================================================
let packageLoadToken = 0;

async function loadPackages(serial, filterMode) {
  const tbody = document.getElementById('package-list-tbody');
  const token = ++packageLoadToken;
  tbody.innerHTML = `<tr><td colspan="5" class="table-empty">Loading packages...</td></tr>`;

  try {
    const pkgs = await invoke('get_packages', {
      serial: serial || null,
      filterMode: filterMode || 'recommended'
    });
    if (token !== packageLoadToken) return; // a newer request superseded this one
    AppState.packages = pkgs || [];
    renderPackagesTable();
  } catch (e) {
    if (token !== packageLoadToken) return;
    tbody.innerHTML = `<tr><td colspan="5" class="table-empty text-danger">Failed to load packages: ${esc(e)}</td></tr>`;
  }
}

const PACKAGE_RENDER_LIMIT = 400;

function getFilteredPackages() {
  const query = (document.getElementById('debloat-search').value || '').toLowerCase();
  if (!query) return AppState.packages;
  return AppState.packages.filter((p) =>
    p.name.toLowerCase().includes(query) || p.package.toLowerCase().includes(query));
}

function renderPackagesTable() {
  const tbody = document.getElementById('package-list-tbody');
  const filtered = getFilteredPackages();

  if (filtered.length === 0) {
    tbody.innerHTML = `<tr><td colspan="5" class="table-empty">No matching packages found</td></tr>`;
    updateDebloatFooter();
    return;
  }

  const shown = filtered.slice(0, PACKAGE_RENDER_LIMIT);
  let html = shown.map((p) => `
    <tr>
      <td><input type="checkbox" class="pkg-check" data-pkg="${esc(p.package)}" ${AppState.selectedPackages.has(p.package) ? 'checked' : ''}></td>
      <td>
        <span class="pkg-title">${esc(p.name)}</span>
        <span class="pkg-id">${esc(p.package)}</span>
      </td>
      <td><span class="badge-category">${esc(p.category)}</span></td>
      <td><span class="badge-risk ${esc(p.risk)}">${esc(p.risk)}</span></td>
      <td>
        <button class="btn btn-secondary btn-icon-tiny btn-pkg-action" data-action="uninstall" data-pkg="${esc(p.package)}" title="Uninstall">🗑️</button>
        <button class="btn btn-secondary btn-icon-tiny btn-pkg-action" data-action="disable" data-pkg="${esc(p.package)}" title="Disable">⏸️</button>
      </td>
    </tr>`).join('');
  if (filtered.length > shown.length) {
    html += `<tr><td colspan="5" class="table-empty">${filtered.length - shown.length} more packages hidden. Use the search box to narrow the list.</td></tr>`;
  }
  tbody.innerHTML = html;
  updateDebloatFooter();
}

// One delegated listener instead of one per row
function setupPackageTableEvents() {
  const tbody = document.getElementById('package-list-tbody');

  tbody.addEventListener('change', (e) => {
    const chk = e.target.closest('.pkg-check');
    if (!chk) return;
    const pkg = chk.getAttribute('data-pkg');
    if (chk.checked) AppState.selectedPackages.add(pkg);
    else AppState.selectedPackages.delete(pkg);
    updateDebloatFooter();
  });

  tbody.addEventListener('click', async (e) => {
    const btn = e.target.closest('.btn-pkg-action');
    if (!btn) return;
    const action = btn.getAttribute('data-action');
    const pkg = btn.getAttribute('data-pkg');
    try {
      if (action === 'uninstall') {
        if (!confirm(`Uninstall ${pkg}?`)) return;
        await invoke('uninstall_package', { serial: AppState.currentAdbDevice, package: pkg });
      } else if (action === 'disable') {
        await invoke('disable_package', { serial: AppState.currentAdbDevice, package: pkg });
      }
      AppState.selectedPackages.delete(pkg);
      loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
    } catch (err) {
      alert(`Failed: ${err}`);
    }
  });
}

/** Run one package command per selected package, with progress and a failure summary. */
async function runPackageBatch(command, verb) {
  const serial = AppState.currentAdbDevice;
  const list = [...AppState.selectedPackages];
  if (!serial || list.length === 0) return;
  const counter = document.getElementById('selected-count');
  const failures = [];
  for (let i = 0; i < list.length; i++) {
    counter.textContent = `${verb} ${i + 1}/${list.length}...`;
    try {
      await invoke(command, { serial, package: list[i] });
      AppState.selectedPackages.delete(list[i]);
    } catch (e) {
      failures.push(`${list[i]}: ${e}`);
    }
  }
  await loadPackages(serial, AppState.currentFilter);
  if (failures.length) {
    alert(`${failures.length} of ${list.length} failed:\n\n${failures.slice(0, 8).join('\n')}${failures.length > 8 ? '\n...' : ''}`);
  }
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
    if (!dirPath) dirPath = await invoke('get_home_dir');
    const items = await invoke('list_pc_directory', { dirPath });
    AppState.pcCurrentPath = dirPath;
    document.getElementById('pc-current-path').value = dirPath;
    document.getElementById('pc-status').textContent = `${items.length} items`;

    renderFileList(container, items, 'pc');
  } catch (e) {
    container.innerHTML = `<div class="table-empty text-danger">Error: ${esc(e)}</div>`;
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
    container.innerHTML = `<div class="table-empty text-danger">Error: ${esc(e)}</div>`;
  }
}

const fileItemsByPanel = { pc: [], android: [] };

function fileIcon(item) {
  if (item.is_dir) return '📁';
  const n = item.name.toLowerCase();
  if (n.endsWith('.apk')) return '📦';
  if (n.endsWith('.zip')) return '🗜️';
  return '📄';
}

function renderFileList(container, items, panelType) {
  fileItemsByPanel[panelType] = items;
  if (items.length === 0) {
    container.innerHTML = '<div class="table-empty">Empty directory</div>';
    return;
  }
  container.innerHTML = items.map((item, i) => `
    <div class="file-row" data-idx="${i}">
      <div class="file-icon">${fileIcon(item)}</div>
      <div class="file-meta">
        <div class="file-name" title="${esc(item.name)}">${esc(item.name)}</div>
        <div class="file-sub">${esc(item.date)} • ${esc(item.permissions)}</div>
      </div>
      <div class="file-size">${esc(item.size_formatted)}</div>
    </div>`).join('');
}

function selectFileRow(container, row, panelType) {
  const prev = container.querySelector('.file-row.selected');
  if (prev) prev.classList.remove('selected');
  row.classList.add('selected');
  const item = fileItemsByPanel[panelType][Number(row.dataset.idx)];
  if (panelType === 'pc') AppState.selectedPcItem = item;
  else AppState.selectedAndroidItem = item;
  return item;
}

// Delegated explorer events (set up once per panel)
function setupFileListEvents(container, panelType) {
  const rowOf = (e) => e.target.closest('.file-row');

  container.addEventListener('click', (e) => {
    const row = rowOf(e);
    if (row) selectFileRow(container, row, panelType);
  });

  container.addEventListener('dblclick', (e) => {
    const row = rowOf(e);
    if (!row) return;
    const item = fileItemsByPanel[panelType][Number(row.dataset.idx)];
    if (item && item.is_dir) {
      if (panelType === 'pc') loadPcDirectory(item.path);
      else loadAndroidDirectory(item.path);
    }
  });

  container.addEventListener('contextmenu', (e) => {
    const row = rowOf(e);
    if (!row) return;
    e.preventDefault();
    const item = selectFileRow(container, row, panelType);
    showContextMenu(e.clientX, e.clientY, item, panelType);
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
  while (screen.childElementCount > 1000) screen.removeChild(screen.firstElementChild);
  screen.scrollTop = screen.scrollHeight;
}

function ansiToHtml(str) {
  // Simple ANSI RGB to HTML converter (input is escaped first)
  return esc(str)
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
  const fileName = basename(filePath);
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
    const res = await runTask('apk', (p) => {
      if (text && p.percent != null) text.textContent = `Installing ${fileName}... ${p.percent}%`;
    }, (taskId) => invoke('install_apk', {
      serial: AppState.currentAdbDevice,
      apkPath: filePath,
      taskId
    }));
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
    if (text) text.textContent = `Install Failed: ${String(err).slice(0, 160)}`;
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

// Fastboot Flasher Helper Functions
const FASTBOOT_LOG_MAX_LINES = 2000;

function appendFastbootLog(line) {
  const consoleElem = document.getElementById('fastboot-terminal-log');
  if (!consoleElem) return;
  if (consoleElem.dataset.idle !== 'false') {
    consoleElem.textContent = '';
    consoleElem.dataset.idle = 'false';
  }
  const div = document.createElement('div');
  div.textContent = line;
  consoleElem.appendChild(div);
  while (consoleElem.childElementCount > FASTBOOT_LOG_MAX_LINES) {
    consoleElem.removeChild(consoleElem.firstElementChild);
  }
  consoleElem.scrollTop = consoleElem.scrollHeight;
}

function setFastbootProgress(pct) {
  const bar = document.getElementById('fastboot-progress-bar');
  if (bar) bar.style.width = `${Math.max(0, Math.min(100, pct))}%`;
}

function autoSelectPartitionChip(partitionName) {
  if (!partitionName) return;
  const clean = partitionName.toLowerCase().trim();
  let matched = false;
  document.querySelectorAll('.p-chip:not(.custom)').forEach(chip => {
    if (chip.getAttribute('data-p').toLowerCase() === clean) {
      document.querySelectorAll('.p-chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      AppState.currentPartition = chip.getAttribute('data-p');
      matched = true;
    }
  });
  if (!matched) {
    AppState.currentPartition = clean;
    const customChip = document.getElementById('chip-custom-partition');
    if (customChip) {
      document.querySelectorAll('.p-chip').forEach(c => c.classList.remove('active'));
      customChip.textContent = clean;
      customChip.classList.add('active');
    }
  }
}

let currentRomPartitions = [];

async function handleRomDirectorySelected(folderPath) {
  if (!folderPath) return;
  document.getElementById('rom-folder-path').value = folderPath;
  appendFastbootLog(`[INFO] Parsing Fastboot ROM folder: ${folderPath}...`);

  try {
    const romInfo = await invoke('parse_rom_directory', { folderPath });
    if (!romInfo) return;

    // 1. Populate Script Select Dropdown
    const scriptSelect = document.getElementById('rom-script-select');
    if (scriptSelect && romInfo.scripts && romInfo.scripts.length > 0) {
      scriptSelect.innerHTML = '';
      const dict = I18N[currentLang] || I18N.en;
      romInfo.scripts.forEach(s => {
        const opt = document.createElement('option');
        opt.value = s;
        let desc = '';
        if (s.startsWith('flash_all.sh') || s.startsWith('flash_all.bat')) {
          desc = dict.script_clean_all || 'Clean All (Recommended)';
        } else if (s.includes('except')) {
          desc = dict.script_save_data || 'Save User Data';
        } else if (s.includes('lock')) {
          desc = dict.script_lock_bl || 'Clean All & Lock Bootloader (CAUTION)';
        }
        opt.textContent = desc ? `${s} (${desc})` : s;
        scriptSelect.appendChild(opt);
      });
      scriptSelect.value = romInfo.default_script;
    }

    // 2. Render Partitions Table
    currentRomPartitions = romInfo.partitions || [];
    renderRomPartitions(currentRomPartitions);
    appendFastbootLog(`[INFO] Loaded ${currentRomPartitions.length} partitions from ROM successfully!`);
  } catch (err) {
    appendFastbootLog(`[ERROR] Failed to parse ROM: ${err}`);
  }
}

function renderRomPartitions(partitions) {
  const tbody = document.getElementById('rom-partitions-tbody');
  if (!tbody) return;

  if (!partitions || partitions.length === 0) {
    tbody.innerHTML = `<tr><td colspan="4" class="empty-state">No partitions found in this ROM</td></tr>`;
    return;
  }

  tbody.innerHTML = partitions.map((item, idx) => {
    const isDangerous = item.is_dangerous;
    const riskBadge = isDangerous
      ? `<span class="badge badge-danger">Dangerous</span>`
      : `<span class="badge badge-safe">Safe</span>`;
    return `
    <tr>
      <td width="35">
        <input type="checkbox" class="rom-part-check" data-partition="${esc(item.partition)}" data-index="${idx}" ${isDangerous ? '' : 'checked'}>
      </td>
      <td class="font-bold ${isDangerous ? 'text-danger' : ''}">${esc(item.partition)}</td>
      <td class="font-mono text-secondary">${esc(item.image_file)}</td>
      <td width="90">${riskBadge}</td>
    </tr>`;
  }).join('');
}

function setupGlobalDragDrop() {
  const dropzone = document.getElementById('apk-dropzone');
  if (dropzone) {
    dropzone.addEventListener('click', selectAndInstallApk);

    // HTML5 Drag & Drop on APK dropzone
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
      // Paths arrive through the native `tauri://drag-drop` event (handled below);
      // the HTML5 File object has no real path inside the webview.
    });
  }

  const btnInstall = document.getElementById('btn-install-apk');
  if (btnInstall) {
    btnInstall.addEventListener('click', selectAndInstallApk);
  }

  // Tauri v2 Native Window Drag & Drop (Context-aware)
  {
    listen('tauri://drag-enter', () => {
      const activeView = document.querySelector('.view.active')?.id;
      if (activeView === 'view-adb' && dropzone) {
        dropzone.classList.add('drag-active');
      }
    });

    listen('tauri://drag-leave', () => {
      if (dropzone) dropzone.classList.remove('drag-active');
    });

    listen('tauri://drag-drop', async (event) => {
      if (dropzone) dropzone.classList.remove('drag-active');
      const paths = event.payload && event.payload.paths ? event.payload.paths : [];
      if (paths.length === 0) return;

      const activeView = document.querySelector('.view.active')?.id || 'view-adb';
      const firstPath = paths[0];
      const lower = firstPath.toLowerCase();

      // 1. FASTBOOT FLASHER TAB
      if (activeView === 'view-fastboot') {
        if (lower.endsWith('.img') || lower.endsWith('.bin')) {
          // Switch to single partition tab and load image
          document.getElementById('tab-flash-single')?.click();
          const imgInput = document.getElementById('single-image-path');
          if (imgInput) imgInput.value = firstPath;
          const fname = basename(firstPath).toLowerCase();
          const cleanName = fname.replace('.img', '').replace('.bin', '').replace('_ab', '').replace('_a', '').replace('_b', '');
          autoSelectPartitionChip(cleanName);
          appendFastbootLog(`[INFO] Loaded partition image: ${firstPath}`);
        } else {
          // It's a directory or ROM package
          document.getElementById('tab-flash-rom')?.click();
          await handleRomDirectorySelected(firstPath);
        }
        return;
      }

      // 2. ADB & DEBLOAT TAB
      if (activeView === 'view-adb') {
        const apkFile = paths.find(p => p.toLowerCase().endsWith('.apk'));
        if (apkFile) {
          installApkFile(apkFile);
        } else if (lower.endsWith('.img') || lower.endsWith('.bin')) {
          // User dropped an image, switch to Fastboot Flasher!
          document.querySelector('.dock-tab[data-target="view-fastboot"]')?.click();
          document.getElementById('tab-flash-single')?.click();
          const imgInput = document.getElementById('single-image-path');
          if (imgInput) imgInput.value = firstPath;
          appendFastbootLog(`[INFO] Switched to Fastboot Flasher and loaded image: ${firstPath}`);
        } else {
          // Could be a ROM directory dropped here
          document.querySelector('.dock-tab[data-target="view-fastboot"]')?.click();
          document.getElementById('tab-flash-rom')?.click();
          await handleRomDirectorySelected(firstPath);
        }
        return;
      }

      // 3. OTHER VIEWS FALLBACK
      if (lower.endsWith('.apk')) {
        installApkFile(firstPath);
      }
    });
  }
}

const LogcatState = {
  isStreaming: false,
  session: null,
  sessionCounter: 0,
  lines: [],        // history (bounded, trimmed in chunks)
  pending: [],      // lines waiting for the next animation frame
  flushScheduled: false,
  maxLines: 2500,
  filterText: '',
  autoScroll: true
};

const LOG_LEVEL_RE = /^\S+\s+\S+\s+([VDIWEF])\//;

function logSeverityClass(line) {
  const m = LOG_LEVEL_RE.exec(line);
  switch (m && m[1]) {
    case 'E': case 'F': return 'log-e';
    case 'W': return 'log-w';
    case 'I': return 'log-i';
    case 'D': return 'log-d';
    default: return 'log-v';
  }
}

function logcatMatches(line) {
  const kw = LogcatState.filterText;
  return !kw || line.toLowerCase().includes(kw);
}

function updateLogcatCounter() {
  const viewer = document.getElementById('logcat-viewer');
  const counter = document.getElementById('logcat-counter');
  if (viewer && counter) counter.textContent = `${viewer.childElementCount} lines`;
}

function buildLogcatFragment(lines) {
  const frag = document.createDocumentFragment();
  for (const line of lines) {
    const div = document.createElement('div');
    div.className = `logcat-line ${logSeverityClass(line)}`;
    div.textContent = line;
    frag.appendChild(div);
  }
  return frag;
}

function flushLogcat() {
  LogcatState.flushScheduled = false;
  const viewer = document.getElementById('logcat-viewer');
  const batch = LogcatState.pending;
  LogcatState.pending = [];
  if (!viewer || batch.length === 0) return;

  const visible = batch.filter(logcatMatches);
  if (visible.length) {
    // never render more than the cap in one go
    viewer.appendChild(buildLogcatFragment(visible.length > LogcatState.maxLines ? visible.slice(-LogcatState.maxLines) : visible));
    let extra = viewer.childElementCount - LogcatState.maxLines;
    while (extra-- > 0) viewer.removeChild(viewer.firstElementChild);
    if (LogcatState.autoScroll) viewer.scrollTop = viewer.scrollHeight;
    updateLogcatCounter();
  }
}

function queueLogcatLines(lines) {
  const st = LogcatState;
  for (const l of lines) st.lines.push(l);
  // trim in chunks so we don't shift() on every line
  if (st.lines.length > st.maxLines * 1.5) st.lines.splice(0, st.lines.length - st.maxLines);
  for (const l of lines) st.pending.push(l);
  if (st.pending.length > st.maxLines * 2) st.pending.splice(0, st.pending.length - st.maxLines);
  if (!st.flushScheduled) {
    st.flushScheduled = true;
    requestAnimationFrame(flushLogcat);
  }
}

function resetLogcatView() {
  const viewer = document.getElementById('logcat-viewer');
  if (viewer) viewer.innerHTML = '';
  LogcatState.lines = [];
  LogcatState.pending = [];
  updateLogcatCounter();
}

async function startLogcatStream() {
  if (!AppState.currentAdbDevice) {
    alert(I18N[currentLang].select_device || 'Please connect an ADB device first.');
    return;
  }

  try {
    const level = document.getElementById('logcat-level-select').value;
    const filter = document.getElementById('logcat-search-filter').value.trim();
    LogcatState.session = `lc-${++LogcatState.sessionCounter}`;
    resetLogcatView();

    await invoke('start_logcat_stream', {
      serial: AppState.currentAdbDevice,
      filter: filter || null,
      level: level || null,
      sessionId: LogcatState.session
    });

    LogcatState.isStreaming = true;
    updateLogcatControls();
  } catch (err) {
    LogcatState.isStreaming = false;
    updateLogcatControls();
    alert(`Failed to start logcat: ${err}`);
  }
}

async function stopLogcatStream() {
  LogcatState.session = null; // ignore anything still in flight
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
  const viewer = document.getElementById('logcat-viewer');

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
      if (LogcatState.isStreaming) stopLogcatStream();
      else startLogcatStream();
    });
  }

  if (clearBtn) {
    clearBtn.addEventListener('click', async () => {
      resetLogcatView();
      try {
        await invoke('clear_logcat', { serial: AppState.currentAdbDevice });
      } catch (e) {
        console.warn('Clear logcat buffer warning:', e);
      }
    });
  }

  if (exportBtn) {
    exportBtn.addEventListener('click', async () => {
      if (LogcatState.lines.length === 0) {
        alert('Log buffer is empty.');
        return;
      }
      try {
        const path = await invoke('pick_save_path', {
          title: 'Export logcat',
          defaultName: `logcat_${AppState.currentAdbDevice || 'device'}_${Date.now()}.txt`
        });
        if (path) {
          await invoke('save_text_file', { path, content: LogcatState.lines.join('\n') });
          alert(`Saved ${LogcatState.lines.length} lines to ${path}`);
        }
      } catch (e) {
        alert(`Export failed: ${e}`);
      }
    });
  }

  if (filterInput) {
    filterInput.addEventListener('input', debounce((e) => {
      LogcatState.filterText = e.target.value.trim().toLowerCase();
      invoke('set_logcat_filter', { filter: e.target.value }).catch(() => {});
      if (!viewer) return;
      viewer.innerHTML = '';
      const shown = LogcatState.lines.filter(logcatMatches);
      viewer.appendChild(buildLogcatFragment(shown.slice(-LogcatState.maxLines)));
      updateLogcatCounter();
      if (LogcatState.autoScroll) viewer.scrollTop = viewer.scrollHeight;
    }, 200));
  }

  if (levelSelect) {
    levelSelect.addEventListener('change', () => {
      if (LogcatState.isStreaming) startLogcatStream();
    });
  }

  if (autoscrollCheck) {
    autoscrollCheck.addEventListener('change', (e) => {
      LogcatState.autoScroll = e.target.checked;
      if (LogcatState.autoScroll && viewer) viewer.scrollTop = viewer.scrollHeight;
    });
  }

  // Scrolling up pauses auto-scroll, scrolling back to the bottom resumes it
  if (viewer) {
    viewer.addEventListener('scroll', () => {
      const atBottom = viewer.scrollHeight - viewer.scrollTop - viewer.clientHeight < 40;
      if (atBottom !== LogcatState.autoScroll) {
        LogcatState.autoScroll = atBottom;
        if (autoscrollCheck) autoscrollCheck.checked = atBottom;
      }
    }, { passive: true });
  }

  listen('logcat-batch', (event) => {
    const p = event.payload;
    if (p && p.session === LogcatState.session && Array.isArray(p.lines)) queueLogcatLines(p.lines);
  });

  listen('logcat-ended', (event) => {
    const p = event.payload;
    if (p && p.session === LogcatState.session && LogcatState.isStreaming) {
      LogcatState.isStreaming = false;
      updateLogcatControls();
      queueLogcatLines(['--- logcat stream ended (device disconnected or adb stopped) ---']);
    }
  });
}

// ==========================================================
// 9.8. NATIVE SCREEN MIRROR (adb screenrecord H.264 -> WebCodecs)
// ==========================================================
const MirrorState = {
  active: false,
  info: null,
  decoder: null,
  decoderCodec: '',
  gotKey: false,
  buf: new Uint8Array(0),
  pendingHeaders: [],
  ts: 0,
  fallbackActive: false,
  pointer: null
};

function concatBytes(a, b) {
  const out = new Uint8Array(a.length + b.length);
  out.set(a, 0);
  out.set(b, a.length);
  return out;
}

/** Index of the next 00 00 01 start code at or after `from`, or -1. */
function findStartCode(buf, from) {
  for (let i = from; i + 2 < buf.length; i++) {
    if (buf[i] === 0 && buf[i + 1] === 0 && buf[i + 2] === 1) return i;
  }
  return -1;
}

function mirrorStatus(text) {
  const el = document.getElementById('mirror-status');
  if (el) el.textContent = text;
}

function configureMirrorDecoder(sps) {
  // profile_idc, constraint flags, level_idc follow the NAL header byte
  const hex = (n) => n.toString(16).padStart(2, '0');
  const codec = `avc1.${hex(sps[1])}${hex(sps[2])}${hex(sps[3])}`;
  if (MirrorState.decoder && MirrorState.decoderCodec === codec) return;
  if (MirrorState.decoder) {
    try { MirrorState.decoder.close(); } catch (_) { /* already closed */ }
  }
  const canvas = document.getElementById('mirror-canvas');
  const ctx = canvas.getContext('2d');
  MirrorState.decoder = new VideoDecoder({
    output: (frame) => {
      if (canvas.width !== frame.displayWidth || canvas.height !== frame.displayHeight) {
        canvas.width = frame.displayWidth;
        canvas.height = frame.displayHeight;
      }
      ctx.drawImage(frame, 0, 0);
      frame.close();
      if (MirrorState.status !== 'live') {
        MirrorState.status = 'live';
        mirrorStatus('● LIVE');
      }
    },
    error: (e) => mirrorStatus(`Decoder error: ${e.message}`)
  });
  MirrorState.decoder.configure({ codec, optimizeForLatency: true });
  MirrorState.decoderCodec = codec;
  MirrorState.gotKey = false;
}

function handleMirrorNal(nal) {
  // nal excludes the start code
  const type = nal[0] & 0x1f;
  const withStart = concatBytes(new Uint8Array([0, 0, 0, 1]), nal);
  if (type === 7) {
    configureMirrorDecoder(nal);
    MirrorState.pendingHeaders = [withStart];
    return;
  }
  if (type === 8 || type === 6 || type === 9) {
    MirrorState.pendingHeaders.push(withStart);
    return;
  }
  if (type !== 1 && type !== 5) return;
  const dec = MirrorState.decoder;
  if (!dec || dec.state !== 'configured') return;

  const isKey = type === 5;
  if (isKey) MirrorState.gotKey = true;
  if (!MirrorState.gotKey) return;
  // stay close to real time: drop delta frames while the decoder is backed up
  if (!isKey && dec.decodeQueueSize > 6) return;

  let data = withStart;
  if (MirrorState.pendingHeaders.length) {
    data = MirrorState.pendingHeaders.reduce(concatBytes, new Uint8Array(0));
    data = concatBytes(data, withStart);
    MirrorState.pendingHeaders = [];
  }
  MirrorState.ts += 33333;
  try {
    dec.decode(new EncodedVideoChunk({ type: isKey ? 'key' : 'delta', timestamp: MirrorState.ts, data }));
  } catch (e) {
    mirrorStatus(`Decode failed: ${e.message}`);
  }
}

function feedMirror(chunk) {
  let bytes;
  if (chunk instanceof ArrayBuffer) bytes = new Uint8Array(chunk);
  else if (ArrayBuffer.isView(chunk)) bytes = new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength);
  else if (Array.isArray(chunk)) bytes = Uint8Array.from(chunk);
  else return;

  let buf = MirrorState.buf.length ? concatBytes(MirrorState.buf, bytes) : bytes;
  let start = findStartCode(buf, 0);
  if (start < 0) { MirrorState.buf = buf.length > 4 ? buf.slice(-4) : buf; return; }

  for (;;) {
    const next = findStartCode(buf, start + 3);
    if (next < 0) break;
    // a 4-byte start code has an extra leading zero that belongs to the previous NAL
    let end = next;
    if (end > start + 3 && buf[end - 1] === 0) end -= 1;
    handleMirrorNal(buf.subarray(start + 3, end));
    start = next;
  }
  MirrorState.buf = buf.slice(start); // keep the unfinished NAL
}

function mirrorToDevice(e) {
  const canvas = document.getElementById('mirror-canvas');
  const r = canvas.getBoundingClientRect();
  const info = MirrorState.info;
  return {
    x: Math.round(Math.min(Math.max((e.clientX - r.left) / r.width, 0), 1) * info.device_width),
    y: Math.round(Math.min(Math.max((e.clientY - r.top) / r.height, 0), 1) * info.device_height)
  };
}

async function stopMirror() {
  MirrorState.active = false;
  MirrorState.fallbackActive = false;
  if (MirrorState.decoder) {
    try { MirrorState.decoder.close(); } catch (_) { /* already closed */ }
    MirrorState.decoder = null;
    MirrorState.decoderCodec = '';
  }
  MirrorState.buf = new Uint8Array(0);
  MirrorState.pendingHeaders = [];
  MirrorState.status = '';
  try { await invoke('stop_mirror'); } catch (_) { /* nothing running */ }
}

async function runMirrorFallback(serial) {
  // Webviews without WebCodecs: poll PNG screenshots
  MirrorState.fallbackActive = true;
  const canvas = document.getElementById('mirror-canvas');
  const ctx = canvas.getContext('2d');
  mirrorStatus('Compatibility mode (screenshots)');
  while (MirrorState.fallbackActive) {
    try {
      const buf = await invoke('capture_frame', { serial });
      const bmp = await createImageBitmap(new Blob([buf], { type: 'image/png' }));
      if (canvas.width !== bmp.width) { canvas.width = bmp.width; canvas.height = bmp.height; }
      ctx.drawImage(bmp, 0, 0);
      bmp.close();
    } catch (e) {
      mirrorStatus(`Capture failed: ${e}`);
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
}

async function openMirror() {
  const serial = AppState.currentAdbDevice;
  if (!serial) {
    alert(I18N[currentLang].select_device || 'Please connect and select an ADB device first.');
    return;
  }
  const modal = document.getElementById('mirror-modal');
  modal.classList.remove('hidden');
  document.getElementById('mirror-device-tag').textContent = serial;
  mirrorStatus('Starting...');
  await stopMirror();
  MirrorState.active = true;

  const hasCodecs = typeof VideoDecoder !== 'undefined' && typeof EncodedVideoChunk !== 'undefined'
    && window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.Channel;

  try {
    if (hasCodecs) {
      const channel = new window.__TAURI__.core.Channel();
      channel.onmessage = feedMirror;
      MirrorState.info = await invoke('start_mirror', {
        serial,
        maxSize: Number(document.getElementById('mirror-quality').value) || 1280,
        bitrateMbps: 8,
        onFrame: channel
      });
    } else {
      const probe = await invoke('capture_frame', { serial });
      const bmp = await createImageBitmap(new Blob([probe], { type: 'image/png' }));
      MirrorState.info = { device_width: bmp.width, device_height: bmp.height, width: bmp.width, height: bmp.height };
      bmp.close();
      runMirrorFallback(serial);
    }
    const canvas = document.getElementById('mirror-canvas');
    canvas.width = MirrorState.info.width;
    canvas.height = MirrorState.info.height;
  } catch (e) {
    mirrorStatus(`Failed to start: ${e}`);
    MirrorState.active = false;
  }
}

function setupMirrorModal() {
  const modal = document.getElementById('mirror-modal');
  const canvas = document.getElementById('mirror-canvas');
  if (!modal || !canvas) return;

  const close = async () => {
    modal.classList.add('hidden');
    await stopMirror();
  };
  document.getElementById('btn-close-mirror').addEventListener('click', close);
  document.getElementById('mirror-quality').addEventListener('change', () => {
    if (MirrorState.active) openMirror();
  });

  const key = (code) => () => invoke('mirror_key', { keycode: code }).catch(() => {});
  document.getElementById('mirror-key-back').addEventListener('click', key(4));
  document.getElementById('mirror-key-home').addEventListener('click', key(3));
  document.getElementById('mirror-key-recents').addEventListener('click', key(187));
  document.getElementById('mirror-key-power').addEventListener('click', key(26));
  document.getElementById('mirror-key-volup').addEventListener('click', key(24));
  document.getElementById('mirror-key-voldown').addEventListener('click', key(25));

  canvas.addEventListener('pointerdown', (e) => {
    if (!MirrorState.info) return;
    canvas.setPointerCapture(e.pointerId);
    MirrorState.pointer = { ...mirrorToDevice(e), t: performance.now() };
  });
  canvas.addEventListener('pointerup', (e) => {
    const start = MirrorState.pointer;
    MirrorState.pointer = null;
    if (!start || !MirrorState.info) return;
    const end = mirrorToDevice(e);
    const dist = Math.hypot(end.x - start.x, end.y - start.y);
    const dur = Math.max(50, Math.round(performance.now() - start.t));
    const cmd = dist < 24
      ? invoke('mirror_tap', { x: end.x, y: end.y })
      : invoke('mirror_swipe', { x1: start.x, y1: start.y, x2: end.x, y2: end.y, durationMs: dur });
    cmd.catch(() => {});
  });

  listen('mirror-ended', (event) => {
    if (!MirrorState.active) return;
    MirrorState.active = false;
    mirrorStatus(`Stopped: ${String(event.payload || '').slice(0, 200)}`);
  });
}

// ==========================================================
// 10. INITIALIZATION & EVENT LISTENERS
// ==========================================================
document.addEventListener('DOMContentLoaded', () => {
  // 1. Initialize Theme & Language
  applyTheme(currentTheme);
  applyLanguage(currentLang);
  clearDeviceSpecs();

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
  document.getElementById('btn-refresh-devices').addEventListener('click', () => refreshDevices(true));
  document.getElementById('btn-refresh-fastboot').addEventListener('click', () => refreshDevices(true));

  // 7. Device Selection Change
  document.getElementById('adb-device-select').addEventListener('change', (e) => {
    AppState.currentAdbDevice = e.target.value || null;
    const dev = AppState.adbDevices.find((d) => d.serial === AppState.currentAdbDevice);
    if (dev) onAdbDeviceSelected(dev);
    else clearDeviceSpecs();
  });

  document.getElementById('fastboot-device-select').addEventListener('change', (e) => {
    AppState.currentFastbootDevice = e.target.value || null;
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
        try {
          await invoke('reboot_device', { serial: AppState.currentAdbDevice, mode });
        } catch (e) {
          alert(`Reboot failed: ${e}`);
        }
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

  document.getElementById('debloat-search').addEventListener('input', debounce(renderPackagesTable, 150));

  document.getElementById('check-all-packages').addEventListener('change', (e) => {
    // only the packages currently listed, not the ones hidden by the search box
    const visible = getFilteredPackages();
    visible.forEach((p) => {
      if (e.target.checked) AppState.selectedPackages.add(p.package);
      else AppState.selectedPackages.delete(p.package);
    });
    renderPackagesTable();
  });

  // Batch actions
  document.getElementById('btn-uninstall-selected').addEventListener('click', async () => {
    if (confirm(`Uninstall ${AppState.selectedPackages.size} selected packages?`)) {
      await runPackageBatch('uninstall_package', 'Uninstalling');
    }
  });
  document.getElementById('btn-disable-selected').addEventListener('click', () => runPackageBatch('disable_package', 'Disabling'));
  document.getElementById('btn-enable-selected').addEventListener('click', () => runPackageBatch('enable_package', 'Enabling'));

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
      try {
        await invoke('restore_package', { serial: AppState.currentAdbDevice, package: pkg });
        restoreModal.classList.add('hidden');
        loadPackages(AppState.currentAdbDevice, AppState.currentFilter);
      } catch (e) {
        alert(`Restore failed: ${e}`);
      }
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

  document.getElementById('btn-screen-mirror').addEventListener('click', openMirror);

  // Setup Smart Drag & Drop and Live Logcat Modal
  setupGlobalDragDrop();
  setupLogcatModal();
  setupMirrorModal();
  setupPackageTableEvents();
  setupFileListEvents(document.getElementById('pc-file-list'), 'pc');
  setupFileListEvents(document.getElementById('android-file-list'), 'android');

  // 10. MT Manager Transfer & Navigation
  document.getElementById('btn-pc-go').addEventListener('click', () => {
    loadPcDirectory(document.getElementById('pc-current-path').value.trim());
  });
  document.getElementById('btn-pc-home').addEventListener('click', () => loadPcDirectory(''));
  document.getElementById('btn-pc-up').addEventListener('click', () => {
    loadPcDirectory(parentOf(AppState.pcCurrentPath));
  });
  document.getElementById('btn-pc-refresh').addEventListener('click', () => loadPcDirectory(AppState.pcCurrentPath));

  document.getElementById('btn-android-go').addEventListener('click', () => {
    loadAndroidDirectory(document.getElementById('android-current-path').value.trim());
  });
  document.getElementById('btn-android-sdcard').addEventListener('click', () => loadAndroidDirectory('/sdcard'));
  document.getElementById('btn-android-root').addEventListener('click', () => loadAndroidDirectory('/'));
  document.getElementById('btn-android-up').addEventListener('click', () => {
    loadAndroidDirectory(parentOf(AppState.androidCurrentPath) || '/');
  });
  document.getElementById('btn-android-refresh').addEventListener('click', () => loadAndroidDirectory(AppState.androidCurrentPath));

  // Bidirectional Transfer Buttons
  document.getElementById('btn-push-to-android').addEventListener('click', async () => {
    if (!AppState.selectedPcItem) {
      alert('Please select a file or folder from the PC panel to push.');
      return;
    }
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    const status = document.getElementById('android-status');
    try {
      await runTask('push', (p) => {
        if (p.percent != null) status.textContent = `Pushing... ${p.percent}%`;
      }, (taskId) => invoke('transfer_pc_to_android', {
        serial: AppState.currentAdbDevice,
        localPath: AppState.selectedPcItem.path,
        remoteDir: AppState.androidCurrentPath,
        rootMode,
        taskId
      }));
      loadAndroidDirectory(AppState.androidCurrentPath);
    } catch (e) {
      status.textContent = 'Transfer failed';
      alert(`Transfer failed: ${e}`);
    }
  });

  document.getElementById('btn-pull-to-pc').addEventListener('click', async () => {
    if (!AppState.selectedAndroidItem) {
      alert('Please select a file or folder from the Android panel to pull.');
      return;
    }
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    const status = document.getElementById('pc-status');
    try {
      await runTask('pull', (p) => {
        if (p.percent != null) status.textContent = `Pulling... ${p.percent}%`;
      }, (taskId) => invoke('transfer_android_to_pc', {
        serial: AppState.currentAdbDevice,
        remotePath: AppState.selectedAndroidItem.path,
        localDir: AppState.pcCurrentPath,
        rootMode,
        taskId
      }));
      loadPcDirectory(AppState.pcCurrentPath);
    } catch (e) {
      status.textContent = 'Transfer failed';
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
      try {
        await invoke('delete_item', {
          serial: AppState.currentAdbDevice,
          path,
          isAndroid: panel === 'android',
          rootMode
        });
      } catch (e) {
        alert(`Delete failed: ${e}`);
      }
      if (panel === 'pc') loadPcDirectory(AppState.pcCurrentPath);
      else loadAndroidDirectory(AppState.androidCurrentPath);
    }
  });

  document.getElementById('ctx-rename').addEventListener('click', async () => {
    const menu = document.getElementById('file-context-menu');
    const path = menu.getAttribute('data-path');
    const panel = menu.getAttribute('data-panel');
    const rootMode = document.getElementById('explorer-root-toggle').checked;
    const newName = prompt('Enter new name:', basename(path));
    if (newName) {
      const sep = panel === 'pc' && path.includes('\\') ? '\\' : '/';
      const newPath = `${parentOf(path).replace(/[\\/]+$/, '')}${sep}${newName}`;
      try {
        await invoke('rename_item', {
          serial: AppState.currentAdbDevice,
          oldPath: path,
          newPath,
          isAndroid: panel === 'android',
          rootMode
        });
      } catch (e) {
        alert(`Rename failed: ${e}`);
      }
      if (panel === 'pc') loadPcDirectory(AppState.pcCurrentPath);
      else loadAndroidDirectory(AppState.androidCurrentPath);
    }
  });

  const ctxOpen = document.getElementById('ctx-open');
  if (ctxOpen) {
    ctxOpen.addEventListener('click', () => {
      const menu = document.getElementById('file-context-menu');
      if (menu.getAttribute('data-isdir') !== 'true') return;
      const path = menu.getAttribute('data-path');
      if (menu.getAttribute('data-panel') === 'pc') loadPcDirectory(path);
      else loadAndroidDirectory(path);
    });
  }

  const ctxExtract = document.getElementById('ctx-extract');
  if (ctxExtract) {
    ctxExtract.addEventListener('click', async () => {
      const menu = document.getElementById('file-context-menu');
      const path = menu.getAttribute('data-path');
      const panel = menu.getAttribute('data-panel');
      if (!path.toLowerCase().endsWith('.zip')) {
        alert('Only .zip archives can be extracted.');
        return;
      }
      const dst = parentOf(path);
      try {
        await invoke('extract_zip_archive', {
          serial: AppState.currentAdbDevice,
          zipPath: path,
          dstFolder: dst,
          isAndroid: panel === 'android',
          rootMode: document.getElementById('explorer-root-toggle').checked
        });
        if (panel === 'pc') loadPcDirectory(AppState.pcCurrentPath);
        else loadAndroidDirectory(AppState.androidCurrentPath);
      } catch (e) {
        alert(`Extract failed: ${e}`);
      }
    });
  }

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
    const logEl = document.getElementById('fastboot-terminal-log');
    logEl.textContent = '[IDLE] Waiting for fastboot commands...';
    logEl.dataset.idle = 'true';
    setFastbootProgress(0);
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
          const fname = basename(path).toLowerCase();
          const cleanName = fname.replace('.img', '').replace('.bin', '').replace('_ab', '').replace('_a', '').replace('_b', '');
          autoSelectPartitionChip(cleanName);
          appendFastbootLog(`[INFO] Loaded partition image: ${path}`);
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
          await handleRomDirectorySelected(path);
        }
      } catch (e) {
        console.error('Pick ROM folder error:', e);
      }
    });
  }

  // Single Partition Flashing
  const btnFlashSingle = document.getElementById('btn-flash-single');
  if (btnFlashSingle) {
    btnFlashSingle.addEventListener('click', async () => {
      const filePath = document.getElementById('single-image-path').value.trim();
      const partition = AppState.currentPartition || 'boot';
      const disableVerity = document.getElementById('check-vbmeta-flags').checked;

      if (!filePath) {
        alert('Please select or drag an image file (.img) first.');
        return;
      }

      if (!confirm(`Flash image to partition "${partition}"?`)) return;

      appendFastbootLog(`[FLASH] Flashing ${partition} with ${filePath}...`);
      setFastbootProgress(0);
      btnFlashSingle.disabled = true;

      try {
        // output + progress stream in through `fastboot-log` / `fastboot-progress`
        await runTask('flash', () => {}, (taskId) => invoke('flash_partition', {
          serial: AppState.currentFastbootDevice,
          partition,
          filePath,
          disableVerity,
          taskId
        }));
        setFastbootProgress(100);
        alert(`Successfully flashed ${partition}!`);
      } catch (e) {
        appendFastbootLog(`[ERROR] ${e}`);
        alert(`Flash failed: ${e}`);
      } finally {
        btnFlashSingle.disabled = false;
      }
    });
  }

  // Temporarily Boot Image
  const btnBootImg = document.getElementById('btn-boot-image');
  if (btnBootImg) {
    btnBootImg.addEventListener('click', async () => {
      const filePath = document.getElementById('single-image-path').value.trim();
      if (!filePath) {
        alert('Please select an image file (.img) first.');
        return;
      }

      appendFastbootLog(`[BOOT] Temporarily booting image: ${filePath}...`);
      try {
        await runTask('boot', () => {}, (taskId) => invoke('boot_image', {
          serial: AppState.currentFastbootDevice,
          filePath,
          taskId
        }));
        alert('Image booted successfully!');
      } catch (e) {
        appendFastbootLog(`[ERROR] ${e}`);
        alert(`Boot failed: ${e}`);
      }
    });
  }

  // Full Fastboot ROM Flashing
  const btnFlashRom = document.getElementById('btn-flash-rom');
  if (btnFlashRom) {
    btnFlashRom.addEventListener('click', async () => {
      const folderPath = document.getElementById('rom-folder-path').value.trim();
      if (!folderPath) {
        alert('Please select or drag a Fastboot ROM directory first.');
        return;
      }

      const scriptSelect = document.getElementById('rom-script-select');
      const scriptName = scriptSelect ? scriptSelect.value : '';
      if (!scriptName) {
        alert(currentLang === 'id' ? 'Silakan pilih skrip flash yang valid terlebih dahulu.' : 'Please select a valid flash script first.');
        return;
      }

      const excludedPartitions = [];
      document.querySelectorAll('.rom-part-check').forEach(cb => {
        if (!cb.checked) {
          excludedPartitions.push(cb.getAttribute('data-partition'));
        }
      });

      let confirmMsg = `Are you sure you want to flash Full Fastboot ROM with script "${scriptName}"?`;
      if (excludedPartitions.length > 0) {
        confirmMsg += `\n\nNote: ${excludedPartitions.length} partitions are excluded and will be SKIPPED:\n${excludedPartitions.slice(0, 4).join(', ')}${excludedPartitions.length > 4 ? '...' : ''}`;
      }
      if (scriptName.includes('lock')) {
        confirmMsg += `\n\n⚠️ CAUTION: THIS SCRIPT WILL LOCK YOUR BOOTLOADER!`;
      }

      if (!confirm(confirmMsg)) return;

      appendFastbootLog(`[START] Initiating Full ROM flash with script: ${scriptName}...`);
      setFastbootProgress(2);
      btnFlashRom.disabled = true;

      try {
        const res = await runTask('rom', () => {}, (taskId) => invoke('flash_rom', {
          serial: AppState.currentFastbootDevice,
          folderPath,
          scriptName,
          excludedPartitions,
          taskId
        }));
        appendFastbootLog(`[SUCCESS] ${res}`);
        setFastbootProgress(100);
        alert('Full ROM flashed successfully!');
      } catch (err) {
        appendFastbootLog(`[ERROR] ${err}`);
        alert(`Flashing error: ${err}`);
      } finally {
        btnFlashRom.disabled = false;
      }
    });
  }

  // Flash script change listener to dynamically refresh partitions
  const romScriptSelect = document.getElementById('rom-script-select');
  if (romScriptSelect) {
    romScriptSelect.addEventListener('change', async (e) => {
      const folderPath = document.getElementById('rom-folder-path').value.trim();
      const scriptName = e.target.value;
      if (!folderPath || !scriptName) return;

      try {
        appendFastbootLog(`[INFO] Switching script to: ${scriptName}...`);
        const romInfo = await invoke('parse_rom_directory', { folderPath, scriptName });
        if (romInfo && romInfo.partitions) {
          currentRomPartitions = romInfo.partitions;
          renderRomPartitions(currentRomPartitions);
          appendFastbootLog(`[INFO] Loaded ${currentRomPartitions.length} partitions for ${scriptName}`);
        }
      } catch (err) {
        appendFastbootLog(`[ERROR] Failed to update partitions for script: ${err}`);
      }
    });
  }

  // Partition exclusion select-all toggle
  const romCheckAll = document.getElementById('rom-check-all');
  if (romCheckAll) {
    romCheckAll.addEventListener('change', (e) => {
      const isChecked = e.target.checked;
      document.querySelectorAll('.rom-part-check').forEach(cb => {
        cb.checked = isChecked;
      });
    });
  }

  // Fastboot Reboot
  const btnFastbootReboot = document.getElementById('btn-fastboot-reboot');
  if (btnFastbootReboot) {
    btnFastbootReboot.addEventListener('click', async () => {
      if (confirm('Reboot Fastboot device to system?')) {
        try {
          await invoke('reboot_fastboot', {
            serial: AppState.currentFastbootDevice,
            mode: 'system'
          });
        } catch (e) {
          alert(`Reboot failed: ${e}`);
        }
      }
    });
  }

  // Fastboot progress & log events from Rust backend
  listen('fastboot-log', (event) => {
    if (event.payload) appendFastbootLog(event.payload);
  });
  listen('fastboot-progress', (event) => setFastbootProgress(Number(event.payload) || 0));

  // Initial Data Fetch
  loadPcDirectory('');
  refreshDevices(true);
  startDevicePolling();
});
