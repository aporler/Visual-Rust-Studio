//! Débogage dans l'IDE : compilation, lancement sous l'adaptateur DAP,
//! points d'arrêt, pas à pas, pile d'appels, variables, espions, paniques.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use vrs_debugger::{DebugEvent, DebugSession, Frame, Thread, Variable};

use crate::app::{BottomTab, LineKind, VrsApp};
use crate::tabs::{Tab, same_path};

/// Requête en attente de réponse.
#[derive(Clone, Debug)]
pub enum DebugPending {
    Threads,
    Stack,
    Scopes,
    Variables(i64),
    Watch(usize),
}

/// État d'une session de débogage.
pub struct DebugState {
    pub session: DebugSession,
    pub thread: Option<i64>,
    pub paused: bool,
    pub frames: Vec<Frame>,
    pub selected_frame: usize,
    pub threads: Vec<Thread>,
    /// Référence des variables locales du cadre sélectionné.
    pub locals: Option<i64>,
    /// Variables chargées, par référence.
    pub variables: HashMap<i64, Vec<Variable>>,
    pub pending: Vec<(i64, DebugPending)>,
    /// Arrêt causé par une panique Rust.
    pub panicked: bool,
}

/// Espion : expression et dernière valeur.
#[derive(Clone, Debug, Default)]
pub struct Watch {
    pub expression: String,
    pub value: Option<String>,
}

impl VrsApp {
    /// Adaptateur de débogage disponible (chemin configuré ou détection).
    pub fn debug_adapter(&self) -> Option<vrs_debugger::Adapter> {
        vrs_debugger::find_adapter(self.settings.build.debugger_path.as_deref())
    }

    pub fn is_debugging(&self) -> bool {
        self.debug.is_some()
    }

    pub fn debug_paused(&self) -> bool {
        self.debug.as_ref().is_some_and(|d| d.paused)
    }

    /// Points d'arrêt de tous les fichiers (onglets ouverts inclus).
    fn collect_breakpoints(&mut self) -> BTreeMap<PathBuf, Vec<usize>> {
        for t in &self.tabs {
            if let Tab::Code(e) = t {
                self.breakpoints
                    .insert(e.path.clone(), e.breakpoints.clone());
            }
        }
        self.breakpoints
            .iter()
            .filter(|(_, l)| !l.is_empty())
            .map(|(p, l)| (p.clone(), l.iter().copied().collect()))
            .collect()
    }

    /// Démarre le débogage : compilation (`cargo build`), puis lancement.
    pub fn start_debugging(&mut self) {
        if self.is_debugging() {
            return;
        }
        if self.debug_adapter().is_none() {
            let msg = self.i18n.tr("debug-no-adapter");
            self.error(msg);
            return;
        }
        self.debug_after_build = true;
        self.execute_cargo(vrs_compiler::CargoCommand::Build);
    }

    /// Appelé à la fin d'une compilation demandée par `start_debugging`.
    pub fn launch_debugger(&mut self) {
        let Some(project) = &self.project else { return };
        let root = project.root.clone();
        let Some(program) = vrs_project::binary_path(&root, self.settings.build.release) else {
            let msg = self.i18n.tr("debug-no-binary");
            self.error(msg);
            return;
        };
        let Some(adapter) = self.debug_adapter() else {
            return;
        };
        let breakpoints = self.collect_breakpoints();
        let ctx = self.repaint_ctx.clone();
        let name = adapter.name();
        match DebugSession::launch(adapter, &program, &root, &breakpoints, move || {
            if let Some(c) = &ctx {
                c.request_repaint();
            }
        }) {
            Ok(session) => {
                self.log(
                    LineKind::Info,
                    self.i18n.trf("debug-started", &[("adapter", name.into())]),
                );
                self.status = self.i18n.tr("debug-running");
                self.show_output = true;
                self.bottom_tab = BottomTab::Debug;
                self.debug = Some(DebugState {
                    session,
                    thread: None,
                    paused: false,
                    frames: Vec::new(),
                    selected_frame: 0,
                    threads: Vec::new(),
                    locals: None,
                    variables: HashMap::new(),
                    pending: Vec::new(),
                    panicked: false,
                });
            }
            Err(e) => {
                let msg = format!("{}\n{e}", self.i18n.tr("debug-no-adapter"));
                self.error(msg);
            }
        }
    }

