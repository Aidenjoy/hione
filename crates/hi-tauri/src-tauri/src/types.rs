use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    pub tmux: bool,
    pub node: bool,
    pub rust: bool,
    pub hi: bool,
    pub hi_monitor: bool,
}

impl Default for SetupStatus {
    fn default() -> Self {
        Self {
            tmux: false,
            node: false,
            rust: false,
            hi: false,
            hi_monitor: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
}

impl Default for ToolInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            installed: false,
            version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub api_key: Option<String>,
    pub api_base_url: Option<String>,
    pub model: Option<String>,
    pub extra_config: serde_json::Value,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            api_key: None,
            api_base_url: None,
            model: None,
            extra_config: serde_json::json!({}),
            enabled: true,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub server_config: serde_json::Value,
    pub enabled_for: Vec<String>,
    pub created_at: i64,
}

impl Default for McpServer {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            server_config: serde_json::json!({}),
            enabled_for: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub repo_url: Option<String>,
    pub local_path: Option<String>,
    pub enabled_for: Vec<String>,
    pub installed_at: i64,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            repo_url: None,
            local_path: None,
            enabled_for: Vec::new(),
            installed_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepo {
    pub id: String,
    pub url: String,
    pub name: String,
}

impl Default for SkillRepo {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: String::new(),
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSession {
    pub work_dir: String,
    pub tools: Vec<String>,
    pub auto_mode: bool,
    pub resume_mode: bool,
    pub last_used: i64,
}

impl Default for RecentSession {
    fn default() -> Self {
        Self {
            work_dir: String::new(),
            tools: Vec::new(),
            auto_mode: false,
            resume_mode: false,
            last_used: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub content: String,
    pub status: String,
    pub created_at: i64,
}

impl Default for TaskRecord {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender: String::new(),
            receiver: String::new(),
            content: String::new(),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTool {
    pub name: String,
    pub auto_flags: Vec<String>,
    pub resume_flags: Vec<String>,
}

impl Default for CustomTool {
    fn default() -> Self {
        Self {
            name: String::new(),
            auto_flags: Vec::new(),
            resume_flags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    pub hi_bin_path: Option<String>,
    pub hi_monitor_bin_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "system".to_string(),
            hi_bin_path: None,
            hi_monitor_bin_path: None,
        }
    }
}