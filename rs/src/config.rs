pub struct Config {
    pub data_dir: String,
    pub image_table_path: String,
}

impl Config {
    pub fn new(data_dir: String) -> Config {
        let image_table_path = format!("{}/image_table.bincode", &data_dir);
        return Config {
            data_dir,
            image_table_path,
        };
    }
}