    /// Arrête la session de débogage.
    pub fn stop_debugging(&mut self) {
        if let Some(d) = self.debug.take() {
            d.session.terminate();
        }
        self.clear_exec_line();
        self.status = self.i18n.tr("debug-stopped");
    }

    fn clear_exec_line(&mut self) {
        for t in &mut self.tabs {
            if let Tab::Code(e) = t {
                e.set_exec_line(None);
            }
        }
    }

    /// Continuer, pas à pas, pause.
    pub fn debug_action(&mut self, action: DebugAction) {
        let Some(d) = &mut self.debug else { return };
        let thread = d.thread.unwrap_or(1);
        let r = match action {
            DebugAction::Continue if d.paused => d.session.continue_(thread),
            DebugAction::StepOver if d.paused => d.session.next(thread),
            DebugAction::StepInto if d.paused => d.session.step_in(thread),
            DebugAction::StepOut if d.paused => d.session.step_out(thread),
            DebugAction::Pause if !d.paused => d.session.pause(thread),
            _ => return,
        };
        if r.is_ok() && action != DebugAction::Pause {
            d.paused = false;
            self.clear_exec_line();
        }
    }

    /// Sélectionne un cadre de la pile : ouvre le fichier et charge ses variables.
    pub fn select_frame(&mut self, index: usize) {
        let Some(d) = &mut self.debug else { return };
        let Some(frame) = d.frames.get(index).cloned() else {
            return;
        };
        d.selected_frame = index;
        d.locals = None;
        if let Ok(id) = d.session.request_scopes(frame.id) {
            d.pending.push((id, DebugPending::Scopes));
        }
        self.refresh_watches();
        if let Some(path) = frame.path.filter(|p| p.is_file()) {
            self.clear_exec_line();
            self.open_file(&path, None);
            if let Some(Tab::Code(e)) = self.tabs.get_mut(self.active) {
                e.set_exec_line(Some(frame.line));
            }
        }
    }

    /// Demande les enfants d'une variable (dépliage).
    pub fn load_variables(&mut self, reference: i64) {
        let Some(d) = &mut self.debug else { return };
        if reference <= 0 || d.variables.contains_key(&reference) {
            return;
        }
        if d.pending
            .iter()
            .any(|(_, p)| matches!(p, DebugPending::Variables(r) if *r == reference))
        {
            return;
        }
        if let Ok(id) = d.session.request_variables(reference) {
            d.pending.push((id, DebugPending::Variables(reference)));
        }
    }

    /// Réévalue les espions dans le cadre sélectionné.
    pub fn refresh_watches(&mut self) {
        let Some(d) = &mut self.debug else { return };
        if !d.paused {
            return;
        }
        let frame = d.frames.get(d.selected_frame).map(|f| f.id);
        for (i, w) in self.watches.iter().enumerate() {
            if w.expression.trim().is_empty() {
                continue;
            }
            if let Ok(id) = d.session.request_evaluate(&w.expression, frame) {
                d.pending.push((id, DebugPending::Watch(i)));
            }
        }
    }

    /// Envoie les points d'arrêt modifiés dans un éditeur.
    fn sync_breakpoints(&mut self) {
        let mut changed: Vec<(PathBuf, BTreeSet<usize>)> = Vec::new();
        for t in &mut self.tabs {
            if let Tab::Code(e) = t
                && e.take_breakpoints_changed()
            {
                changed.push((e.path.clone(), e.breakpoints.clone()));
            }
        }
        for (path, lines) in changed {
            if let Some(d) = &self.debug {
                let list: Vec<usize> = lines.iter().copied().collect();
                let _ = d.session.set_breakpoints(&path, &list);
            }
            self.breakpoints.insert(path, lines);
        }
    }

    /// Applique les points d'arrêt connus à un éditeur qui vient d'être ouvert.
    pub fn restore_breakpoints(&self, path: &Path) -> BTreeSet<usize> {
        self.breakpoints
            .iter()
            .find(|(p, _)| same_path(p, path))
            .map(|(_, l)| l.clone())
            .unwrap_or_default()
    }

