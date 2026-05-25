use tauri::{AppHandle, Emitter, Manager};
use xcap::Monitor;
use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate, Button, Key, Direction};
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose};
use image::codecs::jpeg::JpegEncoder;
use std::io::Cursor;
use std::time::Duration;

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
        true
    }
}

#[tauri::command]
fn get_displays() -> Vec<String> {
    let monitors = Monitor::all().unwrap_or_default();
    monitors
        .iter()
        .map(|m| format!("{} ({}x{})", m.name().unwrap_or_default(), m.width().unwrap_or_default(), m.height().unwrap_or_default()))
        .collect()
}

#[tauri::command]
async fn start_stream(app: AppHandle, display_index: usize) -> Result<(), String> {
    let state = app.state::<Arc<State>>().inner().clone();
    {
        let mut streaming = state.is_streaming.lock().unwrap();
        if *streaming {
            return Ok(());
        }
        *streaming = true;
    }

    let app_clone = app.clone();

    std::thread::spawn(move || {
        let monitors = match Monitor::all() {
            Ok(m) => m,
            Err(_) => return,
        };
        let monitor = match monitors.get(display_index).cloned() {
            Some(m) => m,
            None => return,
        };

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
                    let _ = app_clone.emit("screen-frame", base64_image);
                }
            }
            
            std::thread::sleep(Duration::from_millis(100)); // ~10 FPS for now
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
            if let Ok(monitors) = Monitor::all() {
                if let Some(monitor) = monitors.first() {
                    let abs_x = (x * monitor.width().unwrap_or(1920) as f64) as i32;
                    let abs_y = (y * monitor.height().unwrap_or(1080) as f64) as i32;
                    let _ = enigo.move_mouse(abs_x, abs_y, Coordinate::Abs);
                }
            }
        }
        "mousedown" => {
            let button = match key_or_button.as_str() {
                "left" => Button::Left,
                "right" => Button::Right,
                "middle" => Button::Middle,
                _ => return Err("Invalid button".into()),
            };
            let _ = enigo.button(button, Direction::Press);
        }
        "mouseup" => {
            let button = match key_or_button.as_str() {
                "left" => Button::Left,
                "right" => Button::Right,
                "middle" => Button::Middle,
                _ => return Err("Invalid button".into()),
            };
            let _ = enigo.button(button, Direction::Release);
        }
        "keydown" => {
            // Very basic mapping for now
            if key_or_button.len() == 1 {
                let _ = enigo.key(Key::Unicode(key_or_button.chars().next().unwrap()), Direction::Press);
            }
        }
        "keyup" => {
            if key_or_button.len() == 1 {
                let _ = enigo.key(Key::Unicode(key_or_button.chars().next().unwrap()), Direction::Release);
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
