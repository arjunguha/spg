use super::image_table::ImageTable;
use std::process;
use std::fs;
use std::path::Path;

static INDEX_HTML: &'static str = include_str!("../../html/index.html");
static INDEX_CSS: &'static str = include_str!("../../html/index.css");
static INDEX_BUNDLE_JS: &'static str = include_str!("../../html/index.bundle.js");

fn mkdir_or_exit(p: impl AsRef<Path>) {
    let p = p.as_ref();
    if let Err(err) = fs::create_dir(p) {
        eprintln!("Could not create directory {}.\n{}", p.to_string_lossy(), err);
        process::exit(1);
    }
}

fn create_file_or_exit(p: impl AsRef<Path>, data: &str) {
    let p = p.as_ref();
    if let Err(err) = std::fs::write(p, data) {
        eprintln!("Could not create file {}.\n{}", p.to_string_lossy(), err);
        process::exit(1);
    }
}

pub fn get_data_dir_or_exit(path: Option<String>) -> String {
    path.unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME variable not set");
        format!("{}/.spg", &home)
    })
}

pub fn init(config_path: impl AsRef<Path>) {
    let config_path = config_path.as_ref();
    if config_path.exists() {
        eprintln!("{} already exists. Delete it manually if you really want to reinitialize.",
            config_path.to_string_lossy());
        process::exit(1);
    }

    mkdir_or_exit(config_path);
    let www_path = config_path.join("www");
    mkdir_or_exit(&www_path);
    mkdir_or_exit(config_path.join("converted"));
    create_file_or_exit(www_path.join("index.html"), INDEX_HTML);
    create_file_or_exit(www_path.join("index.css"), INDEX_CSS);
    create_file_or_exit(www_path.join("index.bundle.js"), INDEX_BUNDLE_JS);

    let image_table = ImageTable::new();
    image_table.save(config_path.join("image_table.bincode"));
}