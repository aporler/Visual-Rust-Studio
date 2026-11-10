//! Débogage par le protocole DAP (Debug Adapter Protocol).
//!
//! VRS ne réimplémente pas de débogueur : il pilote un adaptateur existant.
//! - **GDB ≥ 14** (`gdb -i dap`) ; les afficheurs Rust de la chaîne d'outils
//!   (`gdb_load_rust_pretty_printers.py`) sont chargés automatiquement ;
//! - **lldb-dap** (LLVM, anciennement `lldb-vscode`) ;
//! - **CodeLLDB** (`codelldb`).
//!
//! Comme pour le client LSP, les échanges sont non bloquants : les événements
//! sont lus par [`DebugSession::poll`], et les réponses aux requêtes avec
//! [`DebugSession::take_response`].

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

/// Adaptateur de débogage trouvé sur le système.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Adapter {
    /// `gdb -i dap` (GDB 14 ou plus récent).
    Gdb(PathBuf),
    /// `lldb-dap` / `lldb-vscode`.
    LldbDap(PathBuf),
    /// `codelldb --port` non utilisé : mode stdio (`codelldb`).
    CodeLldb(PathBuf),
}

impl Adapter {
    /// Nom affiché.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Gdb(_) => "GDB",
            Self::LldbDap(_) => "lldb-dap",
            Self::CodeLldb(_) => "CodeLLDB",
        }
    }
}

fn in_path(name: &str) -> Option<PathBuf> {
    let exe = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(&exe))
        .find(|p| p.is_file())
}

/// Version majeure de GDB (`GNU gdb (…) 16.3` → 16).
fn gdb_major(gdb: &Path) -> Option<u32> {
    let out = Command::new(gdb).arg("--version").output().ok()?;
    let first = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()?
        .to_owned();
    let version = first.split_whitespace().last()?;
    version.split('.').next()?.parse().ok()
}

/// Cherche un adaptateur : chemin configuré, puis lldb-dap, CodeLLDB, GDB ≥ 14.
pub fn find_adapter(configured: Option<&Path>) -> Option<Adapter> {
    let classify = |p: &Path| -> Option<Adapter> {
        let name = p.file_stem()?.to_string_lossy().to_lowercase();
        if name.contains("codelldb") {
            Some(Adapter::CodeLldb(p.to_path_buf()))
        } else if name.contains("lldb") {
            Some(Adapter::LldbDap(p.to_path_buf()))
        } else if gdb_major(p).is_some_and(|v| v >= 14) {
            Some(Adapter::Gdb(p.to_path_buf()))
        } else {
            None
        }
    };
    if let Some(p) = configured
        && p.is_file()
    {
        return classify(p);
    }
    for name in ["lldb-dap", "lldb-vscode", "codelldb"] {
        if let Some(p) = in_path(name) {
            return classify(&p);
        }
    }
    in_path("gdb").and_then(|p| classify(&p))
}

/// Points d'arrêt par fichier (lignes 1-based).
pub type Breakpoints = Vec<(PathBuf, Vec<usize>)>;

/// Événement du débogueur.
#[derive(Clone, Debug, PartialEq)]
pub enum DebugEvent {
    /// L'adaptateur est prêt à recevoir les points d'arrêt.
    Initialized,
    /// Le programme est arrêté (`reason` : breakpoint, step, pause, exception…).
    Stopped {
        reason: String,
        thread_id: Option<i64>,
        text: Option<String>,
    },
    Continued,
    /// Sortie du programme ou du débogueur.
    Output {
        category: String,
        text: String,
    },
    /// Le programme s'est terminé.
    Exited {
        code: i64,
    },
    /// La session est terminée.
    Terminated,
}

/// Cadre de la pile d'appels.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub id: i64,
    pub name: String,
    pub path: Option<PathBuf>,
    /// Ligne 1-based.
    pub line: usize,
}

/// Variable (ou champ) ; `children` > 0 permet de la déplier.
#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    pub children: i64,
}

/// Fil d'exécution.
#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    pub id: i64,
    pub name: String,
}

#[derive(Default)]
struct Shared {
    responses: HashMap<i64, Result<Value, String>>,
    events: VecDeque<DebugEvent>,
    exited: bool,
}

