use crate::{
    game_manager::GameManager,
    scss::compile_scss,
    site_console::{ConsoleMessage, SiteConsole},
};
use askama::Template;
use axum::{
    Router,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rand::RngExt;
use scryfall_oracle::{
    ScryfallCard,
    bulk_data::oracle_cards::{
        OracleCards,
        filters::{OracleFilter, OracleFilters},
    },
    client::ScryfallClient,
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

pub(crate) mod database;
pub(crate) mod game_manager;
pub(crate) mod scss;
pub(crate) mod site_console;

const MOMIR_VIG_SIMIC_VISIONARY_AVATAR_SCRYFALL_ID: &str = "f5ed5ad3-b970-4720-b23b-308a25f42887";

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
    momir_card: Option<ScryfallCard>,
    db: DatabaseConnection,
    game_manager: GameManager,
    console: SiteConsole,
    oracle: OracleCards,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init Logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("momir_rs=debug,sea_orm=debug,scryfall_oracle=debug")
        }))
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

    let scryfall = ScryfallClient::new()?;
    let cache_path = PathBuf::from("/home/oscar/Documents/Projects/momir_rs_workspace/cache");
    let oracle = OracleCards::new(&scryfall, Some(&cache_path)).await?;
    if let Some(cards) = oracle.cards.as_ref() {
        debug!(
            num_cards_in_oracle_map = cards.len(),
            "Number of 'Creature' cards found in Oracle Card Bulk Data export"
        );
    }

    let rand_cmc = rand::rng().random_range(1..=16) as f64;

    if let Some(card) = oracle.random_creature_by_cmc(rand_cmc, None) {
        debug!(
            rand_cmc,
            card_name = %card.core.name,
            scryfall_id = %card.core.id,
            "Random creature of the day"
        );
    }

    let momir_avatar =
        ScryfallCard::by_id(&scryfall, MOMIR_VIG_SIMIC_VISIONARY_AVATAR_SCRYFALL_ID).await?;

    let shared_state = AppState {
        _scryfall_app_name: "momir_basic_rs/v0.1".to_string(),
        momir_card: Some(momir_avatar.clone()),
        db,
        game_manager: GameManager::new(),
        console: SiteConsole::new(),
        oracle,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/card-by-cmc", get(card_by_cmc))
        .route("/ws/messages/{game_id}", get(websocket))
        .nest_service("/static", ServeDir::new("momir_rs/static"))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let addr = listener.local_addr()?;
    info!(%addr, "Server startup complete");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_database(db: &DatabaseConnection) {
    assert!(db.ping().await.is_ok());
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexHtmlTemplate {
    game_id: String,
    momir_card: Option<ScryfallCard>,
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let game_id = state.game_manager.new_game();

    state.console.send(
        &game_id,
        ConsoleMessage::Text {
            sender: "System".to_string(),
            body: "Game initialized.".to_string(),
        },
    );

    let template = IndexHtmlTemplate {
        game_id,
        momir_card: state.momir_card.clone(),
    };

    Ok(Html(template.render()?))
}

#[derive(Deserialize)]
struct CardByCMCParams {
    cmc: i32,
    game_id: String,

    #[serde(default)]
    unknown_event_filter: bool,

    #[serde(default)]
    modern_filter: bool,

    #[serde(default)]
    premodern_filter: bool,

    #[serde(default)]
    unset_filter: bool,
}

async fn card_by_cmc(Query(params): Query<CardByCMCParams>, State(state): State<AppState>) {
    let filter_checks = [
        (OracleFilter::UnknownEvent, params.unknown_event_filter),
        (OracleFilter::Modern, params.modern_filter),
        (OracleFilter::Premodern, params.premodern_filter),
        (OracleFilter::Unsets, params.unset_filter),
    ];

    let filters = {
        let filters: Vec<_> = filter_checks
            .into_iter()
            .filter_map(|(filter, checked)| (!checked).then_some(filter))
            .collect();

        (!filters.is_empty()).then(|| OracleFilters::from_vec(filters))
    };

    let card = state
        .oracle
        .random_creature_by_cmc(params.cmc.into(), filters.as_ref());
    let message = match card {
        Some(card) => {
            let cmc = card.core.cmc.expect("card selected by CMC must have a CMC");

            debug!(
                card_name = %card.core.name,
                card_cmc = cmc,
                card_id = %card.core.id,
                "Momir generated a card"
            );

            ConsoleMessage::Card {
                sender: "Momir".to_string(),
                card: card.clone(),
            }
        }

        None => ConsoleMessage::Text {
            sender: "Momir".to_string(),
            body: format!("No creature found for CMC {}", params.cmc),
        },
    };

    state.console.send(&params.game_id, message);
}

async fn websocket(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, game_id, state.console))
}

async fn handle_socket(mut socket: WebSocket, game_id: String, console: SiteConsole) {
    let mut rx = console.subscribe(&game_id);

    console.send(
        &game_id,
        ConsoleMessage::Text {
            sender: "System".to_string(),
            body: "WebSocket connected.".to_string(),
        },
    );

    while let Ok(message) = rx.recv().await {
        let rendered = match render_message(&message) {
            Ok(rendered) => rendered,
            Err(err) => {
                tracing::error!(%err, "Failed to render console message");
                continue;
            }
        };

        if socket.send(Message::Text(rendered.into())).await.is_err() {
            break;
        }
    }
}

#[derive(Template)]
#[template(path = "message_fragment.html")]
struct MessageFragmentTemplate<'a> {
    message: &'a ConsoleMessage,
}

fn render_message(message: &ConsoleMessage) -> Result<String, askama::Error> {
    MessageFragmentTemplate { message }.render()
}
