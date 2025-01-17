use std::collections::HashMap;

use serde::Deserialize;


#[derive(Clone, Deserialize, Debug)]
pub struct QuizQuestion {
    pub question: String,
    pub options: QuizOptions,
    pub correct: char,
}

#[derive(Clone, Deserialize, Debug)]
pub struct QuizOptions {
    pub a: String,
    pub b: String,
    pub c: String,
    pub d: String,
}

pub struct QuizManager {
    pub current_quiz: Option<QuizQuestion>,
    pub answers: HashMap<u64, char>
}

impl QuizManager {
    pub fn new() -> Self {
        Self {
            current_quiz: None,
            answers: HashMap::new()
        }
    }

    pub fn set_quiz(&mut self, quiz: QuizQuestion) {
        self.current_quiz = Some(quiz);
        self.answers.clear();
    }

    pub fn set_answer(&mut self, user_id: u64, answer: char) {
        self.answers.insert(user_id, answer);
    }

    pub fn has_user_answered(&self, user_id: u64) -> bool {
        self.answers.contains_key(&user_id)
    }

    pub fn finalize_results(&mut self) -> Option<String> {
        let quiz = match &self.current_quiz {
            Some(q) => q,
            None => {
                println!("No active quiz found!");
                return Some("No active quiz running.".to_string());
            }
        };

        let correct_answer = &quiz.correct;

        let mut summary = String::new();
        summary.push_str(&format!("**Correct answer: {}**\n", correct_answer.to_ascii_uppercase()));
        summary.push_str("**Results:**\n");

        if self.answers.is_empty() {
            return Some("No answers submitted yet.".to_string());
        }

        for (uid, answer) in &self.answers {
            if answer.to_lowercase().cmp(correct_answer.to_lowercase()) == std::cmp::Ordering::Equal {
                summary.push_str(&format!("<@{}> answered {} and got it right!\n", uid, answer));
            } else {
                summary.push_str(&format!("<@{}> answered {} and got it wrong!\n", uid, answer));
            }
        }
        summary.push_str("\n***Quiz ended. Thank you for participating!***");
        
        self.current_quiz = None;
        self.answers.clear();
        Some(summary)
    }
}