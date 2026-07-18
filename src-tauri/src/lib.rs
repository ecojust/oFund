mod fund_crawler;
mod fund_storage;
mod workspace;

use fund_crawler::{fetch_company_list, fetch_fund_list, CrawlProgress, FundItemWithCompany};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AccountItem {
    code: String,
    name: String,
    schedule: HashMap<String, f64>,
}

pub struct AppState {
    pub opencode_child: Arc<Mutex<Option<tauri_plugin_shell::process::CommandChild>>>,
    pub oc_port: Arc<Mutex<Option<u16>>>,
}

use workspace::{execute_opencode_serve, kill_existing_opencode_processes};

#[tauri::command]
async fn get_company_list() -> Result<Vec<fund_crawler::Company>, String> {
    fetch_company_list().await
}

#[tauri::command]
async fn get_fund_list(company_id: String) -> Result<Vec<fund_crawler::FundItem>, String> {
    fetch_fund_list(&company_id).await
}

#[tauri::command]
async fn fetch_all_funds(
    app: tauri::AppHandle,
) -> Result<Vec<fund_crawler::FundItemWithCompany>, String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<CrawlProgress>();

    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = app_clone.emit("crawl-progress", &progress);
        }
    });

    let all_funds = fund_crawler::get_all_funds(Some(tx)).await?;

    fund_storage::save_json(&app, "all_funds.json", &all_funds)?;

    Ok(all_funds)
}

#[tauri::command]
fn get_cached_history_codes(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    fund_storage::list_history_cache(&app)
}

