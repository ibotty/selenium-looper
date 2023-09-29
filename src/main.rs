mod errors;
mod selenium;

use crate::errors::*;
use crate::selenium::Script;

use askama::Template;
use axum::{
    body, debug_handler,
    extract::Path,
    response::{IntoResponse, Response, Result},
    routing::{get, post},
};
use axum_extra::extract::Form;
use axum_htmx::HxBoosted;
use http::{header, HeaderValue, StatusCode};
use include_dir::{include_dir, Dir};
use serde_derive::{Deserialize, Serialize};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, info, instrument};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const STATIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    // install global default tracing subscriber using RUST_LOG env variable
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let app = axum::Router::new()
        .route("/", get(|| async { IndexTemplate {} }))
        .route("/loop", post(generate_loop_script))
        .route("/partials/script-form/", post(post_script_form))
        .route(
            "/partials/script-form/*path",
            get(empty_script_form).delete(|| async {}),
        )
        .route("/assets/*path", get(serve_asset));

    let address = &"0.0.0.0:8080".parse()?;
    info!("Server listening on http://{}", address);
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[derive(Template)]
#[template(path = "script-form.html")]
struct ScriptFormTemplate {
    script_id: String,
    script_name: String,
    script_json: String,
    input_name_suffix: String,
    new: bool,
}

#[derive(Deserialize, Serialize)]
struct PostScript {
    script_id: String,
    script_json: String,
}

#[debug_handler]
#[instrument]
async fn post_script_form(
    Form(PostScript {
        script_id,
        script_json,
    }): Form<PostScript>,
) -> Result<impl IntoResponse> {
    let script_json_decoded: Script =
        serde_json::from_str(&script_json).map_err(AppError::ScriptInvalid)?;

    Ok(ScriptFormTemplate {
        // input_name_suffix: format!("[{}]", script_id.clone()),
        input_name_suffix: "".to_string(),
        new: false,
        script_name: script_json_decoded.name,
        script_id,
        script_json,
    })
}

#[instrument]
#[debug_handler]
async fn empty_script_form(Path(_script_name): Path<String>) -> impl IntoResponse {
    ScriptFormTemplate {
        script_id: "new".to_string(),
        script_name: "".to_string(),
        script_json: "".to_string(),
        input_name_suffix: "".to_string(),
        new: true,
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct LoopForm {
    script_id: Vec<String>,
    // script_json: Vec<String>,
    script_json: String,
    data: String,
}

#[derive(Template)]
#[template(path = "loop-result.html")]
struct LoopResult {
    result_json: String,
}

#[instrument]
#[debug_handler]
async fn generate_loop_script(
    HxBoosted(boosted): HxBoosted,
    Form(input): Form<LoopForm>,
) -> Result<impl IntoResponse> {
    // if boosted {
    // } else {
    // }

    // let script_jsons: std::result::Result<Vec<_>, _> = input.script_json
    let script_jsons: std::result::Result<Vec<_>, _> = vec![input.script_json]
        .into_iter()
        .map(|s| serde_json::from_str(&s).map_err(AppError::ScriptInvalid))
        .collect();
    let scripts = input
        .script_id
        .into_iter()
        .zip(script_jsons?.into_iter())
        .collect();

    let input_data = serde_json::from_str(&input.data).map_err(AppError::InputDataInvalid)?;

    let result = selenium::generate_loop_script(scripts, input_data)?;

    let result_json = serde_json::to_string_pretty(&result).unwrap();
    Ok(LoopResult { result_json })
}

// from https://bloerg.net/posts/serve-static-content-with-axum/
#[debug_handler]
#[instrument]
async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_matches('/');
    let candidates = [path.to_string(), format!("{}.html", path)];

    candidates
        .iter()
        .find_map(|candidate| {
            debug!("considering {}", candidate);
            let mime_type = mime_guess::from_path(candidate).first_or_text_plain();
            STATIC_DIR.get_file(candidate).map(|file| {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                    )
                    .body(body::boxed(body::Full::from(file.contents())))
                    .unwrap()
            })
        })
        .unwrap_or(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::boxed(body::Empty::new()))
                .unwrap(),
        )
}

#[instrument]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("signal received, starting graceful shutdown");
}
