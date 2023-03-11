use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use serde_json::json;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use sysinfo::{CpuExt, System, SystemExt};

#[derive(Clone, Serialize)]
struct AppState {
    cpus: Vec<f32>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState { cpus: vec![] }));

    let thread_state = Arc::clone(&state);
    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let cpus: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            thread_state.write().unwrap().cpus = cpus;
            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/cpu", get(cpu))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();
    Html(markup)
}

async fn cpu(State(sys): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    Json(json!(sys.read().unwrap().clone()))
}
