//! Client LSP minimal pour rust-analyzer.
//!
//! Le serveur tourne dans un processus séparé ; les échanges JSON-RPC passent
//! par son entrée/sortie standard. Toutes les opérations sont **non bloquantes**
//! pour l'interface : une requête retourne un identifiant, et la réponse est
//! récupérée plus tard avec [`LspClient::take_response`].
//!
//! Les positions échangées avec l'IDE sont des (ligne, colonne) 0-based en
//! **caractères** ; la conversion vers l'encodage négocié avec le serveur
//! (UTF-32 de préférence, sinon UTF-16) est faite ici.

mod convert;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

pub use convert::{Encoding, path_to_uri, uri_to_path};

/// Gravité d'un diagnostic LSP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Position (ligne et colonne 0-based, colonne en caractères).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

/// Plage de texte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Range {
    pub start: Pos,
    pub end: Pos,
}

/// Diagnostic publié par le serveur.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Severity,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
}

/// Proposition d'autocomplétion.
#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    pub label: String,
    /// Texte à insérer (sans les marqueurs d'extrait `${1:…}`), `$0` pour le curseur.
    pub insert: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    /// Genre LSP (2 méthode, 3 fonction, 5 champ, 14 mot-clé, 15 extrait…).
    pub kind: u32,
}

/// Emplacement (fichier, plage).
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub path: PathBuf,
    pub range: Range,
}

/// Modification de texte (renommage).
#[derive(Clone, Debug, PartialEq)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Événements internes partagés avec le fil de lecture.
#[derive(Default)]
struct Shared {
    responses: HashMap<i64, Result<Value, String>>,
    diagnostics: HashMap<PathBuf, Vec<Value>>,
    diagnostics_generation: u64,
    initialized: bool,
    exited: bool,
    /// Tâches de fond en cours (indexation…).
    progress: HashMap<String, String>,
}

/// Client connecté à un serveur de langage.
pub struct LspClient {
    child: Mutex<Child>,
    stdin: Arc<Mutex<ChildStdin>>,
    next_id: AtomicI64,
    shared: Arc<Mutex<Shared>>,
    encoding: Mutex<Encoding>,
    init_id: i64,
    root: PathBuf,
    versions: Mutex<HashMap<PathBuf, (i64, String)>>,
}

fn write_message(stdin: &Mutex<ChildStdin>, value: &Value) -> std::io::Result<()> {
    let body = serde_json::to_string(value)?;
    let mut w = stdin.lock().expect("verrou");
    write!(w, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    w.flush()
}

/// Lit un message (en-têtes `Content-Length`, puis corps JSON).
fn read_message(r: &mut impl BufRead) -> Option<Value> {
    let mut len = None;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            len = v.trim().parse::<usize>().ok();
        }
    }
    let mut buf = vec![0; len?];
    r.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

impl LspClient {
    /// Démarre le serveur pour le projet `root`.
    ///
    /// `trusted` : si `false`, l'exécution des scripts `build.rs` et des macros
    /// procédurales est désactivée (code non approuvé). `wake` est appelé à
    /// chaque message reçu (pour redessiner l'interface).
    pub fn start(
        server: &Path,
        root: &Path,
        trusted: bool,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> std::io::Result<Self> {
        let mut child = Command::new(server)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let stdout = child.stdout.take().expect("stdout");
        let shared = Arc::new(Mutex::new(Shared::default()));

        let reader_shared = shared.clone();
        let reader_stdin = stdin.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            while let Some(msg) = read_message(&mut r) {
                handle_incoming(&msg, &reader_shared, &reader_stdin);
                wake();
            }
            reader_shared.lock().expect("verrou").exited = true;
            wake();
        });

