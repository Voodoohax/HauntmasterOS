use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{Multipart, Path}, response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use std::path::PathBuf;
use std::env;

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
    current_playback: Option<String>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState {
        media: vec![],
        current_playback: None,
    }));

    let app = Router::new()
        .route("/api/media", get(list_media))
        .route("/api/upload", post(upload_media))
        .route("/api/media/:id31", delete(delete_media))
        .with_state(state.clone());

    println!("🎃 HauntMaster API running on :3000");
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

        let path = format!("/media/{id}.{ext}");
        let thumb = format!("/thumbs/{id}.jpg");

        tokio::fs::write(format!("../../.{path}"), &data).await.unwrap();

        // Generate thumbnail
        if file_type != "audio" {
            let _ = Command::new("ffmpeg")
                .args(["-i", &format!("../../.{path}"), "-ss", "00:00:01", "-vframes", "1", &format!("../../.{thumb}")])
                .output();
        }

        state.media.push(MediaFile {
            id: id.clone(),
            name,
            path,
            thumb,
            file_type: file_type.to_string(),
        });
    }
    Json(state.media.clone())
}

async fn delete_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    state.media.retain(|f| f.id != id);
    let _ = tokio::fs::remove_file(format!("../../media/{id}.*"));
    let _ = tokio::fs::remove_file(format!("../../thumbs/{id}.jpg"));
    Json(state.media.clone())
}
