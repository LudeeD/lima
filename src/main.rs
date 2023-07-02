use askama::Template;
use axum::{extract::State, http::StatusCode, response::Redirect, routing::get, Form, Router};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use std::{env, net::SocketAddr};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    name: &'a str,
    items: Vec<String>,
}

async fn index(State(state): State<AppState>) -> Result<IndexTemplate<'static>, StatusCode> {
    let pool = state.db_pool;
    let recs = sqlx::query!(
        r#"
SELECT * 
FROM Items
ORDER BY id
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(IndexTemplate {
        name: "world",
        items: recs.iter().map(|record| format!("{:?}", record)).collect(),
    })
}

#[derive(Deserialize, Debug)]
struct NewItem {
    name: String,
    description: String,
    location: String,
    quantity: i64,
}

async fn create_item(
    State(state): State<AppState>,
    Form(input): Form<NewItem>,
) -> Result<Redirect, StatusCode> {
    // Insert the task, then obtain the ID of this row
    let _ = sqlx::query!(
        r#"
INSERT INTO Items ( name, description, location, quantity )
VALUES ( ?1 , ?2, ?3, ?4)
        "#,
        input.name,
        input.description,
        input.location,
        input.quantity
    )
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .last_insert_rowid();

    Ok(Redirect::to("/"))
}

#[derive(Clone)]
struct AppState {
    db_pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await?;

    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/", get(index).post(create_item))
        .with_state(state);

    let app = app.merge(using_serve_dir_with_assets_fallback());

    println!("Hello, world!");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn using_serve_dir_with_assets_fallback() -> Router {
    // `ServeDir` allows setting a fallback if an asset is not found
    // so with this `GET /assets/doesnt-exist.jpg` will return `index.html`
    // rather than a 404
    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/404.html"));

    Router::new()
        .nest_service("/static", serve_dir.clone())
        .fallback_service(serve_dir)
}
