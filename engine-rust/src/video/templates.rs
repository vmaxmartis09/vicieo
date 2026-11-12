//! Template loader / validator / Jinja composer (port of video/templates/{schema,loader,compose}.py).
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::platforms::PlatformPreset;
use super::util::{is_valid_slug, media_duration};
use crate::workspace;

fn d_true() -> bool {
    true
}
fn d_platforms() -> Vec<String> {
    vec!["tiktok".into(), "reels".into(), "shorts".into()]
}
fn d_tiktok() -> String {
    "tiktok".into()
}
fn d_version() -> Value {
    json!(1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub key: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub max_chars: Option<i64>,
    #[serde(default)]
    pub min_chars: Option<i64>,
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationCfg {
    #[serde(default = "d_mode")]
    pub mode: String,
    #[serde(default = "d_min")]
    pub min: f64,
    #[serde(default = "d_max")]
    pub max: f64,
    #[serde(default)]
    pub fixed_duration: Option<f64>,
}
fn d_mode() -> String {
    "auto_from_voice".into()
}
fn d_min() -> f64 {
    5.0
}
fn d_max() -> f64 {
    600.0
}
impl Default for DurationCfg {
    fn default() -> Self {
        Self { mode: d_mode(), min: 5.0, max: 600.0, fixed_duration: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCfg {
    #[serde(default = "d_true")]
    pub required: bool,
    #[serde(default = "d_voice")]
    pub default_voice_id: String,
}
fn d_voice() -> String {
    "soc-nho".into()
}
impl Default for VoiceCfg {
    fn default() -> Self {
        Self { required: true, default_voice_id: d_voice() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionsCfg {
    #[serde(default = "d_anchor")]
    pub style: String,
    #[serde(default = "d_pos")]
    pub position: String,
}
fn d_anchor() -> String {
    "anchor".into()
}
fn d_pos() -> String {
    "lower_third_safe".into()
}
impl Default for CaptionsCfg {
    fn default() -> Self {
        Self { style: d_anchor(), position: d_pos() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgmCfg {
    #[serde(default = "d_mood")]
    pub mood: String,
    #[serde(default = "d_duck")]
    pub duck_db: f64,
}
fn d_mood() -> String {
    "upbeat".into()
}
fn d_duck() -> f64 {
    -12.0
}
impl Default for BgmCfg {
    fn default() -> Self {
        Self { mood: d_mood(), duck_db: -12.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCfg {
    pub role: String,
    #[serde(default)]
    pub dur: Option<Value>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMeta {
    pub id: String,
    pub name: String,
    #[serde(default = "d_version")]
    pub version: Value,
    pub category: String,
    #[serde(default = "d_platforms")]
    pub platforms: Vec<String>,
    #[serde(default = "d_tiktok")]
    pub default_platform: String,
    #[serde(default)]
    pub duration: DurationCfg,
    #[serde(default)]
    pub slots: Vec<Slot>,
    #[serde(default)]
    pub voice: VoiceCfg,
    #[serde(default)]
    pub captions: CaptionsCfg,
    #[serde(default)]
    pub bgm: BgmCfg,
    #[serde(default)]
    pub scenes: Vec<SceneCfg>,
    #[serde(default)]
    pub responsive: HashMap<String, String>,
}

pub fn templates_root() -> PathBuf {
    workspace::data_root().join("shared").join("templates")
}

pub fn template_dir(id: &str) -> Result<PathBuf> {
    if !is_valid_slug(id) {
        bail!("Template id không hợp lệ: {id}");
    }
    let d = templates_root().join(id);
    if !d.is_dir() {
        bail!("Template '{id}' không tồn tại");
    }
    Ok(d)
}

fn load(dir: &Path) -> Option<TemplateMeta> {
    let s = std::fs::read_to_string(dir.join("template.json")).ok()?;
    match serde_json::from_str(&s) {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!("Lỗi đọc template {}: {e}", dir.display());
            None
        }
    }
}

pub fn get_template(id: &str) -> Result<TemplateMeta> {
    load(&template_dir(id)?).ok_or_else(|| anyhow!("Template '{id}' không tồn tại"))
}

pub fn list_templates(category: Option<&str>, platform: Option<&str>) -> Vec<TemplateMeta> {
    let mut v: Vec<TemplateMeta> = std::fs::read_dir(templates_root())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with(['.', '_']))
        .filter_map(|e| load(&e.path()))
        .collect();
    v.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(c) = category.map(|c| c.to_lowercase()) {
        v.retain(|t| t.category.to_lowercase() == c);
    }
    if let Some(p) = platform.map(|p| p.to_lowercase()) {
        v.retain(|t| t.platforms.iter().any(|x| x.to_lowercase() == p));
    }
    v
}

pub fn validate_template_data(t: &TemplateMeta, data: &Map<String, Value>) -> Vec<String> {
    let mut errors = vec![];
    for s in &t.slots {
        let val = data.get(&s.key).filter(|v| !v.is_null());
        let label = s.label.clone().unwrap_or_else(|| s.key.clone());
        if s.required {
            let empty = match val {
                None => true,
                Some(Value::String(x)) => x.trim().is_empty(),
                Some(Value::Array(a)) => a.is_empty(),
                _ => false,
            };
            if empty {
                errors.push(format!("Thiếu trường bắt buộc '{}' ({}).", s.key, label));
                continue;
            }
        }
        let Some(val) = val else { continue };
        if s.kind == "text" {
            if let Some(x) = val.as_str() {
                let n = x.chars().count() as i64;
                if let Some(m) = s.max_chars.filter(|m| n > *m) {
                    errors.push(format!("Trường '{}' quá dài: tối đa {} ký tự (hiện có {} ký tự).", s.key, m, n));
                }
                if let Some(m) = s.min_chars.filter(|m| n < *m) {
                    errors.push(format!("Trường '{}' quá ngắn: tối thiểu {} ký tự (hiện có {} ký tự).", s.key, m, n));
                }
            }
        }
        if s.kind == "media[]" || s.kind == "list" {
            if let Some(a) = val.as_array() {
                let n = a.len() as i64;
                if let Some(m) = s.min.filter(|m| n < *m) {
                    errors.push(format!("Trường '{}' yêu cầu tối thiểu {} mục (hiện có {} mục).", s.key, m, n));
                }
                if let Some(m) = s.max.filter(|m| n > *m) {
                    errors.push(format!("Trường '{}' vượt quá giới hạn: tối đa {} mục (hiện có {} mục).", s.key, m, n));
                }
            }
        }
        if s.kind == "product" {
            match val.as_object() {
                None => errors.push(format!("Trường sản phẩm '{}' phải là một đối tượng JSON hợp lệ.", s.key)),
                Some(o) => {
                    let has = |k: &str| o.get(k).map(|v| v.as_str().map(|s| !s.is_empty()).unwrap_or(!v.is_null())).unwrap_or(false);
                    if !has("title") && !has("name") {
                        errors.push(format!("Sản phẩm trong '{}' cần có tên ('title' hoặc 'name').", s.key));
                    }
                }
            }
        }
    }
    errors
}

pub fn parse_dur_seconds(v: Option<&Value>, default: f64) -> f64 {
    match v {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(default),
        Some(Value::String(s)) => {
            let s = s.trim().to_lowercase();
            s.strip_suffix('s').unwrap_or(&s).trim().parse().unwrap_or(default)
        }
        _ => default,
    }
}

pub struct ComposeInput<'a> {
    pub template: &'a TemplateMeta,
    pub dir: &'a Path,
    pub data: &'a Map<String, Value>,
    pub preset: &'a PlatformPreset,
    pub voice_audio: Option<&'a Path>,
    pub has_bgm: bool,
    pub total_duration: Option<f64>,
}

/// Render composition.html.j2 → HyperFrames HTML. Returns (html, total_duration).
pub fn compose_template(i: &ComposeInput) -> Result<(String, f64)> {
    let errors = validate_template_data(i.template, i.data);
    if !errors.is_empty() {
        bail!("Dữ liệu template '{}' không hợp lệ: {}", i.template.id, errors.join(" | "));
    }
    let dur = if let Some(v) = i.voice_audio.filter(|p| p.exists()) {
        media_duration(v, 4.0)
    } else if let Some(d) = i.total_duration.filter(|d| *d > 0.0) {
        d
    } else if let Some(d) = i.template.duration.fixed_duration {
        d
    } else {
        15.0
    };

    let metas = &i.template.scenes;
    let mut scenes = vec![];
    if metas.is_empty() {
        scenes.push(json!({"role":"main","start":0.0,"duration":r3(dur),"end":r3(dur),"summary":"Main scene"}));
    } else {
        let fixed: f64 = metas.iter().filter(|s| s.dur.is_some()).map(|s| parse_dur_seconds(s.dur.as_ref(), 3.0)).sum();
        let flex_n = metas.iter().filter(|s| s.dur.is_none()).count().max(1) as f64;
        let flex = (dur - fixed).max(1.0) / flex_n;
        let mut t = 0.0;
        for s in metas {
            let d = if s.dur.is_some() { parse_dur_seconds(s.dur.as_ref(), 3.0) } else { flex };
            scenes.push(json!({"role":s.role,"start":r3(t),"duration":r3(d),"end":r3(t + d),"summary":s.summary.clone().unwrap_or_default()}));
            t += d;
        }
    }

    let j2_text = std::fs::read_to_string(i.dir.join("composition.html.j2")).map_err(|_| anyhow!("File composition.html.j2 không tồn tại ở {}", i.dir.display()))?;
    let css_text = std::fs::read_to_string(i.dir.join("styles.css")).unwrap_or_default();

    // Templates are designed on a half-size canvas; `{# design_scale: N #}` overrides (see compose.py).
    let scale = regex::Regex::new(r"\{#\s*design_scale:\s*([0-9.]+)\s*#\}")?
        .captures(&j2_text)
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(2.0);
    let sz = &i.preset.safe_zone;
    let half = |v: u32| (v as f64 / scale).round() as i64;
    let mut env = minijinja::Environment::new();
    env.set_unknown_method_callback(pycompat_callback);
    let css_ctx = json!({
        "safe_zone": {"top":half(sz.top),"bottom":half(sz.bottom),"left":half(sz.left),"right":half(sz.right)},
        "width": i.preset.width, "height": i.preset.height, "fps": i.preset.fps
    });
    let mut css = env.render_str(&css_text, &css_ctx).map_err(|e| anyhow!("styles.css: {e}"))?;
    css.push_str(&format!("\n#root > * {{ zoom: {}; }}\n", scale));

    let ctx = json!({
        "template": i.template,
        "data": i.data,
        "platform": i.preset,
        "width": i.preset.width, "height": i.preset.height, "fps": i.preset.fps,
        "safe_zone": i.preset.safe_zone,
        "total_duration": r3(dur),
        "scenes": scenes,
        "styles_css": css,
        "voiceover_src": "assets/voiceover.mp3",
        "bgm_src": if i.has_bgm { json!("assets/bgm.mp3") } else { Value::Null },
        "bgm_duck_db": i.template.bgm.duck_db,
    });
    let html = env.render_str(&j2_text, &ctx).map_err(|e| anyhow!("composition.html.j2: {e}"))?;
    Ok((html, r3(dur)))
}

fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Jinja2 templates use Python string methods; minijinja-contrib covers split/strip/replace/…, and
/// `"{:,}".format(n)` (thousands separator, as used for prices) is added here.
fn pycompat_callback(state: &minijinja::State, value: &minijinja::Value, method: &str, args: &[minijinja::Value]) -> Result<minijinja::Value, minijinja::Error> {
    if method == "format" {
        if let Some(fmt) = value.as_str() {
            let mut out = fmt.to_string();
            for a in args {
                let (plain, grouped) = match a.as_i64() {
                    Some(n) => {
                        let s = n.abs().to_string();
                        let g = s.as_bytes().rchunks(3).rev().map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(",");
                        (n.to_string(), if n < 0 { format!("-{g}") } else { g })
                    }
                    None => (a.to_string(), a.to_string()),
                };
                if let Some(i) = out.find("{:,}") {
                    out.replace_range(i..i + 4, &grouped);
                } else if let Some(i) = out.find("{}") {
                    out.replace_range(i..i + 2, &plain);
                }
            }
            return Ok(minijinja::Value::from(out));
        }
    }
    minijinja_contrib::pycompat::unknown_method_callback(state, value, method, args)
}
