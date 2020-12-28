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

/// Downloads an original image by hash. Note that we only download an image that is in the
/// ImageTable, and do not give unrestricted file system access.
async fn original(
    hash: String,
    image_table: Arc<ImageTable>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // NOTE(arjun): It is fairly obvious in this code that errors are being silently rejected.
    let hash = u128::from_str_radix(&hash, 16).map_err(|_err| warp::reject())?;
    let row = image_table.get_by_hash(hash).ok_or(warp::reject())?;
    let body = std::fs::read(&row.original_path).map_err(|_err| warp::reject())?;
    let filename = &row.original_path;
    return Ok(warp::reply::with_header(
        body,
        http::header::CONTENT_DISPOSITION,
        format!("attachment; filename={}", filename),
    ));
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

    let original_image_route = {
        let image_table = image_table.clone();
        warp::path!("api" / "original" / String)
            .and(warp::get())
            .and(warp::any().map(move || image_table.clone()))
            .and_then(original)
    };

    let routes = gallery_list_route
        .or(gallery_contents_route)
        .or(original_image_route)
        .or(warp::fs::dir(format!("{}/www", config.data_dir)));

    warp::serve(routes)
        .bind_with_graceful_shutdown(addr, until)
        .1
        .await;
}
