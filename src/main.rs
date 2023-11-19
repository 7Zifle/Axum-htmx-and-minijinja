use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use axum_template::{engine::Engine, Key, RenderHtml};
use minijinja::{path_loader, Environment};
use serde::Serialize;

type AppEngine = Engine<Environment<'static>>;

#[derive(Debug, Serialize)]
pub struct Person {
    name: String,
}

async fn get_name(
    // Obtain the engine
    State(state): State<Arc<AppState>>,
    // Extract the key
    Key(key): Key,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let person = Person { name };

    RenderHtml(key, state.engine.clone(), person)
}

async fn get_test(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    RenderHtml("test.html", state.engine.clone(), {})
}

// Define your application shared state
struct AppState {
    engine: AppEngine,
}

#[tokio::main]
async fn main() {
    let mut jinja = Environment::new();
    jinja
        .add_template("/old/:name", "<h1>Hello Minijinja!</h1><p>{{name}}</p>")
        .unwrap();
    jinja.set_loader(path_loader("templates/"));

    let shared_state = Arc::new(AppState {
        engine: Engine::from(jinja),
    });

    let app = Router::new()
        .route("/old/:name", get(get_name))
        .with_state(shared_state.clone())
        .route("/test", get(get_test))
        .with_state(shared_state.clone());

    println!("Starting server on port 8080");
    Server::bind(&([127, 0, 0, 1], 8080).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
