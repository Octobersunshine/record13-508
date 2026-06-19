use crate::models::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct QuestionBank {
    pub questions: Vec<Question>,
    pub rules: Vec<PaperRule>,
}

impl QuestionBank {
    pub fn new() -> Self {
        let questions = Self::init_questions();
        let rules = Self::init_rules();
        QuestionBank { questions, rules }
    }

    pub fn add_question(&mut self, input: CreateQuestionInput) -> Result<Question, String> {
        if input.score <= 0 {
            return Err("每题分值必须大于0".to_string());
        }
        let content_trim = input.content.trim();
        if content_trim.is_empty() {
            return Err("题目内容不能为空".to_string());
        }
        let exists = self.questions.iter().any(|q| {
            q.q_type == input.q_type && q.content.trim() == content_trim
        });
        if exists {
            return Err(format!(
                "题库中已存在相同内容的「{}」题目",
                input.q_type.label()
            ));
        }
        let q: Question = input.into();
        self.questions.push(q.clone());
        Ok(q)
    }

    pub fn filter_by_type_and_difficulty(
        &self,
        q_type: &QuestionType,
        min_diff: i32,
        max_diff: i32,
    ) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| &q.q_type == q_type && q.difficulty >= min_diff && q.difficulty <= max_diff)
            .collect()
    }

    fn init_rules() -> Vec<PaperRule> {
        vec![
            PaperRule {
                id: Uuid::new_v4().to_string(),
                q_type: QuestionType::SingleChoice,
                count: 10,
                score_per_question: 2,
                total_score: 20,
                min_difficulty: 1,
                max_difficulty: 5,
            },
            PaperRule {
                id: Uuid::new_v4().to_string(),
                q_type: QuestionType::MultipleChoice,
                count: 5,
                score_per_question: 4,
                total_score: 20,
                min_difficulty: 1,
                max_difficulty: 5,
            },
            PaperRule {
                id: Uuid::new_v4().to_string(),
                q_type: QuestionType::TrueOrFalse,
                count: 10,
                score_per_question: 1,
                total_score: 10,
                min_difficulty: 1,
                max_difficulty: 5,
            },
            PaperRule {
                id: Uuid::new_v4().to_string(),
                q_type: QuestionType::FillBlank,
                count: 5,
                score_per_question: 2,
                total_score: 10,
                min_difficulty: 1,
                max_difficulty: 5,
            },
            PaperRule {
                id: Uuid::new_v4().to_string(),
                q_type: QuestionType::ShortAnswer,
                count: 4,
                score_per_question: 5,
                total_score: 20,
                min_difficulty: 1,
                max_difficulty: 5,
            },
            PaperRule {
                id: Uuid::new_v4().to_string(),
                q_type: QuestionType::Essay,
                count: 2,
                score_per_question: 10,
                total_score: 20,
                min_difficulty: 1,
                max_difficulty: 5,
            },
        ]
    }

    fn init_questions() -> Vec<Question> {
        let mut qs = Vec::new();

        let singles = vec![
            ("Rust中哪个关键字用于声明不可变变量？", vec!["let", "let mut", "const", "static"], "A", 2, 1),
            ("以下哪种类型是Rust的原始类型？", vec!["String", "Vec", "i32", "HashMap"], "C", 2, 1),
            ("Rust中用于错误处理的核心trait是？", vec!["Display", "Debug", "Error", "Clone"], "C", 2, 2),
            ("以下哪个不是Rust的所有权规则？", vec!["每个值有一个所有者", "同一时刻只能有一个所有者", "所有者离开作用域值被丢弃", "值可以被多个变量同时拥有"], "D", 2, 2),
            ("Rust中哪个宏用于向标准输出打印？", vec!["printf", "println!", "echo", "print"], "B", 2, 1),
            ("以下哪个是Rust的并发原语？", vec!["thread", "goroutine", "async/await", "promise"], "A", 2, 3),
            ("Rust中trait的作用类似于其他语言的什么？", vec!["类", "接口", "泛型", "枚举"], "B", 2, 1),
            ("Option<T>的Some变体表示什么？", vec!["空值", "有值", "错误", "未知"], "B", 2, 1),
            ("Rust中哪个关键字用于模式匹配？", vec!["if", "switch", "match", "case"], "C", 2, 1),
            ("Rust中的生命周期标注使用什么符号？", vec!["#", "@", "'", "$"], "C", 2, 3),
            ("以下哪个方法用于将Result<T,E>转换为Option<T>？", vec!["ok()", "some()", "unwrap()", "expect()"], "A", 2, 2),
            ("Rust中用于动态大小类型的trait是？", vec!["Sized", "Display", "Debug", "Clone"], "A", 2, 4),
            ("Rust中哪个集合类型存储在堆上？", vec!["[i32;3]", "i32", "String", "f64"], "C", 2, 1),
            ("Vec<T>的增长策略是什么？", vec!["每次加1", "翻倍", "固定大小", "随机增长"], "B", 2, 3),
            ("Rust中的闭包也被称为？", vec!["函数指针", "Lambda", "迭代器", "生成器"], "B", 2, 2),
        ];
        for (i, (content, options, answer, score, diff)) in singles.into_iter().enumerate() {
            qs.push(Question {
                id: format!("sc-{}", i + 1),
                q_type: QuestionType::SingleChoice,
                content: content.to_string(),
                options: options.into_iter().map(String::from).collect(),
                answer: answer.to_string(),
                score,
                difficulty: diff,
            });
        }

        let multiples = vec![
            ("以下哪些是Rust的有效数值类型？", vec!["i32", "f64", "string", "u8"], "ABD", 4, 2),
            ("Rust中实现trait需要实现哪些方法？", vec!["必需方法", "提供默认实现的方法", "私有方法", "静态方法"], "AB", 4, 2),
            ("以下哪些可以用于Rust的异步编程？", vec!["async/await", "tokio", "goroutine", "Future trait"], "ABD", 4, 3),
            ("Rust中以下哪些是有效的字符串类型？", vec!["String", "&str", "CString", "str"], "ABD", 4, 2),
            ("以下哪些是Rust的安全特性？", vec!["所有权系统", "借用检查", "空指针", "数据竞争"], "AB", 4, 1),
            ("Rust中以下哪些集合是线程安全的？", vec!["Vec", "Mutex<Vec>", "RwLock<Vec>", "VecDeque"], "BC", 4, 4),
            ("以下哪些属于Rust的智能指针？", vec!["Box<T>", "Rc<T>", "Arc<T>", "RefCell<T>"], "ABCD", 4, 3),
        ];
        for (i, (content, options, answer, score, diff)) in multiples.into_iter().enumerate() {
            qs.push(Question {
                id: format!("mc-{}", i + 1),
                q_type: QuestionType::MultipleChoice,
                content: content.to_string(),
                options: options.into_iter().map(String::from).collect(),
                answer: answer.to_string(),
                score,
                difficulty: diff,
            });
        }

        let tf = vec![
            ("Rust是一种面向对象编程语言。", "F", 1, 1),
            ("Rust编译器可以在编译期检测数据竞争。", "T", 1, 2),
            ("Rust中变量默认是可变的。", "F", 1, 1),
            ("Rust支持垃圾回收机制。", "F", 1, 1),
            ("Rust中的enum可以携带数据。", "T", 1, 2),
            ("Rust支持函数重载。", "F", 1, 2),
            ("Rust中的match必须穷举所有可能的情况。", "T", 1, 2),
            ("Rust可以在没有标准库的情况下运行。", "T", 1, 3),
            ("Rust中的所有trait都可以被自动派生。", "F", 1, 2),
            ("Rust支持零成本抽象。", "T", 1, 3),
            ("Rust中的引用总是非空的。", "T", 1, 2),
            ("Rust允许同一个可变引用和不可变引用同时存在。", "F", 1, 1),
            ("Rust中的if是表达式而不是语句。", "T", 1, 3),
            ("Rust的Option类型可以替代空值。", "T", 1, 1),
        ];
        for (i, (content, answer, score, diff)) in tf.into_iter().enumerate() {
            qs.push(Question {
                id: format!("tf-{}", i + 1),
                q_type: QuestionType::TrueOrFalse,
                content: content.to_string(),
                options: vec!["正确".to_string(), "错误".to_string()],
                answer: answer.to_string(),
                score,
                difficulty: diff,
            });
        }

        let fills = vec![
            ("Rust中使用____关键字声明一个函数。", "fn", 2, 1),
            ("Rust中使用____关键字引入外部crate。", "use", 2, 1),
            ("Rust中用____将引用标记为可变的。", "mut", 2, 1),
            ("Rust中的宏调用以____符号结尾。", "!", 2, 2),
            ("Rust中用____trait标记可以被格式化打印的类型。", "Display", 2, 2),
            ("Rust的____检查器在编译时确保引用的有效性。", "借用", 2, 3),
            ("Rust中用于声明常量的关键字是____。", "const", 2, 1),
        ];
        for (i, (content, answer, score, diff)) in fills.into_iter().enumerate() {
            qs.push(Question {
                id: format!("fb-{}", i + 1),
                q_type: QuestionType::FillBlank,
                content: content.to_string(),
                options: vec![],
                answer: answer.to_string(),
                score,
                difficulty: diff,
            });
        }

        let shorts = vec![
            ("简述Rust所有权系统的三条核心规则。", "1)每个值有一个所有者 2)同一时刻只有一个所有者 3)所有者离开作用域值被丢弃", 5, 2),
            ("解释Rust中borrowing（借用）的概念。", "借用是获取引用而不获取所有权的方式，分为可变借用和不可变借用", 5, 2),
            ("简述Rust中Option和Result的区别。", "Option表示可能有值或无值，Result表示操作可能成功或失败", 5, 3),
            ("解释Rust中生命周期的概念及其作用。", "生命周期是引用有效性的范围，帮助编译器确保引用不会在其指向的数据之后存在", 5, 4),
            ("简述Rust中trait和impl的用法。", "trait定义行为接口，impl为类型实现具体方法", 5, 2),
            ("解释Rust中泛型的实现方式及其优势。", "Rust通过单态化实现泛型，编译时生成具体类型代码，实现零成本抽象", 5, 3),
        ];
        for (i, (content, answer, score, diff)) in shorts.into_iter().enumerate() {
            qs.push(Question {
                id: format!("sa-{}", i + 1),
                q_type: QuestionType::ShortAnswer,
                content: content.to_string(),
                options: vec![],
                answer: answer.to_string(),
                score,
                difficulty: diff,
            });
        }

        let essays = vec![
            ("论述Rust的并发安全机制，包括Send/Sync trait、Mutex/RwLock以及async/await模型。", "Rust通过所有权系统在编译期保证线程安全：Send trait标记可跨线程传递的类型，Sync trait标记可被多线程共享引用的类型；Mutex提供互斥访问，RwLock提供读写锁；async/await提供异步编程模型。三者结合使Rust能够在编译期消除数据竞争。", 10, 4),
            ("论述Rust的模块系统设计，包括mod、pub、use、crate等关键字的作用以及模块可见性规则。", "Rust模块系统通过mod声明子模块，pub控制可见性，use引入路径，crate代表当前crate根。默认私有，需pub显式暴露。模块可以组织代码层次，控制接口暴露，支持重导出和嵌套。path属性可自定义模块文件路径。", 10, 4),
            ("论述Rust中的零成本抽象原则，结合trait、泛型、闭包等特性进行分析。", "零成本抽象意味着使用高层抽象不会带来运行时开销。Rust通过单态化将泛型展开为具体代码，trait对象使用vtable但仅在需要动态分发时；闭包编译为匿名结构体实现Fn/FnMut/FnOnce trait；迭代器通过内联优化消除循环开销。这些都是编译期完成的，运行时无额外代价。", 10, 5),
        ];
        for (i, (content, answer, score, diff)) in essays.into_iter().enumerate() {
            qs.push(Question {
                id: format!("es-{}", i + 1),
                q_type: QuestionType::Essay,
                content: content.to_string(),
                options: vec![],
                answer: answer.to_string(),
                score,
                difficulty: diff,
            });
        }

        qs
    }
}
