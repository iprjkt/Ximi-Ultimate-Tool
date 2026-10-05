pub mod adb;
pub mod explorer;
pub mod fastboot;
pub mod fastfetch;
pub mod mirror;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(if cfg!(debug_assertions) {
                        log::LevelFilter::Info
                    } else {
                        log::LevelFilter::Warn
                    })
                    .build(),
            )?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            utils::open_url,
            utils::cancel_task,
            utils::pick_save_path,
            utils::save_text_file,
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
            adb::execute_shell,
            adb::start_logcat_stream,
            adb::stop_logcat_stream,
            adb::clear_logcat,
            adb::set_logcat_filter,
            // Native screen mirror
            mirror::start_mirror,
            mirror::stop_mirror,
            mirror::mirror_tap,
            mirror::mirror_swipe,
            mirror::mirror_key,
            mirror::capture_frame,
            // File & Folder Picker
            utils::pick_file,
            utils::pick_folder,
            // Fastboot Commands
            fastboot::get_fastboot_devices,
            fastboot::flash_partition,
            fastboot::boot_image,
            fastboot::parse_rom_directory,
            fastboot::flash_rom,
            fastboot::reboot_fastboot,
            // MT Explorer Commands
            explorer::get_home_dir,
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                // never leave adb children (logcat, screenrecord, running transfers) behind
                adb::stop_logcat_blocking();
                mirror::stop_mirror_blocking();
                utils::cancel_all_tasks();
            }
        });
}
