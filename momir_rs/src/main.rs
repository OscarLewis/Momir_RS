use crate::scss::compile_scss;
use askama::Template;
use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use serde::Deserialize;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::debug;
use tracing_subscriber::EnvFilter;
pub(crate) mod database;
pub(crate) mod scss;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexHtmlTemplate {}

#[derive(Debug)]
enum AppError {
    Template(askama::Error),
}

impl From<askama::Error> for AppError {
    fn from(err: askama::Error) -> Self {
        Self::Template(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Template(err) => {
                eprintln!("Template error: {err}");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render template",
                )
                    .into_response()
            }
        }
    }
}

#[derive(Clone)]
struct AppState {
    _scryfall_app_name: String,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("momir_rs=debug,sea_orm=debug")),
        )
        .init();

    // Compile SCSS styles
    debug!("Compiling scss styles");
    compile_scss()?;

    // Connect to database
    let mut opt = ConnectOptions::new("sqlite://momir.db?mode=rwc");
    opt.sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug)
        .sqlx_slow_statements_logging_settings(log::LevelFilter::Warn, Duration::from_secs(1));

    let db = Database::connect(opt).await?;

    db.execute_unprepared("PRAGMA foreign_keys = ON").await?;

    // Build schema into DB
    db.get_schema_registry("momir_rs::database::*")
        .sync(&db)
        .await?;

    // Check database connection
    check_database(&db).await;

    let shared_state = AppState {
        _scryfall_app_name: "momir_basic_rs/v0.1".to_string(),
        db,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/card-by-cmc", get(card_by_cmc))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let addr = listener.local_addr()?;
    debug!(%addr, "Server startup complete");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_database(db: &DatabaseConnection) {
    assert!(db.ping().await.is_ok());
}

async fn index() -> Result<Html<String>, AppError> {
    let template = IndexHtmlTemplate {};

    Ok(Html(template.render()?))
}

#[derive(Deserialize)]
struct CardByCMCParams {
    cmc: i32,
}

async fn card_by_cmc(Query(params): Query<CardByCMCParams>) -> String {
    format!("You sent: {}", params.cmc)
}
