use std::default::Default;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use tauri::{LogicalPosition, LogicalSize, Manager, State, Window};
use tauri::utils::config::{WindowConfig};
use tokio::sync::mpsc::{UnboundedSender};
use tween::{Tweener};
use crate::window_manager::WindowManagerMessage;
use std::ops::Deref;
use std::process::Command;
use windows_icons::get_icon_base64_by_path;

#[tauri::command]
pub fn fix_window(window: tauri::Window, sender: State<'_, UnboundedSender<WindowManagerMessage>>) {
    if window.is_decorated().unwrap() {
        window.set_decorations(false).unwrap();
        window.set_resizable(false).unwrap();
        save_changes(window, sender);
    }
    else {
        window.set_decorations(true).unwrap();
        window.set_resizable(true).unwrap();
    }
}

#[tauri::command]
pub fn save_changes(window: Window,  sender: State<'_, UnboundedSender<WindowManagerMessage>>) {
    println!("{}", window.title().unwrap());
    sender.send(WindowManagerMessage::SetConfig{
        label: window.label().to_string(),
        config: inner_config(&window)
    }).unwrap()
}

fn inner_config(window: &Window) -> WindowConfig {
    let pos = LogicalPosition::from_physical(window.inner_position().unwrap(), window.scale_factor().unwrap());
    let size = LogicalSize::from_physical(window.inner_size().unwrap(), window.scale_factor().unwrap());
    WindowConfig {
        label: window.label().to_string(),
        url: Default::default(),
        user_agent: None,
        file_drop_enabled: true,
        center: false,
        x: Some(pos.x),
        y: Some(pos.y),
        width: size.width,
        height: size.height,
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        resizable: window.is_resizable().unwrap(),
        maximizable: window.is_maximizable().unwrap(),
        minimizable: window.is_minimizable().unwrap(),
        closable: window.is_closable().unwrap(),
        title: window.title().unwrap(),
        fullscreen: window.is_fullscreen().unwrap(),
        focus: window.is_focused().unwrap(),
        transparent: false,
        maximized: window.is_maximized().unwrap(),
        visible: window.is_visible().unwrap(),
        decorations: window.is_decorated().unwrap(),
        always_on_top: false,
        content_protected: false,
        skip_taskbar: true,
        theme: None,
        title_bar_style: Default::default(),
        hidden_title: false,
        accept_first_mouse: false,
        tabbing_identifier: None,
        additional_browser_args: None,
    }
}

#[tauri::command]
pub fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn expand_window(window: Window, sender: State<'_, UnboundedSender<WindowManagerMessage>>, rx_size: State<'_, Mutex<Receiver<LogicalSize<f64>>>>) {
    sender.send(
        WindowManagerMessage::GetWindowCurrentSize {label: window.label().to_string()}
    ).unwrap();

    let mut size = None;

    while let Ok(size_) = rx_size.lock().unwrap().recv() {
        size = Some(size_);
        break
    }

    if let Some(size) = size {
        let (start, end) = (35, size.height);
        let duration = 2.5;

        let mut tweener = Tweener::sine_in_out(start, end as i32, duration);
        let mut position = 0;

        const DT: f32 = 1.0 / 15.0;

        // and then in your main loop...
        loop {
            position = tweener.move_by(DT);
            if tweener.is_finished() {
                break;
            }
            window.set_size(LogicalSize::new(size.width, position as f64)).unwrap()
        }
    }
}

#[tauri::command]
pub fn roll_up_window(window: Window) {
    let size = LogicalSize::from_physical(window.inner_size().unwrap(), window.scale_factor().unwrap());

    let (start, end) = (size.height, 35);
    let duration = 2.5;

    let mut tweener = Tweener::sine_in_out(start as i32, end , duration);
    let mut position = 0;

    const DT: f32 = 1.0 / 15.0;

    // and then in your main loop...
    loop {
        position = tweener.move_by(DT);
        if tweener.is_finished() {
            break;
        }
        window.set_size(LogicalSize::new(size.width, position as f64)).unwrap()
    }
}

#[tauri::command]
pub fn get_files(window: Window, config_folder: State<'_, PathBuf>) -> Vec<Vec<String>> {
    let wd = walkdir::WalkDir::new(config_folder.deref().join(window.label())).max_depth(1);
    let mut icons = vec![];
    if window.label() != "main" {
        for path in wd {
            if let Ok(path) = path {
                if path.path().is_file() {
                    icons.push(vec![
                        get_icon_base64_by_path(path.path().to_str().unwrap()),
                        path.file_name().to_str().unwrap().split(".").nth(0).unwrap().to_string(),
                        path.path().to_str().unwrap().to_string()
                    ]);
                }
            }
        }
    }
    icons
}

#[tauri::command]
pub fn run_app(path_buf: PathBuf) {
    tokio::spawn( async move {
        Command::new("cmd")
            .args(&["/C", "start", "", path_buf.to_str().unwrap()])
            .spawn()
            .unwrap();
    });
}