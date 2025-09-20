mod database;
mod datatype_endpoint;

use crate::database::database::GameDatabase;
use crate::datatype_endpoint::GameMetadata;
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{post},
};
use once_cell::sync::Lazy;

pub static DATABASE: Lazy<GameDatabase> = Lazy::new(|| GameDatabase::new());

async fn create_game_metadata(Json(payload): Json<GameMetadata>) -> StatusCode {
    DATABASE.add_game_metadata(&payload);
    StatusCode::CREATED
}

#[tokio::main]
async fn main() {
    Lazy::force(&DATABASE);
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new().route("/game_metadata", post(create_game_metadata));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
