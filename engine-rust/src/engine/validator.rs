//! Graph validation + dependency/topological ordering (port of validator.py).
//! Shortcut: capability-driven custom ports (NodeRegistry.effective_ports) are not modelled; ports come from
//! data/shared/node_types.json as declared. Upgrade if custom-port nodes appear in saved graphs.
use crate::workspace;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

pub struct Validation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    /// Same errors keyed by the node they belong to, so the UI can show each one on its own node.
    pub node_errors: Map<String, Value>,
    pub warnings: Vec<String>,
    pub topological_order: Vec<String>,
}

fn blame(m: &mut Map<String, Value>, id: &str, msg: &str) {
    let joined = match m.get(id).and_then(|v| v.as_str()) {
        Some(prev) => format!("{prev}; {msg}"),
        None => msg.to_string(),
    };
    m.insert(id.to_string(), json!(joined));
}

impl Validation {
    pub fn to_json(&self) -> Value {
        json!({"isValid": self.is_valid, "valid": self.is_valid, "errors": self.errors, "warnings": self.warnings, "topologicalOrder": self.topological_order})
    }
}

pub fn node_types() -> HashMap<String, Value> {
    workspace::shared_path(&["node_types.json"], false).ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<Vec<Value>>(&t).ok()).unwrap_or_default()
        .into_iter().filter_map(|d| d["typeName"].as_str().map(|n| (n.to_string(), d.clone()))).collect()
}

pub fn is_bypassed(node: &Value) -> bool {
    match &node["params"]["_bypass"] {
        Value::Bool(b) => *b,
        Value::String(s) => matches!(s.trim().to_lowercase().as_str(), "true" | "1" | "yes"),
        Value::Number(n) => n.as_f64() == Some(1.0),
        _ => false,
    }
}

fn str_of<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}

fn scan_refs(v: &Value, re: &Regex, out: &mut HashSet<String>) {
    match v {
        Value::String(s) => re.captures_iter(s).for_each(|c| {
            out.insert(c[1].to_string());
        }),
        Value::Object(m) => {
            if m.get("$ref").and_then(|x| x.as_str()) == Some("v1") {
                if let Some(e) = m.get("expr").and_then(|x| x.as_str()) {
                    re.captures_iter(e).for_each(|c| {
                        out.insert(c[1].to_string());
                    });
                }
            }
            m.values().for_each(|x| scan_refs(x, re, out));
        }
        Value::Array(a) => a.iter().for_each(|x| scan_refs(x, re, out)),
        _ => {}
    }
}

/// parents[child] = node ids it depends on (edges, node.inputs refs, `{{ $node.<id> }}` refs), in stable order.
/// Returns (node ids in file order, parents, self-reference errors).
pub fn build_deps(graph: &Value) -> (Vec<String>, HashMap<String, Vec<String>>, Vec<(String, String)>) {
    let nodes = graph["nodes"].as_array().cloned().unwrap_or_default();
    let ids: Vec<String> = nodes.iter().map(|n| str_of(n, "id").to_string()).filter(|i| !i.is_empty()).collect();
    let set: HashSet<&String> = ids.iter().collect();
    let mut parents: HashMap<String, Vec<String>> = ids.iter().map(|i| (i.clone(), vec![])).collect();
    let mut errors = vec![];
    let add = |parents: &mut HashMap<String, Vec<String>>, src: &str, tgt: &str| {
        if let Some(p) = parents.get_mut(tgt) {
            if !p.iter().any(|x| x == src) {
                p.push(src.to_string());
            }
        }
    };
    for e in graph["edges"].as_array().into_iter().flatten() {
        let (s, t) = (str_of(e, "sourceNodeId"), str_of(e, "targetNodeId"));
        if set.contains(&s.to_string()) && set.contains(&t.to_string()) {
            add(&mut parents, s, t);
        }
    }
    let re = Regex::new(r"\{\{\s*\$node\.([a-zA-Z0-9_-]+)").unwrap();
    for n in &nodes {
        let t = str_of(n, "id");
        for r in n["inputs"].as_object().into_iter().flat_map(|m| m.values()) {
            if let Some(s) = r.get("nodeId").and_then(|x| x.as_str()) {
                if set.contains(&s.to_string()) {
                    add(&mut parents, s, t);
                }
            }
        }
        let mut found = HashSet::new();
        scan_refs(&n["params"], &re, &mut found);
        let mut found: Vec<_> = found.into_iter().collect();
        found.sort();
        for s in found {
            if s == t {
                errors.push((t.to_string(), format!("Node '{}' tự tham chiếu chính nó qua biểu thức '$node.{}'.", t, t)));
            } else if set.contains(&s) {
                add(&mut parents, &s, t);
            }
        }
    }
    (ids, parents, errors)
}

