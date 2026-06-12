mod cli;
mod config;
mod db;
mod excel;
mod models;

use bytes::Buf;
use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use db::DbPool;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Server) | None => run_server().await,
        Some(Commands::InitDb) => run_init_db(),
        Some(Commands::SetPassword { password }) => run_set_password(password),
        Some(Commands::Import { file }) => run_import(file),
        Some(Commands::Clear) => run_clear(),
    }
}

fn run_init_db() {
    let _pool = db::init_pool("bank_query.db").expect("Failed to init database");
    println!("数据库已初始化");
    // Also ensure config exists
    Config::load();
    println!("配置文件已创建");
}

fn run_set_password(password: &str) {
    let mut cfg = Config::load();
    cfg.admin_password = password.to_string();
    cfg.save();
    println!("管理员密码已更新");
}

fn run_import(file: &str) {
    let pool = db::init_pool("bank_query.db").expect("Failed to init database");
    let data = match std::fs::read(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("读取文件失败: {}", e);
            return;
        }
    };
    match excel::parse_excel(&data) {
        Ok(records) => {
            if records.is_empty() {
                eprintln!("文件中没有有效数据");
                return;
            }
            match db::insert_banks(&pool, &records) {
                Ok(count) => println!("成功导入 {} 条数据", count),
                Err(e) => eprintln!("导入失败: {}", e),
            }
        }
        Err(e) => eprintln!("解析文件失败: {}", e),
    }
}

fn run_clear() {
    let pool = db::init_pool("bank_query.db").expect("Failed to init database");
    match db::clear_banks(&pool) {
        Ok(count) => println!("已清空 {} 条数据", count),
        Err(e) => eprintln!("清空失败: {}", e),
    }
}

async fn run_server() {
    let pool = db::init_pool("bank_query.db").expect("Failed to init database");
    let config = Arc::new(Config::load());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(config.port);

    // GET / — serve the HTML page
    let index = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(include_str!("../templates/index.html")));

    // POST /api/verify-password — verify admin password
    let verify_password = warp::path!("api" / "verify-password")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_config(config.clone()))
        .and_then(handle_verify_password);

    // GET /api/banks?q=xxx — fuzzy search
    let search = warp::path!("api" / "banks")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(pool.clone()))
        .and_then(handle_search);

    // POST /api/banks/import — upload Excel
    let import = warp::path!("api" / "banks" / "import")
        .and(warp::post())
        .and(warp::multipart::form().max_length(10_000_000))
        .and(with_db(pool.clone()))
        .and_then(handle_import);

    // DELETE /api/banks — clear all data
    let clear = warp::path!("api" / "banks")
        .and(warp::delete())
        .and(with_db(pool.clone()))
        .and_then(handle_clear);

    // PUT /api/banks/{id} — update a bank record
    let update = warp::path!("api" / "banks" / i64)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(pool.clone()))
        .and_then(handle_update);

    // DELETE /api/banks/{id} — delete a single record
    let delete_one = warp::path!("api" / "banks" / i64)
        .and(warp::delete())
        .and(with_db(pool.clone()))
        .and_then(handle_delete_one);

    // GET /api/template/download — download Excel template
    let template = warp::path!("api" / "template" / "download")
        .and(warp::get())
        .and_then(handle_template);

    let routes = index
        .or(verify_password)
        .or(search)
        .or(import)
        .or(clear)
        .or(update)
        .or(delete_one)
        .or(template)
        .with(warp::cors().allow_any_origin());

    println!("Server started at http://localhost:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

fn with_config(
    config: Arc<Config>,
) -> impl Filter<Extract = (Arc<Config>,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_db(pool: DbPool) -> impl Filter<Extract = (DbPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

// ── handlers ──

use serde::Deserialize;

#[derive(Deserialize)]
struct PasswordInput {
    password: String,
}

async fn handle_verify_password(
    input: PasswordInput,
    config: Arc<Config>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let valid = input.password == config.admin_password;
    Ok(warp::reply::json(&serde_json::json!({ "valid": valid })))
}

async fn handle_search(
    params: std::collections::HashMap<String, String>,
    pool: DbPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    if q.is_empty() {
        return Ok(warp::reply::json(&serde_json::json!({
            "results": [],
            "total": 0
        })));
    }

    match db::search_banks(&pool, q) {
        Ok(resp) => Ok(warp::reply::json(&resp)),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "error": format!("查询失败: {}", e)
        }))),
    }
}

