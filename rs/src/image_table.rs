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

#[derive(Serialize, Deserialize)]
pub struct ImageTable {
    rows: Vec<Row>,
}

pub struct SimplePhotoGallery {
    image_table: ImageTable,
    config: Config
}

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

fn generate_thumbnail(image: &DynamicImage) -> DynamicImage {
    use image::GenericImageView;
    let (w, h) = image.dimensions();
    // Ideally, we have w / h = 4 / 3 or 3 * w = 4 * h
    if 3 * w == 4 * h {
        return image.thumbnail(200, 150);
    }
    // We can crop either dimension by Δ:
    // - (w - Δ) / h = 4 / 3
    //   Δ = w - (4 * h / 3)
    // - w / (h - Δ) = 4 / 3
    //   Δ = h - (3 * w / 4)
    //
    // To minimize the amount of cropping needed, we calculate both candidate values  of Δ and crop
    // either the width or the height.
    let delta_w = w - (4 * h / 3);
    let delta_h = h - (3 * w / 4);
     // looked at souce code to figure out which number is which coordinate
    let (x1, y1, x2, y2) = image.bounds();
    if delta_w < delta_h {
        return image.crop_imm(x1, y1, x2 - delta_w, y2).thumbnail(200, 150);
    }
    else {
        return image.crop_imm(x1, y1, x2, y2 - delta_h).thumbnail(200, 150);
    }
}


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
        let original_image = self.open_original(&config.data_dir)?;
        let thumbnail = generate_thumbnail(&original_image);
        thumbnail.save_with_format(
            &format!("{}/www/photos/{}", config.data_dir, self.thumbnail_path),
            ImageFormat::Jpeg,
        )?;
        let webview = original_image.resize(1024, 1024, FilterType::Gaussian);
        webview.save_with_format(
            &format!("{}/www/photos/{}", config.data_dir, &self.webview_path),
            ImageFormat::Jpeg,
        )?;
        return Ok(());
    }
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

    fn add_(&mut self, original_path: impl AsRef<Path>) -> Result<(), CommandError> {
        let full_path = original_path.as_ref().canonicalize()?;
        let original_path = full_path.to_string_lossy().to_string();
        match self.image_table.get_by_original_path(&original_path) {
            None => {
                self.image_table.rows.push(Row::new(&self.config, &original_path)?);
                println!("{} added", &original_path);
                return Ok(());
            }
            Some(row) => row.update(&self.config),
        }
    }

    pub fn add(&mut self, filename: String) {
        if let Err(err) = self.add_(&filename) {
            eprintln!("{}\n\nError adding {}", err, &filename);
        }
        self.image_table.save(&self.config.image_table_path);
    }

    fn rm_(&mut self, path: impl AsRef<Path>) -> Result<(), CommandError> {
        let path = path.as_ref();
        let absolute_path = path.canonicalize()?;
        let absolute_path = absolute_path.to_string_lossy();
        let row_index = self.image_table.rows.iter()
            .position(|row| row.original_path == absolute_path)
            .ok_or_else(|| error("file is not in database"))?;
        let row = self.image_table.rows.remove(row_index);
        fs::remove_file(format!("{}/www/photos/{}", self.config.data_dir, row.thumbnail_path))?;
        fs::remove_file(format!("{}/www/photos/{}", self.config.data_dir, row.webview_path))?;
        return Ok(());
    }

    pub fn rm(&mut self, path: String) {
        if let Err(err) = self.rm_(&path) {
            eprintln!("{}\n\nError removing {}", err, &path);
        }
        self.image_table.save(&self.config.image_table_path);
    }

    pub fn add_remove_path(&mut self, root: &str) -> Result<(), CommandError> {
        let images: Vec<_> = WalkDir::new(root).into_iter()
            // Skips all read errors
            .filter_map(|entry| entry.ok())
            .filter(|entry| is_recognized_filename(entry.path()))
            .collect();
        let len = images.len();
        println!("Found {} images.\n", len);
        let mut images_on_disk: HashSet<_> = HashSet::new();
        for (n, image) in images.iter().enumerate() {
            if self.add_(image.path()).is_ok() {
                images_on_disk.insert(image.path().to_string_lossy().to_string());
                self.image_table.save(&self.config.image_table_path);
            }
            if n % 100 == 0 {
                println!("{} remaining.", len - n);
            }
        }
        let images_in_table: HashSet<_> = self.image_table.rows.iter()
            .filter(|row| row.original_path.starts_with(root))
            .map(|row| row.original_path.to_string())
            .collect();
        for original_path in images_in_table.into_iter() {
            if images_on_disk.contains(&original_path) == false {
                if self.rm_(original_path).is_ok() {
                    self.image_table.save(&self.config.image_table_path);
                }
            }
        }

        return Ok(());
    }

    pub fn sync(&mut self, directory: String) {
        if let Err(err) = self.add_remove_path(&directory) {
            eprintln!("{}\n\nError synchronizing directory.", err);
        }
    }

    pub async fn serve(self) {
        crate::server::serve(self.config, self.image_table).await
    }
}