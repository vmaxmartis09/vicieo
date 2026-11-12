//! Episode pipeline (port of video/episode/*): plan → keyframes → voice timeline → render requests
//! (YouTube 16:9 main + three 9:16 shorts). Real scene media is used; nothing is rendered from a template
//! that ignores scenes (yt-animation-episode-v1 only draws title/outro cards, so episodes use the scene compositor).
use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

use super::queue::queue;
use super::render::{RenderRequest, VoiceSpec};
use super::util::{atomic_write, ffmpeg, ffmpeg_bin, is_valid_slug, media_duration};
use crate::audio_mastering::AudioMastering;
use crate::caption::{text_on_timings, KWord};
use crate::workspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueLine {
    pub speaker: String,
    pub line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenePlan {
    pub scene_id: String,
    pub scene_number: u32,
    #[serde(default)]
    pub characters: Vec<String>,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub dialogue: Vec<DialogueLine>,
    #[serde(default)]
    pub camera: String,
    #[serde(default = "d_dur")]
    pub duration_est: f64,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub scale_lock_prompt: String,
}
fn d_dur() -> f64 {
    5.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodePlan {
    pub episode_id: String,
    pub title: String,
    pub scenes: Vec<ScenePlan>,
    #[serde(default)]
    pub estimated_cost_usd: f64,
    #[serde(default)]
    pub total_duration_est: f64,
}

fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

pub fn plan_episode(script: &str, episode_id: &str, title: &str) -> EpisodePlan {
    let location = "Phòng khám Bác sĩ Koala".to_string();
    let scene_re = Regex::new(r"(?i)^(cảnh|scene|phân cảnh)\s*(\d+)[:.-]?\s*(.*)").unwrap();
    let diag_re = Regex::new(r"^([^:：]+)[:：]\s*(.+)$").unwrap();
    let flush = |num: u32, d: &[DialogueLine], action: &str| -> ScenePlan {
        let mut chars = std::collections::BTreeSet::new();
        for l in d {
            let s = l.speaker.to_lowercase();
            if s.contains("bi") || s.contains("sóc") {
                chars.insert("soc_nho".to_string());
            } else if s.contains("koala") || s.contains("bác sĩ") {
                chars.insert("koala".to_string());
            }
        }
        if chars.is_empty() {
            chars.insert("soc_nho".into());
        }
        let words: usize = d.iter().map(|l| l.line.split_whitespace().count()).sum();
        let dur = (r1(words as f64 / 2.8 + 1.2)).max(3.5);
        let both = chars.contains("soc_nho") && chars.contains("koala");
        let scale = if both {
            "Bi the squirrel is two-thirds the height of the koala doctor (100cm vs 150cm), Bi's head reaches the doctor's chest."
        } else {
            "Character Sóc Nhỏ Bi 100cm tall, Pixar 3D style."
        };
        let list: Vec<String> = chars.into_iter().collect();
        ScenePlan {
            scene_id: format!("{episode_id}_sc_{num:02}"),
            scene_number: num,
            prompt: format!(
                "3D Pixar animation shot of {} in {}. Action: {}. {}",
                list.join(", "),
                location,
                if action.trim().is_empty() { "character talking and interacting" } else { action.trim() },
                scale
            ),
            characters: list,
            location: location.clone(),
            action: if action.trim().is_empty() { "Diễn xuất biểu cảm sinh động".into() } else { action.trim().to_string() },
            dialogue: d.to_vec(),
            camera: if both { "Cinematic medium shot".into() } else { "Close-up expression shot".into() },
            duration_est: dur,
            scale_lock_prompt: scale.into(),
        }
    };
    let (mut scenes, mut num, mut dl, mut action) = (vec![], 1u32, Vec::<DialogueLine>::new(), String::new());
    for line in script.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        if let Some(c) = scene_re.captures(line) {
            if !dl.is_empty() || !action.is_empty() {
                scenes.push(flush(num, &dl, &action));
                num += 1;
                dl.clear();
                action.clear();
            }
            let desc = c[3].trim();
            if !desc.is_empty() {
                action = desc.to_string();
            }
            continue;
        }
        if let Some(c) = diag_re.captures(line) {
            dl.push(DialogueLine { speaker: c[1].trim().to_string(), line: c[2].trim().to_string() });
        } else {
            action.push(' ');
            action.push_str(line);
        }
    }
    if !dl.is_empty() || !action.is_empty() {
        scenes.push(flush(num, &dl, &action));
    }
    if scenes.is_empty() {
        scenes.push(ScenePlan {
            scene_id: format!("{episode_id}_sc_01"),
            scene_number: 1,
            characters: vec!["soc_nho".into(), "koala".into()],
            location: "Rừng thông Sóc Nhỏ".into(),
            action: "Bi và Bác sĩ Koala trò chuyện vui vẻ".into(),
            dialogue: vec![DialogueLine { speaker: "Bi".into(), line: script.chars().take(100).collect() }],
            camera: "Medium shot, eye-level cinematic".into(),
            duration_est: 5.0,
            prompt: String::new(),
            scale_lock_prompt: String::new(),
        });
    }
    let total: f64 = scenes.iter().map(|s| s.duration_est).sum();
    EpisodePlan {
        episode_id: episode_id.into(),
        title: if title.is_empty() { "Tập Hoạt Hình Sóc Nhỏ".into() } else { title.into() },
        estimated_cost_usd: (scenes.len() as f64 * 0.015 * 1000.0).round() / 1000.0,
        total_duration_est: r1(total),
        scenes,
    }
}

pub fn estimate_episode_cost(scenes: &[ScenePlan]) -> Value {
    let c = (scenes.len() as f64 * 0.015 * 10000.0).round() / 10000.0;
    json!({"scene_count": scenes.len(), "estimated_image_cost_usd": c, "estimated_voice_cost_usd": 0.0, "total_estimated_usd": c})
}

pub fn ep_dir(project: &str, eid: &str) -> Result<PathBuf> {
    if !is_valid_slug(project) || !is_valid_slug(eid) {
        bail!("project/episode id không hợp lệ");
    }
    workspace::project_path(project, &["output", eid], false)
}

pub fn save_plan(project: &str, plan: &EpisodePlan) -> Result<()> {
    atomic_write(&ep_dir(project, &plan.episode_id)?.join("plan.json"), &serde_json::to_string_pretty(plan)?)
}

const DEFAULT_SCRIPT: &str = "Cảnh 1: Phòng khám Bác sĩ Koala\nBi: Bác sĩ Koala ơi, răng của em bị sâu đau quá!\nKoala: Để bác xem nào, em ăn nhiều kẹo ngọt mà quên đánh răng đúng không?\nCảnh 2: Hướng dẫn chăm sóc răng miệng\nKoala: Mỗi ngày em phải đánh răng 2 lần, sáng và tối nhé!\nBi: Dạ, em nhớ rồi ạ, em sẽ chăm chỉ đánh răng!\nCảnh 3: Kết thúc vui vẻ\nBi: Răng em hết đau rồi, cảm ơn Bác sĩ Koala!\nKoala: Chúc Bi luôn vui khỏe và có nụ cười thật tươi!";

fn dialogue_text(s: &ScenePlan) -> String {
    s.dialogue.iter().map(|d| format!("{}: {}", d.speaker, d.line)).collect::<Vec<_>>().join(" ")
}

/// Keyframe for a scene: an existing generated image (output/<ep>/scenes/<scene_id>.png|jpg, e.g. from the
/// graph's image_gen) wins; otherwise a composite card: coloured backdrop + the characters' own PNGs.
async fn scene_image(project: &str, eid: &str, s: &ScenePlan, cancel: &CancellationToken) -> Result<(PathBuf, bool)> {
    let dir = ep_dir(project, eid)?.join("scenes");
    std::fs::create_dir_all(&dir)?;
    for ext in ["png", "jpg", "jpeg", "webp"] {
        let p = dir.join(format!("{}.{ext}", s.scene_id));
        if p.is_file() {
            return Ok((p, false));
        }
    }
    let h = hex::encode_short(&Sha256::digest(format!("{}:{}", s.prompt, s.scale_lock_prompt)));
    let out = dir.join(format!("{}_{}.png", s.scene_id, h));
    if out.is_file() {
        return Ok((out, true));
    }
    let hue = (u32::from_str_radix(&h[..4], 16).unwrap_or(0) % 6) as usize;
    let bg = ["0x1f3a5f", "0x2d5a3d", "0x5a3d2d", "0x3d2d5a", "0x5a2d4a", "0x2d4a5a"][hue];
    let mut args: Vec<String> = vec!["-f".into(), "lavfi".into(), "-i".into(), format!("color=c={bg}:s=1920x1080:d=1")];
    let mut chain = String::new();
    let mut last = "0:v".to_string();
    let mut k = 0;
    let n_chars = s.characters.len().max(1) as i64;
    for c in &s.characters {
        let cdir = workspace::project_path(project, &["characters", c], false)?;
        let img = std::fs::read_dir(&cdir).into_iter().flatten().flatten().map(|e| e.path()).find(|p| p.extension().map(|e| e == "png").unwrap_or(false));
        if let Some(img) = img {
            args.extend(["-i".into(), img.to_string_lossy().into()]);
            k += 1;
            let x = 1920 * (2 * (k - 1) + 1) / (2 * n_chars);
            chain.push_str(&format!("[{k}:v]scale=-1:820[c{k}];[{last}][c{k}]overlay=x={x}-w/2:y=H-h-80[o{k}];"));
            last = format!("o{k}");
        }
    }
    args.extend(["-filter_complex".into(), if chain.is_empty() { "[0:v]null[o]".into() } else { format!("{}[{last}]null[o]", chain) }, "-map".into(), "[o]".into(), "-frames:v".into(), "1".into(), out.to_string_lossy().into()]);
    ffmpeg(&args, cancel, 120).await?;
    Ok((out, true))
}

mod hex {
    pub fn encode_short(b: &[u8]) -> String {
        b.iter().take(5).map(|x| format!("{x:02x}")).collect::<String>()[..10].to_string()
    }
}

async fn silent_wav(path: &Path, dur: f64, cancel: &CancellationToken) -> Result<()> {
    ffmpeg(&["-f".into(), "lavfi".into(), "-i".into(), "anullsrc=r=44100:cl=mono".into(), "-t".into(), format!("{dur:.3}"), "-c:a".into(), "pcm_s16le".into(), path.to_string_lossy().into()], cancel, 60).await?;
    Ok(())
}

pub async fn build_episode(project: &str, eid: &str, _use_mock: bool) -> Result<Value> {
    let dir = ep_dir(project, eid)?;
    let plan_file = dir.join("plan.json");
    let plan: EpisodePlan = if plan_file.is_file() {
        serde_json::from_str(&std::fs::read_to_string(&plan_file)?)?
    } else {
        let mut p = plan_episode(DEFAULT_SCRIPT, eid, "Sóc Nhỏ và Chiếc Răng Sâu");
        p.estimated_cost_usd = estimate_episode_cost(&p.scenes)["total_estimated_usd"].as_f64().unwrap_or(0.0);
        save_plan(project, &p)?;
        p
    };
    let cancel = CancellationToken::new();
    let mut warnings = vec![];

    // 1) keyframes
    let mut assets = vec![];
    let mut placeholder_img = 0;
    for s in &plan.scenes {
        let (img, is_card) = scene_image(project, eid, s, &cancel).await?;
        placeholder_img += is_card as u32;
        assets.push(json!({"sceneNumber": s.scene_number, "scene_id": s.scene_id, "image": img.to_string_lossy(), "img": img.to_string_lossy(),
            "durationSec": s.duration_est, "dialogue": dialogue_text(s), "characters": s.characters, "camera": s.camera}));
    }
    if placeholder_img > 0 {
        warnings.push(format!("{placeholder_img}/{} cảnh dùng thẻ phác thảo (nền màu + ảnh nhân vật) vì chưa có ảnh sinh ra: chạy node image_gen rồi đặt vào output/{eid}/scenes/<scene_id>.png", plan.scenes.len()));
    }

    // 2) voice timeline: per-scene audio from Track C if present, else silence of the planned duration
    let adir = dir.join("audio");
    std::fs::create_dir_all(&adir)?;
    let mut parts = vec![];
    let mut silent = 0;
    let mut scene_dur = vec![];
    for s in &plan.scenes {
        let mut found = None;
        for ext in ["wav", "mp3", "m4a"] {
            let p = adir.join(format!("{}_voice.{ext}", s.scene_id));
            if p.is_file() {
                found = Some(p);
                break;
            }
        }
        let p = match found {
            Some(p) => p,
            None => {
                silent += 1;
                let p = adir.join(format!("{}_voice.wav", s.scene_id));
                silent_wav(&p, s.duration_est, &cancel).await?;
                p
            }
        };
        scene_dur.push(media_duration(&p, s.duration_est));
        parts.push(p);
    }
    if silent > 0 {
        warnings.push(format!("{silent}/{} cảnh chưa có giọng thật ({{scene_id}}_voice.wav do Track C tạo) nên dùng khoảng lặng — video sẽ im lặng ở các cảnh đó.", plan.scenes.len()));
    }
    let list = adir.join("concat.txt");
    std::fs::write(&list, parts.iter().map(|p| format!("file '{}'\n", p.canonicalize().unwrap_or(p.clone()).display())).collect::<String>())?;
    let raw = adir.join("episode_raw.wav");
    ffmpeg(&["-f".into(), "concat".into(), "-safe".into(), "0".into(), "-i".into(), list.to_string_lossy().into(), "-ar".into(), "44100".into(), "-ac".into(), "1".into(), "-c:a".into(), "pcm_s16le".into(), raw.to_string_lossy().into()], &cancel, 300).await?;
    let master = adir.join("episode_master_voice.mp3");
    if silent == plan.scenes.len() {
        ffmpeg(&["-i".into(), raw.to_string_lossy().into(), "-b:a".into(), "192k".into(), master.to_string_lossy().into()], &cancel, 120).await?;
    } else {
        let (r, m) = (raw.clone(), master.clone());
        tokio::task::spawn_blocking(move || AudioMastering::new(Some(ffmpeg_bin())).master_tiktok(&r, &m, -14.0, 2.0, 1.0, "192k")).await??;
    }

    // 3) word timestamps per scene (script text over each scene's real audio interval)
    let mut words: Vec<KWord> = vec![];
    let mut t = 0.0;
    for (i, s) in plan.scenes.iter().enumerate() {
        let txt = s.dialogue.iter().map(|d| d.line.clone()).collect::<Vec<_>>().join(" ");
        let (a, b) = if parts[i].is_file() { crate::caption::speech_span(&parts[i], scene_dur[i]) } else { (0.0, scene_dur[i]) };
        words.extend(text_on_timings(&txt, t + a, t + b));
        t += scene_dur[i];
    }
    atomic_write(&dir.join("words.json"), &serde_json::to_string_pretty(&words)?)?;

    // 4) chapters + YouTube metadata
    let mut chapters = vec!["00:00 Mở đầu & Giới thiệu tập".to_string()];
    let mut cur = 0.0;
    for (i, s) in plan.scenes.iter().enumerate() {
        let summary: String = if s.action.is_empty() { format!("Cảnh {}", s.scene_number) } else { s.action.chars().take(45).collect() };
        chapters.push(format!("{:02}:{:02} Cảnh {}: {}", (cur / 60.0) as u32, (cur as u32) % 60, s.scene_number, summary));
        cur += scene_dur[i];
    }
    let ch = chapters.join("\n");
    atomic_write(&dir.join("chapters.txt"), &ch)?;
    atomic_write(&dir.join("youtube_meta.json"), &serde_json::to_string_pretty(&json!({
        "title": format!("Sóc Nhỏ TV | {} (Hoạt Hình 3D)", plan.title),
        "description": format!("Chào mừng các bạn nhỏ và quý phụ huynh đến với Sóc Nhỏ TV!\n\nNội dung tập phim: {}\nNhân vật chính: Sóc Nhỏ (Bi) cao 100cm, Bác sĩ Koala cao 150cm.\n\nMỤC LỤC TẬP PHIM:\n{}\n\n© Bản quyền thuộc về Sóc Nhỏ TV. Vui lòng không reup dưới mọi hình thức.", plan.title, ch),
        "tags": ["socnhotv","soc nho bi","hoat hinh 3d","hoat hinh thieu nhi","koala","ke chuyen thieu nhi","bai hoc cuoc song"],
        "category_id": "1"
    }))?)?;

    // 5) render requests: main 16:9 + 3 shorts 9:16 (audio sliced from the master voice)
    let master_s = master.to_string_lossy().to_string();
    let main = RenderRequest {
        project_id: project.into(),
        template_id: String::new(),
        platforms: vec!["youtube".into()],
        data: json!({"scenes": assets, "words": words, "allow_silent": silent == plan.scenes.len(), "auto_draft": true, "title": plan.title, "titleBadge": plan.title, "episode_title": plan.title}).as_object().cloned().unwrap(),
        voice: Some(VoiceSpec { audio_path: Some(master_s.clone()), ..Default::default() }),
        source_ref: None,
        output_name: format!("{eid}_main"),
    };
    let n = plan.scenes.len();
    let starts: Vec<f64> = scene_dur.iter().scan(0.0, |a, d| { let s = *a; *a += d; Some(s) }).collect();
    let ranges = [(0, n.min(2)), (n / 2, (n / 2 + 2).min(n)), (n.saturating_sub(2), n)];
    let mut shorts = vec![];
    for (k, (a, b)) in ranges.iter().enumerate() {
        let (a, b) = if a >= b { (0, n) } else { (*a, *b) };
        let (t0, t1) = (starts[a], starts[b - 1] + scene_dur[b - 1]);
        let clip = adir.join(format!("short_{:02}.mp3", k + 1));
        ffmpeg(&["-ss".into(), format!("{t0:.3}"), "-t".into(), format!("{:.3}", t1 - t0), "-i".into(), master_s.clone(), "-b:a".into(), "192k".into(), clip.to_string_lossy().into()], &cancel, 120).await?;
        let sw: Vec<KWord> = words.iter().filter(|w| w.start >= t0 - 0.01 && w.end <= t1 + 0.3).map(|w| KWord { w: w.w.clone(), start: (w.start - t0).max(0.0), end: (w.end - t0).max(0.0) }).collect();
        shorts.push(RenderRequest {
            project_id: project.into(),
            template_id: String::new(),
            platforms: vec!["shorts".into(), "tiktok".into(), "reels".into()],
            data: json!({"scenes": &assets[a..b], "words": sw, "allow_silent": silent == plan.scenes.len(), "auto_draft": true, "title": format!("Shorts {} - {eid}", k + 1), "titleBadge": "SÓC NHỎ TV"}).as_object().cloned().unwrap(),
            voice: Some(VoiceSpec { audio_path: Some(clip.to_string_lossy().into()), ..Default::default() }),
            source_ref: None,
            output_name: format!("{eid}_short_{:02}", k + 1),
        });
    }
    let q = queue();
    let main_job = q.submit(main, project)?;
    let mut short_ids = vec![];
    for s in shorts {
        short_ids.push(q.submit(s, project)?.job_id);
    }
    let status = json!({"episode_id": eid, "title": plan.title, "main_job_id": main_job.job_id, "shorts_job_ids": short_ids,
        "status": "queued", "scenes_count": n, "estimated_cost_usd": plan.estimated_cost_usd, "warnings": warnings});
    atomic_write(&dir.join("build_status.json"), &serde_json::to_string_pretty(&status)?)?;
    Ok(status)
}

pub fn episode_status(project: &str, eid: &str) -> Result<Value> {
    let dir = ep_dir(project, eid)?;
    let sf = dir.join("build_status.json");
    if !sf.is_file() {
        let pf = dir.join("plan.json");
        if pf.is_file() {
            return Ok(json!({"episode_id": eid, "status": "planned", "plan": serde_json::from_str::<Value>(&std::fs::read_to_string(pf)?)?}));
        }
        return Err(anyhow!("Tập phim {eid} chưa được lập kế hoạch hoặc sản xuất"));
    }
    let mut st: Value = serde_json::from_str(&std::fs::read_to_string(sf)?)?;
    let q = queue();
    let main = st["main_job_id"].as_str().and_then(|j| q.get(project, j).ok());
    let shorts: Vec<_> = st["shorts_job_ids"].as_array().cloned().unwrap_or_default().iter().filter_map(|j| j.as_str().and_then(|j| q.get(project, j).ok())).collect();
    let all: Vec<&str> = main.iter().chain(shorts.iter()).map(|j| j.status.as_str()).collect();
    st["status"] = json!(if all.is_empty() { st["status"].as_str().unwrap_or("unknown") }
        else if all.iter().all(|s| *s == "done") { "done" }
        else if all.contains(&"failed") { "failed" }
        else if all.contains(&"rendering") { "rendering" } else { "queued" });
    st["main_job"] = serde_json::to_value(&main)?;
    st["shorts_jobs"] = serde_json::to_value(&shorts)?;
    if let Ok(c) = std::fs::read_to_string(dir.join("chapters.txt")) {
        st["chapters"] = json!(c);
    }
    if let Some(m) = std::fs::read_to_string(dir.join("youtube_meta.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) {
        st["youtube_meta"] = m;
    }
    Ok(st)
}
