use tauri::{AppHandle, Emitter, Manager};
use xcap::Monitor;
use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate, Button, Key, Direction};
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose};
use image::codecs::jpeg::JpegEncoder;
use std::io::Cursor;
use std::time::Duration;
use axum::{routing::post, Json, Router, extract::State as AxumState};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use local_ip_address::local_ip;

struct State {
    enigo: Mutex<Enigo>,
    is_streaming: Mutex<bool>,
    password: Mutex<Option<String>>,
    is_authenticated: Mutex<bool>,
    is_network_init: Mutex<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IncomingSignal {
    #[serde(rename = "type")]
    msg_type: String,
    from: String,
    from_ip: String,
    sdp: Option<String>,
    candidate: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DiscoveryBeacon {
    id: String,
    ip: String,
}

#[tauri::command]
async fn init_network(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<Arc<State>>();
    {
        let mut is_init = state.is_network_init.lock().unwrap();
        if *is_init { return Ok(()); }
        *is_init = true;
    }

    let app_clone = app.clone();
    tokio::spawn(async move {
        start_discovery(app_clone.clone(), id).await;
        
        let axum_app = Router::new()
            .route("/signaling", post(signaling_handler))
            .with_state(app_clone);
            
        let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
        if let Ok(listener) = tokio::net::TcpListener::bind(&addr).await {
            let _ = axum::serve(listener, axum_app).await;
        }
    });

    Ok(())
}

async fn signaling_handler(
    AxumState(app): AxumState<AppHandle>,
    Json(payload): Json<IncomingSignal>,
) -> &'static str {
    let _ = app.emit("incoming-signal", payload);
    "OK"
}

async fn start_discovery(app: AppHandle, id: String) {
    let local_ip_addr = local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let beacon = DiscoveryBeacon { id: id.clone(), ip: local_ip_addr.to_string() };
    
    // Broadcast task
    let beacon_clone = beacon.clone();
    tokio::spawn(async move {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
            let _ = socket.set_broadcast(true);
            let beacon_json = serde_json::to_string(&beacon_clone).unwrap();
            loop {
                let _ = socket.send_to(beacon_json.as_bytes(), "255.255.255.255:8888").await;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    });

    // Listen task
    tokio::spawn(async move {
        if let Ok(listen_socket) = UdpSocket::bind("0.0.0.0:8888").await {
            let mut buf = [0; 1024];
            loop {
                if let Ok((len, _addr)) = listen_socket.recv_from(&mut buf).await {
                    if let Ok(msg) = String::from_utf8(buf[..len].to_vec()) {
                        if let Ok(received_beacon) = serde_json::from_str::<DiscoveryBeacon>(&msg) {
                            if received_beacon.id != id {
                                let _ = app.emit("device-discovered", received_beacon);
                            }
                        }
                    }
                }
            }
        }
    });
}

#[tauri::command]
async fn send_signal(target_ip: String, mut payload: IncomingSignal) -> Result<(), String> {
    let local_ip_addr = local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    payload.from_ip = local_ip_addr.to_string();
    
    let client = reqwest::Client::new();
    let url = format!("http://{}:8081/signaling", target_ip);
    
    let _ = client.post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(())
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
        is_network_init: Mutex::new(false),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            init_network,
            send_signal,
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
