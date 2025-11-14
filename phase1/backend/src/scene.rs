use axum::{
    routing::post,
    Router, Json, response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct SceneLayer {
    name: String,
    path: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    opacity: f32,
}

#[derive(Serialize, Deserialize)]
struct Scene {
    name: String,
    layers: Vec<SceneLayer>,
}

// PLAY SCENE
async fn play_scene(Json(scene): Json<Scene>) -> impl IntoResponse {
    // Build FFmpeg overlay command
    let mut ffmpeg_args = vec![
        "-f", "lavfi", "-i", "color=s=1920x1080:d=10:c=black",  // Black background
        "-y", "output.mp4"
    ];

    let mut overlay_index = 0;
    for (i, layer) in scene.layers.iter().enumerate() {
        let input = if layer.path.starts_with("/media/") {
            layer.path.strip_prefix("/media/").unwrap().to_string()
        } else {
            layer.path.clone()
        };
        
        ffmpeg_args.extend([
            "-i", &format!("../{}", input),
            "-filter_complex", &format!(
                "[{}:v]scale={}:{},x={},y={},enable='between(t,{},{})'[ov{}]; \
                [{}][ov{}]overlay=shortest=1[out{}]",
                i, layer.width, layer.height, layer.x, layer.y, 
                overlay_index as f32, (overlay_index + 1.0) as f32,
                0, overlay_index, i + 1
            ),
        ]);
        overlay_index += 1;
    }

    // Execute
    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status();

    if status.map_or(false, |s| s.success()) {
        Json(serde_json::json!({"status": "playing", "file": "output.mp4"}))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Scene render failed").into_response()
    }
}

// SAVE SCENE
async fn save_scene(Json(scene): Json<Scene>) -> impl IntoResponse {
    // TODO: Save to database
    Json(serde_json::json!({"status": "saved", "id": "scene-123"}))
}
