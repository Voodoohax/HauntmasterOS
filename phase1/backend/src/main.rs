use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{Multipart, Path}, response::IntoResponse,
    http::header,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tower_http::cors::{CorsLayer, Any};
use glob::glob;

#[derive(Serialize, Deserialize, Clone)]
struct MediaFile {
    id: String,
    name: String,
    path: String,
    thumb: String,
    file_type: String,
}

struct AppState {
    media: Vec<MediaFile>,
}

#[tokio::main]
async fn main() {
    // Create media dirs
    let _ = std::fs::create_dir_all("../../media");
    let _ = std::fs::create_dir_all("../../thumbs");

    let state = Arc::new(Mutex::new(AppState { media: vec![] }));

    let app = Router::new()
        .route("/api/media", get(list_media))
        .route("/api/upload", post(upload_media))
        .route("/api/media/:id", delete(delete_media))
        .route("/api/play", post(play_media))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    println!("HauntMaster API running on :3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// LIST
async fn list_media(axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>) -> Json<Vec<MediaFile>> {
    let state = state.lock().await;
    Json(state.media.clone())
}

// UPLOAD
async fn upload_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let id = Uuid::new_v4().to_string();

        let ext = name.split('.').last().unwrap_or("bin").to_lowercase();
        let file_type = if ["mp4", "webm"].contains(&ext.as_str()) { "video" }
                      else if ["jpg", "png", "webp"].contains(&ext.as_str()) { "image" }
                      else { "audio" };

        let path = format!("../../media/{id}.{ext}");
        let thumb = format!("../../thumbs/{id}.jpg");

        tokio::fs::write(&path, &data).await.unwrap();

        // Thumbnail
        if file_type != "audio" {
            let _ = Command::new("ffmpeg")
                .args(["-i", &path, "-ss", "00:00:01", "-vframes", "1", "-y", &thumb])
                .output();
        }

        state.media.push(MediaFile {
            id: id.clone(),
            name,
            path: format!("/media/{id}.{ext}"),
            thumb: format!("/thumbs/{id}.jpg"),
            file_type: file_type.to_string(),
        });
    }
    Json(state.media.clone())
}

// DELETE
async fn delete_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    state.media.retain(|f| f.id != id);
    let _ = std::fs::remove_file(format!("../../media/{id}.*"));
    let _ = std::fs::remove_file(format!("../../thumbs/{id}.jpg"));
    Json(state.media.clone())
}

// PLAY
async fn play_media(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = payload["id"].as_str().unwrap();
    let hdmi = payload["outputs"]["hdmi"].as_bool().unwrap_or(true);
    let audio = payload["outputs"]["audio"].as_bool().unwrap_or(true);

    let player = if std::path::Path::new("/usr/bin/omxplayer").exists() {
        "omxplayer"
    } else {
        "mpv"
    };

    if hdmi || audio {
        let pattern = format!("../../media/{id}.*");
        if let Ok(mut paths) = glob(&pattern) {
            if let Some(Ok(path)) = paths.next() {
                let path_str = path.to_str().unwrap();
                let mut cmd = Command::new(player);
                if player == "omxplayer" {
                    cmd.args(["--no-osd", path_str]);
                } else {
                    cmd.args(["--no-osd", "--vo=gpu", path_str]);
                }
                let _ = cmd.spawn();
            }
        }
    }

    Json(serde_json::json!({"status": "playing"}))
}
