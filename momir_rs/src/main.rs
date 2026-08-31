use crate::{
    game_manager::GameManager,
    scss::compile_scss,
    site_console::{ConsoleMessage, SiteConsole},
};
use askama::Template;
use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use momir_oracle_config::{AppConfig, load_config};
use oracle_escpos::{OracleNetworkPrinter, test_img_print, test_mdfc_img_print};
use rand::RngExt;
use scryfall_oracle::{
    OracleScryfallCard,
    bulk_data::oracle_cards::{
        OracleCards,
        filters::{OracleFilter, OracleFilters},
    },
    client::ScryfallClient,
    sets::sets::ScryfallSets,
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tokio::{fs, net::TcpListener};
use tower_http::services::ServeDir;
use tracing::{debug, info, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
pub(crate) mod database;
pub(crate) mod game_manager;
pub(crate) mod scss;
pub(crate) mod site_console;

const MOMIR_VIG_SIMIC_VISIONARY_AVATAR_SCRYFALL_ID: &str = "f5ed5ad3-b970-4720-b23b-308a25f42887";

#[derive(Debug)]
enum AppError {
    Template(askama::Error),
    Internal(String),
}

impl From<askama::Error> for AppError {
    fn from(err: askama::Error) -> Self {
        Self::Template(err)
    }
}
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::Internal(err.to_string())
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
            Self::Internal(err) => {
                eprintln!("Internal error: {err}");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal error: {err}"),
                )
                    .into_response()
            }
        }
    }
}

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    momir_card: Option<OracleScryfallCard>,
    db: DatabaseConnection,
    client: ScryfallClient,
    game_manager: GameManager,
    console: SiteConsole,
    oracle: OracleCards,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init Logging
    let log_dir = std::path::Path::new("logs");
    clean_old_logs(log_dir).await?;
    let file_appender = rolling::daily("logs", "momir_rs.jsonl");
    let (non_blocking, _guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("momir_rs=debug,sea_orm=debug,scryfall_oracle=debug,oracle_escpos=debug")
        }))
        .with(fmt::layer())
        .with(fmt::layer().json().with_writer(non_blocking))
        .init();

    // Load config
    let config = load_config()?;
    info!(config = ?config, "Config loaded");

    // Compile SCSS styles
    debug!("Compiling scss styles");
    compile_scss()?;

    // Connect to database
    // TODO move the path for this into Config
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

    let user_agent = config.server.scryfall_user_agent.as_ref();
    let scryfall = ScryfallClient::new(Some(user_agent))?;

    let sets = ScryfallSets::new(&scryfall).await?;
    debug!(num_sets = sets.len(), "Fetched Sets from Scryfall");

    let cache_path =
        std::path::PathBuf::from("/home/oscar/Documents/Projects/momir_rs_workspace/cache");
    let oracle = OracleCards::new(&scryfall, Some(&cache_path), Some(sets)).await?;

    

    // let printer = OraclePrinter::new(config.printer.host.clone(), config.printer.port);

    // let printer = if printer.check_connection().await {
    //     debug!(
    //         printer_host = config.printer.host,
    //         printer_port = config.printer.port,
    //         "Printer connected"
    //     );
    //     Some(printer)
    // } else {
    //     warn!("Printer is not connected");
    //     None
    // };

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
        OracleScryfallCard::by_id_live(&scryfall, MOMIR_VIG_SIMIC_VISIONARY_AVATAR_SCRYFALL_ID)
            .await?;

    let shared_state = AppState {
        config,
        momir_card: Some(momir_avatar.clone()),
        db,
        client: scryfall,
        game_manager: GameManager::new(),
        console: SiteConsole::new(),
        oracle,
    };

    let addr = format!(
        "{}:{}",
        shared_state.config.server.host, shared_state.config.server.port
    )
    .parse::<SocketAddr>()?;

    let app = Router::new()
        .route("/", get(index))
        .route("/games", get(games_handler))
        .route("/card-by-cmc", get(card_by_cmc))
        .route("/test/imgprint", get(test_img_print_handler))
        .route("/test/mdfcprint", get(test_mdfc_img_print_handler))
        .route("/ws/messages/{game_id}", get(websocket))
        .route("/print/token", get(print_momir_token))
        .nest_service("/static", ServeDir::new("momir_rs/static"))
        .with_state(shared_state);

    let listener = TcpListener::bind(addr).await?;

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
    momir_card: Option<OracleScryfallCard>,
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

async fn games_handler(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.game_manager.list())
}

