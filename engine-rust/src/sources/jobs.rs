//! In-memory ingest job queue (3 workers via semaphore) + events appended to runs/events/sources.jsonl.
use super::downloader::download_source;
use super::scenes::detect_and_save_scenes;
use crate::intelligence::util::*;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

static JOBS: Lazy<Mutex<HashMap<String, Value>>> = Lazy::new(Default::default);
static SEM: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(3)));

fn emit(pid: &str, ty: &str, payload: Value) {
    let p = pp(pid, &["runs", "events", "sources.jsonl"], true);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
        let _ = writeln!(f, "{}", json!({"ts": now_iso(), "source": "B", "type": ty, "payload": payload}));
    }
}

fn set(id: &str, kv: &[(&str, Value)]) {
    if let Some(j) = JOBS.lock().unwrap().get_mut(id) { for (k, v) in kv { j[*k] = v.clone(); } }
}

pub fn enqueue_ingest(pid: &str, urls: &[String], rights: &str, note: &str) -> Vec<String> {
    let mut ids = vec![];
    for url in urls.iter().map(|u| u.trim()).filter(|u| !u.is_empty()) {
        let id = format!("job_{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
        JOBS.lock().unwrap().insert(id.clone(), json!({"job_id": id, "project_id": pid, "url": url, "rights": rights, "rights_note": note,
            "status": "queued", "source_id": null, "error": null, "created_at": now_iso()}));
        emit(pid, "source_enqueued", json!({"job_id": id, "url": url}));
        let (jid, pid, url, rights, note) = (id.clone(), pid.to_string(), url.to_string(), rights.to_string(), note.to_string());
        tokio::spawn(async move {
            let _permit = SEM.acquire().await;
            set(&jid, &[("status", json!("downloading")), ("started_at", json!(now_iso()))]);
            emit(&pid, "source_download_start", json!({"job_id": jid, "url": url}));
            match download_source(&pid, &url, &rights, &note).await {
                Ok(item) if item.status == "ready" => {
                    set(&jid, &[("status", json!("processing_scenes"))]);
                    if let Err(e) = detect_and_save_scenes(&pid, &item.id).await { tracing::warn!("auto scene detection: {}", e.1); }
                    set(&jid, &[("status", json!("ready")), ("source_id", json!(item.id)), ("finished_at", json!(now_iso()))]);
                    emit(&pid, "source_ready", json!({"job_id": jid, "source_id": item.id}));
                }
                Ok(item) => {
                    set(&jid, &[("status", json!("failed")), ("error", json!(item.error)), ("finished_at", json!(now_iso()))]);
                    emit(&pid, "source_failed", json!({"job_id": jid, "error": item.error}));
                }
                Err(e) => {
                    set(&jid, &[("status", json!("failed")), ("error", json!(e.to_string())), ("finished_at", json!(now_iso()))]);
                    emit(&pid, "source_failed", json!({"job_id": jid, "error": e.to_string()}));
                }
            }
        });
        ids.push(id);
    }
    ids
}

pub fn get_job_status(id: &str) -> Option<Value> { JOBS.lock().unwrap().get(id).cloned() }
