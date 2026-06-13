use std::sync::{Mutex, mpsc::Sender, mpsc::channel};
use tauri::{AppHandle, State};
mod chipi8;
use chipi8::Chipi8;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(EmulatorState {tx: Mutex::new(None)})
        .invoke_handler(tauri::generate_handler![run_emulator, send_key_event])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

enum EmulatorCommand {
    KeyPress(u16),
    KeyRelease(u16),
    SetPaused(bool),
    Terminate,
    SetSpeed(u16)
}

struct EmulatorState {
    tx: Mutex<Option<Sender<EmulatorCommand>>>
}

#[tauri::command]
fn run_emulator(rom_fpath: String, speed: u32, app: AppHandle, state: State<'_, EmulatorState>) {
    println!("Rom: {rom_fpath}");
    let (tx, rx) = channel::<EmulatorCommand>();
    if let Ok(mut guard) = state.tx.lock() {
        *guard = Some(tx);
    }

    std::thread::spawn(move || {
        let mut chipi8 = Chipi8::new();
        chipi8.set_speed(speed);
        match chipi8.read_rom(&rom_fpath) {
            Ok(res) => {
                if !res {
                    println!("Failed to read file: {rom_fpath}");
                    return;
                }
            },
            Err(s) => {
                println!("Failed to read file: {s}");
                return;
            }
        }
        chipi8.run(app, rx);
    });
}

#[tauri::command]
fn send_key_event(key: u16, is_pressed: bool, state: State<'_, EmulatorState>) {
    if let Ok(guard) = state.tx.lock() {
        if let Some(tx) = &*guard {
            let cmd = if key == 0x10 {
                EmulatorCommand::SetPaused(true)
            } else if key == 0x11 {
                EmulatorCommand::SetPaused(false)
            } else if key == 0x12 {
                EmulatorCommand::Terminate
            } else if key >= 0x20 {
                let speed = key - 0x20;
                EmulatorCommand::SetSpeed(speed)
            } else {
                if is_pressed {
                    EmulatorCommand::KeyPress(key)
                } else {
                    EmulatorCommand::KeyRelease(key)
                }
            };
            let _ = tx.send(cmd);
        }
    }
}