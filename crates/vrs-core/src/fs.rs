//! Opérations de fichiers sûres.
//!
//! Toute écriture effectuée par VRS passe par [`atomic_write`] : le contenu est
//! d'abord écrit dans un fichier temporaire situé dans le même répertoire, puis
//! renommé. Un arrêt brutal ne laisse donc jamais un fichier à moitié écrit.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Écrit `contents` dans `path` de façon atomique (fichier temporaire + renommage).
///
/// Les répertoires parents sont créés si nécessaire.
pub fn atomic_write(path: &Path, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    std::fs::create_dir_all(&parent)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "chemin sans nom de fichier"))?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(file_name);
    tmp_name.push(format!(".vrs-tmp-{}", std::process::id()));
    let tmp_path = parent.join(tmp_name);

    let result = (|| {
        let mut file = std::fs::File::create(&tmp_path)?;
        file.write_all(contents.as_ref())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp_path, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }
    result
}

/// Copie `path` vers `path.bak` (en remplaçant l'éventuelle copie précédente).
///
/// Ne fait rien si `path` n'existe pas. Retourne le chemin de la sauvegarde créée.
pub fn backup_file(path: &Path) -> io::Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut bak = path.as_os_str().to_owned();
    bak.push(".bak");
    let bak = PathBuf::from(bak);
    std::fs::copy(path, &bak)?;
    Ok(Some(bak))
}

/// Écrit `contents` seulement si le contenu actuel diffère. Retourne `true` si
/// le fichier a été (ré)écrit.
pub fn write_if_changed(path: &Path, contents: &str) -> io::Result<bool> {
    if let Ok(existing) = std::fs::read_to_string(path)
        && existing == contents
    {
        return Ok(false);
    }
    atomic_write(path, contents)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_and_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/c.txt");
        atomic_write(&path, "un").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "un");
        atomic_write(&path, "deux").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "deux");
        // Aucun fichier temporaire ne doit rester.
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("vrs-tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn backup_and_write_if_changed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.rs");
        assert!(backup_file(&path).unwrap().is_none());
        assert!(write_if_changed(&path, "fn main() {}").unwrap());
        assert!(!write_if_changed(&path, "fn main() {}").unwrap());
        let bak = backup_file(&path).unwrap().unwrap();
        assert_eq!(std::fs::read_to_string(bak).unwrap(), "fn main() {}");
    }
}
