use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{Multipart, Path}, response::IntoResponse,
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
    let _ = std::fs::create_dir_all("../media");
    let _ = std::fs::create_dir_all("../thumbs");

    let _ = Command::new("chmod").args(["777", "../media"]).output();
    let _ = Command::new("chmod").args(["777", "../thumbs"]).output();

    let state = Arc::new(Mutex::new(AppState { media: vec![] }));

    let app = Router::new()
        .nest_service("/media", ServeDir::new("../media"))
        .nest_service("/thumbs", ServeDir::new("../thumbs"))
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
    async fn play_media(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // FIX: Clone the default map
    let scene_obj = payload["scene"]
        .as_object()
        .unwrap_or(&serde_json::Map::new())
        .clone();

    // FIX: Clone the default vec
    let layers = scene_obj
        .get("layers")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .to_vec();

    if layers.is_empty() {
        return (StatusCode::BAD_REQUEST, "No layers").into_response();
    }

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");

    let mut inputs = vec![];
    let mut filters = vec!["[0:v]null[v0]".to_string()];
    let mut overlay = "[v0]".to_string();

    for (i, layer) in layers.iter().enumerate() {
        let path = layer["path"].as_str().unwrap_or("");
        let x = layer["x"].as_f64().unwrap_or(0.0);
        let y = layer["y"].as_f64().unwrap_or(0.0);
        let w = layer["width"].as_f64().unwrap_or(400.0);
        let h = layer["height"].as_f64().unwrap_or(300.0);
        let opacity = layer["opacity"].as_f64().unwrap_or(1.0);
        let crop = layer.get("crop").and_then(|c| c.as_object());

        let input_path = format!("../media/{}", path.trim_start_matches("/media/"));
        inputs.push(format!("-i {}", input_path));

        let mut filter = format!("[{}:v]", i + 1);

        if let Some(c) = crop {
            let cx = c["x"].as_f64().unwrap_or(0.0);
            let cy = c["y"].as_f64().unwrap_or(0.0);
            let cw = c["width"].as_f64().unwrap_or(w);
            let ch = c["height"].as_f64().unwrap_or(h);
            filter = format!("{}crop={}:{}:{}:{}", filter, cw, ch, cx, cy);
        }

        filter = format!(
            "{}scale={}:{},setsar=1,format=rgba,colorchannelmixer=aa={}[l{}]; \
             {}pad=1920:1080:-1:-1:color=#00000000[p{}]; \
             [p{}][l{}]overlay=x={}:y={}[out{}]",
            filter, w, h, opacity, i,
            overlay, i,
            i, i, x, y, i
        );

        filters.push(filter);
        overlay = format!("[out{}]", i);
    }

    let filter_complex = filters.join(";");
    cmd.args(&inputs);
    cmd.args([
        "-filter_complex", &filter_complex,
        "-map", &format!("{}:v", overlay),
        "-f", "matroska",
        "-c:v", "libx264",
        "-preset", "ultrafast",
        "-tune", "zerolatency",
        "pipe:1"
    ]);

    let ffmpeg = cmd.stdout(std::process::Stdio::piped()).spawn().unwrap();
    let vlc = Command::new("cvlc")
        .args([
            "-", "--fullscreen", "--no-osd", "--play-and-exit",
            "--avcodec-hw=any", "--quiet"
        ])
        .stdin(ffmpeg.stdout.unwrap())
        .spawn();

    if vlc.is_ok() {
        Json(serde_json::json!({"status": "haunting"})).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "VLC failed").into_response()
    }
})
