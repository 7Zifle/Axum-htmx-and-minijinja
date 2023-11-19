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
    todos: Vec<String>,
}

async fn get_test(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut todos = state.todos.lock().await;
    todos.push(String::from("New test"));

    RenderHtml(
        "test.html",
        state.engine.clone(),
        TestParameters {
            todos: todos.clone(),
        },
    )
}

async fn get_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    RenderHtml("index.html", state.engine.clone(), {})
}

async fn get_htmx_resp(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    RenderHtml("htmx-resp.html", state.engine.clone(), {})
}

async fn get_page_home(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    RenderHtml("home-body.html", state.engine.clone(), {})
}

// Define your application shared state
struct AppState {
    engine: AppEngine,
    todos: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let mut jinja = Environment::new();
    let todos = vec![String::from("test"), String::from("test2")];
    jinja.set_loader(path_loader("templates/"));

    let shared_state = Arc::new(AppState {
        engine: Engine::from(jinja),
        todos: Mutex::new(todos),
    });

    let app = Router::new()
        .route("/api/htmx-test", get(get_htmx_resp))
        .route("/test", get(get_test))
        .route("/", get(get_index))
        .route("/page/home", get(get_page_home))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_state.clone());

    println!("Starting server on port 8080");
    Server::bind(&([127, 0, 0, 1], 8080).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
