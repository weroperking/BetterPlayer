mod commands;
mod library;
mod mpv;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(commands::PlayerState::default())
        .invoke_handler(tauri::generate_handler![
            commands::open_path,
            commands::scan_folder,
            commands::play,
            commands::pause,
            commands::toggle_play,
            commands::seek,
            commands::set_volume,
            commands::toggle_mute,
            commands::queue_next,
            commands::queue_previous,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BetterPlayer");
}
