//! Vérifie un répertoire de traductions.
//!
//! Utilisation : `cargo run -p vrs-i18n --bin vrs-i18n-check -- [chemin/vers/locales]`
//!
//! Sans argument, vérifie les traductions intégrées à VRS. Avec un chemin, les
//! fichiers externes sont ajoutés par-dessus (comme dans l'IDE).
//! Code de sortie : 1 s'il existe au moins une erreur.

use vrs_i18n::validate::{Severity, coverage, validate};
use vrs_i18n::{Catalog, default_locale};

fn main() {
    let mut catalog = Catalog::embedded();
    if let Some(dir) = std::env::args().nth(1) {
        for issue in catalog.load_dir(std::path::Path::new(&dir)) {
            eprintln!("[erreur] {} : {}", issue.path.display(), issue.message);
        }
    }
    let reference = default_locale();
    let issues = validate(&catalog, &reference);
    for i in &issues {
        println!("{i}");
    }
    println!();
    println!("Couverture par rapport à {reference} :");
    for (lang, n, total) in coverage(&catalog, &reference) {
        let pct = if total == 0 {
            0.0
        } else {
            n as f64 * 100.0 / total as f64
        };
        println!("  {lang:<8} {n:>4}/{total:<4} {pct:5.1} %");
    }
    let errors = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    if errors > 0 {
        eprintln!("{errors} erreur(s).");
        std::process::exit(1);
    }
}
