//! A two-player game with 9 buttons: board state, winner detection and a helper that returns a button by index.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

use crate::vrs::ButtonState;

/// The 8 winning lines (rows, columns, diagonals).
const LINES: [[usize; 3]; 8] = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8],
    [0, 3, 6], [1, 4, 7], [2, 5, 8],
    [0, 4, 8], [2, 4, 6],
];

#[derive(Default)]
pub struct Form1Data {
    /// The 9 cells: None if empty, Some('X') or Some('O').
    pub board: [Option<char>; 9],
    pub player: Option<char>,
    pub over: bool,
}

/// Returns the winner, if a line contains three identical symbols.
fn winner(board: &[Option<char>; 9]) -> Option<char> {
    LINES.iter().find_map(|[a, b, c]| {
        (board[*a].is_some() && board[*a] == board[*b] && board[*b] == board[*c]).then(|| board[*a]).flatten()
    })
}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        self.play(0);
    }

    /// Handles the Click event of `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        self.play(1);
    }

    /// Handles the Click event of `button3`.
    pub fn button3_click(&mut self, app: &mut AppContext) {
        self.play(2);
    }

    /// Handles the Click event of `button4`.
    pub fn button4_click(&mut self, app: &mut AppContext) {
        self.play(3);
    }

    /// Handles the Click event of `button5`.
    pub fn button5_click(&mut self, app: &mut AppContext) {
        self.play(4);
    }

    /// Handles the Click event of `button6`.
    pub fn button6_click(&mut self, app: &mut AppContext) {
        self.play(5);
    }

    /// Handles the Click event of `button7`.
    pub fn button7_click(&mut self, app: &mut AppContext) {
        self.play(6);
    }

    /// Handles the Click event of `button8`.
    pub fn button8_click(&mut self, app: &mut AppContext) {
        self.play(7);
    }

    /// Handles the Click event of `button9`.
    pub fn button9_click(&mut self, app: &mut AppContext) {
        self.play(8);
    }

    /// Handles the Click event of `button10`.
    pub fn button10_click(&mut self, app: &mut AppContext) {
        self.data = Form1Data::default();
        for i in 0..9 {
            self.cell(i).text.clear();
        }
        self.label1.text = format!("Turn of {}", 'X');
    }

    /// Returns the button of a cell (0 to 8).
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

    /// Plays in a cell, then checks for a winner or a draw.
    fn play(&mut self, i: usize) {
        if self.data.over || self.data.board[i].is_some() {
            return;
        }
        let player = self.data.player.unwrap_or('X');
        self.data.board[i] = Some(player);
        self.cell(i).text = player.to_string();
        if let Some(w) = winner(&self.data.board) {
            self.data.over = true;
            self.label1.text = format!("{} wins!", w);
        } else if self.data.board.iter().all(Option::is_some) {
            self.data.over = true;
            self.label1.text = String::from("Draw!");
        } else {
            let next = if player == 'X' { 'O' } else { 'X' };
            self.data.player = Some(next);
            self.label1.text = format!("Turn of {}", next);
        }
    }
}