#[tauri::command]
fn open_history_dir(app: tauri::AppHandle) -> Result<(), String> {
    let dir = fund_storage::storage_dir(&app)?;
    let history_dir = dir.join("history");
    if !history_dir.exists() {
        std::fs::create_dir_all(&history_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    app.opener()
        .open_path(history_dir.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| format!("打开目录失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn load_cached_funds(
    app: tauri::AppHandle,
) -> Result<Vec<fund_crawler::FundItemWithCompany>, String> {
    Ok(
        fund_storage::load_json::<Vec<fund_crawler::FundItemWithCompany>>(&app, "all_funds.json")?
            .unwrap_or_default(),
    )
}

#[derive(Clone, Serialize)]
struct HistoryCrawlProgress {
    current: usize,
    total: usize,
    fund_code: String,
    fund_name: String,
    status: String,
}

#[tauri::command]
async fn fetch_all_history(
    app: tauri::AppHandle,
    period: Option<fund_crawler::HistoryPeriod>,
) -> Result<(), String> {
    let period = period.unwrap_or(fund_crawler::HistoryPeriod::SinceInception);
    let funds: Vec<FundItemWithCompany> =
        fund_storage::load_json(&app, "all_funds.json")?.ok_or("请先获取基金列表")?;

    let total = funds.len();
    let concurrency = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for fund in funds.into_iter() {
        let sem = semaphore.clone();
        let app_clone = app.clone();
        let counter = counter.clone();
        let period = period.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let code = fund.id.clone();
            let name = fund.name.clone();

            let filename = format!("history/{}.json", code);

            let result = fund_crawler::fetch_fund_history(&code, &period)
                    .await
                    .and_then(|h| {
                        fund_storage::save_json(&app_clone, &filename, &h)?;
                        Ok(())
                    });

            let current = counter.fetch_add(1, Ordering::Relaxed) + 1;

            let status = match &result {
                Ok(_) => "已完成".to_string(),
                Err(e) => format!("失败: {}", e),
            };

            let _ = app_clone.emit(
                "history-crawl-progress",
                &HistoryCrawlProgress {
                    current,
                    total,
                    fund_code: code,
                    fund_name: name,
                    status,
                },
            );
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

#[derive(Serialize)]
struct AlertFundInfo {
    code: String,
    trend: Vec<f64>,
    up_days: usize,
    up_gain: f64,
    total_gain: f64,
}

#[derive(Clone, Serialize)]
struct AlertProgress {
    current: usize,
    total: usize,
    fund_code: String,
    matched: usize,
}

#[tauri::command]
fn get_alert_funds(
    app: tauri::AppHandle,
    days: Option<usize>,
    threshold: Option<f64>,
) -> Result<Vec<AlertFundInfo>, String> {
    let days = days.unwrap_or(3).max(1);
    let threshold = threshold.unwrap_or(10.0);
    let dir = fund_storage::storage_dir(&app)?.join("history");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    let total = entries.len();
    let mut alert_funds = Vec::new();
    for (idx, path) in entries.into_iter().enumerate() {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let history: fund_crawler::FundHistory = match serde_json::from_str(&content) {
            Ok(h) => h,
            Err(_) => {
                let _ = app.emit(
                    "alert-progress",
                    AlertProgress {
                        current: idx + 1,
                        total,
                        fund_code: String::new(),
                        matched: alert_funds.len(),
                    },
                );
                continue;
            }
        };
        let mut data = history.data;
        data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        if data.len() >= days + 1 {
            let mut total_gain = 0.0f64;
            for i in (data.len() - days)..data.len() {
                total_gain += (data[i].value - data[i - 1].value)
                    / (100.0 + data[i - 1].value)
                    * 100.0;
            }
            if total_gain > threshold {
                let trend: Vec<f64> =
                    data.iter().rev().take(10).rev().map(|p| p.value).collect();
                let mut up_days = 0usize;
                let mut up_gain = 0.0f64;
                for i in (1..data.len()).rev() {
                    let gain = (data[i].value - data[i - 1].value)
                        / (100.0 + data[i - 1].value)
                        * 100.0;
                    if gain > 0.0 {
                        up_days += 1;
                        up_gain += gain;
                    } else {
                        break;
                    }
                }
                alert_funds.push(AlertFundInfo {
                    code: history.fund_code.clone(),
                    trend,
                    up_days,
                    up_gain,
                    total_gain,
                });
            }
        }
        let _ = app.emit(
            "alert-progress",
            AlertProgress {
                current: idx + 1,
                total,
                fund_code: history.fund_code,
                matched: alert_funds.len(),
            },
        );
    }
    Ok(alert_funds)
}

#[tauri::command]
fn get_favorites(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    Ok(
        fund_storage::load_json::<Vec<String>>(&app, "favorites.json")?
            .unwrap_or_default(),
    )
}

#[tauri::command]
fn toggle_favorite(app: tauri::AppHandle, fund_code: String) -> Result<Vec<String>, String> {
    let mut favorites = fund_storage::load_json::<Vec<String>>(&app, "favorites.json")?
        .unwrap_or_default();

    if let Some(pos) = favorites.iter().position(|c| c == &fund_code) {
        favorites.remove(pos);
    } else {
        favorites.push(fund_code);
    }

    fund_storage::save_json(&app, "favorites.json", &favorites)?;

    Ok(favorites)
}

#[tauri::command]
fn get_account(app: tauri::AppHandle) -> Result<Vec<AccountItem>, String> {
    Ok(
        fund_storage::load_json::<Vec<AccountItem>>(&app, "account.json")?
            .unwrap_or_default(),
    )
}

#[tauri::command]
fn save_account(app: tauri::AppHandle, data: Vec<AccountItem>) -> Result<(), String> {
    fund_storage::save_json(&app, "account.json", &data)
}

#[tauri::command]
async fn get_fund_history(
    _app: tauri::AppHandle,
    fund_code: String,
    period: Option<fund_crawler::HistoryPeriod>,
) -> Result<fund_crawler::FundHistory, String> {
    let period = period.unwrap_or(fund_crawler::HistoryPeriod::SinceInception);
    fund_crawler::fetch_fund_history(&fund_code, &period).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            opencode_child: Arc::new(Mutex::new(None)),
            oc_port: Arc::new(Mutex::new(None)),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            get_company_list,
            get_fund_list,
            fetch_all_funds,
            load_cached_funds,
            get_cached_history_codes,
            open_history_dir,
            get_fund_history,
            fetch_all_history,
            get_alert_funds,
            get_favorites,
            toggle_favorite,
            get_account,
            save_account,
            execute_opencode_serve,
            kill_existing_opencode_processes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
