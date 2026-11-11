//! Un questionnaire à choix multiples avec des RadioButton, un tableau constant de questions et un score final.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

/// Questions : (texte, trois réponses, index de la bonne réponse).
const QUESTIONS: [(&str, [&str; 3], usize); 3] = [
    ("Quel mot-clé déclare une variable en Rust ?", ["let", "var", "dim"], 0),
    ("Comment rend-on une variable modifiable ?", ["const", "mut", "static"], 1),
    ("Quelle macro écrit du texte dans la console ?", ["print_line!", "echo!", "println!"], 2),
];

#[derive(Default)]
pub struct Form1Data {
    pub index: usize,
    pub score: u32,
}

impl Form1 {
    /// Gère l'événement Load de `Form1`.
    pub fn form1_load(&mut self, app: &mut AppContext) {
        self.show_question();
    }

    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Quel RadioButton est coché ? position() donne Some(index) ou None.
        let answers = [
            self.radio_button1.checked,
            self.radio_button2.checked,
            self.radio_button3.checked,
        ];
        let Some(chosen) = answers.iter().position(|checked| *checked) else {
            self.label2.text = String::from("Choisissez une réponse.");
            return;
        };
        let (_, _, good) = QUESTIONS[self.data.index];
        if chosen == good {
            self.data.score += 1;
            self.label2.text = String::from("Bonne réponse !");
        } else {
            self.label2.text = String::from("Mauvaise réponse.");
        }
        self.data.index += 1;
        if self.data.index == QUESTIONS.len() {
            app.message_box(format!("Score final : {} / {}", self.data.score, QUESTIONS.len()));
            self.data.index = 0;
            self.data.score = 0;
            self.label2.text.clear();
        }
        self.show_question();
    }

    /// Affiche la question en cours et ses réponses.
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
