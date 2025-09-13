use axum::Router;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use decentralized_decision_vote::api::routes::create_router;
use decentralized_decision_vote::core::state::AppState;
use decentralized_decision_vote::cli::{parse_args, execute_cli};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = parse_args();
    if !args.api_mode {
        let code = execute_cli(args).await;
        if code != 0 { std::process::exit(code); }
        return;
    }

    let state = AppState::new().await;
    let app: Router = create_router(state.clone())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("signal received, starting graceful shutdown");
}
