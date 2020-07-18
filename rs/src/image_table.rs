use super::config::Config;
use super::error::*;
use image::imageops::FilterType;
use image::DynamicImage;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process;
use walkdir::WalkDir;

static KNOWN_EXTENSIONS: [&'static str; 6] = [ 
    "heic",
    "HEIC",
    "jpg",
    "JPG",
    "jpeg",
    "JPEG"
];

fn is_recognized_filename(path: &Path) -> bool {
    let filename = path.file_name().and_then(|os_str| os_str.to_str());
    let ext = path.extension().and_then(|os_str| os_str.to_str());
    match (filename, ext) {
        (Some(filename), Some(extension)) => {
            // skip hidden files, including AppleDouble files.
            if filename.starts_with(".") {
                return false;
            }
            return KNOWN_EXTENSIONS.iter().find(|ext| **ext == extension).is_some();
        }
        _ => {
            println!("skipping {}", path.to_string_lossy());
            return false;
        }
    }
}

#[derive(Serialize)]
pub struct RowView<'a> {
    pub thumbnail_path: &'a str,
    pub webview_path: &'a str,
}

#[derive(Serialize, Deserialize)]
struct Row {
    // Path to the original image
    original_path: String,
    // MD5 of the original
    md5: u128,
    // modified time in milliseconds since Unix epoch
    modified: u128,
    // Name of the gallery (derived from original_path)
    gallery: String,
    // Title of the image (derived from original_path)
    title: String,
    // Path to the thumbnail (JPEG) that we build
    thumbnail_path: String,
    // Path to the web-sized (JPEG) that we build
    webview_path: String,
}

// let original_image = image::open(&original_path)?;

fn file_md5(p: impl AsRef<Path>) -> Result<u128, std::io::Error> {
    let buf = fs::read(p)?;
    // 16 bytes, and 16 * 8 = 128 bits
    let arr = md5::compute(&buf).0;
    let mut i: u128 = 0;
    for (index, byte) in arr.iter().enumerate() {
        i += (*byte as u128) << (index * 8);
    }
    return Ok(i);
}

impl Row {
    fn new(config: &Config, original_path: impl Into<String>) -> Result<Self, CommandError> {
        let original_path_str: String = original_path.into();

        let original_path: &Path = original_path_str.as_ref();

        let md5 = file_md5(&original_path)?;

        let title: &Path = original_path.file_name().unwrap().as_ref();
        let title = String::from(title.file_stem().unwrap().to_string_lossy());

        let gallery = String::from(
            original_path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy(),
        );

        let thumbnail_path = format!("{:x}-thumbnail.jpg", md5);
        let webview_path = format!("{:x}-webview.jpg", md5);

        let new_row = Row {
            original_path: original_path_str,
            md5,
            modified: 0,
            title,
            gallery,
            thumbnail_path,
            webview_path,
        };
        new_row.generate_jpegs(config)?;
        return Ok(new_row);
    }

    fn update(&mut self, config: &Config) -> Result<(), CommandError> {
        let current_md5 = file_md5(&self.original_path)?;
        if self.md5 == current_md5 {
            return Ok(());
        }
        self.md5 = current_md5;
        self.generate_jpegs(config)?;
        println!("{} updated", self.original_path);
        return Ok(());
    }

    fn open_original(&self, dir: &str) -> Result<DynamicImage, CommandError> {
        let path = Path::new(&self.original_path);
        let ext = path
            .extension()
            .ok_or(error("filename has no extension"))?
            .to_string_lossy();
        if ext.to_lowercase() == "heic" {
            let output_path_str = format!("{}/converted/{:x}-converted.jpg", dir, self.md5);
            let output_path = Path::new(&output_path_str);
            if output_path.exists() {
                fs::remove_file(output_path)?;
            }
            let mut child_process = Command::new("/usr/bin/heif-convert")
                .arg(path)
                .arg(&output_path_str)
                .spawn()?;
            let exit_code = child_process.wait()?;
            // I believe heif-convert returns exit code zero even if conversion
            // fails. That's why we verify that output_path gets created (below).
            // Note that we delete output_path (above) before calling
            // heif-convert.
            if exit_code.success() == false || output_path.exists() == false {
                println!("Error converting {} to a JPEG", &self.original_path);
                return Err(error("could not convert HEIC to JPEG."));
            }
            return Ok(image::open(output_path)?);
        }
        let original_image = image::open(path)?;
        return Ok(original_image);
    }
    
