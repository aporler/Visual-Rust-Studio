//! Régénère le dossier des exemples : `cargo run -p vrs-app --bin vrs-export-examples -- exemples`

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "exemples".into());
    for lang in vrs_app::EXPORT_LANGUAGES {
        match vrs_app::export_examples(std::path::Path::new(&dir), lang) {
            Ok(list) => {
                for p in list {
                    println!("{}", p.display());
                }
            }
            Err(e) => {
                eprintln!("{lang}: {e}");
                std::process::exit(1);
            }
        }
    }
}