/// Kahn's algorithm. Err(nodes still blocked) when the graph has a cycle.
pub fn topo_sort(ids: &[String], parents: &HashMap<String, Vec<String>>) -> Result<Vec<String>, Vec<String>> {
    let mut indeg: HashMap<&String, usize> = ids.iter().map(|i| (i, parents.get(i).map(|p| p.len()).unwrap_or(0))).collect();
    let mut children: HashMap<&String, Vec<&String>> = HashMap::new();
    for i in ids {
        for p in parents.get(i).into_iter().flatten() {
            if let Some((k, _)) = indeg.get_key_value(p) {
                children.entry(*k).or_default().push(i);
            }
        }
    }
    let mut queue: std::collections::VecDeque<&String> = ids.iter().filter(|i| indeg[*i] == 0).collect();
    let mut order = vec![];
    while let Some(u) = queue.pop_front() {
        order.push(u.clone());
        for v in children.get(u).into_iter().flatten() {
            let d = indeg.get_mut(*v).unwrap();
            *d -= 1;
            if *d == 0 {
                queue.push_back(v);
            }
        }
    }
    if order.len() == ids.len() {
        Ok(order)
    } else {
        let mut stuck: Vec<String> = indeg.into_iter().filter(|(_, d)| *d > 0).map(|(i, _)| i.clone()).collect();
        stuck.sort();
        Err(stuck)
    }
}

fn compatible(src: &str, tgt: &str) -> bool {
    src == tgt || (src == "image" && tgt == "image[]") || (src == "dialogue" && tgt == "text")
}

/// Ports incl. enabled capabilities (params[cap].enabled), mirroring Python NodeRegistry.effective_ports.
fn effective(def: &Value, params: &Value) -> Value {
    let mut d = def.clone();
    for cap in def["capabilities"].as_array().into_iter().flatten() {
        if params[cap["name"].as_str().unwrap_or("")]["enabled"] != true {
            continue;
        }
        for (key, extra) in [("inputPorts", &cap["inputPorts"]), ("outputPorts", &cap["outputPorts"])] {
            for p in extra.as_array().into_iter().flatten() {
                let exists = d[key].as_array().into_iter().flatten().any(|q| q["name"] == p["name"]);
                if !exists {
                    d[key].as_array_mut().map(|a| a.push(p.clone()));
                }
            }
        }
    }
    d
}

fn port<'a>(def: &'a Value, key: &str, name: &str) -> Option<&'a Value> {
    def[key].as_array()?.iter().find(|p| p["name"] == name)
}