/// Session de débogage.
pub struct DebugSession {
    child: Mutex<Child>,
    stdin: Arc<Mutex<ChildStdin>>,
    seq: AtomicI64,
    shared: Arc<Mutex<Shared>>,
    /// Points d'arrêt à envoyer quand l'adaptateur sera initialisé.
    pending_breakpoints: Mutex<Option<Breakpoints>>,
    pub adapter: Adapter,
}

fn write_message(stdin: &Mutex<ChildStdin>, value: &Value) -> std::io::Result<()> {
    let body = serde_json::to_string(value)?;
    let mut w = stdin.lock().expect("verrou");
    write!(w, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    w.flush()
}

fn read_message(r: &mut impl BufRead) -> Option<Value> {
    let mut len = None;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            if len.is_some() {
                break;
            }
            continue;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            len = v.trim().parse::<usize>().ok();
        }
    }
    let mut buf = vec![0; len?];
    std::io::Read::read_exact(r, &mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

fn parse_event(msg: &Value) -> Option<DebugEvent> {
    let body = &msg["body"];
    Some(match msg["event"].as_str()? {
        "initialized" => DebugEvent::Initialized,
        "stopped" => DebugEvent::Stopped {
            reason: body["reason"].as_str().unwrap_or_default().to_owned(),
            thread_id: body["threadId"].as_i64(),
            text: body["text"]
                .as_str()
                .or_else(|| body["description"].as_str())
                .map(str::to_owned),
        },
        "continued" => DebugEvent::Continued,
        "output" => DebugEvent::Output {
            category: body["category"].as_str().unwrap_or("console").to_owned(),
            text: body["output"].as_str().unwrap_or_default().to_owned(),
        },
        "exited" => DebugEvent::Exited {
            code: body["exitCode"].as_i64().unwrap_or_default(),
        },
        "terminated" => DebugEvent::Terminated,
        _ => return None,
    })
}

/// Chemin du script des afficheurs Rust pour GDB (`String`, `Vec`… lisibles).
fn rust_gdb_printers() -> Option<PathBuf> {
    let rustc = in_path("rustc").or_else(|| {
        let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })?;
        let p = PathBuf::from(home).join(".cargo/bin/rustc");
        p.is_file().then_some(p)
    })?;
    let out = Command::new(rustc)
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let p = PathBuf::from(sysroot).join("lib/rustlib/etc/gdb_load_rust_pretty_printers.py");
    p.is_file().then_some(p)
}

impl DebugSession {
    /// Démarre l'adaptateur et lance `program` (arrêt aux points d'arrêt donnés).
    ///
    /// `breakpoints` : fichier → lignes (1-based). Les paniques Rust arrêtent
    /// le programme (`rust_panic`).
    pub fn launch(
        adapter: Adapter,
        program: &Path,
        cwd: &Path,
        breakpoints: &BTreeMap<PathBuf, Vec<usize>>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> std::io::Result<Self> {
        let mut cmd = match &adapter {
            Adapter::Gdb(p) => {
                let mut c = Command::new(p);
                c.args(["-i", "dap", "-q"]);
                if let Some(printers) = rust_gdb_printers() {
                    if let Some(dir) = printers.parent() {
                        c.arg("-iex")
                            .arg(format!("add-auto-load-safe-path {}", dir.display()));
                        c.arg("-iex").arg(format!(
                            "python import sys; sys.path.insert(0, '{}')",
                            dir.display()
                        ));
                    }
                    c.arg("-iex").arg(format!("source {}", printers.display()));
                }
                c
            }
            Adapter::LldbDap(p) | Adapter::CodeLldb(p) => Command::new(p),
        };
        let mut child = cmd
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let stdout = child.stdout.take().expect("stdout");
        let shared = Arc::new(Mutex::new(Shared::default()));

        let reader_shared = shared.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            while let Some(msg) = read_message(&mut r) {
                let mut s = reader_shared.lock().expect("verrou");
                match msg["type"].as_str() {
                    Some("response") => {
                        if let Some(id) = msg["request_seq"].as_i64() {
                            let r = if msg["success"].as_bool().unwrap_or(false) {
                                Ok(msg.get("body").cloned().unwrap_or(Value::Null))
                            } else {
                                Err(msg["message"].as_str().unwrap_or("erreur").to_owned())
                            };
                            s.responses.insert(id, r);
                        }
                    }
                    Some("event") => {
                        if let Some(e) = parse_event(&msg) {
                            s.events.push_back(e);
                        }
                    }
                    _ => {}
                }
                drop(s);
                wake();
            }
            reader_shared.lock().expect("verrou").exited = true;
            wake();
        });

