use crate::AppState;
use std::net::TcpListener;
use std::time::Duration;
use std::{fs, path::PathBuf};
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

fn opencode_serve_args(port: u16, dev_url: &str) -> Vec<String> {
    let mut args = vec![
        "serve".to_string(),
        "--port".to_string(),
        port.to_string(),
        "--print-logs".to_string(),
    ];
    for origin in [
        dev_url,
        "http://127.0.0.1:1520",
        "http://localhost:1520",
        "tauri://localhost",
        "http://tauri.localhost",
        "https://tauri.localhost",
    ] {
        args.push("--cors".to_string());
        args.push(origin.to_string());
    }
    args
}

#[tauri::command]
pub async fn execute_opencode_serve(
    workspace: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    {
        let guard = state.oc_port.lock().map_err(|e| e.to_string())?;
        if let Some(port) = *guard {
            return Ok(format!("http://127.0.0.1:{port}"));
        }
    }

    let port = {
        let listener =
            TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("端口分配失败：{e}"))?;
        listener.local_addr().map_err(|e| e.to_string())?.port()
    };

    let base_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("oFund")
        .join("workspaces");
    let target_workspace = base_dir.join(&workspace);
    fs::create_dir_all(&target_workspace)
        .map_err(|e| format!("创建工作区目录失败：{e}"))?;

    let config_path = target_workspace.join("opencode.json");
    let config_content =
        r#"{"$schema":"https://opencode.ai/config.json","permission":"allow"}"#;
    fs::write(&config_path, config_content)
        .map_err(|e| format!("写入 opencode.json 失败：{e}"))?;

    let dev_url = format!("http://127.0.0.1:{}", port);
    let cmd = app
        .shell()
        .sidecar("opencode")
        .map_err(|e| format!("sidecar 加载失败：{e}"))?
        .args(opencode_serve_args(port, &dev_url))
        .current_dir(&target_workspace);

    let (mut rx, child) = cmd.spawn().map_err(|e| format!("启动失败：{e}"))?;

    if let Ok(mut guard) = state.opencode_child.lock() {
        *guard = Some(child);
    }
    if let Ok(mut guard) = state.oc_port.lock() {
        *guard = Some(port);
    }

    let url = format!("http://127.0.0.1:{port}");

    // Wait for server to be ready
    let mut ready = false;
    for _ in 0..30 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    if !ready {
        if let Ok(mut guard) = state.opencode_child.lock() {
            if let Some(child) = guard.take() {
                let _ = child.kill();
            }
        }
        if let Ok(mut guard) = state.oc_port.lock() {
            *guard = None;
        }
        return Err("opencode 服务启动超时".into());
    }

    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("oFund");
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("opencode-server.log");

    tauri::async_runtime::spawn(async move {
        use std::io::Write;
        let mut log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        loop {
            match rx.recv().await {
                Some(CommandEvent::Stdout(bytes)) => {
                    if let Some(ref mut f) = log_file {
                        let _ = writeln!(f, "[serve] {}", String::from_utf8_lossy(&bytes));
                    }
                }
                Some(CommandEvent::Stderr(bytes)) => {
                    if let Some(ref mut f) = log_file {
                        let _ = writeln!(f, "[serve:err] {}", String::from_utf8_lossy(&bytes));
                    }
                }
                Some(CommandEvent::Terminated(_)) | Some(CommandEvent::Error(_)) | None => break,
                _ => {}
            }
        }
        if let Ok(mut guard) = app.state::<AppState>().opencode_child.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = app.state::<AppState>().oc_port.lock() {
            *guard = None;
        }
    });

    Ok(url)
}

#[tauri::command]
pub async fn kill_existing_opencode_processes(app: tauri::AppHandle) -> Result<(), String> {
    if let Ok(mut guard) = app.state::<AppState>().opencode_child.lock() {
        if let Some(child) = guard.take() {
            let _ = child.kill();
        }
    }
    if let Ok(mut guard) = app.state::<AppState>().oc_port.lock() {
        *guard = None;
    }
    Ok(())
}