        let client = Self {
            child: Mutex::new(child),
            stdin,
            next_id: AtomicI64::new(1),
            shared,
            encoding: Mutex::new(Encoding::Utf16),
            init_id: 1,
            root: root.to_path_buf(),
            versions: Mutex::new(HashMap::new()),
        };
        let init_id = client.next_id.fetch_add(1, Ordering::SeqCst);
        debug_assert_eq!(init_id, client.init_id);
        let options = if trusted {
            json!({ "checkOnSave": true })
        } else {
            json!({
                "cargo": { "buildScripts": { "enable": false } },
                "procMacro": { "enable": false },
                "checkOnSave": false
            })
        };
        write_message(
            &client.stdin,
            &json!({
                "jsonrpc": "2.0",
                "id": init_id,
                "method": "initialize",
                "params": {
                    "processId": std::process::id(),
                    "rootUri": path_to_uri(root),
                    "workspaceFolders": [{ "uri": path_to_uri(root), "name": "root" }],
                    "initializationOptions": options,
                    "capabilities": {
                        "general": { "positionEncodings": ["utf-32", "utf-16"] },
                        "window": { "workDoneProgress": true },
                        "textDocument": {
                            "synchronization": { "didSave": true },
                            "completion": {
                                "completionItem": {
                                    "snippetSupport": true,
                                    "documentationFormat": ["plaintext", "markdown"]
                                }
                            },
                            "hover": { "contentFormat": ["plaintext", "markdown"] },
                            "definition": {},
                            "rename": { "prepareSupport": false },
                            "publishDiagnostics": { "relatedInformation": false }
                        },
                        "workspace": { "workspaceEdit": { "documentChanges": false } }
                    }
                }
            }),
        )?;
        Ok(client)
    }

    /// Répertoire du projet.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Le serveur a-t-il répondu à l'initialisation ?
    pub fn is_ready(&mut self) -> bool {
        let mut s = self.shared.lock().expect("verrou");
        if s.initialized {
            return true;
        }
        if let Some(resp) = s.responses.remove(&self.init_id) {
            if let Ok(result) = resp {
                let enc = result
                    .pointer("/capabilities/positionEncoding")
                    .and_then(Value::as_str)
                    .unwrap_or("utf-16");
                *self.encoding.lock().expect("verrou") = if enc == "utf-32" {
                    Encoding::Utf32
                } else if enc == "utf-8" {
                    Encoding::Utf8
                } else {
                    Encoding::Utf16
                };
            }
            s.initialized = true;
            drop(s);
            let _ = self.notify("initialized", json!({}));
            // Ouvrir les documents déjà connus.
            let docs: Vec<(PathBuf, String)> = self
                .versions
                .lock()
                .expect("verrou")
                .iter()
                .map(|(p, (_, t))| (p.clone(), t.clone()))
                .collect();
            for (p, t) in docs {
                let _ = self.send_open(&p, 1, &t);
            }
            return true;
        }
        false
    }

    /// Le serveur s'est-il arrêté ?
    pub fn has_exited(&self) -> bool {
        self.shared.lock().expect("verrou").exited
    }

    /// Tâches en cours côté serveur (indexation, vérification…).
    pub fn progress(&self) -> Vec<String> {
        let s = self.shared.lock().expect("verrou");
        let mut v: Vec<String> = s.progress.values().cloned().collect();
        v.sort();
        v
    }

    fn notify(&self, method: &str, params: Value) -> std::io::Result<()> {
        write_message(
            &self.stdin,
            &json!({ "jsonrpc": "2.0", "method": method, "params": params }),
        )
    }

    fn request(&self, method: &str, params: Value) -> Option<i64> {
        if !self.shared.lock().expect("verrou").initialized {
            return None;
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        write_message(
            &self.stdin,
            &json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
        )
        .ok()?;
        Some(id)
    }

    /// Réponse d'une requête, si elle est arrivée.
    pub fn take_response(&self, id: i64) -> Option<Result<Value, String>> {
        self.shared.lock().expect("verrou").responses.remove(&id)
    }

    fn send_open(&self, path: &Path, version: i64, text: &str) -> std::io::Result<()> {
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": {
                "uri": path_to_uri(path), "languageId": "rust", "version": version, "text": text
            }}),
        )
    }

    /// Signale l'ouverture ou la modification d'un document (texte complet).
    pub fn sync_document(&self, path: &Path, text: &str) {
        let mut versions = self.versions.lock().expect("verrou");
        let ready = self.shared.lock().expect("verrou").initialized;
        match versions.get_mut(path) {
            Some((v, t)) if t == text => {
                let _ = v;
            }
            Some((v, t)) => {
                *v += 1;
                *t = text.to_owned();
                if ready {
                    let _ = self.notify(
                        "textDocument/didChange",
                        json!({
                            "textDocument": { "uri": path_to_uri(path), "version": *v },
                            "contentChanges": [{ "text": text }]
                        }),
                    );
                }
            }
            None => {
                versions.insert(path.to_path_buf(), (1, text.to_owned()));
                if ready {
                    let _ = self.send_open(path, 1, text);
                }
            }
        }
    }

    /// Signale l'enregistrement (déclenche la vérification par cargo).
    pub fn did_save(&self, path: &Path) {
        let _ = self.notify(
            "textDocument/didSave",
            json!({ "textDocument": { "uri": path_to_uri(path) } }),
        );
    }

    /// Signale la fermeture d'un document.
    pub fn did_close(&self, path: &Path) {
        if self.versions.lock().expect("verrou").remove(path).is_some() {
            let _ = self.notify(
                "textDocument/didClose",
                json!({ "textDocument": { "uri": path_to_uri(path) } }),
            );
        }
    }

    fn text_of(&self, path: &Path) -> Option<String> {
        self.versions
            .lock()
            .expect("verrou")
            .get(path)
            .map(|(_, t)| t.clone())
    }

    fn position(&self, path: &Path, pos: Pos) -> Value {
        let enc = *self.encoding.lock().expect("verrou");
        let text = self.text_of(path).unwrap_or_default();
        let col = convert::char_col_to_encoded(&text, pos.line, pos.col, enc);
        json!({ "line": pos.line, "character": col })
    }

    fn text_position(&self, path: &Path, pos: Pos) -> Value {
        json!({ "textDocument": { "uri": path_to_uri(path) }, "position": self.position(path, pos) })
    }

    /// Demande des propositions d'autocomplétion.
    pub fn request_completion(&self, path: &Path, pos: Pos) -> Option<i64> {
        self.request("textDocument/completion", self.text_position(path, pos))
    }

    /// Demande l'infobulle (type, documentation) à une position.
    pub fn request_hover(&self, path: &Path, pos: Pos) -> Option<i64> {
        self.request("textDocument/hover", self.text_position(path, pos))
    }

    /// Demande la définition du symbole à une position.
    pub fn request_definition(&self, path: &Path, pos: Pos) -> Option<i64> {
        self.request("textDocument/definition", self.text_position(path, pos))
    }

    /// Demande le renommage d'un symbole.
    pub fn request_rename(&self, path: &Path, pos: Pos, new_name: &str) -> Option<i64> {
        let mut params = self.text_position(path, pos);
        params["newName"] = json!(new_name);
        self.request("textDocument/rename", params)
    }

    fn decode_range(&self, path: &Path, v: &Value) -> Range {
        let enc = *self.encoding.lock().expect("verrou");
        let text = self
            .text_of(path)
            .or_else(|| std::fs::read_to_string(path).ok())
            .unwrap_or_default();
        let get = |p: &str| {
            let line = v
                .pointer(&format!("/{p}/line"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            let ch = v
                .pointer(&format!("/{p}/character"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            Pos {
                line,
                col: convert::encoded_col_to_char(&text, line, ch, enc),
            }
        };
        Range {
            start: get("start"),
            end: get("end"),
        }
    }

    /// Diagnostics actuels d'un fichier et numéro de génération (change à
    /// chaque publication du serveur).
    pub fn diagnostics(&self, path: &Path) -> (u64, Vec<Diagnostic>) {
        let (generation, raw) = {
            let s = self.shared.lock().expect("verrou");
            (
                s.diagnostics_generation,
                s.diagnostics.get(path).cloned().unwrap_or_default(),
            )
        };
        let list = raw
            .iter()
            .map(|d| Diagnostic {
                range: self.decode_range(path, &d["range"]),
                severity: match d["severity"].as_u64() {
                    Some(1) => Severity::Error,
                    Some(2) => Severity::Warning,
                    Some(3) => Severity::Information,
                    _ => Severity::Hint,
                },
                code: match &d["code"] {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                },
                source: d["source"].as_str().map(str::to_owned),
                message: d["message"].as_str().unwrap_or_default().to_owned(),
            })
            .collect();
        (generation, list)
    }

    /// Décode une réponse d'autocomplétion.
    pub fn parse_completion(&self, result: &Value) -> Vec<Completion> {
        let items = match result {
            Value::Array(a) => a.clone(),
            Value::Object(_) => result["items"].as_array().cloned().unwrap_or_default(),
            _ => Vec::new(),
        };
        items
            .iter()
            .take(200)
            .map(|i| {
                let label = i["label"].as_str().unwrap_or_default().to_owned();
                let raw = i
                    .pointer("/textEdit/newText")
                    .or(i.get("insertText"))
                    .and_then(Value::as_str)
                    .unwrap_or(&label)
                    .to_owned();
                let insert = if i["insertTextFormat"].as_u64() == Some(2) {
                    convert::strip_snippet(&raw)
                } else {
                    raw
                };
                Completion {
                    label,
                    insert,
                    detail: i["detail"].as_str().map(str::to_owned),
                    documentation: markup_text(&i["documentation"]),
                    kind: i["kind"].as_u64().unwrap_or(1) as u32,
                }
            })
            .collect()
    }

    /// Décode une réponse d'infobulle.
    pub fn parse_hover(&self, result: &Value) -> Option<String> {
        let text = markup_text(&result["contents"])?;
        let cleaned: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n");
        let cleaned = cleaned.trim().to_owned();
        (!cleaned.is_empty()).then_some(cleaned)
    }

    /// Décode une réponse de définition.
    pub fn parse_locations(&self, result: &Value) -> Vec<Location> {
        let items = match result {
            Value::Array(a) => a.clone(),
            Value::Object(_) => vec![result.clone()],
            _ => Vec::new(),
        };
        items
            .iter()
            .filter_map(|l| {
                let uri = l["uri"].as_str().or_else(|| l["targetUri"].as_str())?;
                let path = uri_to_path(uri)?;
                let range = l.get("range").or_else(|| l.get("targetSelectionRange"))?;
                Some(Location {
                    range: self.decode_range(&path, range),
                    path,
                })
            })
            .collect()
    }

    /// Décode un `WorkspaceEdit` (renommage) : modifications par fichier.
    pub fn parse_workspace_edit(&self, result: &Value) -> Vec<(PathBuf, Vec<TextEdit>)> {
        let mut out = Vec::new();
        if let Some(changes) = result["changes"].as_object() {
            for (uri, edits) in changes {
                if let Some(path) = uri_to_path(uri) {
                    out.push((path.clone(), self.edits(&path, edits)));
                }
            }
        }
        if let Some(docs) = result["documentChanges"].as_array() {
            for d in docs {
                if let Some(path) = d
                    .pointer("/textDocument/uri")
                    .and_then(Value::as_str)
                    .and_then(uri_to_path)
                {
                    out.push((path.clone(), self.edits(&path, &d["edits"])));
                }
            }
        }
        out
    }

    fn edits(&self, path: &Path, edits: &Value) -> Vec<TextEdit> {
        edits
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|e| TextEdit {
                        range: self.decode_range(path, &e["range"]),
                        new_text: e["newText"].as_str().unwrap_or_default().to_owned(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Arrête le serveur.
    pub fn shutdown(&self) {
        let _ = self.request("shutdown", Value::Null);
        let _ = self.notify("exit", Value::Null);
        if let Ok(mut c) = self.child.lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn markup_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("value").and_then(Value::as_str).map(str::to_owned),
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().filter_map(markup_text).collect();
            (!parts.is_empty()).then(|| parts.join("\n"))
        }
        _ => None,
    }
}

/// Traite un message du serveur (réponse, notification ou requête).
fn handle_incoming(msg: &Value, shared: &Mutex<Shared>, stdin: &Mutex<ChildStdin>) {
    let method = msg["method"].as_str();
    match (msg.get("id"), method) {
        // Réponse à une de nos requêtes.
        (Some(id), None) => {
            if let Some(id) = id.as_i64() {
                let r = match msg.get("error") {
                    Some(e) => Err(e["message"].as_str().unwrap_or("erreur").to_owned()),
                    None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
                };
                shared.lock().expect("verrou").responses.insert(id, r);
            }
        }
        // Requête du serveur : réponse neutre.
        (Some(id), Some(m)) => {
            let result = match m {
                "workspace/configuration" => {
                    let n = msg
                        .pointer("/params/items")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);
                    Value::Array(vec![Value::Null; n])
                }
                _ => Value::Null,
            };
            let _ = write_message(
                stdin,
                &json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            );
        }
        (None, Some("textDocument/publishDiagnostics")) => {
            let uri = msg
                .pointer("/params/uri")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if let Some(path) = uri_to_path(uri) {
                let list = msg
                    .pointer("/params/diagnostics")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let mut s = shared.lock().expect("verrou");
                s.diagnostics.insert(path, list);
                s.diagnostics_generation += 1;
            }
        }
        (None, Some("$/progress")) => {
            let token = msg
                .pointer("/params/token")
                .map(|t| t.to_string())
                .unwrap_or_default();
            let kind = msg.pointer("/params/value/kind").and_then(Value::as_str);
            let title = msg.pointer("/params/value/title").and_then(Value::as_str);
            let mut s = shared.lock().expect("verrou");
            match kind {
                Some("begin") => {
                    s.progress
                        .insert(token, title.unwrap_or_default().to_owned());
                }
                Some("end") => {
                    s.progress.remove(&token);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn rust_analyzer() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        let p = PathBuf::from(home).join(".cargo/bin/rust-analyzer");
        let ok = Command::new(&p)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        ok.then_some(p)
    }

    fn wait<T>(timeout: u64, mut f: impl FnMut() -> Option<T>) -> Option<T> {
        let deadline = Instant::now() + Duration::from_secs(timeout);
        while Instant::now() < deadline {
            if let Some(v) = f() {
                return Some(v);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        None
    }

    #[test]
    fn framing_roundtrip() {
        let body = r#"{"jsonrpc":"2.0","id":1,"result":null}"#;
        let raw = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let mut r = BufReader::new(raw.as_bytes());
        let v = read_message(&mut r).unwrap();
        assert_eq!(v["id"], 1);
    }

    /// Avec le vrai rust-analyzer : complétion, survol, définition, diagnostics.
    #[test]
    #[ignore = "lent : lance rust-analyzer"]
    fn real_rust_analyzer() {
        let Some(ra) = rust_analyzer() else {
            eprintln!("rust-analyzer absent");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        std::fs::create_dir(dir.path().join("src")).unwrap();
        let main = dir.path().join("src/main.rs");
        let text = "/// Calcule le double.\nfn double(x: i32) -> i32 {\n    x * 2\n}\n\nfn main() {\n    let été = String::new();\n    let n: i32 = \"x\";\n    println!(\"{}\", double(n) + été.len() as i32);\n    été.\n}\n";
        std::fs::write(&main, text).unwrap();
        let mut c = LspClient::start(&ra, dir.path(), true, || {}).unwrap();
        assert!(wait(60, || c.is_ready().then_some(())).is_some());
        c.sync_document(&main, text);

        // Survol de `double` (ligne 8, après « été » : vérifie la conversion des colonnes).
        let line8 = text.lines().nth(8).unwrap();
        let col = line8.chars().collect::<String>().find("double").unwrap();
        let col = line8[..col].chars().count();
        let hover = wait(120, || {
            let id = c.request_hover(
                &main,
                Pos {
                    line: 8,
                    col: col + 1,
                },
            )?;
            let r = wait(20, || c.take_response(id))?.ok()?;
            c.parse_hover(&r)
        });
        let hover = hover.expect("infobulle");
        assert!(hover.contains("fn double(x: i32) -> i32"), "{hover}");
        assert!(hover.contains("Calcule le double"), "{hover}");

        // Définition : ligne 1.
        let id = c
            .request_definition(
                &main,
                Pos {
                    line: 8,
                    col: col + 1,
                },
            )
            .unwrap();
        let r = wait(30, || c.take_response(id)).unwrap().unwrap();
        let locs = c.parse_locations(&r);
        assert_eq!(locs[0].range.start.line, 1);

        // Complétion après `été.`
        let line9 = text.lines().nth(9).unwrap();
        let id = c
            .request_completion(
                &main,
                Pos {
                    line: 9,
                    col: line9.chars().count(),
                },
            )
            .unwrap();
        let r = wait(30, || c.take_response(id)).unwrap().unwrap();
        let items = c.parse_completion(&r);
        assert!(
            items.iter().any(|i| i.label.starts_with("push_str")),
            "{:?}",
            items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );

        // Diagnostic de type (rustc via cargo check, ou natif).
        c.did_save(&main);
        let diags = wait(240, || {
            let (_, d) = c.diagnostics(&main);
            d.iter()
                .any(|d| d.range.start.line == 7 && d.severity == Severity::Error)
                .then_some(d)
        });
        assert!(diags.is_some(), "aucun diagnostic sur la ligne 8");

        // Renommage de `double`.
        let id = c
            .request_rename(&main, Pos { line: 1, col: 4 }, "doubler")
            .unwrap();
        let r = wait(30, || c.take_response(id)).unwrap().unwrap();
        let edits = c.parse_workspace_edit(&r);
        assert_eq!(edits[0].1.len(), 2);
        c.shutdown();
    }
}
