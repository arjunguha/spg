use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct SerializedConfig {
    cache_path: String,
}

pub struct Config {
    pub cache_path: String,
    pub image_table_path: String,
}

fn path_if_exists(s: &str) -> Option<PathBuf> {
    let p = Path::new(s);
    if !p.exists() {
        return None;
    }
    return Some(p.to_owned());
}

fn get_config_path(opt_path: Option<&str>) -> Option<PathBuf> {
    let home = std::env::var("HOME").expect("reading HOME variable");
    let home_path = format!("{}/.spg_config.json", &home);

    return opt_path
        .and_then(path_if_exists)
        .or_else(|| path_if_exists(&home_path));
}

impl Config {
    pub fn from_default_paths(opt_path: Option<&str>) -> Option<Config> {
        let config_str = std::fs::read_to_string(get_config_path(opt_path)?).unwrap();
        let c: SerializedConfig = serde_json::from_str(&config_str).unwrap();
        let cache_path = c.cache_path;
        let image_table_path = format!("{}/image_table.bincode", &cache_path);
        return Some(Config {
            cache_path,
            image_table_path,
        });
    }
}
