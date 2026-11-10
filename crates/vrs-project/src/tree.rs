//! Arborescence des fichiers affichée dans l'explorateur de projet.

use std::path::{Path, PathBuf};

/// Nœud de l'arborescence.
#[derive(Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

const MAX_DEPTH: usize = 8;
const MAX_ENTRIES: usize = 5000;
const IGNORED_DIRS: &[&str] = &["target", ".git", "node_modules"];

/// Construit l'arborescence (dossiers d'abord, ordre alphabétique).
/// Les dossiers `target`, `.git` et les fichiers cachés sont ignorés.
pub fn file_tree(root: &Path) -> FileNode {
    let mut budget = MAX_ENTRIES;
    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.display().to_string());
    FileNode {
        name,
        path: root.to_path_buf(),
        is_dir: true,
        children: children(root, 0, &mut budget),
    }
}

fn children(dir: &Path, depth: usize, budget: &mut usize) -> Vec<FileNode> {
    if depth >= MAX_DEPTH {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut nodes = Vec::new();
    for e in entries.filter_map(Result::ok) {
        if *budget == 0 {
            break;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != ".gitignore" {
            continue;
        }
        if name.ends_with(".bak") {
            continue;
        }
        let path = e.path();
        // Ne pas suivre les liens symboliques vers des dossiers (boucles possibles).
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir && IGNORED_DIRS.contains(&name.as_str()) {
            continue;
        }
        *budget -= 1;
        let kids = if is_dir {
            children(&path, depth + 1, budget)
        } else {
            Vec::new()
        };
        nodes.push(FileNode {
            name,
            path,
            is_dir,
            children: kids,
        });
    }
    nodes.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_skips_target_and_hidden() {
        let dir = tempfile::tempdir().unwrap();
        for p in [
            "src/main.rs",
            "target/debug/x",
            ".git/HEAD",
            "Cargo.toml",
            "b.rs.bak",
        ] {
            let full = dir.path().join(p);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, "").unwrap();
        }
        let t = file_tree(dir.path());
        let names: Vec<_> = t.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["src", "Cargo.toml"]);
        assert_eq!(t.children[0].children[0].name, "main.rs");
    }
}