use futures_util::StreamExt;
use warp::multipart::FormData;

async fn handle_import(
    mut form: FormData,
    pool: DbPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut file_data = Vec::new();

    while let Some(part_result) = form.next().await {
        let mut part = match part_result {
            Ok(p) => p,
            Err(_) => {
                return Ok(warp::reply::json(&serde_json::json!({
                    "success": false,
                    "message": "读取上传文件失败，请检查文件名是否为中文或特殊字符"
                })));
            }
        };
        while let Some(chunk_result) = part.data().await {
            let mut chunk = match chunk_result {
                Ok(c) => c,
                Err(_) => {
                    return Ok(warp::reply::json(&serde_json::json!({
                        "success": false,
                        "message": "读取文件内容失败"
                    })));
                }
            };
            let len = chunk.remaining();
            let bytes = chunk.copy_to_bytes(len);
            file_data.extend_from_slice(&bytes);
        }
    }

    if file_data.is_empty() {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "message": "上传文件为空"
        })));
    }

    match excel::parse_excel(&file_data) {
        Ok(records) => {
            if records.is_empty() {
                return Ok(warp::reply::json(&serde_json::json!({
                    "success": false,
                    "message": "未能从文件中解析到有效数据"
                })));
            }
            match db::insert_banks(&pool, &records) {
                Ok(count) => Ok(warp::reply::json(&serde_json::json!({
                    "success": true,
                    "count": count
                }))),
                Err(e) => Ok(warp::reply::json(&serde_json::json!({
                    "success": false,
                    "message": format!("数据库写入失败: {}", e)
                }))),
            }
        }
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "message": format!("文件解析失败: {}", e)
        }))),
    }
}

async fn handle_clear(pool: DbPool) -> Result<impl warp::Reply, warp::Rejection> {
    match db::clear_banks(&pool) {
        Ok(count) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "count": count
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "message": format!("清空失败: {}", e)
        }))),
    }
}

async fn handle_update(
    id: i64,
    body: models::BankUpdate,
    pool: DbPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db::update_bank(&pool, id, &body.code, &body.name) {
        Ok(true) => Ok(warp::reply::json(&serde_json::json!({"success": true}))),
        Ok(false) => Ok(warp::reply::json(&serde_json::json!({
            "success": false, "message": "未找到该记录"
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false, "message": format!("更新失败: {}", e)
        }))),
    }
}

async fn handle_delete_one(id: i64, pool: DbPool) -> Result<impl warp::Reply, warp::Rejection> {
    match db::delete_bank(&pool, id) {
        Ok(true) => Ok(warp::reply::json(&serde_json::json!({"success": true}))),
        Ok(false) => Ok(warp::reply::json(&serde_json::json!({
            "success": false, "message": "未找到该记录"
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false, "message": format!("删除失败: {}", e)
        }))),
    }
}

async fn handle_template() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    match excel::generate_template() {
        Ok(data) => {
            let resp = warp::http::Response::builder()
                .header(
                    "Content-Type",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                )
                .header(
                    "Content-Disposition",
                    "attachment; filename=\"bank_template.xlsx\"",
                )
                .body(data)
                .map_err(|_| warp::reject::reject())?;
            Ok(Box::new(resp))
        }
        Err(e) => Ok(Box::new(warp::reply::json(&serde_json::json!({
            "error": format!("生成模板失败: {}", e)
        })))),
    }
}
