//! Lancement de Cargo dans un processus séparé.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::diagnostics::{CargoLine, Diagnostic, parse_cargo_line};

/// Commande Cargo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CargoCommand {
    Build,
    Run,
    Check,
    Clippy,
    Test,
    Clean,
}

impl CargoCommand {
    fn subcommand(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Run => "run",
            Self::Check => "check",
            Self::Clippy => "clippy",
            Self::Test => "test",
            Self::Clean => "clean",
        }
    }

    /// Produit-elle des diagnostics du compilateur ?
    pub fn emits_diagnostics(self) -> bool {
        !matches!(self, Self::Clean)
    }
}

/// Flux d'origine d'une ligne.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Événement envoyé à l'IDE.
#[derive(Debug)]
pub enum BuildEvent {
    /// Ligne de commande lancée.
    Started(String),
    /// Ligne de texte (progression de Cargo, sortie du programme…).
    Line(OutputStream, String),
    Diagnostic(Box<Diagnostic>),
    /// Fin du processus.
    Finished {
        success: bool,
        code: Option<i32>,
        duration: Duration,
        killed: bool,
    },
    /// Impossible de lancer Cargo.
    SpawnError(String),
}

/// Processus en cours.
pub struct BuildHandle {
    pub command: CargoCommand,
    rx: Receiver<BuildEvent>,
    child: Arc<Mutex<Option<Child>>>,
    killed: Arc<Mutex<bool>>,
    finished: bool,
}

impl BuildHandle {
    /// Récupère les événements disponibles sans bloquer.
    pub fn poll(&mut self) -> Vec<BuildEvent> {
        let events: Vec<BuildEvent> = self.rx.try_iter().collect();
        if events
            .iter()
            .any(|e| matches!(e, BuildEvent::Finished { .. } | BuildEvent::SpawnError(_)))
        {
            self.finished = true;
        }
        events
    }

    /// Le processus est-il terminé (et tous ses événements lus) ?
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Arrête le processus.
    pub fn kill(&self) {
        *self.killed.lock().expect("verrou") = true;
        if let Some(child) = self.child.lock().expect("verrou").as_mut() {
            let _ = child.kill();
        }
    }
}

/// Cherche un outil de la chaîne Rust (`cargo`, `rustfmt`…) : chemin configuré,
/// `PATH`, puis `~/.cargo/bin`.
pub fn find_tool(name: &str, configured: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = configured
        && p.is_file()
    {
        return Some(p.to_path_buf());
    }
    let exe = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(&exe);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                .map(|h| PathBuf::from(h).join(".cargo"))
        })?;
    let candidate = cargo_home.join("bin").join(&exe);
    candidate.is_file().then_some(candidate)
}

/// Cherche `cargo`.
pub fn find_cargo(configured: Option<&Path>) -> Option<PathBuf> {
    find_tool("cargo", configured)
}

