//! USD spend guard persisted in <project>/runs/spend.json (same shape as spend_guard.py).
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Mutex;

static LOCK: Mutex<()> = Mutex::new(());

pub struct SpendGuard {
    path: PathBuf,
    pub cap: f64,
    pub daily_cap: f64,
}

fn env_f64(names: &[&str]) -> Option<f64> {
    names.iter().find_map(|n| std::env::var(n).ok().and_then(|v| v.trim().parse().ok()))
}

impl SpendGuard {
    pub fn new(runs_dir: PathBuf) -> Self {
        let cap = env_f64(&["VICIEO_SPEND_CAP_USD", "SOCNHOTV_SPEND_CAP_USD"]).unwrap_or(5.0);
        let daily_cap = env_f64(&["VICIEO_DAILY_CAP_USD", "SOCNHOTV_DAILY_CAP_USD"]).unwrap_or(cap);
        Self { path: runs_dir.join("spend.json"), cap, daily_cap }
    }

    fn read(&self) -> Value {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_else(|| json!({"totalUsd": 0.0, "capUsd": self.cap, "entries": []}))
    }

    fn spent_24h(&self, now: f64) -> f64 {
        self.read()["entries"].as_array().map(|es| {
            es.iter()
                .filter(|e| e["ts"].as_f64().map(|t| t >= now - 86400.0).unwrap_or(true))
                .map(|e| e["usd"].as_f64().unwrap_or(0.0))
                .sum::<f64>()
                .max(0.0)
        }).unwrap_or(0.0)
    }

    pub fn preview(&self, amount: f64) -> Value {
        let now = chrono::Utc::now().timestamp() as f64;
        let total = self.read()["totalUsd"].as_f64().unwrap_or(0.0);
        let daily = self.spent_24h(now);
        let reason = if total + amount > self.cap + 1e-9 {
            Some(format!("Vượt trần chi tiêu dự án: đã chi ${:.2}, cần thêm ${:.2}, trần ${:.2}.", total, amount, self.cap))
        } else if daily + amount > self.daily_cap + 1e-9 {
            Some(format!("Vượt trần chi tiêu 24h: đã chi ${:.2}, cần thêm ${:.2}, trần ngày ${:.2}.", daily, amount, self.daily_cap))
        } else {
            None
        };
        json!({
            "canProceed": reason.is_none(), "reason": reason,
            "requestedUsd": amount, "projectSpentUsd": total, "projectCapUsd": self.cap,
            "dailySpentUsd": daily, "dailyCapUsd": self.daily_cap,
            "remainingBudgetUsd": (self.cap - total).min(self.daily_cap - daily).max(0.0),
        })
    }

    pub fn reserve(&self, amount: f64, label: &str) -> anyhow::Result<()> {
        if amount <= 0.0 {
            return Ok(());
        }
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let check = self.preview(amount);
        if check["canProceed"] != true {
            anyhow::bail!("{} ({})", check["reason"].as_str().unwrap_or("spend cap"), label);
        }
        self.append(amount, label)
    }

    pub fn refund(&self, amount: f64, label: &str) {
        if amount > 0.0 {
            let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let _ = self.append(-amount, &format!("hoàn: {}", label));
        }
    }

    fn append(&self, usd: f64, label: &str) -> anyhow::Result<()> {
        let mut d = self.read();
        let total = (d["totalUsd"].as_f64().unwrap_or(0.0) + usd).max(0.0);
        d["totalUsd"] = json!((total * 10000.0).round() / 10000.0);
        d["capUsd"] = json!(self.cap);
        d["dailyCapUsd"] = json!(self.daily_cap);
        let entry = json!({"label": label, "usd": usd, "source": "api", "ts": chrono::Utc::now().timestamp()});
        match d["entries"].as_array_mut() {
            Some(a) => a.push(entry),
            None => d["entries"] = json!([entry]),
        }
        if let Some(p) = self.path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&self.path, serde_json::to_string_pretty(&d)? + "\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reserve_respects_cap() {
        let d = tempfile::tempdir().unwrap();
        let mut g = SpendGuard::new(d.path().to_path_buf());
        g.cap = 1.0;
        g.daily_cap = 1.0;
        g.reserve(0.6, "a").unwrap();
        assert!(g.reserve(0.6, "b").is_err());
        g.refund(0.6, "a");
        g.reserve(0.6, "c").unwrap();
    }
}
