use crate::models::*;
use crate::bank::QuestionBank;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use uuid::Uuid;

pub struct PaperEngine {
    pub rules: Vec<PaperRule>,
    pub papers: Vec<ExamPaper>,
}

impl PaperEngine {
    pub fn new(rules: Vec<PaperRule>) -> Self {
        PaperEngine {
            rules,
            papers: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, input: CreateRuleInput) -> Result<PaperRule, String> {
        if input.count <= 0 {
            return Err("题目数量必须大于0".to_string());
        }
        if input.score_per_question <= 0 {
            return Err("每题分值必须大于0".to_string());
        }
        let rule = input.into_rule();
        self.rules.push(rule.clone());
        Ok(rule)
    }

    pub fn generate(&mut self, bank: &QuestionBank) -> Result<ExamPaper, String> {
        if self.rules.is_empty() {
            return Err("没有设置组卷规则，请先添加规则".to_string());
        }

        let mut rng = rand::rng();
        let mut sections = Vec::new();
        let mut total_score = 0;
        let mut used_ids: HashSet<String> = HashSet::new();
        let mut used_contents: HashSet<String> = HashSet::new();

        for rule in &self.rules {
            let candidates = bank.filter_by_type_and_difficulty(
                &rule.q_type,
                rule.min_difficulty,
                rule.max_difficulty,
            );

            let available: Vec<&Question> = candidates
                .into_iter()
                .filter(|q| {
                    !used_ids.contains(&q.id)
                        && !used_contents.contains(&q.content.trim().to_string())
                })
                .collect();

            if available.len() < rule.count as usize {
                return Err(format!(
                    "题型「{}」需要{}题，但符合条件且未重复的题目仅有{}题（总量{}题，已用{}题）",
                    rule.q_type.label(),
                    rule.count,
                    available.len(),
                    bank.filter_by_type_and_difficulty(
                        &rule.q_type,
                        rule.min_difficulty,
                        rule.max_difficulty,
                    ).len(),
                    used_ids.len(),
                ));
            }

            let mut pool = available;
            pool.shuffle(&mut rng);

            let selected: Vec<&Question> = pool.into_iter().take(rule.count as usize).collect();

            for q in &selected {
                used_ids.insert(q.id.clone());
                used_contents.insert(q.content.trim().to_string());
            }

            let questions: Vec<QuestionInPaper> = selected
                .iter()
                .map(|q| QuestionInPaper::from_question(*q, rule.score_per_question))
                .collect();

            total_score += rule.total_score;

            sections.push(SectionInPaper {
                q_type: rule.q_type.clone(),
                label: format!("{}、{}（每题{}分，共{}分）", 
                    chinese_num(sections.len() + 1),
                    rule.q_type.label(),
                    rule.score_per_question,
                    rule.total_score,
                ),
                count: rule.count,
                score_per_question: rule.score_per_question,
                total_score: rule.total_score,
                questions,
            });
        }

        let paper = ExamPaper {
            id: Uuid::new_v4().to_string(),
            title: generate_paper_title(),
            total_score,
            sections,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        Self::validate_no_duplicates(&paper)?;

        self.papers.push(paper.clone());
        Ok(paper)
    }

    fn validate_no_duplicates(paper: &ExamPaper) -> Result<(), String> {
        let mut id_set = HashSet::new();
        let mut content_set = HashSet::new();
        let mut total_count = 0usize;

        for section in &paper.sections {
            for q in &section.questions {
                total_count += 1;
                let content_key = format!("{:?}|{}", q.q_type, q.content.trim());
                if !id_set.insert(q.id.clone()) {
                    return Err(format!(
                        "试卷校验失败：发现重复题目ID「{}」（题型：{}）",
                        q.id,
                        q.q_type.label(),
                    ));
                }
                if !content_set.insert(content_key.clone()) {
                    return Err(format!(
                        "试卷校验失败：发现重复题目内容「{}」（题型：{}）",
                        q.content,
                        q.q_type.label(),
                    ));
                }
            }
        }

        assert_eq!(
            id_set.len(),
            total_count,
            "内部错误：去重集合大小与题目总数不一致"
        );
        Ok(())
    }
}

fn chinese_num(n: usize) -> &'static str {
    const CN: &[&str] = &[
        "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
    ];
    CN.get(n - 1).unwrap_or(&"?")
}
