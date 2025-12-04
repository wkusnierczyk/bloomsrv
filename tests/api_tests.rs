use axum::{
    body::Body,
    http::{Request, StatusCode},
    // Router is removed here because type inference handles it
};
use http_body_util::BodyExt; // specific dependency for reading bodies
use tower::ServiceExt; // for `oneshot`

// !!! IMPORTANT: Replace 'bloom_daemon' with the actual name of your package from Cargo.toml !!!
use bloomsrv::{create_app, SharedState};

// --- Helper to convert response body to Serde Value ---
async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

// --- Unit/Integration Tests ---
// Note: These are now top-level functions, not inside a 'mod tests'

#[tokio::test]
async fn test_create_filter_validation() {
    let state = SharedState::default();
    let app = create_app(state);

    // Case 1: Missing required params (neither hash_count nor fp_rate)
    let payload = serde_json::json!({
        "name": "bad_filter",
        "item_count": 1000
    });

    let req = Request::builder()
        .method("POST")
        .uri("/filters")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_non_existent() {
    let state = SharedState::default();
    let app = create_app(state);

    let req = Request::builder()
        .method("DELETE")
        .uri("/filters/ghost_filter")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_full_filter_lifecycle() {
    let state = SharedState::default();

    // 1. CREATE a filter
    let create_payload = serde_json::json!({
        "name": "login_attempts",
        "item_count": 1000,
        "false_positive_rate": 0.01
    });

    let req = Request::builder()
        .method("POST")
        .uri("/filters")
        .header("content-type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let json = response_json(response).await;
    let filter_id = json.get("id").unwrap().as_str().unwrap();
    assert!(!filter_id.is_empty());

    // 2. LIST to verify it exists
    let req = Request::builder()
        .method("GET")
        .uri("/filters")
        .body(Body::empty())
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = response_json(response).await;
    let list = json.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "login_attempts");

    // 3. LOOKUP (Should be empty/false)
    let req = Request::builder()
        .method("GET")
        .uri("/filters/login_attempts/items")
        .body(Body::from("user_123")) // Body acts as the item
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["contains"], false);

    // 4. INSERT item
    let req = Request::builder()
        .method("POST")
        .uri("/filters/login_attempts/items")
        .body(Body::from("user_123"))
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. LOOKUP (Should be true)
    let req = Request::builder()
        .method("GET")
        .uri("/filters/login_attempts/items")
        .body(Body::from("user_123"))
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    let json = response_json(response).await;
    assert_eq!(json["contains"], true);

    // 6. CLEAR the filter
    let req = Request::builder()
        .method("PUT")
        .uri("/filters/login_attempts/clear")
        .body(Body::empty())
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 7. LOOKUP (Should be false again)
    let req = Request::builder()
        .method("GET")
        .uri("/filters/login_attempts/items")
        .body(Body::from("user_123"))
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    let json = response_json(response).await;
    assert_eq!(json["contains"], false);

    // 8. DELETE by ID
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/filters/{}", filter_id))
        .body(Body::empty())
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 9. VERIFY DELETION (List should be empty)
    let req = Request::builder()
        .method("GET")
        .uri("/filters")
        .body(Body::empty())
        .unwrap();

    let response = create_app(state.clone()).oneshot(req).await.unwrap();
    let json = response_json(response).await;
    let list = json.as_array().unwrap();
    assert_eq!(list.len(), 0);
}