        let session = Self {
            child: Mutex::new(child),
            stdin,
            seq: AtomicI64::new(1),
            shared,
            pending_breakpoints: Mutex::new(Some(
                breakpoints
                    .iter()
                    .map(|(p, l)| (p.clone(), l.clone()))
                    .collect(),
            )),
            adapter,
        };
        session.request(
            "initialize",
            json!({
                "clientID": "vrs",
                "clientName": "Visual Rust Studio",
                "adapterID": "rust",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "pathFormat": "path",
                "supportsVariableType": true
            }),
        )?;
        session.request(
            "launch",
            json!({
                "program": program,
                "cwd": cwd,
                "args": [],
                "stopAtBeginningOfMainSubprogram": false,
                "stopOnEntry": false
            }),
        )?;
        // Les points d'arrêt sont envoyés à la réception de `initialized` (voir `poll`).
        Ok(session)
    }

    /// Envoie une requête ; retourne son numéro.
    pub fn request(&self, command: &str, arguments: Value) -> std::io::Result<i64> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        write_message(
            &self.stdin,
            &json!({ "seq": seq, "type": "request", "command": command, "arguments": arguments }),
        )?;
        Ok(seq)
    }

    /// Réponse d'une requête, si elle est arrivée.
    pub fn take_response(&self, seq: i64) -> Option<Result<Value, String>> {
        self.shared.lock().expect("verrou").responses.remove(&seq)
    }

    /// Événements reçus depuis le dernier appel. Les points d'arrêt sont
    /// envoyés automatiquement à la réception de `Initialized`.
    pub fn poll(&self) -> Vec<DebugEvent> {
        let events: Vec<DebugEvent> = self
            .shared
            .lock()
            .expect("verrou")
            .events
            .drain(..)
            .collect();
        if events.contains(&DebugEvent::Initialized) {
            let pending = self
                .pending_breakpoints
                .lock()
                .expect("verrou")
                .take()
                .unwrap_or_default();
            for (path, lines) in pending {
                let _ = self.set_breakpoints(&path, &lines);
            }
            let _ = self.request(
                "setFunctionBreakpoints",
                // Arrêt sur panique : nom récent (mangling v0) et ancien nom.
                json!({ "breakpoints": [{ "name": "__rustc::rust_panic" }, { "name": "rust_panic" }] }),
            );
            let _ = self.request("configurationDone", json!({}));
        }
        events
    }

    /// La session est-elle terminée (adaptateur arrêté) ?
    pub fn has_exited(&self) -> bool {
        self.shared.lock().expect("verrou").exited
    }

    /// Remplace les points d'arrêt d'un fichier (lignes 1-based).
    pub fn set_breakpoints(&self, path: &Path, lines: &[usize]) -> std::io::Result<i64> {
        let bps: Vec<Value> = lines.iter().map(|l| json!({ "line": l })).collect();
        self.request(
            "setBreakpoints",
            json!({ "source": { "path": path }, "breakpoints": bps }),
        )
    }

    pub fn continue_(&self, thread: i64) -> std::io::Result<i64> {
        self.request("continue", json!({ "threadId": thread }))
    }

    /// Pas à pas principal (Step Over).
    pub fn next(&self, thread: i64) -> std::io::Result<i64> {
        self.request("next", json!({ "threadId": thread }))
    }

    /// Pas à pas détaillé (Step Into).
    pub fn step_in(&self, thread: i64) -> std::io::Result<i64> {
        self.request("stepIn", json!({ "threadId": thread }))
    }

    /// Pas à pas sortant (Step Out).
    pub fn step_out(&self, thread: i64) -> std::io::Result<i64> {
        self.request("stepOut", json!({ "threadId": thread }))
    }

    pub fn pause(&self, thread: i64) -> std::io::Result<i64> {
        self.request("pause", json!({ "threadId": thread }))
    }

    pub fn request_threads(&self) -> std::io::Result<i64> {
        self.request("threads", json!({}))
    }

    pub fn request_stack(&self, thread: i64) -> std::io::Result<i64> {
        self.request(
            "stackTrace",
            json!({ "threadId": thread, "startFrame": 0, "levels": 50 }),
        )
    }

    pub fn request_scopes(&self, frame: i64) -> std::io::Result<i64> {
        self.request("scopes", json!({ "frameId": frame }))
    }

    pub fn request_variables(&self, reference: i64) -> std::io::Result<i64> {
        self.request("variables", json!({ "variablesReference": reference }))
    }

    /// Évalue une expression (espion) dans un cadre.
    pub fn request_evaluate(&self, expression: &str, frame: Option<i64>) -> std::io::Result<i64> {
        let mut args = json!({ "expression": expression, "context": "watch" });
        if let Some(f) = frame {
            args["frameId"] = json!(f);
        }
        self.request("evaluate", args)
    }

    /// Arrête le programme et la session.
    pub fn terminate(&self) {
        let _ = self.request("disconnect", json!({ "terminateDebuggee": true }));
        if let Ok(mut c) = self.child.lock() {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for DebugSession {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// Décode `threads`.
pub fn parse_threads(body: &Value) -> Vec<Thread> {
    body["threads"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|t| Thread {
                    id: t["id"].as_i64().unwrap_or_default(),
                    name: t["name"].as_str().unwrap_or_default().to_owned(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Décode `stackTrace`.
pub fn parse_stack(body: &Value) -> Vec<Frame> {
    body["stackFrames"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|f| Frame {
                    id: f["id"].as_i64().unwrap_or_default(),
                    name: f["name"].as_str().unwrap_or_default().to_owned(),
                    path: f
                        .pointer("/source/path")
                        .and_then(Value::as_str)
                        .map(PathBuf::from),
                    line: f["line"].as_u64().unwrap_or_default() as usize,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Décode `scopes` : (nom, référence des variables).
pub fn parse_scopes(body: &Value) -> Vec<(String, i64)> {
    body["scopes"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|s| {
                    (
                        s["name"].as_str().unwrap_or_default().to_owned(),
                        s["variablesReference"].as_i64().unwrap_or_default(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Décode `variables`.
pub fn parse_variables(body: &Value) -> Vec<Variable> {
    body["variables"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| Variable {
                    name: v["name"].as_str().unwrap_or_default().to_owned(),
                    value: v["value"].as_str().unwrap_or_default().to_owned(),
                    type_name: v["type"].as_str().map(str::to_owned),
                    children: v["variablesReference"].as_i64().unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Décode `evaluate` : (résultat, référence pour déplier).
pub fn parse_evaluate(body: &Value) -> (String, i64) {
    (
        body["result"].as_str().unwrap_or_default().to_owned(),
        body["variablesReference"].as_i64().unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn framing_and_events() {
        let body =
            r#"{"type":"event","event":"stopped","body":{"reason":"breakpoint","threadId":1}}"#;
        let raw = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let v = read_message(&mut BufReader::new(raw.as_bytes())).unwrap();
        assert_eq!(
            parse_event(&v),
            Some(DebugEvent::Stopped {
                reason: "breakpoint".into(),
                thread_id: Some(1),
                text: None
            })
        );
        let stack = parse_stack(
            &serde_json::json!({"stackFrames":[{"id":3,"name":"main","line":4,"source":{"path":"/x/main.rs"}}]}),
        );
        assert_eq!(stack[0].path.as_deref(), Some(Path::new("/x/main.rs")));
    }

    fn wait_for<T>(secs: u64, mut f: impl FnMut() -> Option<T>) -> Option<T> {
        let deadline = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < deadline {
            if let Some(v) = f() {
                return Some(v);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        None
    }

    /// Avec un vrai débogueur : `VRS_TEST_DEBUGGER=/chemin/gdb cargo test -p vrs-debugger -- --ignored`.
    #[test]
    #[ignore = "nécessite un débogueur (VRS_TEST_DEBUGGER)"]
    fn real_debugger_breakpoint_step_variables_panic() {
        let dbg = std::env::var_os("VRS_TEST_DEBUGGER").map(PathBuf::from);
        let Some(adapter) = find_adapter(dbg.as_deref()) else {
            eprintln!("aucun débogueur");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("main.rs");
        std::fs::write(
            &src,
            "fn double(x: i32) -> i32 {\n    x * 2\n}\n\nfn main() {\n    let nom = String::from(\"Rust\");\n    let n = 21;\n    let r = double(n);\n    println!(\"{nom} {r}\");\n    let v: Vec<i32> = Vec::new();\n    let _ = v[r as usize];\n}\n",
        )
        .unwrap();
        let exe = dir.path().join("prog");
        let rustc = in_path("rustc").unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap()).join(".cargo/bin/rustc")
        });
        let ok = Command::new(rustc)
            .args(["-g", "-o"])
            .arg(&exe)
            .arg(&src)
            .status()
            .unwrap()
            .success();
        assert!(ok);

        let mut bps = BTreeMap::new();
        bps.insert(src.clone(), vec![8]);
        let s = DebugSession::launch(adapter, &exe, dir.path(), &bps, || {}).unwrap();
        let mut all = Vec::new();
        let stopped = wait_for(30, || {
            let ev = s.poll();
            all.extend(ev.clone());
            ev.into_iter().find_map(|e| match e {
                DebugEvent::Stopped {
                    thread_id, reason, ..
                } => Some((thread_id.unwrap_or(1), reason)),
                _ => None,
            })
        });
        let (thread, reason) = stopped.unwrap_or_else(|| panic!("pas d'arrêt : {all:?}"));
        assert_eq!(reason, "breakpoint");

        let id = s.request_stack(thread).unwrap();
        let stack = parse_stack(&wait_for(10, || s.take_response(id)).unwrap().unwrap());
        assert!(stack[0].name.contains("main"), "{stack:?}");
        assert_eq!(stack[0].line, 8);

        let id = s.request_scopes(stack[0].id).unwrap();
        let scopes = parse_scopes(&wait_for(10, || s.take_response(id)).unwrap().unwrap());
        let id = s.request_variables(scopes[0].1).unwrap();
        let vars = parse_variables(&wait_for(10, || s.take_response(id)).unwrap().unwrap());
        let n = vars.iter().find(|v| v.name == "n").expect("variable n");
        assert_eq!(n.value, "21");
        let nom = vars.iter().find(|v| v.name == "nom").expect("variable nom");
        assert!(nom.value.contains("Rust"), "{nom:?}");

        let id = s.request_evaluate("n * 2", Some(stack[0].id)).unwrap();
        let (v, _) = parse_evaluate(&wait_for(10, || s.take_response(id)).unwrap().unwrap());
        assert_eq!(v, "42");

        // Pas à pas : une ligne Rust peut demander plusieurs pas (appel de fonction).
        let mut line = 8;
        for _ in 0..4 {
            s.next(thread).unwrap();
            wait_for(10, || {
                s.poll()
                    .into_iter()
                    .find(|e| matches!(e, DebugEvent::Stopped { .. }))
            })
            .unwrap();
            let id = s.request_stack(thread).unwrap();
            let stack = parse_stack(&wait_for(10, || s.take_response(id)).unwrap().unwrap());
            line = stack[0].line;
            if line != 8 {
                break;
            }
        }
        assert_eq!(line, 9);

        // Continuer : la panique (index hors limites) arrête le programme.
        s.continue_(thread).unwrap();
        let panic_stop = wait_for(20, || {
            s.poll().into_iter().find_map(|e| match e {
                DebugEvent::Stopped { reason, .. } => Some(reason),
                _ => None,
            })
        });
        assert!(panic_stop.is_some(), "pas d'arrêt sur la panique");
        let id = s.request_stack(thread).unwrap();
        let stack = parse_stack(&wait_for(10, || s.take_response(id)).unwrap().unwrap());
        assert!(
            stack
                .iter()
                .any(|f| f.name.contains("rust_panic") || f.name.contains("panic")),
            "{stack:?}"
        );
        assert!(
            stack
                .iter()
                .any(|f| f.line == 11 && f.path.as_deref() == Some(src.as_path())),
            "{stack:?}"
        );
        s.terminate();
    }
}