    /// Une image : événements et réponses du débogueur.
    pub fn poll_debug(&mut self) {
        self.sync_breakpoints();
        let Some(d) = &mut self.debug else { return };
        let events = d.session.poll();
        let exited = d.session.has_exited();
        let mut log = Vec::new();
        let mut finished = None;
        for ev in events {
            match ev {
                DebugEvent::Stopped {
                    reason, thread_id, ..
                } => {
                    d.paused = true;
                    d.thread = thread_id.or(d.thread).or(Some(1));
                    d.panicked = reason.contains("function breakpoint") || reason == "exception";
                    let t = d.thread.unwrap_or(1);
                    if let Ok(id) = d.session.request_stack(t) {
                        d.pending.push((id, DebugPending::Stack));
                    }
                    if let Ok(id) = d.session.request_threads() {
                        d.pending.push((id, DebugPending::Threads));
                    }
                }
                DebugEvent::Continued => d.paused = false,
                DebugEvent::Output { category, text } => {
                    if category != "telemetry" {
                        let kind = if category == "stderr" {
                            LineKind::Stderr
                        } else {
                            LineKind::Stdout
                        };
                        for l in text.lines() {
                            log.push((kind, l.to_owned()));
                        }
                    }
                }
                DebugEvent::Exited { code } => finished = Some(code),
                DebugEvent::Terminated => finished = finished.or(Some(0)),
                DebugEvent::Initialized => {}
            }
        }
        // Réponses.
        let mut done = Vec::new();
        for (i, (id, _)) in d.pending.iter().enumerate() {
            if let Some(r) = d.session.take_response(*id) {
                done.push((i, r));
            }
        }
        let mut select_top = false;
        let mut watch_values = Vec::new();
        for (i, r) in done.into_iter().rev() {
            let (_, what) = d.pending.remove(i);
            let Ok(body) = r else { continue };
            match what {
                DebugPending::Threads => d.threads = vrs_debugger::parse_threads(&body),
                DebugPending::Stack => {
                    d.frames = vrs_debugger::parse_stack(&body);
                    select_top = true;
                }
                DebugPending::Scopes => {
                    let scopes = vrs_debugger::parse_scopes(&body);
                    d.locals = scopes.first().map(|(_, r)| *r);
                    d.variables.clear();
                    if let Some(r) = d.locals
                        && let Ok(id) = d.session.request_variables(r)
                    {
                        d.pending.push((id, DebugPending::Variables(r)));
                    }
                }
                DebugPending::Variables(r) => {
                    d.variables.insert(r, vrs_debugger::parse_variables(&body));
                }
                DebugPending::Watch(w) => {
                    watch_values.push((w, vrs_debugger::parse_evaluate(&body).0))
                }
            }
        }
        for (kind, line) in log {
            self.log(kind, line);
        }
        for (w, v) in watch_values {
            if let Some(watch) = self.watches.get_mut(w) {
                watch.value = Some(v);
            }
        }
        if select_top {
            // Premier cadre dans le code du projet (utile après une panique).
            let root = self.project.as_ref().map(|p| p.root.clone());
            let d = self.debug.as_ref().expect("session");
            let index = d
                .frames
                .iter()
                .position(|f| {
                    f.path
                        .as_ref()
                        .zip(root.as_ref())
                        .is_some_and(|(p, r)| p.starts_with(r))
                })
                .unwrap_or(0);
            // La raison donnée varie selon l'adaptateur : la pile fait foi.
            let panicked = d.panicked
                || d.frames
                    .iter()
                    .any(|f| f.name.contains("rust_panic") || f.name.contains("panicking"));
            self.status = if panicked {
                self.i18n.tr("debug-panic")
            } else {
                self.i18n.tr("debug-paused")
            };
            self.select_frame(index);
        }
        if let Some(code) = finished.or(exited.then_some(0)) {
            self.log(
                LineKind::Info,
                self.i18n.trf("debug-exited", &[("code", code.into())]),
            );
            self.debug = None;
            self.clear_exec_line();
            self.status = self.i18n.tr("debug-stopped");
        }
    }
}

/// Commande de contrôle de l'exécution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugAction {
    Continue,
    StepOver,
    StepInto,
    StepOut,
    Pause,
}
