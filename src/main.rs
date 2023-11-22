mod todo;

use std::{sync::Arc, time::Duration};

use crate::todo::Todo;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Form, Router, Server,
};
use axum_template::{engine::Engine, RenderHtml};
use minijinja::{path_loader, Environment};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use uuid::Uuid;

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
struct TodosParameters {
    todos: Vec<Todo>,
}

async fn get_todos(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let todos = match Todo::db_get_todos(&state.pool).await {
        Ok(todos) => todos,
        Err(_) => Vec::<Todo>::new(),
    };
    RenderHtml("test.html", state.engine.clone(), TodosParameters { todos })
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

async fn get_edit_form(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    let todo = Todo::db_find_by_uuid(uuid, &state.pool).await.ok();
    RenderHtml("form.html", state.engine.clone(), todo)
}

async fn delete_todo(State(state): State<Arc<AppState>>, Path(uuid): Path<Uuid>) -> StatusCode {
    match Todo::db_delete(uuid, &state.pool).await {
        Ok(_rows) => StatusCode::OK,
        Err(e) => {
            println!("Error deleting from database E: {}", e.to_string());
            StatusCode::BAD_REQUEST
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TodoForm {
    description: String,
}

async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Form(payload): Form<TodoForm>,
) -> impl IntoResponse {
    let todo = Todo {
        uuid,
        name: payload.description,
    };
    let _ = todo.db_update(&state.pool).await;
    RenderHtml("todo.html", state.engine.clone(), todo)
}

async fn post_todo(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<TodoForm>,
) -> impl IntoResponse {
    let todo = Todo {
        uuid: Uuid::now_v7(),
        name: payload.description,
    };
    let _ = todo.db_insert(&state.pool).await;
    RenderHtml("todo.html", state.engine.clone(), todo)
}

// Define your application shared state
struct AppState {
    engine: AppEngine,
    pool: PgPool,
}

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect("postgres:///todo_app")
        .await
        .expect("Can't connect to database");

    let mut jinja = Environment::new();
    jinja.set_loader(path_loader("templates/"));

    let shared_state = Arc::new(AppState {
        engine: Engine::from(jinja),
        pool,
    });

    let app = Router::new()
        .route("/api/htmx-test", get(get_htmx_resp))
        .route("/api/todos", get(get_todos))
        .route("/", get(get_index))
        .route("/page/home", get(get_page_home))
        .route("/api/add-todo", post(post_todo))
        .route("/api/delete/:id", delete(delete_todo))
        .route("/api/update/:id", patch(update_todo))
        .route("/api/edit/:id", get(get_edit_form))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_state.clone());

    println!("Starting server on port 8080");
    Server::bind(&([127, 0, 0, 1], 8080).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
