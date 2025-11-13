use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{Multipart, Path}, response::{IntoResponse, Response}, http::{header, StatusCode},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;
use tower_http::cors::CorsLayer;
use glob::glob;
use std::path::PathBuf;
use mime;

// ... (MediaFile, AppState same)

#[tokio::main]
async fn main() {
    let _ = std::fs::create_dir_all("../media");
    let _ = std::fs::create_dir_all("../thumbs");
    let _ = Command::new("chmod").args(["777", "../media"]).output();
    let _ = Command::new("chmod").args(["777", "../thumbs"]).output();

    let state = Arc::new(Mutex::new(AppState { media: vec![] }));

    let app = Router::new()
        .route("/media/:filename", get(serve_media))      // ← CUSTOM
        .route("/thumbs/:filename", get(serve_thumbs))    // ← CUSTOM
        .route("/api/media", get(list_media))
        .route("/api/upload", post(upload_media))
        .route("/api/play", post(play_media))
        .route("/api/media/:id", delete(delete_media))
        .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    println!("HauntMaster API running on :3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// CUSTOM MEDIA SERVER
async fn serve_media(Path(filename): Path<String>) -> impl IntoResponse {
    serve_file(&format!("../media/{}", filename)).await
}

// CUSTOM THUMBS SERVER
async fn serve_thumbs(Path(filename): Path<String>) -> impl IntoResponse {
    serve_file(&format!("../thumbs/{}", filename)).await
}

// UNIVERSAL FILE SERVER
async fn serve_file(path: &str) -> impl IntoResponse {
    match File::open(path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = axum::body::Body::from_stream(stream);
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.to_string())
                .body(body)
                .unwrap()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
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
        let name = match field.file_name() { Some(n) => n.to_string(), None => continue };
        let data = match field.bytes().await { Ok(d) => d, Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "File too large").into_response() };

        let id = Uuid::new_v4().to_string();
        let ext = name.split('.').last().unwrap_or("bin").to_lowercase();
        let file_type = if ["mp4", "webm", "mov", "avi", "mkv"].contains(&ext.as_str()) { "video" }
                      else if ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg"].contains(&ext.as_str()) { "image" }
                      else { "audio" };

        let path = format!("../media/{id}.{ext}");
        let thumb = format!("../thumbs/{id}.webp");

        let mut write_success = false;
        for _ in 0..3 {
            if tokio::fs::write(&path, &data).await.is_ok() {
                write_success = true;
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        if !write_success {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Write failed").into_response();
        }

        if file_type == "image" {
            let status = Command::new("cwebp")
                .args(["-q", "80", &path, "-o", &thumb])
                .status();
            if !status.map_or(false, |s| s.success()) {
                let _ = std::fs::copy(&path, &thumb);
            }
        } else if file_type == "video" {
            let mut success = false;
            for _ in 0..3 {
                let status = Command::new("ffmpeg")
                    .args([
                        "-i", &path,
                        "-ss", "00:00:01",
                        "-vframes", "1",
                        "-vf", "scale=400:-1",
                        "-f", "webm",
                        "-c:v", "libwebp",
                        "-q:v", "80",
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
        }

        let media_file = MediaFile {
            id: id.clone(),
            name,
            path: format!("/media/{id}.{ext}"),
            thumb: format!("/thumbs/{id}.webp"),
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
    let pattern = format!("../media/{id}.*");
    for path in glob(&pattern).unwrap().filter_map(|x| x.ok()) {
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_file(format!("../thumbs/{id}.webp"));
    Json(state.media.clone())
}

async fn play_media(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = payload["id"].as_str().unwrap_or("");  // ← FIXED: Added (
    let hdmi = payload["outputs"]["hdmi"].as_bool().unwrap_or(true);
    let audio = payload["outputs"]["audio"].as_bool().unwrap_or(true);

    let player = if std::path::Path::new("/usr/bin/omxplayer").exists() {
        "omxplayer"
    } else {
        "mpv"
    };

    if hdmi || audio {
        let pattern = format!("../media/{id}.*");
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
