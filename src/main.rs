mod models;
mod bank;
mod engine;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 试卷生成服务已启动 -> http://localhost:3000");
    println!("  GET  /api/questions        - 查看题库");
    println!("  POST /api/questions        - 添加题目");
    println!("  GET  /api/rules            - 查看组卷规则");
    println!("  POST /api/rules            - 添加组卷规则");
    println!("  POST /api/paper/generate   - 根据规则生成试卷");
    println!("  GET  /api/papers           - 查看已生成试卷列表");
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
