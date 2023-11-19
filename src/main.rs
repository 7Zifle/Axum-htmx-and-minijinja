use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Router, Server};
use axum_template::{engine::Engine, RenderHtml};
use minijinja::{path_loader, Environment};
use serde::Serialize;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

type AppEngine = Engine<Environment<'static>>;

//#[derive(Debug, Serialize)]
//pub struct Person {
//name: String,
//}

//async fn get_name(
//// Obtain the engine
//State(state): State<Arc<AppState>>,
//// Extract the key
//Key(key): Key,
//Path(name): Path<String>,
//) -> impl IntoResponse {
//let person = Person { name };

//RenderHtml(key, state.engine.clone(), person)
//}

#[derive(Serialize)]
struct TestParameters {
    names: Vec<String>,
}

async fn get_test(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut names = state.names.lock().await;
    names.push(String::from("New test"));

    RenderHtml(
        "test.html",
        state.engine.clone(),
        TestParameters {
            names: names.clone(),
        },
    )
}

async fn get_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    RenderHtml("index.html", state.engine.clone(), {})
}

async fn get_htmx_resp(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    RenderHtml("htmx-resp.html", state.engine.clone(), {})
}

// Define your application shared state
struct AppState {
    engine: AppEngine,
    names: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let mut jinja = Environment::new();
    let names = vec![String::from("test"), String::from("test2")];
    jinja.set_loader(path_loader("templates/"));

    let shared_state = Arc::new(AppState {
        engine: Engine::from(jinja),
        names: Mutex::new(names),
    });

    let app = Router::new()
        .route("/test", get(get_test))
        .route("/", get(get_index))
        .route("/htmx-test", get(get_htmx_resp))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_state.clone());

    println!("Starting server on port 8080");
    Server::bind(&([127, 0, 0, 1], 8080).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
