//! A multiple-choice quiz with RadioButtons, a constant array of questions and a final score.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

/// Questions: (text, three answers, index of the right answer).
const QUESTIONS: [(&str, [&str; 3], usize); 3] = [
    ("Which keyword declares a variable in Rust?", ["let", "var", "dim"], 0),
    ("How do you make a variable modifiable?", ["const", "mut", "static"], 1),
    ("Which macro writes text to the console?", ["print_line!", "echo!", "println!"], 2),
];

#[derive(Default)]
pub struct Form1Data {
    pub index: usize,
    pub score: u32,
}

impl Form1 {
    /// Handles the Load event of `Form1`.
    pub fn form1_load(&mut self, app: &mut AppContext) {
        self.show_question();
    }

    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Which RadioButton is checked? position() gives Some(index) or None.
        let answers = [
            self.radio_button1.checked,
            self.radio_button2.checked,
            self.radio_button3.checked,
        ];
        let Some(chosen) = answers.iter().position(|checked| *checked) else {
            self.label2.text = String::from("Choose an answer.");
            return;
        };
        let (_, _, good) = QUESTIONS[self.data.index];
        if chosen == good {
            self.data.score += 1;
            self.label2.text = String::from("Right answer!");
        } else {
            self.label2.text = String::from("Wrong answer.");
        }
        self.data.index += 1;
        if self.data.index == QUESTIONS.len() {
            app.message_box(format!("Final score: {} / {}", self.data.score, QUESTIONS.len()));
            self.data.index = 0;
            self.data.score = 0;
            self.label2.text.clear();
        }
        self.show_question();
    }

    /// Shows the current question and its answers.
    fn show_question(&mut self) {
        let (question, answers, _) = QUESTIONS[self.data.index];
        self.label1.text = question.to_owned();
        for (radio, answer) in [
            &mut self.radio_button1,
            &mut self.radio_button2,
            &mut self.radio_button3,
        ]
        .into_iter()
        .zip(answers)
        {
            radio.text = answer.to_owned();
            radio.checked = false;
        }
    }
}
