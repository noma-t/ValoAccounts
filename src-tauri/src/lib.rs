mod commands;
mod crypto;
mod db;
mod fs;
mod process;
mod shop;
mod skins;

use std::sync::atomic::Ordering;

use db::initialize_database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting valo-accounts application");

    #[cfg(debug_assertions)]
    if std::env::args().any(|a| a == "--demo") {
        commands::util::DEMO_MODE.store(true, Ordering::Relaxed);
        log::info!("Demo mode enabled");
    }

    if let Err(e) = initialize_database(None) {
        log::error!("Failed to initialize database: {}", e);
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = skins::initialize_skins_db(None) {
        log::error!("Failed to initialize skins database: {}", e);
    }

    tauri::Builder::default()
        .setup(|app| {
            process::start_process_monitor(app.handle().clone());

            tauri::async_runtime::spawn(async {
                match skins::sync_skins_database().await {
                    Ok(true) => log::info!("Skins database synced successfully"),
                    Ok(false) => log::info!("Skins database already up to date"),
                    Err(e) => log::warn!("Failed to sync skins database: {}", e),
                }
            });

            let window = app
                .get_webview_window("main")
                .ok_or("main window not found")?;
            window.show().map_err(|e| e.to_string())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::util::greet,
            commands::util::get_app_dir,
            commands::util::get_default_riot_client_service_path,
            commands::util::get_default_riot_client_data_path,
            commands::settings::get_app_settings,
            commands::settings::update_app_settings,
            commands::account::add_account,
            commands::account::list_accounts,
            commands::account::edit_account,
            commands::account::check_current_data_available,
            commands::util::mark_launched,
            commands::account::switch_account,
            commands::process::get_riot_client_status,
            commands::process::kill_riot_client,
            commands::process::launch_riot_client,
            commands::process::get_valorant_status,
            commands::util::copy_account_password,
            commands::cookies::get_account_cookies,
            commands::shop::get_shop,
            commands::skins::get_skin_info,
            commands::skins::get_skin_info_batch,
            commands::skins::get_buddy_info,
            commands::skins::get_buddy_info_batch,
            commands::skins::get_flex_info,
            commands::skins::get_flex_info_batch,
            commands::skins::get_title_info_batch,
            commands::skins::get_playercard_info,
            commands::skins::get_playercard_info_batch,
            commands::skins::get_spray_info,
            commands::skins::get_spray_info_batch,
            commands::skins::sync_skins,
            commands::util::open_shop_window,
            commands::util::is_demo_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
