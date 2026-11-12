//! Manual edit decisions + selective re-render (port of core/engine/edit_decisions.py).
//! script.json is never modified; decisions live in edit_decisions.json next to it.
use anyhow::{anyhow, bail, Result};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

use super::util::{atomic_write, ffmpeg, is_valid_slug, now_iso};
use crate::workspace;

pub struct Editor {
    pub project: String,
    pub proj_dir: PathBuf,
}

fn sha(s: &str) -> String {
    hex_str(&Sha256::digest(s.as_bytes()))
}
fn hex_str(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Typed so handlers can map to 404 / 400 / 500 like the Python routes.
#[derive(Debug)]
pub enum EditorError {
    NotFound(String),
    Invalid(String),
    Other(anyhow::Error),
}
impl From<anyhow::Error> for EditorError {
    fn from(e: anyhow::Error) -> Self {
        EditorError::Other(e)
    }
}
impl From<std::io::Error> for EditorError {
    fn from(e: std::io::Error) -> Self {
        EditorError::Other(e.into())
    }
}
type R<T> = std::result::Result<T, EditorError>;

impl Editor {
    pub fn new(project: &str) -> R<Self> {
        if !is_valid_slug(project) {
            return Err(EditorError::NotFound(format!("Dự án '{project}' không tồn tại")));
        }
        let d = workspace::project_path(project, &[], false).map_err(EditorError::Other)?;
        if !d.is_dir() {
            return Err(EditorError::NotFound(format!("Dự án '{project}' không tồn tại")));
        }
        Ok(Self { project: project.into(), proj_dir: d })
    }

    fn ep_dir(&self, eid: &str) -> R<PathBuf> {
        if !is_valid_slug(eid) {
            return Err(EditorError::Invalid(format!("episode id không hợp lệ: {eid}")));
        }
        Ok(self.proj_dir.join("content/episodes").join(eid))
    }

    pub fn load_base_script(&self, eid: &str) -> R<Value> {
        let p = self.ep_dir(eid)?.join("script.json");
        let s = std::fs::read_to_string(&p).map_err(|_| EditorError::NotFound(format!("Không tìm thấy kịch bản cho tập '{eid}'")))?;
        serde_json::from_str(&s).map_err(|e| EditorError::Other(e.into()))
    }

    pub fn load_decisions(&self, eid: &str) -> R<Option<Value>> {
        let p = self.ep_dir(eid)?.join("edit_decisions.json");
        Ok(std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()))
    }

    fn save_decisions(&self, eid: &str, d: &Value) -> R<()> {
        atomic_write(&self.ep_dir(eid)?.join("edit_decisions.json"), &serde_json::to_string_pretty(d).map_err(|e| anyhow!(e))?)?;
        Ok(())
    }

    /// script.json (scenes[].shots[]) → flat shot list (episode_script.script_to_scenes).
    fn base_shots(&self, eid: &str) -> R<Vec<Value>> {
        let script = self.load_base_script(eid)?;
        let mut out = vec![];
        let mut n = 0;
        for sc in script["scenes"].as_array().cloned().unwrap_or_default() {
            for sh in sc["shots"].as_array().cloned().unwrap_or_default() {
                n += 1;
                let mut s = sh.clone();
                let o = s.as_object_mut().ok_or_else(|| EditorError::Invalid("shot không hợp lệ".into()))?;
                o.insert("sceneNumber".into(), json!(n));
                o.insert("title".into(), json!(format!("{} · {}", sc["title"].as_str().unwrap_or(""), sh["id"].as_str().unwrap_or(""))));
                o.entry("sceneIndex").or_insert_with(|| sc.get("index").cloned().unwrap_or(json!(1)));
                out.push(s);
            }
        }
        Ok(out)
    }

    pub fn get_effective_shots(&self, eid: &str) -> R<Vec<Value>> {
        let raw = self.base_shots(eid)?;
        let dec = self.load_decisions(eid)?.unwrap_or(json!({}));
        let overrides = dec["shots"].as_object().cloned().unwrap_or_default();
        let custom: Vec<String> = dec["shotOrder"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut base: Map<String, Value> = Map::new();
        let mut original = vec![];
        for sc in &raw {
            let id = sc["id"].as_str().map(String::from).unwrap_or_else(|| format!("shot_{}", sc["sceneNumber"]));
            original.push(id.clone());
            let mut media = ["video", "img", "image", "src"].iter().find_map(|k| sc[*k].as_str().filter(|s| !s.is_empty())).unwrap_or("").to_string();
            if media.is_empty() && self.ep_dir(eid)?.join("clips").join(format!("{id}.mp4")).is_file() {
                media = format!("content/episodes/{eid}/clips/{id}.mp4");
            }
            let dialogue = sc["dialogue"].as_str().unwrap_or("");
            base.insert(
                id.clone(),
                json!({
                    "shotId": id, "sceneNumber": sc["sceneNumber"], "sceneIndex": sc["sceneIndex"],
                    "title": sc["title"], "durationSec": sc["durationSec"].as_f64().unwrap_or(20.0),
                    "dialogue": dialogue, "subtitle": dialogue, "speaker": sc["speaker"].as_str().unwrap_or(""),
                    "speakerId": sc["speakerId"].as_str().unwrap_or("soc_nho"), "camera": sc["camera"].as_str().unwrap_or("push-in"),
                    "priority": sc.get("priority").filter(|p| !p.is_null()).cloned().unwrap_or(json!(3)),
                    "mediaSrc": media, "enabled": true, "isModified": false,
                    "action": sc["action"].as_str().unwrap_or(""), "sfx": sc["sfx"].as_str().unwrap_or(""), "bgm": sc["bgm"].as_str().unwrap_or(""),
                }),
            );
        }
        let active = if custom.is_empty() { original.clone() } else { custom };
        let mut order: Vec<String> = vec![];
        for id in active.iter().chain(original.iter()) {
            if base.contains_key(id) && !order.contains(id) {
                order.push(id.clone());
            }
        }
        let mut out = vec![];
        let mut t = 0.0;
        for id in order {
            let mut s = base[&id].clone();
            if let Some(ov) = overrides.get(&id).and_then(|o| o.as_object()).filter(|o| !o.is_empty()) {
                s["isModified"] = json!(true);
                if let Some(d) = ov.get("durationSec").and_then(|d| d.as_f64()) {
                    s["durationSec"] = json!(d.max(0.5));
                }
                for k in ["mediaSrc", "dialogue", "subtitle", "camera"] {
                    if let Some(v) = ov.get(k) {
                        s[k] = json!(v.as_str().map(String::from).unwrap_or_else(|| v.to_string()));
                    }
                }
                if let Some(e) = ov.get("enabled") {
                    s["enabled"] = json!(e.as_bool().unwrap_or(true));
                }
            }
            let dur = s["durationSec"].as_f64().unwrap_or(20.0);
            let enabled = s["enabled"].as_bool().unwrap_or(true);
            s["startSec"] = json!((t * 1000.0f64).round() / 1000.0);
            s["endSec"] = json!(((t + if enabled { dur } else { 0.0 }) * 1000.0f64).round() / 1000.0);
            if enabled {
                t += dur;
            }
            out.push(s);
        }
        Ok(out)
    }

    pub fn apply_decision(&self, eid: &str, action: &str, shot_id: Option<&str>, params: &Value) -> R<Value> {
        let script = self.load_base_script(eid)?;
        let mut dec = self.load_decisions(eid)?.unwrap_or_else(|| {
            json!({"schemaVersion": 1, "episodeId": eid, "baseScriptSha256": sha(&serde_json::to_string(&script).unwrap_or_default()),
                   "createdAt": now_iso(), "updatedAt": now_iso(), "shotOrder": [], "shots": {}, "history": []})
        });
        let ts = now_iso();
        let entry = json!({"timestamp": ts, "action": action, "shotId": shot_id, "params": params});
        let bad = || EditorError::Invalid(format!("Hành động không hợp lệ hoặc thiếu shot_id: {action}"));
        match (action, shot_id) {
            ("reset_all", _) => {
                let _ = std::fs::remove_file(self.ep_dir(eid)?.join("edit_decisions.json"));
                return Ok(json!({"status": "reset", "effectiveShots": self.get_effective_shots(eid)?}));
            }
            ("reset_shot", Some(id)) => {
                if let Some(m) = dec["shots"].as_object_mut() {
                    m.remove(id);
                }
                if let Some(a) = dec["shotOrder"].as_array_mut() {
                    a.retain(|x| x.as_str() != Some(id));
                }
            }
            ("reorder", _) => {
                if let Some(o) = params["order"].as_array().filter(|a| !a.is_empty()) {
                    dec["shotOrder"] = json!(o.iter().map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).collect::<Vec<_>>());
                }
            }
            ("update_duration", Some(id)) => {
                let d = params["durationSec"].as_f64().unwrap_or(20.0);
                dec["shots"][id]["durationSec"] = json!(d.max(0.5));
            }
            ("replace_media", Some(id)) => dec["shots"][id]["mediaSrc"] = json!(params["mediaSrc"].as_str().unwrap_or("")),
            ("toggle_shot", Some(id)) => dec["shots"][id]["enabled"] = json!(params["enabled"].as_bool().unwrap_or(true)),
            ("update_subtitle", Some(id)) => {
                let sub = params["subtitle"].as_str().unwrap_or("");
                dec["shots"][id]["subtitle"] = json!(sub);
                dec["shots"][id]["dialogue"] = json!(params["dialogue"].as_str().unwrap_or(sub));
            }
            _ => return Err(bad()),
        }
        dec["updatedAt"] = json!(ts);
        if let Some(h) = dec["history"].as_array_mut() {
            h.push(entry);
        }
        self.save_decisions(eid, &dec)?;
        let shots = self.get_effective_shots(eid)?;
        let total: f64 = shots.iter().filter(|s| s["enabled"].as_bool().unwrap_or(true)).map(|s| s["durationSec"].as_f64().unwrap_or(0.0)).sum();
        Ok(json!({"status": "success", "action": action, "shotId": shot_id, "effectiveShots": shots, "totalDurationSec": total}))
    }

    fn shot_hash(shot: &Value) -> String {
        let sub = shot["subtitle"].as_str().filter(|s| !s.is_empty()).or_else(|| shot["dialogue"].as_str()).unwrap_or("");
        let mut m = std::collections::BTreeMap::new();
        m.insert("shotId", shot["shotId"].clone());
        m.insert("durationSec", json!((shot["durationSec"].as_f64().unwrap_or(20.0) * 1000.0).round() / 1000.0));
        m.insert("mediaSrc", json!(shot["mediaSrc"].as_str().unwrap_or("")));
        m.insert("subtitle", json!(sub));
        m.insert("camera", json!(shot["camera"].as_str().unwrap_or("")));
        m.insert("enabled", json!(shot["enabled"].as_bool().unwrap_or(true)));
        sha(&serde_json::to_string(&m).unwrap_or_default())
    }

    async fn render_segment(&self, shot: &Value, out: &std::path::Path, w: u32, h: u32, fps: u32, cancel: &CancellationToken) -> R<()> {
        let dur = shot["durationSec"].as_f64().unwrap_or(5.0);
        let media = shot["mediaSrc"].as_str().filter(|s| !s.is_empty()).map(|m| self.proj_dir.join(m)).filter(|p| p.is_file());
        let mut a: Vec<String> = vec![];
        match &media {
            Some(p) if ["mp4", "mov", "webm"].contains(&p.extension().and_then(|e| e.to_str()).unwrap_or("")) => {
                a.extend(["-stream_loop".into(), "-1".into(), "-t".into(), dur.to_string(), "-i".into(), p.to_string_lossy().into()]);
            }
            Some(p) => a.extend(["-loop".into(), "1".into(), "-t".into(), dur.to_string(), "-i".into(), p.to_string_lossy().into()]),
            None => a.extend(["-f".into(), "lavfi".into(), "-i".into(), format!("color=c=0x18181b:s={w}x{h}:d={dur}")]),
        }
        a.extend(["-f".into(), "lavfi".into(), "-i".into(), format!("anullsrc=r=44100:cl=stereo:d={dur}")]);
        a.extend([
            "-filter_complex".into(),
            format!("[0:v]scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1,format=yuv420p[v]"),
            "-map".into(), "[v]".into(), "-map".into(), "1:a".into(),
            "-c:v".into(), "libx264".into(), "-preset".into(), "ultrafast".into(), "-crf".into(), "18".into(),
            "-c:a".into(), "aac".into(), "-b:a".into(), "192k".into(), "-r".into(), fps.to_string(), "-t".into(), dur.to_string(),
            out.to_string_lossy().into(),
        ]);
        if let Some(p) = out.parent() {
            std::fs::create_dir_all(p)?;
        }
        ffmpeg(&a, cancel, 300).await.map_err(|e| EditorError::Other(anyhow!("Lỗi render segment {}: {e}", shot["shotId"])))?;
        Ok(())
    }

    pub async fn selective_rerender(&self, eid: &str, force_all: bool, w: u32, h: u32, fps: u32) -> R<Value> {
        let t0 = Instant::now();
        let shots: Vec<Value> = self.get_effective_shots(eid)?.into_iter().filter(|s| s["enabled"].as_bool().unwrap_or(true)).collect();
        if shots.is_empty() {
            return Err(EditorError::Invalid(format!("Tập {eid} không có shot nào được bật để render.")));
        }
        let cache = self.proj_dir.join("runs/cache/shots").join(eid);
        std::fs::create_dir_all(&cache)?;
        let out_dir = self.proj_dir.join("output").join(eid);
        std::fs::create_dir_all(&out_dir)?;
        let out = out_dir.join(format!("{eid}.mp4"));
        let cancel = CancellationToken::new();
        let (mut segs, mut cached, mut rendered, mut missing) = (vec![], 0, 0, vec![]);
        for s in &shots {
            let id = s["shotId"].as_str().unwrap_or("shot");
            if !is_valid_slug(id) {
                return Err(EditorError::Invalid(format!("shotId không hợp lệ: {id}")));
            }
            let seg = cache.join(format!("{id}_{}_{w}x{h}.mp4", &Self::shot_hash(s)[..12]));
            if !(seg.is_file() && !force_all && std::fs::metadata(&seg).map(|m| m.len() > 1024).unwrap_or(false)) {
                if s["mediaSrc"].as_str().map(|m| m.is_empty() || !self.proj_dir.join(m).is_file()).unwrap_or(true) {
                    missing.push(id.to_string());
                }
                self.render_segment(s, &seg, w, h, fps, &cancel).await?;
                rendered += 1;
            } else {
                cached += 1;
            }
            segs.push(seg);
        }
        let list = cache.join("concat_list.txt");
        std::fs::write(&list, segs.iter().map(|p| format!("file '{}'\n", p.canonicalize().unwrap_or(p.clone()).display())).collect::<String>())?;
        ffmpeg(&["-f".into(), "concat".into(), "-safe".into(), "0".into(), "-i".into(), list.to_string_lossy().into(), "-c".into(), "copy".into(), out.to_string_lossy().into()], &cancel, 600)
            .await
            .map_err(|e| EditorError::Other(anyhow!("Lỗi ghép nối video: {e}")))?;
        super::hyperframes::extract_thumbnail(&out, &out_dir.join("thumb.jpg"), 0.5);
        let total: f64 = shots.iter().map(|s| s["durationSec"].as_f64().unwrap_or(0.0)).sum();
        let _ = bail_if_empty(&out);
        Ok(json!({
            "status": "success", "episodeId": eid,
            "outputVideo": out.strip_prefix(&self.proj_dir).map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            "outputFullPath": out.to_string_lossy(), "renderedShotsCount": rendered, "cachedShotsCount": cached,
            "totalShotsCount": shots.len(), "elapsedSec": (t0.elapsed().as_secs_f64() * 1000.0).round() / 1000.0,
            "totalDurationSec": total, "engine": "selective_shot_concat", "shotsWithoutMedia": missing,
        }))
    }

    pub fn timeline(&self, eid: &str) -> R<Value> {
        let script = self.load_base_script(eid)?;
        let decisions = self.load_decisions(eid)?;
        let shots = self.get_effective_shots(eid)?;
        let total: f64 = shots.iter().filter(|s| s["enabled"].as_bool().unwrap_or(true)).map(|s| s["durationSec"].as_f64().unwrap_or(0.0)).sum();
        let out_dir = self.proj_dir.join("output").join(eid);
        let has = out_dir.join(format!("{eid}.mp4")).is_file();
        let p = &self.project;
        Ok(json!({
            "episodeId": eid, "title": script["title"].as_str().unwrap_or(eid),
            "aspect": script["aspect"].as_str().unwrap_or("16:9 Widescreen"),
            "baseDurationSec": script["durationSec"].as_f64().unwrap_or(600.0),
            "effectiveDurationSec": (total * 1000.0).round() / 1000.0,
            "hasDecisions": decisions.is_some(), "decisions": decisions, "shots": shots,
            "outputVideoUrl": has.then(|| format!("/projects/{p}/files/output/{eid}/{eid}.mp4")),
            "thumbUrl": out_dir.join("thumb.jpg").is_file().then(|| format!("/projects/{p}/files/output/{eid}/thumb.jpg")),
        }))
    }
}

fn bail_if_empty(p: &std::path::Path) -> Result<()> {
    if std::fs::metadata(p).map(|m| m.len() == 0).unwrap_or(true) {
        bail!("video rỗng");
    }
    Ok(())
}
