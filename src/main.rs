use axum::body;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use http::{header, HeaderValue, StatusCode};
use include_dir::{include_dir, Dir};
use tracing::info;

const STATIC_DIR: Dir = include_dir!("./static");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let app = axum::Router::new()
        .route("/static/*path", get(static_path));

    let address = &"0.0.0.0:8080".parse()?;
    info!("Server listening on http://{}", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// from https://bloerg.net/posts/serve-static-content-with-axum/
async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(body::Empty::new()))
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(body::Full::from(file.contents())))
            .unwrap(),
    }
}
