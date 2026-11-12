//! Node execution interface. Feature tracks implement `NodeExecutor` and register it in
//! `ExecutorRegistry::with_defaults()`; the graph runner resolves by node type and falls back
//! to `FallbackExecutor` (placeholder previews) for types nobody owns yet.
use crate::workspace;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct NodeCtx {
    pub project_id: String,
    pub run_id: String,
    pub node_id: String,
    pub node_type: String,
    pub custom_name: String,
    pub params: Value,
    /// Outputs of upstream nodes keyed by source node id.
    pub inputs: HashMap<String, Value>,
    /// SSE broadcast channel (same one /events subscribes to).
    pub tx: broadcast::Sender<String>,
    pub cancel: CancellationToken,
    pub mock: bool,
}

#[derive(Debug, Clone)]
pub struct NodeOutput {
    /// "text" | "image" | "video" | "audio"
    pub preview_type: String,
    pub preview_url: Option<String>,
    pub summary: String,
    pub cost_usd: f64,
    pub outputs: Value,
    pub artifacts: Vec<PathBuf>,
}

#[async_trait]
pub trait NodeExecutor: Send + Sync {
    fn node_types(&self) -> &'static [&'static str];
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput>;
}

#[derive(Clone, Default)]
pub struct ExecutorRegistry {
    map: HashMap<String, Arc<dyn NodeExecutor>>,
    fallback: Option<Arc<dyn NodeExecutor>>,
}

impl ExecutorRegistry {
    pub fn with_defaults() -> Self {
        let mut r = Self { map: HashMap::new(), fallback: Some(Arc::new(FallbackExecutor)) };
        crate::routes::voice_impl::register(&mut r);
        crate::video::register(&mut r);
        let _ = &mut r;
        r
    }

    pub fn register(&mut self, ex: Arc<dyn NodeExecutor>) {
        for t in ex.node_types() {
            self.map.insert((*t).to_string(), ex.clone());
        }
    }

    pub fn get(&self, node_type: &str) -> Option<Arc<dyn NodeExecutor>> {
        self.map.get(node_type).cloned().or_else(|| self.fallback.clone())
    }
}

/// Placeholder previews (no real generation). Preserves the pre-split behaviour.
pub struct FallbackExecutor;

#[async_trait]
impl NodeExecutor for FallbackExecutor {
    fn node_types(&self) -> &'static [&'static str] {
        &[]
    }

    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let (preview_type, preview_url, summary) = fallback_preview(ctx);
        Ok(NodeOutput {
            preview_type,
            preview_url,
            summary,
            cost_usd: 0.0,
            outputs: json!({}),
            artifacts: vec![],
        })
    }
}

fn fallback_preview(ctx: &NodeCtx) -> (String, Option<String>, String) {
                let r = match ctx.node_type.as_str() {
                "script" => (
                    "text".to_string(),
                    None,
                    format!("Kịch bản phân cảnh ({} cảnh)", ctx.params.get("targetSceneCount").and_then(|c| c.as_i64()).unwrap_or(6))
                ),
                "quality_check" => (
                    "text".to_string(),
                    None,
                    "Đạt kiểm định 100% (An toàn chính sách)".to_string()
                ),
                "image_gen" | "image_edit" | "image" | "character" | "product" | "bg_remove" | "depth_layers" => {
                    let mut img_url = ctx.params.get("image").or_else(|| ctx.params.get("variant")).and_then(|s| s.as_str()).map(|s| s.to_string());
                    if img_url.is_none() {
                        let char_dir = workspace::project_path(&ctx.project_id, &["characters", "soc_nho"], false).unwrap_or_default();
                        if let Ok(entries) = fs::read_dir(char_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().and_then(|e| e.to_str()) == Some("png") {
                                    img_url = Some(format!("/data/projects/{}/files/characters/soc_nho/{}", ctx.project_id, p.file_name().unwrap().to_str().unwrap()));
                                    break;
                                }
                            }
                        }
                    } else if let Some(ref u) = img_url {
                        if !u.starts_with("http") && !u.starts_with('/') {
                            img_url = Some(format!("/data/projects/{}/files/{}", ctx.project_id, u));
                        }
                    }
                    ("image".to_string(), img_url, format!("Hoàn thành tạo ảnh {}", ctx.custom_name))
                },
                "video_gen" | "hyperframes_render" | "handdraw" | "scene_cut" | "export" => {
                    let mut vid_url = ctx.params.get("video").and_then(|s| s.as_str()).map(|s| s.to_string());
                    if vid_url.is_none() {
                        let out_dir = workspace::project_path(&ctx.project_id, &["output"], false).unwrap_or_default();
                        if let Ok(entries) = fs::read_dir(out_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().and_then(|e| e.to_str()) == Some("mp4") {
                                    vid_url = Some(format!("/data/projects/{}/files/output/{}", ctx.project_id, p.file_name().unwrap().to_str().unwrap()));
                                    break;
                                }
                            }
                        }
                    } else if let Some(ref u) = vid_url {
                        if !u.starts_with("http") && !u.starts_with('/') {
                            vid_url = Some(format!("/data/projects/{}/files/{}", ctx.project_id, u));
                        }
                    }
                    ("video".to_string(), vid_url, format!("Hoàn thành dựng video {}", ctx.custom_name))
                },
                "voice_tts" | "voice_clone" => {
                    let voice_id = ctx.params.get("voiceId").and_then(|s| s.as_str()).unwrap_or("soc-nho");
                    ("audio".to_string(), Some(format!("/voices/{}/sample", voice_id)), format!("Đã tạo giọng đọc ({})", voice_id))
                },
                "captions" | "word_captions" => (
                    "text".to_string(),
                    None,
                    "Phụ đề kinetic karaoke (khớp mốc thời gian từng từ)".to_string()
                ),
                _ => (
                    "text".to_string(),
                    None,
                    format!("Hoàn thành {}", ctx.custom_name)
                ),
            };
    r
}
