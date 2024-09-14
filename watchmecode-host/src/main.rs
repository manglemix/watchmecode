#![feature(
    const_collections_with_hasher,
    build_hasher_default_const_new,
    try_blocks
)]
use chrono::{Local, Timelike};
use parking_lot::RwLock;
use tower::ServiceBuilder;
use notify::{event::{DataChange, ModifyKind}, Event, EventKind, Watcher};
use std::{io::{BufReader, Read, SeekFrom}, sync::atomic::{AtomicUsize, Ordering}};

use axum::{extract::{ws::Message, WebSocketUpgrade}, response::Response, routing::get, Router};

use tokio::{
    fs::{create_dir_all, read_dir, remove_file, File},
    io::{AsyncSeekExt, AsyncWriteExt, BufWriter}, sync::Notify,
};
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer}};
use clap::{Parser, Subcommand};
use ngrok::{config::TunnelBuilder, prelude::TunnelExt};
use ngrok::tunnel::UrlTunnel;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Ngrok,
}

static HOST_CODE: RwLock<String> = RwLock::new(String::new());
static HOST_CODE_CHANGED: Notify = Notify::const_new();

async fn code_ws(ws: WebSocketUpgrade) -> Response {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    ws.on_upgrade(|mut ws| async move {
        let mut name = COUNTER.fetch_add(1, Ordering::Relaxed).to_string();
        let mut filename = format!("incoming/{name}.txt");
        let mut file = BufWriter::new(File::create(&filename).await.expect("Failed to create file"));
        
        {
            let host_code = HOST_CODE.read().clone();
            ws.send(Message::Text(host_code)).await.expect("Failed to send message");
        }
        
        loop {
            tokio::select! {
                option = ws.recv() => {
                    if let Some(Ok(msg)) = option {
                        let Message::Text(msg) = msg else { continue; };
                        if msg.len() <= 6 {
                            continue;
                        }
                        
                        match &msg[0..6] {
                            "CODE: " => {
                                let code = &msg.as_bytes()[6..];
                                let datetime = Local::now();
                                let timestamp = format!(
                                    "{:0>2}:{:0>2}:{:0>2}",
                                    datetime.hour(),
                                    datetime.minute(),
                                    datetime.second(),
                                );
                                let result: std::io::Result<()> = try {
                                    const DIVIDER: &[u8] = b"\n-------------------\n";
                                    let len = timestamp.len() + DIVIDER.len() + code.len();
                                    file.seek(SeekFrom::Start(0)).await.expect("Failed to seek to start");
                                    file.get_mut().set_len(len as u64).await.expect("Failed to set length of file");
                                    file.write_all(timestamp.as_bytes()).await?;
                                    file.write_all(DIVIDER).await?;
                                    file.write_all(code).await?;
                                };
                                result.expect("Failed to write to file");
                                file.flush().await.expect("Failed to flush");
                            }
                            "NAME: " => {
                                drop(file);
                                let old_filename = filename;
                                name = msg;
                                name.drain(0..6);
                                filename = format!("incoming/{name}.txt");
                                let mut tmp_file = File::create(&filename).await.expect("Failed to create file");
                                tokio::io::copy(&mut File::open(&old_filename).await.expect("Failed to open file for copying"), &mut tmp_file).await.expect("Failed to copy file");
                                file = BufWriter::new(tmp_file);
                                file.flush().await.expect("Failed to flush file");
                                remove_file(&old_filename).await.expect("Failed to delete file");
                            }
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }
                _ = HOST_CODE_CHANGED.notified() => {
                    let host_code = HOST_CODE.read().clone();
                    ws.send(Message::Text(host_code)).await.expect("Failed to send message");
                }
            }
        }
    })
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    create_dir_all("incoming").await.unwrap();
    let mut read_dir_iter = read_dir("incoming").await.unwrap();

    while let Some(entry) = read_dir_iter.next_entry().await.unwrap() {
        tokio::spawn(remove_file(entry.path()));
    }
    
    std::fs::File::create("host_code.txt").expect("Failed to create host_code.txt");
    
    let mut watcher = notify::recommended_watcher(|res: notify::Result<Event>| {
        match res.expect("Failed to observe change in host_code.txt").kind {
            EventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                let mut host_code_file = BufReader::new(std::fs::File::open("host_code.txt").expect("Failed to open host_code.txt"));
                {
                    let mut writer = HOST_CODE.write();
                    writer.clear();
                    if host_code_file.read_to_string(&mut *writer).is_err() {
                        return;
                    }
                }
                HOST_CODE_CHANGED.notify_waiters();
            }
            _ => {}
        }
    }).expect("Failed to set up filesystem watcher");

    watcher.watch("host_code.txt".as_ref(), notify::RecursiveMode::Recursive).expect("Failed to watch host_code.txt");

    let app = Router::new()
        .route("/code/", get(code_ws))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new().br(true))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                )
        );
    
    match cli.command {
        Some(Commands::Ngrok) => {
            let mut tun = ngrok::Session::builder()
                // Read the token from the NGROK_AUTHTOKEN environment variable
                .authtoken_from_env()
                // Connect the ngrok session
                .connect()
                .await
                .expect("Failed to init ngrok")
                // Start a tunnel with an HTTP edge
                .http_endpoint()
                .compression()
                .listen()
                .await
                .expect("Failed to start tunnel");
        
            println!("Deployed on: https://manglemix.github.io/watchmecode/?host=wss://{}", &tun.url()[8..]);
            tokio::spawn(async move {
                tun.forward_tcp("127.0.0.1:80").await.expect("Failed to forward TCP");
            });
        }
        None => {
            println!("Deployed on: https://manglemix.github.io/watchmecode/?host=<insert host address here>");
            println!("Port 80 must be deployed manually");
        }
    }
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
