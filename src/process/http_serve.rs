use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    // std 里的 SocketAddr
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Serving {:?} on port {}", path, addr);

    let state = HttpServeState { path };

    let router = Router::new()
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));

    // tokio TcpListener
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

// pattern match 写法，如果是直接 state: State<Arc<HttpServeState>> 这样的写法，那访问 state 就需要 state.0
// Path 是 extract::Path
async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    // format!("{:?}, {:?}", state, path)
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        (
            StatusCode::NOT_FOUND,
            format!("File {} not found", p.display()),
        )
    } else {
        match tokio::fs::read_to_string(p).await {
            Ok(content) => {
                info!("Read {} bytes", content.len());
                (StatusCode::OK, content)
            }
            Err(e) => {
                warn!("Error reading file {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
    }
}