async fn test_img_print_handler(
    State(_state): State<AppState>,
) -> Result<(StatusCode, String), AppError> {
    test_img_print()?;

    Ok((StatusCode::OK, "Testing IMG for printer...".into()))
}

async fn test_mdfc_img_print_handler(
    State(_state): State<AppState>,
) -> Result<(StatusCode, String), AppError> {
    test_mdfc_img_print()?;

    Ok((StatusCode::OK, "Testing IMG for printer...".into()))
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

    #[serde(default)]
    everything_else_filter: bool,
}

#[axum::debug_handler]
async fn card_by_cmc(
    Query(params): Query<CardByCMCParams>,
    State(state): State<AppState>,
) -> Result<(), AppError> {
    let filter_checks = [
        (OracleFilter::UnknownEvent, params.unknown_event_filter),
        (OracleFilter::Modern, params.modern_filter),
        (OracleFilter::Premodern, params.premodern_filter),
        (OracleFilter::Unsets, params.unset_filter),
        (OracleFilter::EverythingElse, params.everything_else_filter),
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

            // TODO Enable printing
            // if let Some(printer) = &state.printer {
            //     printer
            //         .print_oracle_scryfall_card(&card, None)
            //         .await
            //         .map_err(|e| AppError::Internal(e.to_string()))?;
            // }

            // let printer = OraclePrinter::from(&state.config);

            // if printer.check_connection().await {
            //     let card = card.clone();

            //     tokio::spawn(async move {
            //         if let Err(e) = printer.print_oracle_scryfall_card(&card, None).await {
            //             warn!(
            //                 error = %e,
            //                 card_name = %card.core.name,
            //                 "Failed to print card"
            //             );
            //         }
            //     });
            // } else {
            //     warn!("Printer is not reachable");
            // }

            ConsoleMessage::Card {
                sender: "Momir".to_string(),
                card: card,
            }
        }

        None => ConsoleMessage::Text {
            sender: "Momir".to_string(),
            body: format!("No creature found for CMC {}", params.cmc),
        },
    };

    state.console.send(&params.game_id, message);
    Ok(())
}

#[derive(Serialize)]
struct PrintResponse {
    success: bool,
    message: String,
}

async fn print_momir_token(
    State(state): State<AppState>,
) -> Result<Json<PrintResponse>, StatusCode> {
    // let printer = OraclePrinter::from(&state.config);

    // if !printer.check_connection().await {
    //     return Err(StatusCode::SERVICE_UNAVAILABLE);
    // }

    let momir = state.momir_card.clone().ok_or(StatusCode::NOT_FOUND)?;

    // printer
    //     .print_oracle_scryfall_card(&momir, None)
    //     .await
    //     .map_err(|e| {
    //         warn!(
    //             error = %e,
    //             card_name = %momir.core.name,
    //             "Failed to print card"
    //         );

    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;

    Ok(Json(PrintResponse {
        success: true,
        message: "Token printed!".to_string(),
    }))
}

async fn websocket(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, game_id, state.console, state.config.clone()))
}

async fn handle_socket(
    mut socket: WebSocket,
    game_id: String,
    console: SiteConsole,
    config: AppConfig,
) {
    let mut rx = console.subscribe(&game_id);

    console.send(
        &game_id,
        ConsoleMessage::Text {
            sender: "System".to_string(),
            body: "Websocket connected".to_string(),
        },
    );

    // let printer = OraclePrinter::from(&config);

    // if printer.check_connection().await {
    //     console.send(
    //         &game_id,
    //         ConsoleMessage::Text {
    //             sender: "System".to_string(),
    //             body: format!(
    //                 "Printer connected at {}:{}",
    //                 &config.printer.host, config.printer.port
    //             ),
    //         },
    //     );
    // }

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

async fn clean_old_logs(log_dir: &std::path::Path) -> std::io::Result<()> {
    if !tokio::fs::try_exists(log_dir).await? {
        tokio::fs::create_dir_all(log_dir).await?;
    }

    let mut entries = tokio::fs::read_dir(log_dir).await?;
    let mut logs = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();

        if name
            .to_str()
            .is_some_and(|name| name.starts_with("momir_rs.jsonl."))
        {
            logs.push(entry);
        }
    }

    logs.sort_by_key(|entry| entry.file_name());

    while logs.len() > 3 {
        let oldest = logs.remove(0);
        tokio::fs::remove_file(oldest.path()).await?;
    }

    Ok(())
}
