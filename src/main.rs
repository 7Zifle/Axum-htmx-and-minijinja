use std::sync::Arc;

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

#[derive(Serialize, Clone)]
struct Todo {
    id: Uuid,
    name: String,
}

async fn get_todos(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let todos = state.todos.lock().await;
    RenderHtml(
        "test.html",
        state.engine.clone(),
        TodosParameters {
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

async fn get_edit_form(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    let todos = state.todos.lock().await.clone();
    let todo = todos.into_iter().find(|todo| todo.id == uuid);
    RenderHtml("form.html", state.engine.clone(), todo)
}

async fn delete_todo(State(state): State<Arc<AppState>>, Path(uuid): Path<Uuid>) -> StatusCode {
    let mut todos = state.todos.lock().await;
    todos.retain(|todo| todo.id != uuid);
    StatusCode::OK
}

#[derive(Serialize, Deserialize)]
struct TodoForm {
    description: String,
}

async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<String>,
    Form(payload): Form<TodoForm>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().await;

    match todos.iter_mut().find(|todo| todo.id.to_string() == uuid) {
        Some(todo) => {
            todo.name.clear();
            todo.name.push_str(&payload.description);
            ([("HX-Trigger", "reloadTodos")], StatusCode::OK)
        }
        None => ([("HX-Trigger", "reloadTodos")], StatusCode::BAD_REQUEST),
    }
}

async fn post_todo(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<TodoForm>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().await;
    todos.push(Todo {
        id: Uuid::new_v4(),
        name: payload.description,
    });
    [("HX-Trigger", "reloadTodos")]
}

// Define your application shared state
struct AppState {
    engine: AppEngine,
    todos: Mutex<Vec<Todo>>,
}

#[tokio::main]
async fn main() {
    let mut jinja = Environment::new();
    jinja.set_loader(path_loader("templates/"));

    let shared_state = Arc::new(AppState {
        engine: Engine::from(jinja),
        todos: Mutex::new(Vec::<Todo>::new()),
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
