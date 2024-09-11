// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_manager;

use std::sync::Mutex;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu, WindowBuilder, Manager, AppHandle, WindowEvent, FileDropEvent, State};
use tauri_plugin_autostart::MacosLauncher;
use crate::window_manager::{WindowManager, WindowManagerMessage};
use crate::window_manager::window_commands::{fix_window, close_window, expand_window, roll_up_window, get_files, run_app, save_changes};

#[tokio::main]
async fn main() {
    let (window_manager, (sender, rx_size)) = WindowManager::init();
    let config_folder = window_manager.get_config_folder();
    let window_manager_task = tokio::task::spawn(async move {
        window_manager.run().await
    });

    sender.send(WindowManagerMessage::ScanConfigFolder).unwrap();

    let setup_sender = sender.clone();
    let tray_event_sender = sender.clone();
    let window_event_sender = sender.clone();

    tauri::Builder::default()
        .manage(sender.clone())
        .manage(Mutex::new(rx_size))
        .manage(config_folder)
        .setup(move |app| {
            let tray = SystemTray::new()
                .with_menu(
                    SystemTrayMenu::new().add_submenu(SystemTraySubmenu::new(
                        "Tab",
                        SystemTrayMenu::new()
                            .add_item(CustomMenuItem::new("create_tab", "Create tab")),
                    ))
                        .add_submenu(SystemTraySubmenu::new(
                            "Config",
                            SystemTrayMenu::new()
                                .add_item(CustomMenuItem::new("open_config_folder", "Open config folder"))
                        ))
                        .add_native_item(SystemTrayMenuItem::Separator)
                        .add_item(CustomMenuItem::new("exit", "Exit"))
                )
                .build(app)?;


            setup_sender.send(WindowManagerMessage::OpenLoadWindows {app: app.handle()}).unwrap();
            Ok(())
        })
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "create_tab" => {
                    tray_event_sender.send(WindowManagerMessage::CreateWindow {app: app.clone()}).unwrap()
                }
                "open_config_folder" => {
                    tray_event_sender.send(WindowManagerMessage::OpenConfigFolder).unwrap()
                }
                "exit" => {
                    let wins = app.windows();
                    for (label, win) in wins {
                        win.close().unwrap()
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(move |event| {
            let window = event.window();
            match event.event() {
                WindowEvent::FileDrop(event) => {
                    match event {
                        FileDropEvent::Dropped(paths) => {
                            for path in paths {
                                window_event_sender.send(WindowManagerMessage::CopyReq {
                                    path: path.clone(),
                                    label: window.label().to_string()
                                }).unwrap();
                                window.eval("window.getIcons()").unwrap()
                            }
                        }
                        _ => {}
                    }
                }
                WindowEvent::Resized(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                _ => {}
            }
        })
        .plugin(tauri_plugin_context_menu::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![fix_window, close_window, expand_window, roll_up_window, get_files, run_app, save_changes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


