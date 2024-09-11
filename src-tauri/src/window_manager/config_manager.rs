use std::collections::HashMap;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::api::dialog::MessageDialogBuilder;
use tauri::utils::config::WindowConfig;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;

pub struct ConfigManager {
    configs: HashMap<String, Config>
}

impl ConfigManager {
    pub fn init() -> Self {
        Self {
            configs: HashMap::new()
        }
    }

    fn add_config(&mut self, config: Config) {
        self.configs.insert(config.window_config.label.clone(), config);
    }

    pub async fn create_config<P: AsRef<Path>>(&mut self, path: P) -> Config {
        let conf = Config::new(&path);

        File::create(path.as_ref().join(format!("{}.conf.json", &conf.external.tab_path.display()))).await.unwrap()
            .write_all(serde_json::to_string(&conf).unwrap().as_bytes()).await.unwrap();

        if let Err(e) = create_dir(path.as_ref().join(conf.external.tab_path.display().to_string())) {
            MessageDialogBuilder::new("Error", e.to_string()).show( |_ev| {} )
        }

        self.add_config(conf.clone());

        conf
    }

    pub async fn scan_config_folder<P: AsRef<Path>>(&mut self, path: P) {
        let walkdir = WalkDir::new(path.as_ref()).max_depth(1);
        for dir in walkdir {
            if let Ok(dir) = dir {
                if dir.path().is_file() {
                    let mut buf = "".to_string();
                    let mut file = File::open(dir.path()).await.unwrap();
                    file.read_to_string(&mut buf).await.unwrap();
                    let config = serde_json::from_str::<Config>(&buf).unwrap();
                    self.add_config(config)
                }
            }
        }
    }

    #[inline]
    pub fn get_configs(&self) -> &HashMap<String, Config> {
        &self.configs
    }
    #[inline]
    pub fn get_mut_configs(&mut self) -> &mut HashMap<String, Config> {
        &mut self.configs
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    window_config: WindowConfig,
    external: ExternalConfig
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut window_config = WindowConfig::default();

        let label: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        window_config.label = label.clone();
        window_config.title = "tab".to_string();
        window_config.decorations = false;
        window_config.skip_taskbar = true;
        window_config.width = 400.0;
        window_config.height = 300.0;
        window_config.x = Some(150.0);
        window_config.y = Some(150.0);
        window_config.resizable = false;
        window_config.closable = false;
        window_config.hidden_title = true;

        let path = path.as_ref().join(label);

        Self {
            window_config,
            external: ExternalConfig::new(path)
        }
    }

    pub fn get_win_conf(&self) -> WindowConfig {
        self.window_config.clone()
    }

    pub fn set_window_config(&mut self, window_config: WindowConfig) {
        self.window_config = window_config
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExternalConfig {
    tab_path: PathBuf
}

impl ExternalConfig {
    fn new<P: AsRef<Path>>(path: P) -> ExternalConfig {
        Self {
            tab_path: path.as_ref().to_path_buf()
        }
    }
}