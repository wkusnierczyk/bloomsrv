use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use bloomlib::BloomFilter;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

// --- Data Structures ---

/// Container holding the filter and its configuration.
///
/// This struct is used to store the state of a specific bloom filter
/// inside the global HashMap.
///
/// # Examples
///
/// ```
/// use bloomsrv::{FilterContainer, CreationMode};
/// // Note: Requires bloomlib dependency to construct the inner filter
/// // This is just a structural example.
/// ```
pub struct FilterContainer {
    pub id: String,
    pub name: String,
    pub filter: BloomFilter<String>,
    pub capacity: usize,
    pub creation_mode: CreationMode,
}

/// Defines how the Bloom Filter was calculated during creation.
///
/// This is stored so that if we need to "clear" (re-create) the filter,
/// we know which parameters to use.
///
/// # Examples
///
/// ```
/// use bloomsrv::CreationMode;
///
/// let mode_rate = CreationMode::FalsePositiveRate(0.01);
/// let mode_hash = CreationMode::HashCount(5);
/// ```
#[derive(Clone, Copy)]
pub enum CreationMode {
    FalsePositiveRate(f64),
    HashCount(u32),
}

/// Global Thread-Safe State.
pub type SharedState = Arc<RwLock<HashMap<String, FilterContainer>>>;

// --- API Request/Response Models ---

#[derive(Deserialize)]
struct CreateRequest {
    name: String,
    item_count: usize,
    hash_count: Option<u32>,
    false_positive_rate: Option<f64>,
}

#[derive(Serialize)]
struct FilterResponse {
    id: String,
    name: String,
    message: String,
}

#[derive(Serialize)]
struct ListItem {
    id: String,
    name: String,
    item_count: usize,
    config: String,
}

// --- The App Factory ---

/// Creates the main Axum application router with the defined routes.
///
/// This function is the entry point for both the `main` binary and
/// integration tests.
///
/// # Arguments
///
/// * `state` - The shared state (Arc<RwLock<...>>) holding the filters.
///
/// # Examples
///
/// ```
/// use bloomsrv::{create_app, SharedState};
///
/// // Initialize the empty state
/// let state = SharedState::default();
///
/// // Create the router
/// let app = create_app(state);
///
/// // The app is now ready to be passed to axum::serve or used in tests
/// ```
pub fn create_app(state: SharedState) -> Router {
    Router::new()
        .route("/filters", post(filters_create))
        .route("/filters", get(filters_list))
        .route("/filters/:name", delete(filters_delete))
        .route("/filters/:name/items", post(filter_insert))
        .route("/filters/:name/items", get(filter_lookup))
        .route("/filters/:name/clear", put(filter_clear))
        .with_state(state)
}

// --- Request Handlers ---

async fn filters_create(
    State(state): State<SharedState>,
    Json(payload): Json<CreateRequest>,
) -> impl IntoResponse {
    let mut db = state.write();
    let filter_name = payload.name.clone();

    if db.contains_key(&filter_name) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": format!("Cannot create filter '{filter_name}', name is already in use") })),
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();

    let (filter, creation_mode) = if let Some(false_positive_rate) = payload.false_positive_rate {
        (
            BloomFilter::<String>::new(payload.item_count, false_positive_rate),
            CreationMode::FalsePositiveRate(false_positive_rate),
        )
    } else if let Some(hash_count) = payload.hash_count {
        (
            BloomFilter::<String>::new(payload.item_count, hash_count),
            CreationMode::HashCount(hash_count),
        )
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Must provide either false_positive_rate or hash_count" })),
        )
            .into_response();
    };

    let container = FilterContainer {
        id: id.clone(),
        name: filter_name.clone(),
        filter,
        capacity: payload.item_count,
        creation_mode,
    };

    db.insert(filter_name, container);

    let name = payload.name.clone();
    (
        StatusCode::CREATED,
        Json(FilterResponse {
            id: id.clone(),
            name: name.clone(),
            message: format!("Filter '{name}' created"),
        }),
    )
        .into_response()
}

async fn filters_delete(
    Path(id_or_name): Path<String>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let mut db = state.write();
    if db.remove(&id_or_name).is_some() {
        return (
            StatusCode::OK,
            Json(
                serde_json::json!({ "message": format!("Filter '{id_or_name}' has been deleted") }),
            ),
        );
    }
    let key = db
        .iter()
        .find(|(_, c)| c.id == id_or_name)
        .map(|(k, _)| k.clone());
    if let Some(name) = key {
        db.remove(&name);
        (
            StatusCode::OK,
            Json(serde_json::json!({ "message": format!("Filter '{name}' has been deleted") })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Filter '{id_or_name}' not found") })),
        )
    }
}

async fn filters_list(State(state): State<SharedState>) -> impl IntoResponse {
    let db = state.read();
    let list: Vec<ListItem> = db
        .values()
        .map(|c| {
            let config = match c.creation_mode {
                CreationMode::FalsePositiveRate(r) => format!("False positive rate: {}", r),
                CreationMode::HashCount(h) => format!("Hash count: {}", h),
            };
            ListItem {
                id: c.id.clone(),
                name: c.name.clone(),
                item_count: c.capacity,
                config,
            }
        })
        .collect();
    Json(list)
}

async fn filter_insert(
    Path(name): Path<String>,
    State(state): State<SharedState>,
    item: String,
) -> impl IntoResponse {
    let mut db = state.write();
    if let Some(c) = db.get_mut(&name) {
        c.filter.insert(&item);
        (
            StatusCode::OK,
            Json(
                serde_json::json!({ "response": format!("Item '{item}' inserted into filter '{name}'") }),
            ),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Filter '{name}' not found") })),
        )
    }
}

async fn filter_lookup(
    Path(name): Path<String>,
    State(state): State<SharedState>,
    item: String,
) -> impl IntoResponse {
    let db = state.read();
    if let Some(container) = db.get(&name) {
        let contains = container.filter.contains(&item);
        (
            StatusCode::OK,
            Json(serde_json::json!(
            {
                "contains": contains,
                "message": if contains {
                    format!("Item '{item}' may have been seen by filter '{name}'")
                } else {
                    format!("Item '{item}' cannot have been seen by filter '{name}'")
                }})),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Filter '{name}' not found") })),
        )
    }
}

async fn filter_clear(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let mut db = state.write();
    if let Some(container) = db.get_mut(&name) {
        container.filter.clear();
        (
            StatusCode::OK,
            Json(serde_json::json!({ "message": format!("Filter '{name}' has been cleared") })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Filter '{name}' not found") })),
        )
    }
}
