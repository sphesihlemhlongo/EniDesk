use tauri::{AppHandle, Emitter, Manager};
use xcap::Monitor;
use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate};
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose};
use image::codecs::jpeg::JpegEncoder;
use std::io::Cursor;

struct State {
    enigo: Mutex<Enigo>,
    is_streaming: Mutex<bool>,
    password: Mutex<Option<String>>,
    is_authenticated: Mutex<bool>,
}

#[tauri::command]
fn set_password(state: tauri::State<'_, Arc<State>>, password: String) {
    let mut p = state.password.lock().unwrap();
    *p = Some(password);
}

#[tauri::command]
fn verify_password(state: tauri::State<'_, Arc<State>>, password: String) -> bool {
    let p = state.password.lock().unwrap();
    if let Some(ref saved) = *p {
        let auth = saved == &password;
        let mut authed = state.is_authenticated.lock().unwrap();
        *authed = auth;
        auth
    } else {
        // If no password set, allow connection (standard AnyDesk prompt style would be next)
        true
    }
}

#[tauri::command]
fn get_displays() -> Vec<String> {
    let monitors = Monitor::all().unwrap_or_default();
    monitors
        .iter()
        .map(|m| format!("{} ({}x{})", m.name(), m.width(), m.height()))
        .collect()
}

#[tauri::command]
async fn start_stream(app: AppHandle, display_index: usize) -> Result<(), String> {
    let state = app.state::<Arc<State>>();
    {
        let mut streaming = state.is_streaming.lock().unwrap();
        if *streaming {
            return Ok(());
        }
        *streaming = true;
    }

    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let monitor = monitors.get(display_index).cloned().ok_or("Display not found")?;

    tokio::spawn(async move {
        loop {
            {
                let streaming = state.is_streaming.lock().unwrap();
                if !*streaming {
                    break;
                }
            }

            if let Ok(image) = monitor.capture_image() {
                let mut buffer = Cursor::new(Vec::new());
                let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 60);
                if encoder.encode_image(&image).is_ok() {
                    let base64_image = general_purpose::STANDARD.encode(buffer.get_ref());
                    let _ = app.emit("screen-frame", base64_image);
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // ~10 FPS for now
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_stream(app: AppHandle) {
    let state = app.state::<Arc<State>>();
    let mut streaming = state.is_streaming.lock().unwrap();
    *streaming = false;
}

#[tauri::command]
fn inject_input(state: tauri::State<'_, Arc<State>>, event_type: String, key_or_button: String, x: f64, y: f64) -> Result<(), String> {
    let mut enigo = state.enigo.lock().unwrap();
    
    match event_type.as_str() {
        "mousemove" => {
            let _ = enigo.move_mouse(x as i32, y as i32, Coordinate::Abs);
        }
        "mousedown" => {
            let button = match key_or_button.as_str() {
                "left" => Mouse::Left,
                "right" => Mouse::Right,
                "middle" => Mouse::Middle,
                _ => return Err("Invalid button".into()),
            };
            let _ = enigo.button(button, enigo::Direction::Press);
        }
        "mouseup" => {
            let button = match key_or_button.as_str() {
                "left" => Mouse::Left,
                "right" => Mouse::Right,
                "middle" => Mouse::Middle,
                _ => return Err("Invalid button".into()),
            };
            let _ = enigo.button(button, enigo::Direction::Release);
        }
        "keydown" => {
            // Very basic mapping for now
            if key_or_button.len() == 1 {
                let _ = enigo.key(Keyboard::Unicode(key_or_button.chars().next().unwrap()), enigo::Direction::Press);
            }
        }
        "keyup" => {
            if key_or_button.len() == 1 {
                let _ = enigo.key(Keyboard::Unicode(key_or_button.chars().next().unwrap()), enigo::Direction::Release);
            }
        }
        _ => {}
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(State {
        enigo: Mutex::new(Enigo::new(&Settings::default()).unwrap()),
        is_streaming: Mutex::new(false),
        password: Mutex::new(None),
        is_authenticated: Mutex::new(false),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_displays,
            start_stream,
            stop_stream,
            inject_input,
            set_password,
            verify_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
