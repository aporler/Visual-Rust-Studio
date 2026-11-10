//! Noyau partagé de Visual Rust Studio.
//!
//! Ce crate ne dépend d'aucune bibliothèque graphique. Il regroupe :
//! - l'écriture atomique de fichiers et les copies de sauvegarde ([`fs`]) ;
//! - la validation et la conversion d'identifiants Rust ([`ident`]).

pub mod fs;
pub mod ident;
