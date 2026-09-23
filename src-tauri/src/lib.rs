pub mod adb;
pub mod explorer;
pub mod fastboot;
pub mod fastfetch;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            utils::open_url,
            // ADB Commands
            adb::get_adb_devices,
            adb::get_device_specs,
            adb::get_packages,
            adb::uninstall_package,
            adb::restore_package,
            adb::disable_package,
            adb::enable_package,
            adb::reboot_device,
            adb::install_apk,
            adb::take_screenshot,
            adb::screen_mirror,
            adb::execute_shell,
            // Fastboot Commands
            fastboot::get_fastboot_devices,
            fastboot::flash_partition,
            fastboot::boot_image,
            fastboot::parse_rom_directory,
            fastboot::reboot_fastboot,
            // MT Explorer Commands
            explorer::list_pc_directory,
            explorer::list_android_directory,
            explorer::transfer_pc_to_android,
            explorer::transfer_android_to_pc,
            explorer::create_item,
            explorer::delete_item,
            explorer::rename_item,
            explorer::read_file_text,
            explorer::write_file_text,
            explorer::extract_zip_archive,
            // Fastfetch
            fastfetch::run_fastfetch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
