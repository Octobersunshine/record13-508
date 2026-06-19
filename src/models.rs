use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    SingleChoice,
    MultipleChoice,
    TrueOrFalse,
    FillBlank,
    ShortAnswer,
    Essay,
}

impl QuestionType {
    pub fn label(&self) -> &str {
        match self {
            QuestionType::SingleChoice => "单选题",
            QuestionType::MultipleChoice => "多选题",
            QuestionType::TrueOrFalse => "判断题",
            QuestionType::FillBlank => "填空题",
            QuestionType::ShortAnswer => "简答题",
            QuestionType::Essay => "论述题",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub q_type: QuestionType,
    pub content: String,
    pub options: Vec<String>,
    pub answer: String,
    pub score: i32,
    pub difficulty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionInput {
    pub q_type: QuestionType,
    pub content: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub answer: String,
    pub score: i32,
    #[serde(default = "default_difficulty")]
    pub difficulty: i32,
}

fn default_difficulty() -> i32 {
    3
}

impl From<CreateQuestionInput> for Question {
    fn from(input: CreateQuestionInput) -> Self {
        Question {
            id: Uuid::new_v4().to_string(),
            q_type: input.q_type,
            content: input.content,
            options: input.options,
            answer: input.answer,
            score: input.score,
            difficulty: input.difficulty,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperRule {
    pub id: String,
    pub q_type: QuestionType,
    pub count: i32,
    pub score_per_question: i32,
    pub total_score: i32,
    #[serde(default)]
    pub min_difficulty: i32,
    #[serde(default = "default_max_difficulty")]
    pub max_difficulty: i32,
}

fn default_max_difficulty() -> i32 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleInput {
    pub q_type: QuestionType,
    pub count: i32,
    pub score_per_question: i32,
    #[serde(default)]
    pub min_difficulty: i32,
    #[serde(default = "default_max_difficulty")]
    pub max_difficulty: i32,
}

impl CreateRuleInput {
    pub fn into_rule(self) -> PaperRule {
        let total_score = self.count * self.score_per_question;
        PaperRule {
            id: Uuid::new_v4().to_string(),
            q_type: self.q_type,
            count: self.count,
            score_per_question: self.score_per_question,
            total_score,
            min_difficulty: self.min_difficulty,
            max_difficulty: self.max_difficulty,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInPaper {
    pub id: String,
    pub q_type: QuestionType,
    pub content: String,
    pub options: Vec<String>,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInPaper {
    pub q_type: QuestionType,
    pub label: String,
    pub count: i32,
    pub score_per_question: i32,
    pub total_score: i32,
    pub questions: Vec<QuestionInPaper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamPaper {
    pub id: String,
    pub title: String,
    pub total_score: i32,
    pub sections: Vec<SectionInPaper>,
    pub created_at: String,
}

pub fn generate_paper_title() -> String {
    let mut rng = rand::rng();
    let year = 2025 + rng.random_range(0..3);
    let term = if rng.random_bool(0.5) { "上" } else { "下" };
    let variant = (b'A' + rng.random_range(0..4)) as char;
    format!("{}年{}学期期末考试试卷（{}卷）", year, term, variant)
}

impl QuestionInPaper {
    pub fn from_question(q: &Question, score: i32) -> Self {
        QuestionInPaper {
            id: q.id.clone(),
            q_type: q.q_type.clone(),
            content: q.content.clone(),
            options: q.options.clone(),
            score,
        }
    }
}
