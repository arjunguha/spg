use super::config::Config;
use std::path::Path;

static INDEX_HTML: &'static str = include_str!("../../html/index.html");
static INDEX_CSS: &'static str = include_str!("../../html/index.css");
static INDEX_BUNDLE_JS: &'static str = include_str!("../../html/index.bundle.js");

fn create_file_unless_exists(base_path: impl AsRef<Path>, name: &str, data: &str) {
    let base_path: &Path = base_path.as_ref();
    let name = base_path.join(name);
    let name = name.as_path();
    if name.exists() {
        return;
    }
    std::fs::write(name, data).unwrap();
}

pub fn create_static_resources(config: &Config) {
    let base = &config.cache_path;
    create_file_unless_exists(base, "www/index.html", INDEX_HTML);
    create_file_unless_exists(base, "www/index.css", INDEX_CSS);
    create_file_unless_exists(base, "www/index.bundle.js", INDEX_BUNDLE_JS);
}
