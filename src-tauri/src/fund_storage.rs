use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

pub fn storage_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {}", e))?;
    Ok(dir)
}

pub fn save_json<T: Serialize>(app: &AppHandle, filename: &str, data: &T) -> Result<(), String> {
    let path = storage_dir(app)?.join(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}

pub fn load_json<T: DeserializeOwned>(
    app: &AppHandle,
    filename: &str,
) -> Result<Option<T>, String> {
    let path = storage_dir(app)?.join(filename);
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    serde_json::from_str(&json)
        .map(Some)
        .map_err(|e| format!("反序列化失败: {}", e))
}

pub fn list_history_cache(app: &AppHandle) -> Result<Vec<String>, String> {
    let dir = storage_dir(app)?.join("history");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut codes = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        if let Some(name) = entry.file_name().to_str() {
            if let Some(code) = name.strip_suffix(".json") {
                codes.push(code.to_string());
            }
        }
    }
    Ok(codes)
}
