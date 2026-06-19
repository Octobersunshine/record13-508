mod models;
mod bank;
mod engine;
mod export;

use axum::{
    Json, Router,
    body::Body,
    extract::{State, Path},
    http::{StatusCode, HeaderMap, HeaderValue, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use models::*;
use bank::QuestionBank;
use engine::PaperEngine;

type SharedState = Arc<RwLock<AppState>>;

struct AppState {
    bank: QuestionBank,
    engine: PaperEngine,
}

#[tokio::main]
async fn main() {
    let bank = QuestionBank::new();
    let engine = PaperEngine::new(bank.rules.clone());
    let state = Arc::new(RwLock::new(AppState { bank, engine }));

    let app = Router::new()
        .route("/api/questions", get(list_questions))
        .route("/api/questions", post(add_question))
        .route("/api/rules", get(list_rules))
        .route("/api/rules", post(add_rule))
        .route("/api/paper/generate", post(generate_paper))
        .route("/api/papers", get(list_papers))
        .route("/api/paper/export/latest", get(export_latest_paper))
        .route("/api/paper/export/{id}", get(export_paper_by_id))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 试卷生成服务已启动 -> http://localhost:3000");
    println!("  GET  /api/questions                - 查看题库");
    println!("  POST /api/questions                - 添加题目");
    println!("  GET  /api/rules                    - 查看组卷规则");
    println!("  POST /api/rules                    - 添加组卷规则");
    println!("  POST /api/paper/generate           - 根据规则生成试卷");
    println!("  GET  /api/papers                   - 查看已生成试卷列表");
    println!("  GET  /api/paper/export/latest      - 导出最新试卷为 Word");
    println!("  GET  /api/paper/export/:id         - 按 ID 导出试卷为 Word");
    axum::serve(listener, app).await.unwrap();
}

async fn list_questions(State(state): State<SharedState>) -> Json<Vec<Question>> {
    let s = state.read().await;
    Json(s.bank.questions.clone())
}

async fn add_question(
    State(state): State<SharedState>,
    Json(input): Json<CreateQuestionInput>,
) -> Result<(StatusCode, Json<Question>), (StatusCode, Json<serde_json::Value>)> {
    let mut s = state.write().await;
    match s.bank.add_question(input) {
        Ok(q) => Ok((StatusCode::CREATED, Json(q))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e })),
        )),
    }
}

async fn list_rules(State(state): State<SharedState>) -> Json<Vec<PaperRule>> {
    let s = state.read().await;
    Json(s.engine.rules.clone())
}

async fn add_rule(
    State(state): State<SharedState>,
    Json(input): Json<CreateRuleInput>,
) -> Result<(StatusCode, Json<PaperRule>), (StatusCode, Json<serde_json::Value>)> {
    let mut s = state.write().await;
    match s.engine.add_rule(input) {
        Ok(r) => Ok((StatusCode::CREATED, Json(r))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e })),
        )),
    }
}

async fn generate_paper(
    State(state): State<SharedState>,
) -> Result<Json<ExamPaper>, (StatusCode, Json<serde_json::Value>)> {
    let mut s = state.write().await;
    let bank_snapshot = s.bank.clone();
    match s.engine.generate(&bank_snapshot) {
        Ok(paper) => Ok(Json(paper)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": e })),
        )),
    }
}

async fn list_papers(State(state): State<SharedState>) -> Json<Vec<ExamPaper>> {
    let s = state.read().await;
    Json(s.engine.papers.clone())
}

async fn export_latest_paper(
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let s = state.read().await;
    let paper = s.engine.papers.last().cloned().ok_or_else(|| (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "还没有生成任何试卷，请先调用 /api/paper/generate" })),
    ))?;
    drop(s);
    build_docx_response(paper).await
}

async fn export_paper_by_id(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let s = state.read().await;
    let paper = s.engine.papers
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("未找到 ID 为「{}」的试卷", id) })),
        ))?;
    drop(s);
    build_docx_response(paper).await
}

fn filename_utf8_percent_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'(' | b')' | b'[' | b']' => (b as char).to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

async fn build_docx_response(
    paper: ExamPaper,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let filename = export::filename_for_paper(&paper);
    let paper_for_export = paper.clone();

    let bytes = tokio::task::spawn_blocking(move || {
        export::paper_to_docx_bytes(&paper_for_export)
    })
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("导出任务失败：{}", e) })),
    ))?
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e })),
    ))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
    );
    let safe_ascii: String = paper.title.chars().map(|c| {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '(' || c == ')' || c == '[' || c == ']' { c } else { '_' }
    }).collect();
    let ascii_filename = format!("{}_{}.docx", safe_ascii, &paper.id[..8]);
    let utf8_encoded = filename_utf8_percent_encode(&filename);
    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        ascii_filename, utf8_encoded
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from(bytes.len() as u64),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    headers.insert(
        "X-Paper-Title",
        HeaderValue::from_str(&paper.title).unwrap_or_else(|_| HeaderValue::from_static("Exam Paper")),
    );

    Ok((headers, Body::from(bytes)))
}
