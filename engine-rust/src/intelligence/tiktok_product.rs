//! Collect a TikTok Shop product (no login): link -> pdp page JSON -> product.json + downloaded images.
//! Background removal (Toolbox `bg-remove`) is not wired in the Rust engine yet; reported as a warning.
use super::products::*;
use super::util::*;
use regex::Regex;
use serde_json::{json, Value};
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
const LEGAL_SPECS: &[&str] = &["Địa chỉ tổ chức", "Tên tổ chức"];

pub struct CollectError(pub String);
type CResult<T> = Result<T, CollectError>;
fn ce<T>(m: impl Into<String>) -> CResult<T> { Err(CollectError(m.into())) }

fn find_in(u: &str) -> (Option<String>, Option<String>) {
    let dec = percent_decode(&percent_decode(u));
    let m = Regex::new(r"/(?:view/product|pdp(?:/[^/?#]*)?)/(\d{12,})").unwrap().captures(&dec).map(|c| c[1].to_string())
        .or_else(|| Regex::new(r#"product_id"?[=:]"?(\d{12,})"#).unwrap().captures(&dec).map(|c| c[1].to_string()));
    let region = Regex::new(r"shop\.tiktok\.com/([a-z]{2})/").unwrap().captures(u).map(|c| c[1].to_string())
        .or_else(|| url::Url::parse(u).ok().and_then(|x| x.query_pairs().find(|(k, _)| k == "share_region").map(|(_, v)| v.to_lowercase())).filter(|r| !r.is_empty()));
    (m, region)
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) { out.push(v); i += 3; continue; }
        }
        out.push(b[i]); i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// (product_id, region) from any share-link form; follows short links.
pub async fn parse_product_link(url: &str) -> CResult<(String, String)> {
    let url = url.trim();
    if !(url.to_lowercase().starts_with("http://") || url.to_lowercase().starts_with("https://")) { return ce("Link phải bắt đầu bằng http:// hoặc https://"); }
    let host = url::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_lowercase())).unwrap_or_default();
    if !(host == "tiktok.com" || host.ends_with(".tiktok.com")) { return ce(format!("Chưa hỗ trợ trang '{host}'. Hiện chỉ lấy được sản phẩm TikTok Shop.")); }
    let (mut pid, mut region) = find_in(url);
    if pid.is_none() && host != "shop.tiktok.com" {
        let c = reqwest::Client::builder().timeout(Duration::from_secs(20)).user_agent(UA).build().map_err(|e| CollectError(e.to_string()))?;
        let r = c.get(url).send().await.map_err(|e| CollectError(format!("Không mở được link: {e}")))?;
        let (p2, r2) = find_in(r.url().as_str());
        pid = p2; region = region.or(r2);
    }
    match pid { Some(p) => Ok((p, region.unwrap_or_else(|| "vn".into()))), None => ce("Không thấy mã sản phẩm trong link. Dán link chia sẻ sản phẩm từ TikTok Shop.") }
}

pub async fn fetch_product_page(pid: &str, region: &str) -> CResult<Value> {
    let lang = if region == "vn" { "vi-VN,vi;q=0.9" } else { "en-US,en;q=0.9" };
    let c = reqwest::Client::builder().timeout(Duration::from_secs(30)).user_agent(UA).build().map_err(|e| CollectError(e.to_string()))?;
    let r = c.get(format!("https://shop.tiktok.com/{region}/pdp/{pid}")).header("Accept-Language", lang).send().await.map_err(|e| CollectError(format!("Không tải được trang sản phẩm: {e}")))?;
    if !r.status().is_success() { return ce(format!("TikTok Shop trả mã {} cho sản phẩm {pid}", r.status().as_u16())); }
    let html = r.text().await.map_err(|e| CollectError(e.to_string()))?;
    let mut scripts: Vec<String> = Regex::new(r"(?s)<script[^>]*>(.*?)</script>").unwrap().captures_iter(&html).map(|c| c[1].to_string()).collect();
    scripts.sort_by_key(|s| std::cmp::Reverse(s.len()));
    for sc in scripts.iter().take(4) {
        let Ok(data) = serde_json::from_str::<Value>(sc) else { continue };
        for (key, ld) in data.get("loaderData").and_then(|l| l.as_object()).into_iter().flatten() {
            if !key.contains("pdp") || !ld.is_object() { continue; }
            for comp in ld.pointer("/page_config/components_map").and_then(|c| c.as_array()).into_iter().flatten() {
                if let Some(cd) = comp.get("component_data") {
                    if cd.pointer("/product_info/title").is_some_and(|t| t.as_str().is_some_and(|t| !t.is_empty())) { return Ok(cd.clone()); }
                }
            }
        }
    }
    ce("Trang không có dữ liệu sản phẩm (sản phẩm đã gỡ, sai vùng, hoặc TikTok đang yêu cầu xác minh). Thử mở link trên trình duyệt để kiểm tra.")
}

