#![feature(
    const_collections_with_hasher,
    build_hasher_default_const_new,
    try_blocks
)]
use chrono::{Local, Timelike};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

use axum::{extract::Path, routing::post, Router};
use fxhash::{FxBuildHasher, FxHashMap};

use tokio::{
    fs::{create_dir_all, read_dir, remove_file},
    io::{AsyncWriteExt, BufWriter},
};
use tower_http::compression::CompressionLayer;

struct UserInfo {
    name: Arc<str>,
}

static USERS: RwLock<FxHashMap<String, UserInfo>> =
    RwLock::new(HashMap::with_hasher(FxBuildHasher::new()));

async fn post_code(id: String, name: Option<String>, code: String) {
    let owned_name: Arc<str>;
    let name = if let Some(name) = name {
        'result: {
            if let Some(info) = USERS.read().get(&id) {
                if &*info.name == &name {
                    owned_name = info.name.clone();
                    break 'result &*owned_name;
                }
            }
            let mut writer = USERS.write();
            owned_name = Arc::from(name.into_boxed_str());
            writer.insert(
                id.to_owned(),
                UserInfo {
                    name: owned_name.clone(),
                },
            );
            &*owned_name
        }
    } else {
        if let Some(info) = USERS.read().get(&id) {
            owned_name = info.name.clone();
            &*owned_name
        } else {
            ""
        }
    };

    let mut file = BufWriter::new(
        tokio::fs::File::create(format!("incoming/{id}.txt"))
            .await
            .expect("Failed to open file"),
    );

    let result: std::io::Result<()> = try {
        if !name.is_empty() {
            file.write_all(b"Name: ").await?;
            file.write_all(name.as_bytes()).await?;
            file.write_all(b" | ").await?;
        }
        let datetime = Local::now();
        let timestamp = format!(
            "{:0>2}:{:0>2}:{:0>2}",
            datetime.hour(),
            datetime.minute(),
            datetime.second(),
        );
        file.write_all(timestamp.as_bytes()).await?;
        file.write_all(b"\n-------------------\n").await?;
        file.write_all(code.as_bytes()).await?;
    };
    result.expect("Failed to write to file");
    file.flush().await.expect("Failed to flush");
}

#[axum::debug_handler]
async fn post_code_with_name(Path((id, name)): Path<(String, String)>, code: String) {
    post_code(id, Some(name), code).await
}

#[axum::debug_handler]
async fn post_code_without_name(Path((id,)): Path<(String,)>, code: String) {
    post_code(id, None, code).await
}

#[tokio::main]
async fn main() {
    create_dir_all("incoming").await.unwrap();
    let mut read_dir_iter = read_dir("incoming").await.unwrap();

    while let Some(entry) = read_dir_iter.next_entry().await.unwrap() {
        tokio::spawn(remove_file(entry.path()));
    }

    let app = Router::new()
        .route("/code/:id/:name/", post(post_code_with_name))
        .route("/code/:id/", post(post_code_without_name))
        .layer(CompressionLayer::new().br(true));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
