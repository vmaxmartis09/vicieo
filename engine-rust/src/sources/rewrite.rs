use crate::intelligence::util::*;
use serde_json::{json, Value};

pub fn rewrite_script_lines(lines: &[Value], style: &str) -> Vec<Value> {
    let n = lines.len();
    lines.iter().enumerate().map(|(i, l)| {
        let text = l.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let (st, en) = (l["start"].as_f64().unwrap_or(0.0), l["end"].as_f64().unwrap_or(0.0));
        let new = match style {
            "hook_boost" if i == 0 => format!("Dừng lại 3 giây! {text} bạn đã biết chưa?"),
            "hook_boost" if i == n - 1 => format!("{text}. Thả tim và theo dõi ngay nhé!"),
            "character" if i == 0 => format!("Bi chào các bạn nhỏ nè! Hôm nay cùng Bi xem {text} nhé!"),
            "character" if i == n - 1 => format!("{text}. Hẹn gặp lại các bạn ở tập sau nha!"),
            "character" => text.replace("mình", "Bi").replace("tôi", "Bi"),
            _ => text.clone(),
        };
        json!({"line_idx": i + 1, "start": (st * 100.0).round() / 100.0, "end": (en * 100.0).round() / 100.0,
               "target_duration_s": ((en - st).max(1.0) * 100.0).round() / 100.0, "text": new, "original_vi": text})
    }).collect()
}

pub fn run_rewrite(pid: &str, source_id: &str, style: &str) -> ApiResult<Vec<Value>> {
    let dir = pp(pid, &["raw", safe_id(source_id).map_err(|e| ApiError::bad(e.to_string()))?, "x"], false).parent().unwrap().to_path_buf();
    let vi = read_json(&dir.join("transcript.vi.json")).and_then(|v| v.as_array().cloned()).ok_or_else(|| ApiError::nf("Vietnamese transcript not found"))?;
    let out = rewrite_script_lines(&vi, style);
    write_json(&pp(pid, &["content", "scripts", &format!("{source_id}.{style}.json")], true), &out)?;
    write_json(&dir.join(format!("script.{style}.json")), &out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn styles() {
        let l = vec![json!({"text":"a","start":0.0,"end":2.0}), json!({"text":"mình thử","start":2.0,"end":4.0})];
        assert!(rewrite_script_lines(&l, "hook_boost")[0]["text"].as_str().unwrap().starts_with("Dừng lại"));
        assert!(rewrite_script_lines(&l, "character")[1]["text"].as_str().unwrap().contains("Hẹn gặp"));
        assert_eq!(rewrite_script_lines(&l, "faithful")[1]["text"], "mình thử");
    }
}
