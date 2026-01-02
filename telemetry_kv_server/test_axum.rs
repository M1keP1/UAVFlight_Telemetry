use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9091")
        .await
        .unwrap();
    
    println!("Test server on http://localhost:9091");
    
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
