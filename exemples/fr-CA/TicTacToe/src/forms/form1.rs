//! Un jeu à deux joueurs avec 9 boutons : état de la grille, détection du gagnant et une méthode qui renvoie un bouton par son index.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

use crate::vrs::ButtonState;

/// Les 8 lignes gagnantes (lignes, colonnes, diagonales).
const LINES: [[usize; 3]; 8] = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8],
    [0, 3, 6], [1, 4, 7], [2, 5, 8],
    [0, 4, 8], [2, 4, 6],
];

#[derive(Default)]
pub struct Form1Data {
    /// Les 9 cases : None si vide, Some('X') ou Some('O').
    pub board: [Option<char>; 9],
    pub player: Option<char>,
    pub over: bool,
}

/// Renvoie le gagnant, si une ligne contient trois symboles identiques.
fn winner(board: &[Option<char>; 9]) -> Option<char> {
    LINES.iter().find_map(|[a, b, c]| {
        (board[*a].is_some() && board[*a] == board[*b] && board[*b] == board[*c]).then(|| board[*a]).flatten()
    })
}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        self.play(0);
    }

    /// Gère l'événement Click de `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        self.play(1);
    }

    /// Gère l'événement Click de `button3`.
    pub fn button3_click(&mut self, app: &mut AppContext) {
        self.play(2);
    }

    /// Gère l'événement Click de `button4`.
    pub fn button4_click(&mut self, app: &mut AppContext) {
        self.play(3);
    }

    /// Gère l'événement Click de `button5`.
    pub fn button5_click(&mut self, app: &mut AppContext) {
        self.play(4);
    }

    /// Gère l'événement Click de `button6`.
    pub fn button6_click(&mut self, app: &mut AppContext) {
        self.play(5);
    }

    /// Gère l'événement Click de `button7`.
    pub fn button7_click(&mut self, app: &mut AppContext) {
        self.play(6);
    }

    /// Gère l'événement Click de `button8`.
    pub fn button8_click(&mut self, app: &mut AppContext) {
        self.play(7);
    }

    /// Gère l'événement Click de `button9`.
    pub fn button9_click(&mut self, app: &mut AppContext) {
        self.play(8);
    }

    /// Gère l'événement Click de `button10`.
    pub fn button10_click(&mut self, app: &mut AppContext) {
        self.data = Form1Data::default();
        for i in 0..9 {
            self.cell(i).text.clear();
        }
        self.label1.text = format!("Au tour de {}", 'X');
    }

    /// Renvoie le bouton d'une case (0 à 8).
    fn cell(&mut self, i: usize) -> &mut ButtonState {
        match i {
            0 => &mut self.button1,
            1 => &mut self.button2,
            2 => &mut self.button3,
            3 => &mut self.button4,
            4 => &mut self.button5,
            5 => &mut self.button6,
            6 => &mut self.button7,
            7 => &mut self.button8,
            _ => &mut self.button9,
        }
    }

    /// Joue dans une case, puis vérifie s'il y a un gagnant ou une égalité.
    fn play(&mut self, i: usize) {
        if self.data.over || self.data.board[i].is_some() {
            return;
        }
        let player = self.data.player.unwrap_or('X');
        self.data.board[i] = Some(player);
        self.cell(i).text = player.to_string();
        if let Some(w) = winner(&self.data.board) {
            self.data.over = true;
            self.label1.text = format!("{} a gagné !", w);
        } else if self.data.board.iter().all(Option::is_some) {
            self.data.over = true;
            self.label1.text = String::from("Match nul !");
        } else {
            let next = if player == 'X' { 'O' } else { 'X' };
            self.data.player = Some(next);
            self.label1.text = format!("Au tour de {}", next);
        }
    }
}
