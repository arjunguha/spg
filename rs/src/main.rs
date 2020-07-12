mod config;
mod error;
mod image_table;
mod resources;
mod server;

use clap::Clap;
use config::Config;
use image_table::ImageTable;

#[derive(Clap)]
#[clap(version = "1.0", author = "Arjun Guha")]
struct Opts {
    #[clap(short, long, about = "Path to configuration file")]
    config_path: Option<String>,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Add(Add),
    Sync(Sync),
    Serve,
}

#[derive(Clap)]
struct Add {
    filename: String,
}

#[derive(Clap)]
struct Sync {
    directory: String,
}
// fn file_timestamp(p : impl AsRef<Path>) -> Result<u128, std::io::Error> {
//     let metadata = fs::metadata(p)?;
//     let modified_time = metadata.modified()?;
//     let duration_since_epoch = modified_time.duration_since(std::time::UNIX_EPOCH)
//         .expect("file modification time is bogus");
//     return Ok(duration_since_epoch.as_millis());
// }

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let config = Config::from_default_paths(opts.config_path.as_deref())
        .expect("could not open configuration");

    let mut image_table = ImageTable::open_or_new(&config.image_table_path);

    match opts.subcmd {
        SubCommand::Add(add) => {
            image_table.add(&config, add.filename).unwrap();
            image_table.save(config.image_table_path);
        }
        SubCommand::Sync(sync) => {
            image_table.add_remove_path(&config, &sync.directory).unwrap();
        }
        SubCommand::Serve => {
            resources::create_static_resources(&config);
            server::serve(config, image_table).await
        }
    };
}
