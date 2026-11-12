//! Track B tests. They run the real ffmpeg/ffprobe (and HyperFrames when HYPERFRAMES_BIN/node_modules exist).
use super::executors::VideoExecutor;
use super::platforms::get_platform;
use super::render::{render_request, RenderCtx, RenderRequest, VoiceSpec};
use super::templates::{self, ComposeInput};
use super::util::probe_sync;
use crate::engine::executor::{NodeCtx, NodeExecutor};
use crate::workspace;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

fn tmp_project(name: &str) -> String {
    let d = workspace::project_path(name, &[], false).unwrap();
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    name.to_string()
}

#[test]
fn every_template_composes_with_sample_data() {
    let mut failed = vec![];
    for t in templates::list_templates(None, None) {
        let dir = templates::template_dir(&t.id).unwrap();
        let sample: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("sample.data.json")).unwrap()).unwrap();
        let preset = get_platform(&t.default_platform).unwrap();
        let r = templates::compose_template(&ComposeInput { template: &t, dir: &dir, data: sample.as_object().unwrap(), preset: &preset, voice_audio: None, has_bgm: false, total_duration: Some(5.0) });
        match r {
            Ok((html, d)) => assert!(html.contains("data-composition-id") && d > 0.0, "{}", t.id),
            Err(e) => failed.push(format!("{}: {e}", t.id)),
        }
    }
    assert!(failed.is_empty(), "template không compose được: {failed:?}");
}

#[tokio::test]
async fn sample_template_renders_valid_mp4() {
    let pid = tmp_project("_test_video_tpl");
    let tdir = templates::template_dir("tiktok-sales-v1").unwrap();
    let sample: Value = serde_json::from_str(&std::fs::read_to_string(tdir.join("sample.data.json")).unwrap()).unwrap();
    let voice = tdir.join("assets/voiceover.mp3");
    let req = RenderRequest {
        project_id: pid.clone(),
        template_id: "tiktok-sales-v1".into(),
        platforms: vec!["tiktok".into(), "youtube".into()],
        data: sample.as_object().unwrap().clone(),
        voice: Some(VoiceSpec { audio_path: Some(voice.to_string_lossy().into()), ..Default::default() }),
        output_name: "t".into(),
        ..Default::default()
    };
    let work = std::env::temp_dir().join("vicieo_test_work");
    let ctx = RenderCtx { cancel: CancellationToken::new(), progress: Arc::new(|_, _| {}), work_dir: work };
    let res = render_request(&req, &ctx).await.expect("render");
    for (plat, rel) in &res.outputs {
        let p = workspace::project_path(&pid, &[rel], false).unwrap();
        let pr = probe_sync(&p).expect("ffprobe");
        let preset = get_platform(plat).unwrap();
        assert_eq!((pr.width, pr.height), (preset.width, preset.height), "{plat}");
        assert!(pr.has_audio && pr.duration > 1.0 && pr.video_codec == "h264", "{plat}: {pr:?}");
        assert_eq!(res.verify[plat]["pass_all"], true, "{plat}: {}", res.verify[plat]);
    }
    let _ = std::fs::remove_dir_all(workspace::project_path(&pid, &[], false).unwrap());
}

fn node(pid: &str, ty: &str, params: Value, inputs: HashMap<String, Value>) -> NodeCtx {
    let (tx, _) = tokio::sync::broadcast::channel(4);
    NodeCtx { project_id: pid.into(), run_id: "run1".into(), node_id: "n1".into(), node_type: ty.into(), custom_name: ty.into(), params, inputs, tx, cancel: CancellationToken::new(), mock: false }
}

#[tokio::test]
async fn handdraw_scene_cut_and_captions_nodes() {
    let pid = tmp_project("_test_video_nodes");
    let img = templates::template_dir("tiktok-sales-v1").unwrap().join("assets/product.png");
    let voice = templates::template_dir("tiktok-sales-v1").unwrap().join("assets/voiceover.mp3");
    let ex = VideoExecutor;

    let o = ex.execute(&node(&pid, "handdraw", json!({"image": img, "durationSec": 3}), HashMap::new())).await.expect("handdraw");
    let url = o.preview_url.unwrap();
    let rel = url.split("/files/").nth(1).unwrap();
    let p = workspace::project_path(&pid, &[rel], false).unwrap();
    assert!(probe_sync(&p).unwrap().duration > 2.5);

    let o = ex.execute(&node(&pid, "scene_cut", json!({"video": p, "segmentSec": 1.0}), HashMap::new())).await.expect("scene_cut");
    assert!(o.outputs["clips"].as_array().unwrap().len() >= 2);

    let inputs = HashMap::from([("voice".to_string(), json!({"audioUrl": voice, "text": "Bí kíp săn deal hot giảm năm mươi phần trăm"}))]);
    let o = ex.execute(&node(&pid, "word_captions", json!({}), inputs)).await.expect("captions");
    let words = o.outputs["words"].as_array().unwrap();
    assert_eq!(words.len(), 10);
    assert!(words.windows(2).all(|w| w[0]["end"].as_f64() <= w[1]["start"].as_f64().map(|s| s + 1e-6)));
    let _ = std::fs::remove_dir_all(workspace::project_path(&pid, &[], false).unwrap());
}

#[tokio::test]
async fn hyperframes_render_node_uses_upstream_images_voice_and_words() {
    let pid = tmp_project("_test_video_hf");
    let t = templates::template_dir("tiktok-sales-v1").unwrap();
    let (img, voice) = (t.join("assets/product.png"), t.join("assets/voiceover.mp3"));
    let inputs = HashMap::from([
        ("img1".to_string(), json!({"previewUrl": img})),
        ("voice".to_string(), json!({"audioUrl": voice, "words": [{"w":"Xin","start":0.2,"end":0.6},{"w":"chào","start":0.6,"end":1.2}]})),
    ]);
    let o = VideoExecutor.execute(&node(&pid, "hyperframes_render", json!({"platform": "tiktok"}), inputs)).await.expect("render node");
    let rel = o.preview_url.unwrap().split("/files/").nth(1).unwrap().to_string();
    let pr = probe_sync(&workspace::project_path(&pid, &[&rel], false).unwrap()).unwrap();
    assert_eq!((pr.width, pr.height, pr.has_audio), (1080, 1920, true));
    assert_eq!(o.outputs["verify"]["tiktok"]["pass_all"], true);
    let _ = std::fs::remove_dir_all(workspace::project_path(&pid, &[], false).unwrap());
}
