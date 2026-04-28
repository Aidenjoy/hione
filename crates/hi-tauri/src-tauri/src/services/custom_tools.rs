use crate::types::CustomTool;
use crate::error::AppError;
use std::path::PathBuf;
use std::fs;

#[derive(serde::Deserialize)]
struct ToolsToml {
    tools: std::collections::HashMap<String, ToolConfig>,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct ToolConfig {
    #[serde(default)]
    auto_flags: Vec<String>,
    #[serde(default)]
    resume_flags: Vec<String>,
}

pub fn read_tools(work_dir: &str) -> Result<Vec<CustomTool>, AppError> {
    let path = PathBuf::from(work_dir).join(".hione").join("tools.toml");
    
    if !path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&path)?;
    let toml: ToolsToml = toml::from_str(&content)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    
    let tools: Vec<CustomTool> = toml
        .tools
        .into_iter()
        .map(|(name, config)| CustomTool {
            name,
            auto_flags: config.auto_flags,
            resume_flags: config.resume_flags,
        })
        .collect();
    
    Ok(tools)
}

pub fn write_tools(work_dir: &str, tools: &[CustomTool]) -> Result<(), AppError> {
    let path = PathBuf::from(work_dir).join(".hione").join("tools.toml");
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut toml_content = String::new();
    
    for tool in tools {
        toml_content.push_str(&format!("[tools.{}]\n", tool.name));
        
        if !tool.auto_flags.is_empty() {
            let flags = tool
                .auto_flags
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ");
            toml_content.push_str(&format!("auto_flags = [{}]\n", flags));
        }
        
        if !tool.resume_flags.is_empty() {
            let flags = tool
                .resume_flags
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ");
            toml_content.push_str(&format!("resume_flags = [{}]\n", flags));
        }
        
        toml_content.push('\n');
    }
    
    fs::write(&path, toml_content)?;
    
    Ok(())
}