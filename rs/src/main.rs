mod config;
mod error;
mod image_table;
mod resources;
mod server;

use clap::Clap;

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
    Init,
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

    let data_dir = resources::get_data_dir_or_exit(opts.config_path);

    match opts.subcmd {
        SubCommand::Init => {
            resources::init(data_dir);
        }
        SubCommand::Add(add) => {
            let mut spg = image_table::SimplePhotoGallery::new(data_dir);
            spg.add(add.filename);
        }
        SubCommand::Sync(sync) => {
            let mut spg = image_table::SimplePhotoGallery::new(data_dir);
            spg.sync(sync.directory);
        }
        SubCommand::Serve => {
            let spg = image_table::SimplePhotoGallery::new(data_dir);
            spg.serve().await;
        }
    };
}
