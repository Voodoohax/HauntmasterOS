use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{Multipart, Path, DefaultBodyLimit}, response::IntoResponse,
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
    let _ = std::fs::create_dir_all("../../media");
    let _ = std::fs::create_dir_all("../../thumbs");

    let state = Arc::new(Mutex::new(AppState { media: vec![] }));

    let app = Router::new()
        .route("/api/media", get(list_media))
        .route("/api/upload", post(upload_media))
        .route("/api/play", post(play_media))
        .route("/api/media/:id", delete(delete_media))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024))  // 2GB
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    println!("HauntMaster API running on :3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_media(axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>) -> Json<Vec<MediaFile>> {
    let state = state.lock().await;
    Json(state.media.clone())
}

async fn upload_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
    DefaultBodyLimit(_): DefaultBodyLimit,
    mut multipart: Multipart,
) -> Result<Json<Vec<MediaFile>>, (axum::http::StatusCode, String)> {
    let mut state = state.lock().await;
    while let Some(field) = multipart.next_field().await.map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))? {
        let name = field.file_name().ok_or((axum::http::StatusCode::BAD_REQUEST, "Missing filename".to_string()))?.to_string();
        let data = field.bytes().await.map_err(|e| (axum::http::StatusCode::PAYLOAD_TOO_LARGE, e.to_string()))?;
        
        let id = Uuid::new_v4().to_string();
        let ext = name.split('.').last().unwrap_or("bin").to_lowercase();
        let file_type = if ["mp4", "webm", "mov"].contains(&ext.as_str()) { "video" }
                      else if ["jpg", "jpeg", "png", "webp", "gif"].contains(&ext.as_str()) { "image" }
                      else { "audio" };

        let path = format!("../../media/{id}.{ext}");
        let thumb = format!("../../thumbs/{id}.jpg");

        tokio::fs::write(&path, &data).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // THUMBNAIL FOR VIDEO + IMAGE
        if file_type == "video" || file_type == "image" {
            let input = &path;
            let output = &thumb;
            let status = if file_type == "video" {
                Command::new("ffmpeg")
                    .args(["-i", input, "-ss", "00:00:01", "-vframes", "1", "-q:v", "2", "-y", output])
                    .status()
            } else {
                Command::new("ffmpeg")
                    .args(["-i", input, "-vf", "scale=400:-1", "-q:v", "2", "-y", output])
                    .status()
            }.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if !status.success() {
                // Fallback: copy original if thumbnail fails
                let _ = std::fs::copy(input, output);
            }
        }

        state.media.push(MediaFile {
            id: id.clone(),
            name,
            path: format!("/media/{id}.{ext}"),
            thumb: format!("/thumbs/{id}.jpg"),
            file_type: file_type.to_string(),
        });
    }
    Ok(Json(state.media.clone()))
}

// ... rest unchanged (delete_media, play_media)