    fn generate_jpegs(&self, config: &Config) -> Result<(), CommandError> {
        let original_image = self.open_original(&config.cache_path)?;
        let thumbnail = original_image.thumbnail(200, 200);
        thumbnail.save_with_format(
            &format!("{}/www/photos/{}", config.cache_path, self.thumbnail_path),
            ImageFormat::Jpeg,
        )?;
        let webview = original_image.resize(1024, 1024, FilterType::Gaussian);
        webview.save_with_format(
            &format!("{}/www/photos/{}", config.cache_path, &self.webview_path),
            ImageFormat::Jpeg,
        )?;
        return Ok(());
    }
}

#[derive(Serialize, Deserialize)]
pub struct ImageTable {
    rows: Vec<Row>,
}

pub struct SimplePhotoGallery {
    image_table: ImageTable,
    config: Config
}

impl ImageTable {
    pub fn new() -> Self {
        return ImageTable { rows: vec![] };
    }

    pub fn open(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let bytes = std::fs::read(path).unwrap();
        return bincode::deserialize(&bytes).unwrap();
    }

    pub fn save(&self, path: impl AsRef<Path>) {
        let path: &Path = path.as_ref();
        let bytes = bincode::serialize(self).unwrap();
        std::fs::write(path, bytes).expect(&format!("Could not write to {:?}", path));
    }

    fn get_by_original_path(&mut self, p: &str) -> Option<&mut Row> {
        return self.rows.iter_mut().find(|row| row.original_path == p);
    }

    pub fn add(&mut self, config: &Config, original_path: impl AsRef<Path>) -> Result<(), CommandError> {
        let full_path = original_path.as_ref().canonicalize()?;
        let original_path = full_path.to_string_lossy().to_string();
        match self.get_by_original_path(&original_path) {
            None => {
                self.rows.push(Row::new(config, &original_path)?);
                println!("{} added", &original_path);
                return Ok(());
            }
            Some(row) => row.update(config),
        }
    }

    pub fn gallery_list(&self) -> HashSet<&str> {
        return self.rows.iter().map(|row| row.gallery.as_str()).collect();
    }

    pub fn gallery_contents(&self, gallery: &str) -> Vec<RowView> {
        return self
            .rows
            .iter()
            .filter(|row| row.gallery == gallery)
            .map(|row| RowView {
                thumbnail_path: row.thumbnail_path.as_str(),
                webview_path: row.webview_path.as_str(),
            })
            .collect();
    }


    pub fn add_remove_path(&mut self, config: &Config, root: &str) -> Result<(), CommandError> {
        let images: Vec<_> = WalkDir::new(root).into_iter()
            // Skips all read errors
            .filter_map(|entry| entry.ok())
            .filter(|entry| is_recognized_filename(entry.path()))
            .collect();
        for image in images.iter() {
            if self.add(config, image.path()).is_ok() {
                self.save(&config.image_table_path);
            }
        }
        return Ok(());
    }
}

impl SimplePhotoGallery {

    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref();
        if !data_dir.is_dir() {
            eprintln!("Data directory not found. Run \'spg init\'.");
            process::exit(1);
        }
        let config = Config::new(data_dir.to_string_lossy().to_string());
        let image_table = ImageTable::open(&config.image_table_path);
        return Self { config, image_table };
    }

    pub fn add(&mut self, filename: String) {
        unwrap_or_exit(self.image_table.add(&self.config, filename),
            "Error adding file.");
        self.image_table.save(&self.config.image_table_path);
    }

    pub fn sync(&mut self, directory: String) {
        unwrap_or_exit(self.image_table.add_remove_path(&self.config, &directory),
            "Error synchronizing directory.");
    }

    pub async fn serve(self) {
        crate::server::serve(self.config, self.image_table).await
    }
}