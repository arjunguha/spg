use super::config::Config;
use super::image_table::ImageTable;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::Filter;

async fn gallery_list(image_table: Arc<ImageTable>) -> Result<impl warp::Reply, warp::Rejection> {
    return Ok(warp::reply::json(&image_table.gallery_list()));
}

async fn gallery_contents(
    gallery: String,
    image_table: Arc<ImageTable>,
) -> Result<impl warp::Reply, warp::Rejection> {
    return Ok(warp::reply::json(&image_table.gallery_contents(&gallery)));
}

pub async fn serve(
    addr: impl Into<SocketAddr> + 'static,
    until: impl Future<Output = ()> + Send + 'static,
    config: Config,
    image_table: ImageTable,
) {
    let image_table = Arc::new(image_table);
    let config = Arc::new(config);

    let gallery_list_route = {
        let image_table = image_table.clone();
        warp::path!("api" / "list_galleries")
            .and(warp::get())
            .and(warp::any().map(move || image_table.clone()))
            .and_then(gallery_list)
    };

    let gallery_contents_route = {
        let image_table = image_table.clone();
        warp::path!("api" / "gallery_contents")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || image_table.clone()))
            .and_then(gallery_contents)
    };

    let routes = gallery_list_route
        .or(gallery_contents_route)
        .or(warp::fs::dir(format!("{}/www", config.data_dir)));

    warp::serve(routes)
        .bind_with_graceful_shutdown(addr, until)
        .1
        .await;
}
