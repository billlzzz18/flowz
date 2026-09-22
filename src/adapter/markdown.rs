use crate::domain::{
    CronDefinition, CronExecutionMode, DefinitionSource, MisfirePolicy, OverlapPolicy,
    ResolvedCommand, generate_id,
};
use crate::error::{FlowzError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub cron: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub skill: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub no_agent: bool,
    #[serde(default)]
    pub execution_mode: Option<CronExecutionMode>,
    #[serde(default = "default_overlap")]
    pub overlap: OverlapPolicy,
    #[serde(default = "default_misfire")]
    pub misfire: MisfirePolicy,
    #[serde(default)]
    pub iterations: Option<u64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}
fn default_overlap() -> OverlapPolicy {
    OverlapPolicy::Skip
}
fn default_misfire() -> MisfirePolicy {
    MisfirePolicy::RunOnce
}
fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    Wikilink(String),
    Url(String),
    File(String),
}

#[derive(Debug, Clone)]
pub struct MarkdownDefinition {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub links: Vec<Link>,
    pub source_path: PathBuf,
}

impl MarkdownDefinition {
    pub fn parse(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = content.lines();
        if lines
            .next()
            .map(|line| line.strip_prefix('\u{feff}').unwrap_or(line))
            != Some("---")
        {
            return Err(FlowzError::Validation(format!(
                "{} must start with YAML frontmatter",
                path.display()
            )));
        }
        let mut yaml = String::new();
        let mut body_lines = Vec::new();
        let mut closed = false;
        for line in lines {
            if !closed && line.trim() == "---" {
                closed = true;
                continue;
            }
            if closed {
                body_lines.push(line);
            } else {
                yaml.push_str(line);
                yaml.push('\n');
            }
        }
        if !closed {
            return Err(FlowzError::Validation(format!(
                "{} has unterminated frontmatter",
                path.display()
            )));
        }
        let frontmatter: Frontmatter = serde_yaml::from_str(&yaml).map_err(|e| {
            FlowzError::Validation(format!("invalid frontmatter in {}: {}", path.display(), e))
        })?;
        if frontmatter.name.trim().is_empty() {
            return Err(FlowzError::Validation("frontmatter name must not be empty".into()));
        }
        let body = body_lines.join("\n").trim().to_string();
        let links = extract_links(&body);
        Ok(Self {
            frontmatter,
            body,
            links,
            source_path: path.to_path_buf(),
        })
    }
}

fn extract_links(body: &str) -> Vec<Link> {
    body.split_whitespace()
        .filter_map(|word| {
            if word.starts_with("[[") && word.ends_with("]]") && word.len() > 4 {
                return Some(Link::Wikilink(word[2..word.len() - 2].to_string()));
            }
            let clean = word.trim_matches(|c: char| "()[]{}<>,.;\"'".contains(c));
            if clean.starts_with("http://") || clean.starts_with("https://") {
                Some(Link::Url(clean.to_string()))
            } else if clean.starts_with('@') && clean.len() > 1 {
                Some(Link::File(clean[1..].to_string()))
            } else {
                None
            }
        })
        .collect()
}

pub fn compile(def: &MarkdownDefinition, timezone: &str) -> Result<CronDefinition> {
    let fm = &def.frontmatter;
    let expression = fm.cron.clone().ok_or_else(|| {
        FlowzError::Validation(format!("{} requires cron", def.source_path.display()))
    })?;
    let command = if let Some(tool) = &fm.tool {
        ResolvedCommand::Tool {
            name: tool.clone(),
            args: serde_json::json!({"prompt": def.body}),
        }
    } else if let Some(skill) = &fm.skill {
        // Sanitize skill path to prevent path traversal
        let skill_path = PathBuf::from(skill);
        if skill_path.is_absolute() || skill_path.components().any(|c| c.as_os_str() == "..") {
            return Err(FlowzError::Validation(
                "skill path must be relative and not contain '..'".into(),
            ));
        }
        ResolvedCommand::Skill {
            path: skill_path,
            body: def.body.clone(),
        }
    } else if fm.execution_mode == Some(CronExecutionMode::NoAgent) || (fm.execution_mode.is_none() && fm.no_agent) {
        ResolvedCommand::Shell {
            command: def.body.clone(),
            args: Vec::new(),
        }
    } else {
        // WithAgent mode without tool/skill: treat body as agent prompt via a default tool
        ResolvedCommand::Tool {
            name: "agent".to_string(),
            args: serde_json::json!({"prompt": def.body}),
        }
    };
    let execution_mode = fm.execution_mode.clone().unwrap_or(if fm.no_agent {
        CronExecutionMode::NoAgent
    } else {
        CronExecutionMode::WithAgent
    });
    let definition = CronDefinition {
        id: generate_id(),
        name: fm.name.clone(),
        description: fm.description.clone(),
        expression,
        timezone: fm.timezone.clone().unwrap_or_else(|| timezone.to_string()),
        command,
        execution_mode,
        project: fm.project.clone(),
        enabled: fm.enabled,
        overlap_policy: fm.overlap,
        misfire_policy: fm.misfire,
        max_runs: fm.iterations,
        source: DefinitionSource::Markdown { path: def.source_path.clone() },
    };
    crate::cron::validate_cron_definition(&definition)
        .map_err(|e| FlowzError::Validation(e.to_string()))?;
    Ok(definition)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_frontmatter_and_supplements() {
        let path = std::env::temp_dir().join(format!("flowz-definition-{}.md", generate_id()));
        std::fs::write(&path, "---\nname: Review\ncron: '0 9 * * 1-5'\n---\nCheck [[review-checklist]] https://example.com @notes.md").unwrap();
        let parsed = MarkdownDefinition::parse(&path).unwrap();
        assert_eq!(parsed.frontmatter.name, "Review");
        assert_eq!(parsed.links.len(), 3);
        let compiled = compile(&parsed, "Asia/Bangkok").unwrap();
        assert!(matches!(compiled.source, DefinitionSource::Markdown { .. }));
        let _ = std::fs::remove_file(path);
    }
}