/// Lance `cargo <commande>` dans `manifest_dir`. `wake` est appelé à chaque
/// événement (pour redessiner l'interface).
pub fn spawn(
    cargo: &Path,
    manifest_dir: &Path,
    command: CargoCommand,
    release: bool,
    wake: impl Fn() + Send + Sync + 'static,
) -> BuildHandle {
    let (tx, rx) = channel();
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(wake);
    let child_slot: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let killed = Arc::new(Mutex::new(false));

    let mut cmd = Command::new(cargo);
    cmd.arg(command.subcommand());
    if command.emits_diagnostics() {
        cmd.arg("--message-format=json");
    }
    if release && command != CargoCommand::Clean {
        cmd.arg("--release");
    }
    if command == CargoCommand::Clean && release {
        cmd.arg("--release");
    }
    cmd.current_dir(manifest_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("CARGO_TERM_COLOR", "never")
        .env("CARGO_TERM_PROGRESS_WHEN", "never");

    let cmdline = format!(
        "cargo {}{}",
        command.subcommand(),
        if release { " --release" } else { "" }
    );
    let _ = tx.send(BuildEvent::Started(cmdline));
    let start = Instant::now();

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(BuildEvent::SpawnError(format!("{}: {e}", cargo.display())));
            wake();
            return BuildHandle {
                command,
                rx,
                child: child_slot,
                killed,
                finished: false,
            };
        }
    };

    let root = manifest_dir.to_path_buf();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    *child_slot.lock().expect("verrou") = Some(child);

    let mut readers = Vec::new();
    if let Some(out) = stdout {
        let tx = tx.clone();
        let wake = wake.clone();
        let root = root.clone();
        readers.push(std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                let event = match parse_cargo_line(&line, &root) {
                    CargoLine::Diagnostic(d) => BuildEvent::Diagnostic(d),
                    CargoLine::Text => BuildEvent::Line(OutputStream::Stdout, line),
                    CargoLine::BuildFinished(_) | CargoLine::Ignored => continue,
                };
                if tx.send(event).is_err() {
                    break;
                }
                wake();
            }
        }));
    }
    if let Some(err) = stderr {
        let tx = tx.clone();
        let wake = wake.clone();
        readers.push(std::thread::spawn(move || {
            for line in BufReader::new(err).lines().map_while(Result::ok) {
                if tx
                    .send(BuildEvent::Line(OutputStream::Stderr, line))
                    .is_err()
                {
                    break;
                }
                wake();
            }
        }));
    }

    let slot = child_slot.clone();
    let killed_flag = killed.clone();
    std::thread::spawn(move || {
        let status = loop {
            {
                let mut guard = slot.lock().expect("verrou");
                match guard.as_mut().map(|c| c.try_wait()) {
                    Some(Ok(Some(status))) => break Some(status),
                    Some(Ok(None)) => {}
                    Some(Err(_)) | None => break None,
                }
            }
            std::thread::sleep(Duration::from_millis(40));
        };
        for r in readers {
            let _ = r.join();
        }
        let killed = *killed_flag.lock().expect("verrou");
        let _ = tx.send(BuildEvent::Finished {
            success: status.is_some_and(|s| s.success()) && !killed,
            code: status.and_then(|s| s.code()),
            duration: start.elapsed(),
            killed,
        });
        wake();
    });

    BuildHandle {
        command,
        rx,
        child: child_slot,
        killed,
        finished: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wait(handle: &mut BuildHandle) -> Vec<BuildEvent> {
        let mut all = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(120);
        while !handle.is_finished() && Instant::now() < deadline {
            all.extend(handle.poll());
            std::thread::sleep(Duration::from_millis(20));
        }
        all
    }

    fn project(src: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), src).unwrap();
        dir
    }

    #[test]
    fn check_reports_type_error() {
        let Some(cargo) = find_cargo(None) else {
            eprintln!("cargo introuvable : test ignoré");
            return;
        };
        let dir = project("fn main() {\n    let x: String = 5;\n    let _ = x;\n}\n");
        let mut h = spawn(&cargo, dir.path(), CargoCommand::Check, false, || {});
        let events = wait(&mut h);
        let diag = events
            .iter()
            .find_map(|e| match e {
                BuildEvent::Diagnostic(d) if d.code.as_deref() == Some("E0308") => Some(d),
                _ => None,
            })
            .expect("E0308 attendu");
        let span = diag.span.as_ref().unwrap();
        assert_eq!(span.line_start, 2);
        assert!(span.file.ends_with("src/main.rs"));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, BuildEvent::Finished { success: false, .. }))
        );
    }

    #[test]
    fn run_captures_program_output() {
        let Some(cargo) = find_cargo(None) else {
            return;
        };
        let dir = project("fn main() { println!(\"bonjour-vrs\"); }\n");
        let mut h = spawn(&cargo, dir.path(), CargoCommand::Run, false, || {});
        let events = wait(&mut h);
        assert!(
            events.iter().any(
                |e| matches!(e, BuildEvent::Line(OutputStream::Stdout, l) if l == "bonjour-vrs")
            )
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, BuildEvent::Finished { success: true, .. }))
        );
    }

    #[test]
    fn kill_stops_process() {
        let Some(cargo) = find_cargo(None) else {
            return;
        };
        let dir = project(
            "fn main() { println!(\"start\"); std::thread::sleep(std::time::Duration::from_secs(60)); }\n",
        );
        let mut h = spawn(&cargo, dir.path(), CargoCommand::Run, false, || {});
        let deadline = Instant::now() + Duration::from_secs(120);
        let mut started = false;
        while !started && Instant::now() < deadline {
            started = h
                .poll()
                .iter()
                .any(|e| matches!(e, BuildEvent::Line(_, l) if l == "start"));
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(started);
        let t = Instant::now();
        h.kill();
        let events = wait(&mut h);
        assert!(t.elapsed() < Duration::from_secs(10));
        assert!(events.iter().any(|e| matches!(
            e,
            BuildEvent::Finished {
                killed: true,
                success: false,
                ..
            }
        )));
    }

    #[test]
    fn missing_cargo_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let mut h = spawn(
            Path::new("/nonexistent/cargo"),
            dir.path(),
            CargoCommand::Build,
            false,
            || {},
        );
        let events = h.poll();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, BuildEvent::SpawnError(_)))
        );
        assert!(h.is_finished());
    }
}