pub fn validate(graph: &Value) -> Validation {
    let types = node_types();
    let (mut errors, mut warnings, mut node_errors) = (vec![], vec![], Map::new());
    let nodes = graph["nodes"].as_array().cloned().unwrap_or_default();
    let mut map: HashMap<String, &Value> = HashMap::new();
    for n in &nodes {
        let id = str_of(n, "id");
        if map.contains_key(id) {
            let msg = format!("Trùng lặp mã Node ID '{}' trong đồ thị.", id);
            blame(&mut node_errors, id, &msg);
            errors.push(msg);
            continue;
        }
        map.insert(id.to_string(), n);
        if !types.contains_key(str_of(n, "type")) {
            let msg = format!("Loại node '{}' tại node '{}' không có trong NodeRegistry.", str_of(n, "type"), id);
            blame(&mut node_errors, id, &msg);
            errors.push(msg);
        }
    }
    let mut connected: HashSet<(String, String)> = HashSet::new();
    for e in graph["edges"].as_array().into_iter().flatten() {
        let (sid, tid, eid) = (str_of(e, "sourceNodeId"), str_of(e, "targetNodeId"), str_of(e, "id"));
        let (Some(sn), Some(tn)) = (map.get(sid), map.get(tid)) else {
            if !map.contains_key(sid) {
                let msg = format!("Dây nối '{}' trỏ từ node không tồn tại '{}'.", eid, sid);
                blame(&mut node_errors, tid, &msg);
                errors.push(msg);
            } else {
                let msg = format!("Dây nối '{}' trỏ tới node không tồn tại '{}'.", eid, tid);
                blame(&mut node_errors, sid, &msg);
                errors.push(msg);
            }
            continue;
        };
        connected.insert((tid.to_string(), str_of(e, "targetPort").to_string()));
        let (Some(sd), Some(td)) = (types.get(str_of(sn, "type")), types.get(str_of(tn, "type"))) else { continue };
        let (sd, td) = (effective(sd, &sn["params"]), effective(td, &tn["params"]));
        let sp = port(&sd, "outputPorts", str_of(e, "sourcePort"));
        let tp = port(&td, "inputPorts", str_of(e, "targetPort"));
        if sp.is_none() {
            let msg = format!("Không có cổng ra tên '{}' (loại {}).", str_of(e, "sourcePort"), str_of(sn, "type"));
            blame(&mut node_errors, sid, &msg);
            errors.push(format!("Node '{}' {}", sid, msg));
        }
        if tp.is_none() {
            let msg = format!("Không có cổng vào tên '{}' (loại {}).", str_of(e, "targetPort"), str_of(tn, "type"));
            blame(&mut node_errors, tid, &msg);
            errors.push(format!("Node '{}' {}", tid, msg));
        }
        if let (Some(sp), Some(tp)) = (sp, tp) {
            let (a, b) = (str_of(sp, "portType"), str_of(tp, "portType"));
            if !compatible(a, b) {
                let msg = format!("Không tương thích kiểu cổng: Dây nối '{}' từ cổng ra '{}' ({}) của node '{}' không thể nối vào cổng vào '{}' ({}) của node '{}'.",
                    eid, str_of(e, "sourcePort"), a, sid, str_of(e, "targetPort"), b, tid);
                blame(&mut node_errors, tid, &msg);
                blame(&mut node_errors, sid, &msg);
                errors.push(msg);
            }
        }
    }
    for n in &nodes {
        for (pn, r) in n["inputs"].as_object().into_iter().flatten() {
            if r.get("nodeId").is_some() {
                connected.insert((str_of(n, "id").to_string(), pn.clone()));
            }
        }
    }
    let (ids, parents, self_errs) = build_deps(graph);
    for (id, msg) in self_errs {
        blame(&mut node_errors, &id, &msg);
        errors.push(msg);
    }
    let order = match topo_sort(&ids, &parents) {
        Ok(o) => o,
        Err(stuck) => {
            let msg = format!("Phát hiện chu trình (vòng lặp vô hạn) trong đồ thị liên quan đến các node: {:?}. Đồ thị phải là DAG để có thể thực thi.", stuck);
            for id in &stuck { blame(&mut node_errors, id, &msg); }
            errors.push(msg);
            vec![]
        }
    };
    for n in &nodes {
        let id = str_of(n, "id");
        let Some(def) = types.get(str_of(n, "type")).map(|d| effective(d, &n["params"])) else { continue };
        if is_bypassed(n) {
            continue;
        }
        let p = &n["params"];
        let conn = |port: &str| connected.contains(&(id.to_string(), port.to_string()));
        match str_of(n, "type") {
            "script" => match str_of(p, "mode") {
                "episode" if str_of(p, "episodeId").trim().is_empty() => warnings.push(format!("Node '{}' (script) đang ở chế độ 'Kịch bản tập' nhưng chưa chọn tập (episodeId rỗng).", id)),
                "paste" if str_of(p, "rawScript").trim().is_empty() => warnings.push(format!("Node '{}' (script) đang ở chế độ 'Dán kịch bản' nhưng chưa dán kịch bản (rawScript rỗng).", id)),
                "episode" | "paste" => {}
                _ => {
                    let has = !format!("{}{}", str_of(p, "text"), str_of(p, "content")).trim().is_empty();
                    if !(conn("text") || has) {
                        warnings.push(format!("Node '{}' (script) chưa nhận ý tưởng (cổng 'text' chưa nối và chưa nhập text).", id));
                    }
                }
            },
            "voice_tts" if !(conn("text") || !str_of(p, "text").trim().is_empty()) => warnings.push(format!("Node '{}' (voice_tts) chưa nhận lời thoại cần đọc (cổng 'text' chưa nối và chưa nhập text).", id)),
            "image" | "character_image" if str_of(p, "uri").trim().is_empty() => warnings.push(format!("Node '{}' ({}) chưa có ảnh được chọn hoặc tải lên (uri rỗng).", id, str_of(n, "type"))),
            _ => {}
        }
        for rp in def["inputPorts"].as_array().into_iter().flatten().filter(|x| x["required"] == true) {
            let name = str_of(rp, "name");
            let has_in = n["inputs"].get(name).map(|v| !v.is_null()).unwrap_or(false);
            let has_param = p.get(name).map(|v| !v.is_null()).unwrap_or(false) || !matches!(rp["defaultValue"], Value::Null) && rp["defaultValue"] != "";
            if !(conn(name) || has_in || has_param) {
                warnings.push(format!("Node '{}' ({}) chưa nhận dữ liệu cho cổng bắt buộc '{}'.", id, str_of(n, "type"), name));
            }
        }
    }
    Validation { is_valid: errors.is_empty(), errors, node_errors, warnings, topological_order: order }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(edges: &[(&str, &str)], ids: &[&str]) -> Value {
        json!({"nodes": ids.iter().map(|i| json!({"id": i, "type": "x", "params": {}})).collect::<Vec<_>>(),
               "edges": edges.iter().map(|(a, b)| json!({"id": format!("{a}{b}"), "sourceNodeId": a, "targetNodeId": b})).collect::<Vec<_>>()})
    }

    #[test]
    fn self_reference_is_blamed_on_its_node() {
        let graph = json!({"nodes": [{"id": "a", "type": "text", "params": {"content": "{{ $node.a.out }}"}}], "edges": []});
        let v = validate(&graph);
        assert!(v.node_errors.get("a").and_then(|m| m.as_str()).is_some_and(|m| m.contains("tự tham chiếu")), "{:?}", v.node_errors);
    }

    #[test]
    fn bad_port_is_blamed_on_its_node() {
        let graph = json!({"nodes": [{"id": "c", "type": "character", "params": {}}, {"id": "t", "type": "image_gen", "params": {}}],
            "edges": [{"id": "e", "sourceNodeId": "c", "sourcePort": "nope", "targetNodeId": "t", "targetPort": "prompt"}]});
        let v = validate(&graph);
        assert!(!v.is_valid && v.node_errors.contains_key("c"), "{:?}", v.node_errors);
    }

    #[test]
    fn capability_ports_only_when_enabled() {
        let def = json!({"inputPorts": [], "outputPorts": [{"name": "character"}],
            "capabilities": [{"name": "speak", "inputPorts": [{"name": "lines"}], "outputPorts": [{"name": "voice"}]}]});
        assert!(port(&effective(&def, &json!({"speak": {"enabled": false}})), "outputPorts", "voice").is_none());
        let on = effective(&def, &json!({"speak": {"enabled": true}}));
        assert!(port(&on, "outputPorts", "voice").is_some() && port(&on, "inputPorts", "lines").is_some());
    }

    #[test]
    fn topo_orders_parents_first() {
        let (ids, parents, _) = build_deps(&g(&[("a", "b"), ("b", "c"), ("a", "c")], &["c", "b", "a"]));
        let o = topo_sort(&ids, &parents).unwrap();
        let pos = |x: &str| o.iter().position(|y| y == x).unwrap();
        assert!(pos("a") < pos("b") && pos("b") < pos("c"));
    }

    #[test]
    fn detects_cycle() {
        let (ids, parents, _) = build_deps(&g(&[("a", "b"), ("b", "c"), ("c", "b")], &["a", "b", "c"]));
        assert_eq!(topo_sort(&ids, &parents).unwrap_err(), vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn node_ref_in_params_is_a_dependency() {
        let graph = json!({"nodes": [{"id": "a", "type": "x", "params": {}}, {"id": "b", "type": "x", "params": {"t": "{{ $node.a.text }}"}}], "edges": []});
        let (_, parents, _) = build_deps(&graph);
        assert_eq!(parents["b"], vec!["a".to_string()]);
    }
}
