mod auth;
mod background;
mod error;
mod google;

use auth::AuthState;
use error::Result;
use tauri::{AppHandle, State};

struct Http(reqwest::Client);

#[tauri::command]
fn is_logged_in() -> bool {
    auth::is_logged_in()
}

#[tauri::command]
async fn login(app: AppHandle, auth: State<'_, AuthState>, http: State<'_, Http>) -> Result<()> {
    auth::login(&app, &auth, &http.0).await
}

#[tauri::command]
async fn logout(auth: State<'_, AuthState>, http: State<'_, Http>) -> Result<()> {
    auth::logout(&auth, &http.0).await
}

#[tauri::command]
async fn list_calendars(
    auth: State<'_, AuthState>,
    http: State<'_, Http>,
) -> Result<Vec<google::Calendar>> {
    let token = auth::access_token(&auth, &http.0).await?;
    google::list_calendars(&http.0, &token).await
}

#[tauri::command]
async fn list_events(
    calendar_ids: Vec<String>,
    time_min: String,
    time_max: String,
    auth: State<'_, AuthState>,
    http: State<'_, Http>,
) -> Result<Vec<google::Event>> {
    let token = auth::access_token(&auth, &http.0).await?;
    google::list_events(&http.0, &token, &calendar_ids, &time_min, &time_max).await
}

#[tauri::command]
async fn get_background(app: AppHandle) -> Result<Option<String>> {
    background::get(&app)
}

#[tauri::command]
async fn set_background(app: AppHandle, path: String) -> Result<String> {
    background::set(&app, &path)
}

#[tauri::command]
async fn clear_background(app: AppHandle) -> Result<()> {
    background::clear(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AuthState::default())
        .manage(Http(reqwest::Client::new()))
        .invoke_handler(tauri::generate_handler![
            is_logged_in,
            login,
            logout,
            list_calendars,
            list_events,
            get_background,
            set_background,
            clear_background
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
