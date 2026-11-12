//! Render job queue (port of queue.py): tokio workers, job state in `runs/render_queue/<id>.json`
//! (same files/shape as Python + progress fields), cancel kills the child process group.
use anyhow::{anyhow, bail, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use super::render::{render_request, RenderCtx, RenderRequest};
use super::util::{atomic_write, is_valid_slug, now_iso};
use crate::workspace;

fn d_queued() -> String {
    "queued".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    #[serde(default)]
    pub project_id: String,
    /// queued | rendering | done | failed
    #[serde(default = "d_queued")]
    pub status: String,
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub created_at: String,
    // --- additive fields (Python's pydantic model ignores unknown keys) ---
    #[serde(default)]
    pub progress: f64,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub engines: HashMap<String, String>,
    #[serde(default)]
    pub verify: HashMap<String, Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
}

pub struct RenderQueue {
    sem: Arc<Semaphore>,
    tokens: Mutex<HashMap<String, CancellationToken>>,
}

static QUEUE: Lazy<RenderQueue> = Lazy::new(|| {
    let n = std::env::var("SOCNHOTV_RENDER_PARALLEL").ok().and_then(|v| v.parse().ok()).unwrap_or(2usize).max(1);
    RenderQueue { sem: Arc::new(Semaphore::new(n)), tokens: Mutex::new(HashMap::new()) }
});

pub fn queue() -> &'static RenderQueue {
    &QUEUE
}

fn dir(project: &str) -> Result<PathBuf> {
    if !is_valid_slug(project) {
        bail!("project_id không hợp lệ");
    }
    workspace::project_path(project, &["runs", "render_queue"], false)
}

fn job_file(project: &str, job: &str) -> Result<PathBuf> {
    if !is_valid_slug(job) {
        bail!("job_id không hợp lệ");
    }
    Ok(dir(project)?.join(format!("{job}.json")))
}

fn save(job: &Job) -> Result<()> {
    atomic_write(&job_file(&job.project_id, &job.job_id)?, &serde_json::to_string_pretty(job)?)
}

fn event(project: &str, typ: &str, payload: Value) {
    if let Ok(p) = workspace::project_path(project, &["runs", "events", "render.jsonl"], true) {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{}", json!({"ts": now_iso(), "source": "B", "type": typ, "payload": payload}));
        }
    }
}

