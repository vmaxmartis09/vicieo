use super::util::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AffiliateLinks { pub tiktok_shop: String, pub shopee: String, pub lazada: String, pub other: String }

fn cat() -> String { "general".into() }
fn vnd() -> String { "VND".into() }
fn chua() -> String { "chua_lam".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub brand: String,
    #[serde(default = "cat")] pub category: String,
    pub price: f64,
    pub sale_price: Option<f64>,
    #[serde(default = "vnd")] pub currency: String,
    pub affiliate_links: AffiliateLinks,
    pub usp: Vec<String>,
    pub pain_points: Vec<String>,
    pub specs: BTreeMap<String, String>,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub target_audience: String,
    pub compliance_notes: String,
    pub description: String,
    pub source_url: String,
    pub shop_name: String,
    pub original_price: Option<f64>,
    pub discount: String,
    pub sold_count: i64,
    pub rating: Option<f64>,
    pub review_count: i64,
    pub colors: Vec<Value>,
    pub sizes: Vec<String>,
    pub variants: Vec<Value>,
    pub cutout_images: Vec<String>,
    pub raw_images: Vec<String>,
    #[serde(default = "chua")] pub review_status: String,
    pub review_videos: Vec<String>,
    pub reviewed_at: String,
    pub notes: String,
}

impl Default for Product {
    fn default() -> Self { Product::new("", "") }
}
impl Product {
    pub fn new(id: &str, name: &str) -> Self {
        Product {
            id: id.into(), name: name.into(), brand: String::new(), category: cat(), price: 0.0, sale_price: None,
            currency: vnd(), affiliate_links: AffiliateLinks::default(), usp: vec![], pain_points: vec![],
            specs: BTreeMap::new(), images: vec![], videos: vec![], target_audience: String::new(),
            compliance_notes: String::new(), description: String::new(), source_url: String::new(),
            shop_name: String::new(), original_price: None, discount: String::new(), sold_count: 0, rating: None,
            review_count: 0, colors: vec![], sizes: vec![], variants: vec![], cutout_images: vec![], raw_images: vec![],
            review_status: chua(), review_videos: vec![], reviewed_at: String::new(), notes: String::new(),
        }
    }
}

pub fn list_products(pid: &str) -> Vec<Product> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(pp(pid, &["products"], false)) {
        let mut dirs: Vec<_> = rd.flatten().filter(|e| e.path().is_dir()).collect();
        dirs.sort_by_key(|e| e.file_name());
        for e in dirs {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with('.') || n.starts_with('_') { continue; }
            if let Some(v) = read_json(&e.path().join("product.json")) {
                if let Ok(p) = serde_json::from_value::<Product>(v) { out.push(p); }
            }
        }
    }
    out
}

pub fn get_product(pid: &str, id: &str) -> Option<Product> {
    safe_id(id).ok()?;
    serde_json::from_value(read_json(&pp(pid, &["products", id, "product.json"], false))?).ok()
}

pub fn save_product(pid: &str, p: &Product) -> anyhow::Result<()> {
    write_json(&pp(pid, &["products", safe_id(&p.id)?, "product.json"], true), p)
}

pub fn delete_product(pid: &str, id: &str) -> bool {
    if safe_id(id).is_err() { return false; }
    let dir = pp(pid, &["products", id], false);
    if !dir.exists() { return false; }
    let trash = pp(pid, &["products", "_trash", id], true);
    let _ = std::fs::remove_dir_all(&trash);
    std::fs::rename(&dir, &trash).is_ok()
}

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

fn meta(html: &str, prop: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"(?is)<meta\s+(?:property|name)=["']{prop}["']\s+content=["'](.*?)["']"#)).ok()?;
    re.captures(html).map(|c| c[1].trim().to_string())
}

/// Public OpenGraph metadata only (no login / cookies), same heuristics as Python.
pub async fn import_product_from_url(url: &str) -> Value {
    static PRICE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)([0-9]{2,3}(?:\.[0-9]{3})+)\s*(?:đ|VND|d)").unwrap());
    static TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<title>(.*?)</title>").unwrap());
    let (mut title, mut image, mut desc, mut price) = (String::new(), String::new(), String::new(), 0.0);
    if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).user_agent(UA).build() {
        if let Ok(r) = client.get(url).send().await {
            if r.status().is_success() {
                let html = r.text().await.unwrap_or_default();
                title = meta(&html, "og:title").or_else(|| TITLE.captures(&html).map(|c| c[1].trim().to_string())).unwrap_or_default();
                image = meta(&html, "og:image").unwrap_or_default();
                desc = meta(&html, "og:description").unwrap_or_default();
                if let Some(c) = PRICE.captures(&html) { price = c[1].replace('.', "").parse().unwrap_or(0.0); }
            }
        }
    }
    json!({"title": if title.is_empty() { "Sản phẩm mới".to_string() } else { title }, "description": desc, "image": image, "price": price, "url": url})
}
