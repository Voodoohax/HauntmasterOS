use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{Multipart, Path, DefaultBodyLimit}, response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
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
        .nest_service("/media", ServeDir::new("../../media"))
        .nest_service("/thumbs", ServeDir::new("../../thumbs"))
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

async fn list_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
) -> Json<Vec<MediaFile>> {
    let state = state.lock().await;
    Json(state.media.clone())
}

async fn upload_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    let mut new_media = vec![];

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = match field.file_name() {
            Some(n) => n.to_string(),
            None => continue,
        };
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "File too large".to_string()).into_response(),
        };

        let id = Uuid::new_v4().to_string();
        let ext = name.split('.').last().unwrap_or("bin").to_lowercase();
        let file_type = if ["mp4", "webm", "mov", "avi", "mkv"].contains(&ext.as_str()) { "video" }
                      else if ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg"].contains(&ext.as_str()) { "image" }
                      else { "audio" };

        let path = format!("../../media/{id}.{ext}");
        let thumb = format!("../../thumbs/{id}.jpg");

        if tokio::fs::write(&path, &data).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Write failed").into_response();
        }

                        // THUMBNAIL: VIDEO = FFmpeg, IMAGE = cwebp → NO WARNINGS
        if file_type == "video" {
            // Video: FFmpeg (warning harmless, thumbnail works)
            let mut success = false;
            for _ in 0..3 {
                let status = Command::new("ffmpeg")
                    .args([
                        "-i", &path,
                        "-ss", "00:00:01",
                        "-vframes", "1",
                        "-vf", "scale=400:-1",
                        "-q:v", "2",
                        "-y", &thumb,
                    ])
                    .status();

                if status.map_or(false, |s| s.success()) {
                    success = true;
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            if !success {
                let _ = std::fs::copy(&path, &thumb);
            }
        } else if file_type == "image" {
            // Image: Use cwebp → WebP → convert to JPG
            let webp_temp = format!("{}.webp", thumb.strip_suffix(".jpg").unwrap());
            let status1 = Command::new("cwebp")
                .args(["-q", "80", &path, "-o", &webp_temp])
                .status();

            if status1.map_or(false, |s| s.success()) {
                let status2 = Command::new("ffmpeg")
                    .args(["-i", &webp_temp, "-y", &thumb])
                    .status();
                let _ = std::fs::remove_file(&webp_temp);
                if !status2.map_or(false, |s| s.success()) {
                    let _ = std::fs::copy(&path, &thumb);
                }
            } else {
                let _ = std::fs::copy(&path, &thumb);
            }
        }
        
        let media_file = MediaFile {
            id: id.clone(),
            name,
            path: format!("/media/{id}.{ext}"),
            thumb: format!("/thumbs/{id}.jpg"),
            file_type: file_type.to_string(),
        };
        new_media.push(media_file.clone());
        state.media.push(media_file);
    }

    Json(state.media.clone()).into_response()
}

async fn delete_media(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    state.media.retain(|f| f.id != id);
    let pattern = format!("../../media/{id}.*");
    for path in glob(&pattern).unwrap().filter_map(|x| x.ok()) {
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_file(format!("../../thumbs/{id}.jpg"));
    Json(state.media.clone())
}

async fn play_media(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = payload["id"].as_str().unwrap_or("");
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

    Json(serde_json::json!({"status": "playing"})).into_response()
}
