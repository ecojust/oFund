use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub creat_time: String,
    pub level: u8,
    pub fund_count: u32,
    pub manager_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FundItem {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FundItemWithCompany {
    pub id: String,
    pub name: String,
    pub company_id: String,
    pub company_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FundHistoryPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FundHistory {
    pub fund_code: String,
    pub fund_name: String,
    pub data: Vec<FundHistoryPoint>,
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("创建 HTTP 客户端失败")
}

/// 获取所有基金公司列表
pub async fn fetch_company_list() -> Result<Vec<Company>, String> {
    let url = "http://fund.eastmoney.com/company/default.html";
    let html = http_client()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求公司列表失败: {}", e))?
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let document = Html::parse_document(&html);
    let row_selector =
        Selector::parse("#gspmTbl tbody tr").map_err(|e| format!("选择器错误: {}", e))?;
    let td_selector = Selector::parse("td").map_err(|e| format!("选择器错误: {}", e))?;
    let a_selector = Selector::parse("a").map_err(|e| format!("选择器错误: {}", e))?;
    let star_selector =
        Selector::parse(".sprite-star3").map_err(|e| format!("选择器错误: {}", e))?;
    let id_re = Regex::new(r"[1-9][0-9]*").map_err(|e| format!("正则错误: {}", e))?;

    let mut companies = Vec::new();

    for row in document.select(&row_selector) {
        let a_elem = match row.select(&a_selector).next() {
            Some(a) => a,
            None => continue,
        };
        let href = match a_elem.value().attr("href") {
            Some(h) => h,
            None => continue,
        };
        let name = a_elem.text().collect::<String>().trim().to_string();
        let id = match id_re.find(href) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };

        let tds: Vec<_> = row.select(&td_selector).collect();
        if tds.len() < 8 {
            continue;
        }

        let creat_time = tds[3].text().collect::<String>().trim().to_string();
        let star_count = row.select(&star_selector).count() as u8;
        let level = 5u8.saturating_sub(star_count);
        let fund_count = tds[6]
            .text()
            .collect::<String>()
            .trim()
            .parse::<u32>()
            .unwrap_or(0);
        let manager_count = tds[7]
            .text()
            .collect::<String>()
            .trim()
            .parse::<u32>()
            .unwrap_or(0);

        companies.push(Company {
            id,
            name,
            creat_time,
            level,
            fund_count,
            manager_count,
        });
    }

    companies.sort_by(|a, b| b.level.cmp(&a.level));
    Ok(companies)
}

/// 获取指定基金公司下的所有基金
pub async fn fetch_fund_list(company_id: &str) -> Result<Vec<FundItem>, String> {
    let url = format!(
        "http://fund.eastmoney.com/Company/f10/jjjz_{}.html",
        company_id
    );
    let html = http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求基金列表失败: {}", e))?
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let document = Html::parse_document(&html);
    let td_selector = Selector::parse("td.fund-name-code")
        .map_err(|e| format!("选择器错误: {}", e))?;
    let a_selector = Selector::parse("a").map_err(|e| format!("选择器错误: {}", e))?;

    let mut funds = Vec::new();

    for td in document.select(&td_selector) {
        let links: Vec<_> = td.select(&a_selector).collect();
        if links.len() < 2 {
            continue;
        }
        let name = links[0].text().collect::<String>().trim().to_string();
        let id = links[1].text().collect::<String>().trim().to_string();
        if name.is_empty() || id.is_empty() {
            continue;
        }
        funds.push(FundItem { id, name });
    }

    Ok(funds)
}

/// 获取所有基金（遍历所有公司，带并发控制）
pub async fn get_all_funds(
    progress: Option<tokio::sync::mpsc::UnboundedSender<CrawlProgress>>,
) -> Result<Vec<FundItemWithCompany>, String> {
    let companies = fetch_company_list().await?;
    let total = companies.len();
    let concurrency = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for company in companies.into_iter() {
        let company_id = company.id.clone();
        let company_name = company.name.clone();
        let sem = semaphore.clone();
        let tx = progress.clone();
        let counter = counter.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let result = fetch_fund_list(&company_id).await;
            drop(_permit);

            let current = counter.fetch_add(1, Ordering::Relaxed) + 1;

            if let Some(ref tx) = tx {
                let _ = tx.send(CrawlProgress {
                    current,
                    total,
                    company_name: company_name.clone(),
                    status: match &result {
                        Ok(f) => format!("{} 只基金", f.len()),
                        Err(e) => format!("失败: {}", e),
                    },
                });
            }

            match result {
                Ok(funds) => funds
                    .into_iter()
                    .map(|f| FundItemWithCompany {
                        id: f.id,
                        name: f.name,
                        company_id: company_id.clone(),
                        company_name: company_name.clone(),
                    })
                    .collect::<Vec<_>>(),
                Err(_) => Vec::new(),
            }
        }));
    }

    let mut all_funds = Vec::new();
    for handle in handles {
        if let Ok(funds) = handle.await {
            all_funds.extend(funds);
        }
    }

    Ok(all_funds)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrawlProgress {
    pub current: usize,
    pub total: usize,
    pub company_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryPeriod {
    #[serde(rename = "1m")]
    OneMonth,
    #[serde(rename = "3m")]
    ThreeMonths,
    #[serde(rename = "6m")]
    SixMonths,
    #[serde(rename = "1y")]
    OneYear,
    #[serde(rename = "all")]
    SinceInception,
}

impl HistoryPeriod {
    fn api_type(&self) -> &str {
        match self {
            HistoryPeriod::OneMonth => "m",
            HistoryPeriod::ThreeMonths => "q",
            HistoryPeriod::SixMonths => "hy",
            HistoryPeriod::OneYear => "y",
            HistoryPeriod::SinceInception => "se",
        }
    }
}

/// 获取单只基金的历史净值数据
/// 使用 eastmoney 的 LJSYLZS（累计收益率指数）API
pub async fn fetch_fund_history(fund_code: &str, period: &HistoryPeriod) -> Result<FundHistory, String> {
    let client = http_client();

    let url = format!(
        "https://api.fund.eastmoney.com/pinzhong/LJSYLZS?fundCode={}&indexcode=000300&type={}",
        fund_code, period.api_type()
    );

    let resp = client
        .get(&url)
        .header("Referer", format!("https://fund.eastmoney.com/{}.html", fund_code))
        .send()
        .await
        .map_err(|e| format!("请求历史数据失败: {}", e))?
        .json::<LJSYLZSResponse>()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if resp.ErrCode != 0 {
        return Err(format!("API 错误: {}", resp.ErrMsg.unwrap_or_default()));
    }

    let fund_data = resp.Data.first().ok_or("无累计收益率数据")?;
    let fund_name = fund_data.name.clone();
    let points = &fund_data.data;

    if points.is_empty() {
        return Err("无累计收益率记录".to_string());
    }

    let data: Vec<FundHistoryPoint> = points
        .iter()
        .map(|p| FundHistoryPoint {
            timestamp: p[0] as i64,
            value: p[1],
        })
        .collect();

    Ok(FundHistory {
        fund_code: fund_code.to_string(),
        fund_name,
        data,
    })
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct LJSYLZSResponse {
    Data: Vec<LJSYLZSData>,
    ErrCode: i32,
    ErrMsg: Option<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct LJSYLZSData {
    data: Vec<[f64; 2]>,
    name: String,
}