impl RenderQueue {
    pub fn submit(&'static self, mut req: RenderRequest, project: &str) -> Result<Job> {
        req.project_id = project.to_string();
        if req.platforms.is_empty() {
            bail!("platforms không được rỗng");
        }
        for p in &req.platforms {
            super::platforms::get_platform(p)?;
        }
        if req.output_name.trim().is_empty() {
            bail!("output_name là bắt buộc");
        }
        let job_id = format!("render_{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
        let job = Job {
            job_id: job_id.clone(),
            project_id: project.to_string(),
            status: "queued".into(),
            outputs: HashMap::new(),
            error: None,
            created_at: now_iso(),
            progress: 0.0,
            stage: "queued".into(),
            engines: HashMap::new(),
            verify: HashMap::new(),
            warnings: vec![],
            finished_at: None,
        };
        save(&job)?;
        atomic_write(&dir(project)?.join(format!("{job_id}.req.json")), &serde_json::to_string_pretty(&req)?)?;
        event(project, "render_queued", json!({"job_id": job_id, "template_id": req.template_id, "platforms": req.platforms, "output_name": req.output_name}));
        let token = CancellationToken::new();
        self.tokens.lock().unwrap().insert(job_id.clone(), token.clone());
        let j = job.clone();
        tokio::spawn(async move { self.worker(j, req, token).await });
        Ok(job)
    }

    pub fn get(&self, project: &str, job: &str) -> Result<Job> {
        let s = std::fs::read_to_string(job_file(project, job)?).map_err(|_| anyhow!("Job {job} không tồn tại"))?;
        let mut j: Job = serde_json::from_str(&s)?;
        if j.project_id.is_empty() {
            j.project_id = project.to_string();
        }
        Ok(j)
    }

    pub fn list(&self, project: &str, status: Option<&str>) -> Vec<Job> {
        let mut v: Vec<Job> = std::fs::read_dir(dir(project).unwrap_or_default())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                (n.starts_with("render_") && n.ends_with(".json") && !n.ends_with(".req.json")).then_some(e.path())
            })
            .filter_map(|p| serde_json::from_str::<Job>(&std::fs::read_to_string(p).ok()?).ok())
            .filter(|j| status.map(|s| j.status == s).unwrap_or(true))
            .collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }

    pub fn cancel(&self, project: &str, job: &str) -> Result<Job> {
        let mut j = self.get(project, job)?;
        if let Some(t) = self.tokens.lock().unwrap().get(job) {
            t.cancel(); // kills the child process group
        }
        if j.status == "queued" || j.status == "rendering" {
            j.status = "failed".into();
            j.error = Some("Cancelled by user".into());
            j.stage = "cancelled".into();
            j.finished_at = Some(now_iso());
            save(&j)?;
            event(project, "render_cancelled", json!({"job_id": job}));
        }
        Ok(j)
    }

    async fn worker(&'static self, job: Job, req: RenderRequest, token: CancellationToken) {
        let pid = job.project_id.clone();
        let jid = job.job_id.clone();
        let _permit = tokio::select! {
            p = self.sem.clone().acquire_owned() => p.ok(),
            _ = token.cancelled() => None,
        };
        let finish = |this: &Self| {
            this.tokens.lock().unwrap().remove(&jid);
        };
        if token.is_cancelled() || _permit.is_none() {
            finish(self);
            return; // cancel() already persisted the failed state
        }
        let state = Arc::new(Mutex::new(job));
        {
            let mut j = state.lock().unwrap();
            j.status = "rendering".into();
            j.stage = "bắt đầu".into();
            let _ = save(&j);
        }
        event(&pid, "render_started", json!({"job_id": jid}));
        let work_dir = dir(&pid).unwrap_or_default().join(&jid);
        let _ = std::fs::create_dir_all(&work_dir);
        let last = Arc::new(Mutex::new(Instant::now()));
        let st = state.clone();
        let progress: super::render::Progress = Arc::new(move |p, stage| {
            let mut l = last.lock().unwrap();
            if l.elapsed().as_millis() < 400 && p < 100.0 {
                return;
            }
            *l = Instant::now();
            let mut j = st.lock().unwrap();
            if j.status != "rendering" {
                return; // cancelled meanwhile: don't resurrect
            }
            j.progress = (p * 10.0).round() / 10.0;
            j.stage = stage.to_string();
            let _ = save(&j);
        });
        let ctx = RenderCtx { cancel: token.clone(), progress, work_dir: work_dir.clone() };
        let result = render_request(&req, &ctx).await;
        let mut j = state.lock().unwrap();
        match result {
            Ok(r) => {
                j.status = "done".into();
                j.progress = 100.0;
                j.stage = "done".into();
                j.outputs = r.outputs.clone();
                j.engines = r.engines;
                j.verify = r.verify;
                j.warnings = r.warnings;
                event(&pid, "render_completed", json!({"job_id": jid, "outputs": r.outputs}));
                if req.data.get("auto_draft").and_then(|v| v.as_bool()).unwrap_or(false) {
                    event(&pid, "auto_draft_skipped", json!({"job_id": jid, "reason": "social hub (Track E) tạo draft"}));
                }
            }
            Err(e) => {
                j.status = "failed".into();
                j.error = Some(if token.is_cancelled() { "Cancelled by user".into() } else { format!("{e:#}") });
                j.stage = if token.is_cancelled() { "cancelled".into() } else { "failed".into() };
                event(&pid, "render_failed", json!({"job_id": jid, "error": j.error}));
            }
        }
        j.finished_at = Some(now_iso());
        let _ = save(&j);
        drop(j);
        finish(self);
    }
}