fn money(v: Option<&Value>) -> Option<f64> {
    let s = match v? { Value::String(s) => s.clone(), Value::Number(n) => n.to_string(), _ => return None };
    s.replace(['.', ','], "").parse().ok()
}
fn vs(v: Option<&Value>) -> String { v.and_then(|x| x.as_str()).unwrap_or("").to_string() }

/// Page JSON -> Product fields (+ `_gallery`, colors[].`_image`).
pub fn extract_product(cd: &Value, product_id: &str, region: &str) -> Value {
    let pi = &cd["product_info"];
    let skus = pi["skus"].as_array().cloned().unwrap_or_default();
    let sale: Vec<f64> = skus.iter().filter_map(|s| money(s.pointer("/price/sale_price_decimal"))).filter(|x| *x != 0.0).collect();
    let orig: Vec<f64> = skus.iter().filter_map(|s| money(s.pointer("/price/origin_price_decimal"))).filter(|x| *x != 0.0).collect();
    let props = pi["sale_props"].as_array().cloned().unwrap_or_default();
    let (mut colors, mut sizes): (Vec<Value>, Vec<String>) = (vec![], vec![]);
    for p in &props {
        let vals = p["sale_prop_values"].as_array().cloned().unwrap_or_default();
        let pn = vs(p.get("prop_name")).to_lowercase();
        if p["has_image"] == true || pn.contains("màu") || pn.contains("color") {
            colors.extend(vals.iter().map(|v| json!({"name": vs(v.get("prop_value")), "_image": v.get("image").cloned().unwrap_or(json!({}))})));
        } else { sizes.extend(vals.iter().map(|v| vs(v.get("prop_value")))); }
    }
    let desc = pi["desc_blocks"].as_array().cloned().unwrap_or_default().iter().filter(|b| b["type"] == "text").map(|b| vs(b.get("text")).trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join("\n");
    let care_at = Regex::new(r"(?i)CÁCH GIẶT|HƯỚNG DẪN BẢO QUẢN|BẢO QUẢN").unwrap().find(&desc).map(|m| m.start());
    let (body, care) = match care_at { Some(i) => (&desc[..i], &desc[i..]), None => (&desc[..], "") };
    let bullets = |t: &str| t.lines().filter(|l| l.trim().starts_with('*')).map(|l| l.trim().trim_start_matches('*').trim().to_string()).collect::<Vec<_>>();
    let (bul, care_lines) = (bullets(body), bullets(care));
    let mut specs = serde_json::Map::new();
    for sp in pi["specifications"].as_array().into_iter().flatten() {
        let n = vs(sp.get("name"));
        if !n.is_empty() && !LEGAL_SPECS.iter().any(|l| n.starts_with(l)) { specs.insert(n, json!(vs(sp.get("value")))); }
    }
    if !sizes.is_empty() { specs.insert("Kích cỡ".into(), json!(sizes.iter().map(|s| s.replace("Size ", "")).collect::<Vec<_>>().join(", "))); }
    if !colors.is_empty() { specs.insert("Màu sắc".into(), json!(colors.iter().map(|c| vs(c.get("name"))).collect::<Vec<_>>().join(", "))); }
    if !care_lines.is_empty() { specs.insert("Bảo quản".into(), json!(care_lines.join("; "))); }
    let cats = cd["categories"].as_array().cloned().unwrap_or_default().iter().map(|c| vs(c.get("category_name"))).filter(|c| !c.is_empty()).collect::<Vec<_>>();
    let first_prop = props.first().map(|p| vs(p.get("prop_name"))).unwrap_or_default();
    let max = |v: &[f64]| v.iter().cloned().fold(f64::MIN, f64::max);
    let min = |v: &[f64]| v.iter().cloned().fold(f64::MAX, f64::min);
    let variants: Vec<Value> = skus.iter().map(|s| {
        let sp = s["sku_sale_props"].as_array().cloned().unwrap_or_default();
        json!({"color": sp.iter().find(|q| vs(q.get("prop_name")) == first_prop).map(|q| vs(q.get("prop_value"))).unwrap_or_default(),
               "size": if props.is_empty() { String::new() } else { sp.iter().find(|q| vs(q.get("prop_name")) != first_prop).map(|q| vs(q.get("prop_value"))).unwrap_or_default() },
               "stock": s.get("stock").cloned().unwrap_or(json!(0)), "price": s.pointer("/price/sale_price_decimal").cloned().unwrap_or(Value::Null)})
    }).collect();
    json!({
        "id": product_id, "name": pi["title"], "category": if cats.is_empty() { "general".to_string() } else { cats.join(" > ") },
        "price": if !orig.is_empty() { max(&orig) } else if !sale.is_empty() { max(&sale) } else { 0.0 },
        "sale_price": if sale.is_empty() { Value::Null } else { json!(min(&sale)) }, "original_price": if orig.is_empty() { Value::Null } else { json!(max(&orig)) },
        "discount": vs(pi.pointer("/price/discount")), "currency": Some(vs(pi.pointer("/price/currency"))).filter(|c| !c.is_empty()).unwrap_or_else(|| "VND".into()),
        "shop_name": Some(vs(pi.pointer("/seller/name"))).filter(|c| !c.is_empty()).unwrap_or_else(|| vs(cd.pointer("/shop_info/shop_name"))),
        "sold_count": money(pi.get("sold_count")).unwrap_or(0.0) as i64,
        "rating": pi.pointer("/review/product_rating").and_then(|r| match r { Value::String(s) => s.parse::<f64>().ok(), Value::Number(n) => n.as_f64(), _ => None }),
        "review_count": money(pi.pointer("/review/review_count")).unwrap_or(0.0) as i64,
        "description": desc, "usp": bul.into_iter().take(8).collect::<Vec<_>>(), "specs": specs, "colors": colors, "sizes": sizes, "variants": variants,
        "_gallery": pi["images"].clone(), "source_url": format!("https://shop.tiktok.com/{region}/pdp/{product_id}"),
    })
}

async fn download(c: &reqwest::Client, img: &Value, dest: &std::path::Path) -> bool {
    for u in img["url_list"].as_array().into_iter().flatten().filter_map(|u| u.as_str()) {
        if let Ok(r) = c.get(u).send().await {
            if r.status().is_success() { if let Ok(b) = r.bytes().await { if !b.is_empty() && std::fs::write(dest, &b).is_ok() { return true; } } }
        }
    }
    false
}

pub async fn collect_product(pid: &str, url: &str, cutout: bool) -> CResult<(Product, Vec<String>)> {
    let mut warnings = vec![];
    let (prod_id, region) = parse_product_link(url).await?;
    safe_id(&prod_id).map_err(|e| CollectError(e.to_string()))?;
    let mut info = extract_product(&fetch_product_page(&prod_id, &region).await?, &prod_id, &region);
    let rel = format!("products/{prod_id}/images");
    let base = pp(pid, &["products", &prod_id, "images", "raw", "x"], true).parent().unwrap().parent().unwrap().to_path_buf();
    let raw_dir = base.join("raw");
    let gallery_items = info.as_object_mut().unwrap().remove("_gallery").and_then(|g| g.as_array().cloned()).unwrap_or_default();
    let (mut gallery, mut variants, mut seen) = (vec![], vec![], std::collections::HashSet::new());
    let c = reqwest::Client::builder().timeout(Duration::from_secs(30)).user_agent(UA).build().map_err(|e| CollectError(e.to_string()))?;
    for (i, im) in gallery_items.iter().enumerate() {
        let name = format!("gallery_{:02}.webp", i + 1);
        if download(&c, im, &raw_dir.join(&name)).await { gallery.push(name); seen.insert(vs(im.get("uri"))); } else { warnings.push(format!("Không tải được ảnh bộ sưu tập {}", i + 1)); }
    }
    if let Some(cols) = info["colors"].as_array_mut() {
        for col in cols.iter_mut() {
            let im = col.as_object_mut().and_then(|o| o.remove("_image")).unwrap_or(json!({}));
            let uri = vs(im.get("uri"));
            if im["url_list"].as_array().is_some_and(|a| !a.is_empty()) && !seen.contains(&uri) {
                let stem = format!("variant_{}", slug(&vs(col.get("name"))));
                if download(&c, &im, &raw_dir.join(format!("{stem}.webp"))).await { variants.push(format!("{stem}.webp")); seen.insert(uri); col["image"] = json!(format!("{rel}/cutout/{stem}.png")); }
            } else if seen.contains(&uri) { col["image"] = Value::Null; }
        }
    }
    if variants.is_empty() { warnings.push("Sản phẩm không có ảnh theo biến thể: chưa tách nền (ảnh bộ sưu tập thường có người mẫu).".into()); }
    let mut cutouts: Vec<String> = vec![];
    if cutout && !variants.is_empty() { warnings.push("Tách nền (Toolbox bg-remove) chưa được nối vào engine Rust: giữ ảnh gốc, hãy chạy công cụ tách nền riêng.".into()); }
    let old = get_product(pid, &prod_id);
    let mut prod: Product = serde_json::from_value(info.clone()).map_err(|e| CollectError(format!("Dữ liệu sản phẩm không hợp lệ: {e}")))?;
    if let Some(o) = old {
        prod.review_status = o.review_status; prod.review_videos = o.review_videos; prod.reviewed_at = o.reviewed_at; prod.notes = o.notes;
        prod.brand = o.brand; prod.target_audience = o.target_audience; prod.compliance_notes = o.compliance_notes; prod.pain_points = o.pain_points;
        let proj = pp(pid, &[], false);
        let mut extra: Vec<String> = o.cutout_images.into_iter().filter(|c| !cutouts.contains(c) && proj.join(c).is_file()).collect();
        extra.extend(cutouts); cutouts = extra;
    }
    let raw_rel: Vec<String> = gallery.iter().chain(&variants).map(|n| format!("{rel}/raw/{n}")).collect();
    prod.affiliate_links = AffiliateLinks { tiktok_shop: prod.source_url.clone(), ..Default::default() };
    prod.images = cutouts.iter().cloned().chain(raw_rel.iter().filter(|r| r.contains("/gallery_")).cloned()).collect();
    prod.cutout_images = cutouts; prod.raw_images = raw_rel;
    let _ = write_json(&base.parent().unwrap().join("collected.json"), &json!({"url": url, "region": region, "warnings": warnings}));
    save_product(pid, &prod).map_err(|e| CollectError(e.to_string()))?;
    if prod.currency == "VND" && prod.sale_price.is_some() && prod.original_price.is_some() {
        warnings.push("Giá lấy theo phiên không đăng nhập; giá trong ứng dụng có thể khác (voucher, vùng). Kiểm tra lại trước khi đọc giá trong video.".into());
    }
    Ok((prod, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn link_and_extract() {
        let (p, r) = find_in("https://shop.tiktok.com/vn/pdp/slug/1729384756012345");
        assert_eq!((p.as_deref(), r.as_deref()), (Some("1729384756012345"), Some("vn")));
        assert_eq!(find_in("https://www.tiktok.com/view/product/1729384756012345?share_region=SG").1.as_deref(), Some("sg"));
        let cd = json!({"product_info": {"title": "Áo", "skus": [{"price": {"sale_price_decimal": "199.000", "origin_price_decimal": "299.000"}, "stock": 3, "sku_sale_props": [{"prop_name": "Màu", "prop_value": "Đen"}, {"prop_name": "Size", "prop_value": "M"}]}],
            "sale_props": [{"prop_name": "Màu", "has_image": true, "sale_prop_values": [{"prop_value": "Đen", "image": {"uri": "u1", "url_list": ["http://x"]}}]}, {"prop_name": "Size", "sale_prop_values": [{"prop_value": "Size M"}]}],
            "desc_blocks": [{"type": "text", "text": "* Vải dày\n* Form chuẩn\nCÁCH GIẶT\n* Giặt tay"}]}});
        let v = extract_product(&cd, "1", "vn");
        assert_eq!(v["price"], 299000.0);
        assert_eq!(v["sale_price"], 199000.0);
        assert_eq!(v["usp"].as_array().unwrap().len(), 2);
        assert_eq!(v["specs"]["Bảo quản"], "Giặt tay");
        assert_eq!(v["variants"][0]["size"], "M");
    }
}
