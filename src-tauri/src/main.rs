mod commands;

use commands::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use vantage::config::VantageConfig;
use vantage::local_api::LocalClient;

pub struct AppState {
    pub client: Arc<Mutex<Option<LocalClient>>>,
    pub config: Arc<Mutex<VantageConfig>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = VantageConfig::load().unwrap_or_default();

    let state = AppState {
        client: Arc::new(Mutex::new(None)),
        config: Arc::new(Mutex::new(config)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            init_client,
            get_current_phase,
            get_match_data,
            get_post_game,
            get_match_history,
            get_config,
            save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
