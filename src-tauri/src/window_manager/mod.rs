pub mod window_commands;
mod config_manager;

use std::any::Any;
use std::collections::HashMap;
use std::fs::create_dir;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use rand::Rng;
use tauri::{AppHandle, LogicalSize, WindowBuilder};
use tauri::utils::config::WindowConfig;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use window_shadows::set_shadow;
use crate::window_manager::config_manager::ConfigManager;

pub struct WindowManager {
    config_path: PathBuf,
    unbounded_receiver: UnboundedReceiver<WindowManagerMessage>,
    config_manager: ConfigManager,
    size_sender: Sender<LogicalSize<f64>>
}

impl WindowManager {
    pub fn init() -> (Self, (UnboundedSender<WindowManagerMessage>, Receiver<LogicalSize<f64>>)) {
        let path = PathBuf::from(format!("C:\\users\\{}\\.mocha-tabs", whoami::username()));
        if !path.exists() {
            let _ = create_dir(&path);
        }

        let (tx, rx) = unbounded_channel();
        let (tx_size, rx_size) = channel();
        (
            Self {
                unbounded_receiver: rx,
                config_path: path,
                config_manager: ConfigManager::init(),
                size_sender: tx_size
            },
            (tx, rx_size)
        )
    }

    pub fn get_config_folder(&self) -> PathBuf {
        self.config_path.clone()
    }

    pub async fn run(mut self) {
        while let Some(message) = &self.unbounded_receiver.recv().await {
            match message {
                WindowManagerMessage::CreateWindow {app} => self.create_window(app).await,
                WindowManagerMessage::OpenConfigFolder => open::that(&self.config_path).unwrap(),
                WindowManagerMessage::ScanConfigFolder => self.config_manager.scan_config_folder(&self.config_path).await,
                WindowManagerMessage::OpenLoadWindows {app} => self.load_windows(app).await,
                WindowManagerMessage::SetConfig {label, config} => self.set_config(label, config).await,
                WindowManagerMessage::GetWindowCurrentSize {label} => if let Some(conf) = self.config_manager.get_configs().get(label.as_str()) {
                    self.size_sender.send(LogicalSize::new(conf.get_win_conf().width, conf.get_win_conf().height)).unwrap();
                }
                WindowManagerMessage::CopyReq {path, label} => { std::fs::copy(path.clone(), PathBuf::from(self.config_path.join(label)).join(path.iter().last().unwrap())).unwrap(); }
            }
        }
    }

    async fn create_window(&mut self, app: &AppHandle) {
        let mut config = self.config_manager.create_config(&self.config_path).await.get_win_conf();
        config.height = 35.0;
        let win = WindowBuilder::from_config(app, config)
            .build()
            .unwrap();

        #[cfg(any(windows, target_os = "macos"))]
        set_shadow(&win, true).unwrap();
    }

    async fn load_windows(&self, app: &AppHandle) {
        for (_, config) in self.config_manager.get_configs() {
            let mut conf = config.get_win_conf();
            conf.height = 35.0;
            let win = WindowBuilder::from_config(app, conf)
                .build().unwrap();

            #[cfg(any(windows, target_os = "macos"))]
            set_shadow(&win, true).unwrap();
        }
    }

    async fn set_config(&mut self, label: &String, new_config: &WindowConfig) {
        if let Some(conf) = self.config_manager.get_mut_configs().get_mut(label) {
            conf.set_window_config(new_config.clone());
            self.save_config(label).await;
        }
    }

    async fn save_config(&self, label: &String) {
        if let Some(conf) = self.config_manager.get_configs().get(label) {
            File::create(self.config_path.join(format!("{}.conf.json", label))).await.unwrap()
                .write_all(serde_json::to_string(conf).unwrap().as_bytes()).await.unwrap();
        }
    }
}

pub enum WindowManagerMessage {
    CreateWindow { app: AppHandle },
    OpenConfigFolder,
    ScanConfigFolder,
    OpenLoadWindows { app: AppHandle },
    SetConfig { label: String, config: WindowConfig },
    GetWindowCurrentSize {label: String},
    CopyReq {path: PathBuf, label: String}
}