mod config;
mod error;
mod image_table;
mod monitor_fs;
mod resources;
mod server;
#[cfg(test)]
mod tests;

use clap::Clap;
use futures::prelude::*;
use std::net::SocketAddrV4;

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
    Rm(Rm),
    Sync(Sync),
    Stat(Stat),
    Serve(Serve),
    Init,
}

#[derive(Clap)]
struct Serve {
    /// Port to listen on
    #[clap(long, short)]
    port: u16,
    #[clap(long, short, default_value = "127.0.0.1")]
    bind_address: String,
}

#[derive(Clap)]
struct Add {
    filename: String,
}

#[derive(Clap)]
struct Rm {
    filename: String,
}

#[derive(Clap)]
struct Sync {
    directory: String,
}

#[derive(Clap)]
struct Stat {
    filename: String,
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
        SubCommand::Rm(add) => {
            let mut spg = image_table::SimplePhotoGallery::new(data_dir);
            spg.rm(add.filename);
        }
        SubCommand::Sync(sync) => {
            let mut spg = image_table::SimplePhotoGallery::new(data_dir);
            spg.sync(sync.directory);
        }
        SubCommand::Stat(stat) => {
            let mut spg = image_table::SimplePhotoGallery::new(data_dir);
            spg.stat(stat.filename);
        }
        SubCommand::Serve(serve) => {
            let mut changes = monitor_fs::monitor_changes(&data_dir);
            loop {
                let until = changes.next().await.expect("receive");
                let spg = image_table::SimplePhotoGallery::new(&data_dir);
                let sock_addr = SocketAddrV4::new(
                    serve.bind_address.parse().expect("invalid address"),
                    serve.port,
                );
                server::serve(sock_addr, until.map(|_| ()), spg.config, spg.image_table).await;
                eprintln!("Restarting server");
            }
        }
    };
}
