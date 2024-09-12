use axum::{extract::Path, http::Method, routing::post, Router};
use tokio::fs::{create_dir_all, read_dir, remove_file, write};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer}};

#[axum::debug_handler]
async fn post_code(Path((id,)): Path<(String,)>, code: String) {
    write(&format!("incoming/{id}.txt"), code.as_bytes())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    create_dir_all("incoming").await.unwrap();
    let mut read_dir_iter = read_dir("incoming").await.unwrap();

    while let Some(entry) = read_dir_iter.next_entry().await.unwrap() {
        tokio::spawn(remove_file(entry.path()));
    }

    let app = Router::new().route("/code/:id", post(post_code)).layer(
        ServiceBuilder::new().layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::POST]),
        )
        .layer(
            CompressionLayer::new()
                .br(true)
        ),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
