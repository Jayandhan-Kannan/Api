#![allow(unused)]

use std::net::SocketAddr;
use axum::{
    extract::{Json, Path, State}, http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Router
};
use serde::{Deserialize, Serialize};
use serde_json::de;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct Item {
    id: i32,
    name: String,
}

type Details = Arc<Mutex<Vec<Item>>>;

#[tokio::main]
async fn main() {
    // Shared in-memory database
    let db: Details = Arc::new(Mutex::new(Vec::new()));

    // Router definition
    let app = Router::new()
        .route("/test", get(test))
        .route("/student", post(create_student).get(get_all_students))
        .route("/student/:id", get(get_student).delete(delete_student).put(update_student))
        .with_state(db);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn test() -> impl IntoResponse {
    "Test call get successfully"
}

async fn create_student(
    State(db): State<Details>,
    Json(details): Json<Item>,
) -> (StatusCode, Json<Item>) {
    let mut db = db.lock().unwrap();
    let new_item = Item {
        id: details.id,
        name: details.name.clone(),
    };
    db.push(new_item.clone());
    (StatusCode::CREATED, Json(new_item))    
}

async fn get_student(State(db): State<Details>, Path(id):Path<i32>) -> Result<Json<Item>, StatusCode>
{
    let db = db.lock().unwrap();
    db.iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NO_CONTENT)
}

async fn get_all_students(State(db): State<Details>) -> Result<Json<Vec<Item>>, StatusCode>
{
    let db = db.lock().unwrap();
    if db.len() == 0
    {
        Err(StatusCode::NO_CONTENT)
    }
    else {
        Ok(Json(db.clone()))
    }
    
}

async fn update_student(State(db): State<Details>,
    Path(id):Path<i32>,
    Json(details): Json<Item>) -> Result<Json<Item>, StatusCode>
{
    let mut db = db.lock().unwrap();
    if let Some(item) = db.iter_mut().find(|item| item.id == id)
    {
        item.id = details.id.clone();
        item.name = details.name.clone();
        Ok(Json(item.clone()))
    }
    else if db.len() == 0 
    {
        Err(StatusCode::NO_CONTENT)
    }
    else 
    {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_student(State(db): State<Details>,Path(id):Path<i32>) -> StatusCode
{
    let mut db = db.lock().unwrap();
    if db.iter().any(|item| item.id == id)
    {
        db.retain(|item| item.id != id);
        StatusCode::OK
    }
    else if db.len() == 0 
    {
        StatusCode::NO_CONTENT
    }
    else
    {
        StatusCode::NOT_FOUND
    }
}

