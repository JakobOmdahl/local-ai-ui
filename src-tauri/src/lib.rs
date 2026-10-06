// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod llm;
use llama_cpp_2::llama_backend::LlamaBackend;
use llm::LlmEngine;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};

pub struct AppState {
    pub backend: Arc<LlamaBackend>,
    pub llm: Arc<Mutex<Option<LlmEngine>>>,
    pub model_path: Arc<Mutex<Option<String>>>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    model_path: String,
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;

    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    Ok(dir.join("config.json"))
}

fn save_model_path(app: &AppHandle, model_path: &str) -> Result<(), String> {
    let config = Config {
        model_path: model_path.to_string(),
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

    fs::write(config_path(app)?, json).map_err(|e| e.to_string())?;

    Ok(())
}

fn load_config(app: &AppHandle) -> Result<Option<Config>, String> {
    let path = config_path(app)?;

    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let config = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    Ok(Some(config))
}

#[derive(Serialize)]
struct ModelStatus {
    loaded: bool,
    path: Option<String>,
    error: Option<String>,
}

#[tauri::command]
async fn initialize_model(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ModelStatus, String> {
    let config = match load_config(&app)? {
        Some(config) => config,

        None => {
            return Ok(ModelStatus {
                loaded: false,
                path: None,
                error: None,
            });
        }
    };

    {
        let current_path = state.model_path.lock().map_err(|e| e.to_string())?;

        if current_path.as_deref() == Some(&config.model_path) {
            return Ok(ModelStatus {
                loaded: true,
                path: Some(config.model_path),
                error: None,
            });
        }
    }

    let path = PathBuf::from(&config.model_path);

    if !path.is_file() {
        return Ok(ModelStatus {
            loaded: false,
            path: Some(config.model_path),
            error: Some("The saved model path does not exist.".to_string()),
        });
    }
    let backend = Arc::clone(&state.backend);
    let model_path = config.model_path.clone();

    let path_for_load = model_path.clone();

    let engine =
        tauri::async_runtime::spawn_blocking(move || LlmEngine::load(&backend, &path_for_load))
            .await
            .map_err(|e| e.to_string())??;

    let mut llm = state.llm.lock().map_err(|e| e.to_string())?;

    *llm = Some(engine);

    Ok(ModelStatus {
        loaded: true,
        path: Some(model_path),
        error: None,
    })
}

#[tauri::command]
async fn set_model_path(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    model_path: String,
) -> Result<ModelStatus, String> {
    let backend = Arc::clone(&state.backend);
    let llm = Arc::clone(&state.llm);

    let path = model_path.clone();

    let new_engine = tauri::async_runtime::spawn_blocking(move || LlmEngine::load(&backend, &path))
        .await
        .map_err(|e| e.to_string())??;
    {
        let mut current = llm.lock().map_err(|e| e.to_string())?;

        *current = Some(new_engine);
    }

    save_model_path(&app, &model_path)?;

    Ok(ModelStatus {
        loaded: true,
        path: Some(model_path),
        error: None,
    })
}

#[tauri::command]
async fn greet(state: tauri::State<'_, AppState>, prompt: String) -> Result<String, String> {
    let llm = Arc::clone(&state.llm);
    let backend = Arc::clone(&state.backend);
    tauri::async_runtime::spawn_blocking(move || {
        let mut llm = llm.lock().map_err(|e| e.to_string())?;
        let engine = llm
            .as_mut()
            .ok_or_else(|| "No model is currently loaded.".to_string())?;
        engine.generate(&backend, &prompt)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let backend = Arc::new(LlamaBackend::init().expect("Failed to initialize llama.cpp backend"));

    tauri::Builder::default()
        .manage(AppState {
            backend,
            llm: Arc::new(Mutex::new(None)),
            model_path: Arc::new(Mutex::new(None)),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            initialize_model,
            set_model_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
