pub struct Config {
    pub cache_path: String,
    pub image_table_path: String,
}

impl Config {
    pub fn new(cache_path: String) -> Config {
        let image_table_path = format!("{}/image_table.bincode", &cache_path);
        return Config {
            cache_path,
            image_table_path,
        };
    }
}
