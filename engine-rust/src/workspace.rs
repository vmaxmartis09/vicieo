use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub active_project: Option<String>,
    pub projects: Option<Vec<ProjectEntry>>,
}

/// Traverse up from the current directory until reaching a directory containing `data/workspace.json`.
pub fn repo_root() -> PathBuf {
    if let Ok(mut current) = std::env::current_dir() {
        loop {
            if current.join("data").join("workspace.json").exists() {
                return current;
            }
            if !current.pop() {
                break;
            }
        }
    }
    PathBuf::from(".")
}

pub fn data_root() -> PathBuf {
    repo_root().join("data")
}

pub fn read_workspace_data() -> WorkspaceManifest {
    let ws_file = data_root().join("workspace.json");
    if !ws_file.exists() {
        return WorkspaceManifest {
            active_project: Some("soc-nho-tv".to_string()),
            projects: None,
        };
    }
    match fs::read_to_string(&ws_file) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(WorkspaceManifest {
            active_project: Some("soc-nho-tv".to_string()),
            projects: None,
        }),
        Err(_) => WorkspaceManifest {
            active_project: Some("soc-nho-tv".to_string()),
            projects: None,
        },
    }
}

pub fn active_project_id() -> String {
    read_workspace_data()
        .active_project
        .unwrap_or_else(|| "soc-nho-tv".to_string())
}

pub fn list_projects() -> Vec<ProjectEntry> {
    read_workspace_data().projects.unwrap_or_default()
}

pub fn project_path(project_id: &str, subpaths: &[&str], ensure: bool) -> Result<PathBuf> {
    let mut target = data_root().join("projects").join(project_id);
    for sub in subpaths {
        target = target.join(sub);
    }
    if ensure {
        if subpaths.is_empty() {
            fs::create_dir_all(&target)?;
        } else if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(target)
}

pub fn set_active_project(id: &str) -> Result<()> {
    let manifest_path = data_root().join("workspace.json");
    let mut manifest: Value = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };
    manifest["active_project"] = json!(id);
    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

pub fn shared_path(subpaths: &[&str], ensure: bool) -> Result<PathBuf> {
    let mut target = data_root().join("shared");
    for sub in subpaths {
        target = target.join(sub);
    }
    if ensure {
        if subpaths.is_empty() {
            fs::create_dir_all(&target)?;
        } else if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(target)
}
