use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AIAuthorship {
    pub file_path: String,
    pub ai_lines: Vec<(usize, usize)>,
    pub session_id: String,
    pub trace_id: String,
    pub model: String,
    pub timestamp: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitAIMetadata {
    pub schema_version: String,
    pub base_commit_sha: String,
    pub git_ai_version: String,
    pub sessions: HashMap<String, SessionRecord>,
    pub humans: HashMap<String, HumanRecord>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub agent_id: AgentId,
    pub human_author: Option<String>,
    pub custom_attributes: HashMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanRecord {
    pub author: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentId {
    pub tool: String,
    pub id: String,
    pub model: String,
}

pub struct GitAIExtractor;
impl GitAIExtractor {
    pub fn get_ai_authorship(
        file_path: &str,
        commit_sha: Option<&str>,
    ) -> Result<Vec<AIAuthorship>, Box<dyn std::error::Error>> {
        let note = Self::read_note(commit_sha)?;
        Self::parse_attestations(&note, file_path)
    }
    pub fn get_git_ai_metadata(
        commit_sha: Option<&str>,
    ) -> Result<GitAIMetadata, Box<dyn std::error::Error>> {
        let note = Self::read_note(commit_sha)?;
        let json = note
            .split("---")
            .nth(1)
            .ok_or("invalid Git AI note: metadata separator missing")?;
        let v: Value = serde_json::from_str(json.trim())?;
        Ok(Self::metadata_from_value(&v))
    }
    pub fn has_ai_code(commit_sha: Option<&str>) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(Self::read_note(commit_sha).is_ok())
    }
    pub fn get_ai_percentage(
        file_path: &str,
        commit_sha: Option<&str>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let total = fs::read_to_string(file_path)?.lines().count();
        if total == 0 {
            return Ok(0.0);
        }
        let ranges = Self::get_ai_authorship(file_path, commit_sha)?;
        let ai = ranges
            .into_iter()
            .flat_map(|x| x.ai_lines)
            .map(|(s, e)| if e >= s { e - s + 1 } else { 0 })
            .sum::<usize>();
        Ok((ai as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
    }
    pub fn get_ai_authors(
        commit_sha: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let m = Self::get_git_ai_metadata(commit_sha)?;
        let mut v = m
            .sessions
            .values()
            .filter_map(|s| s.human_author.clone())
            .collect::<Vec<_>>();
        v.sort();
        v.dedup();
        Ok(v)
    }
    pub fn get_ai_models(
        commit_sha: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let m = Self::get_git_ai_metadata(commit_sha)?;
        let mut v = m
            .sessions
            .values()
            .map(|s| s.agent_id.model.clone())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        v.sort();
        v.dedup();
        Ok(v)
    }
    pub fn is_line_ai_authored(
        file_path: &str,
        line: usize,
        commit_sha: Option<&str>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(Self::get_ai_authorship(file_path, commit_sha)?
            .iter()
            .flat_map(|a| a.ai_lines.iter())
            .any(|(s, e)| line >= *s && line <= *e))
    }
    pub fn parse_ranges(input: &str) -> Result<Vec<(usize, usize)>, Box<dyn std::error::Error>> {
        let mut out = Vec::new();
        for token in input.split(',').map(str::trim).filter(|x| !x.is_empty()) {
            if let Some((a, b)) = token.split_once('-') {
                let s: usize = a.parse()?;
                let e: usize = b.parse()?;
                if s == 0 || e < s {
                    return Err("invalid line range".into());
                }
                out.push((s, e));
            } else {
                let n: usize = token.parse()?;
                if n == 0 {
                    return Err("line numbers start at 1".into());
                }
                out.push((n, n));
            }
        }
        Ok(out)
    }
    pub fn parse_attestations(
        content: &str,
        file_path: &str,
    ) -> Result<Vec<AIAuthorship>, Box<dyn std::error::Error>> {
        let mut current = "";
        let mut out = Vec::new();
        for line in content.lines() {
            if line.trim() == "---" {
                break;
            }
            if line
                .chars()
                .next()
                .map(|c| !c.is_whitespace())
                .unwrap_or(false)
            {
                current = line.trim();
                continue;
            }
            if current == file_path {
                let p = line.split_whitespace().collect::<Vec<_>>();
                if p.len() >= 2 {
                    let id = p[0];
                    let (session, trace) = id.split_once("::").unwrap_or((id, ""));
                    out.push(AIAuthorship {
                        file_path: file_path.into(),
                        ai_lines: Self::parse_ranges(p[1])?,
                        session_id: session.into(),
                        trace_id: trace.into(),
                        model: "unknown".into(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }
        Ok(out)
    }
    fn read_note(sha: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let out = Command::new("git")
            .args(["notes", "--ref=refs/notes/ai", "show", sha.unwrap_or("HEAD")])
            .output()?;
        if !out.status.success() {
            return Err("Git AI note not found".into());
        }
        Ok(String::from_utf8(out.stdout)?)
    }
    fn metadata_from_value(v: &Value) -> GitAIMetadata {
        let mut sessions = HashMap::new();
        if let Some(obj) = v["sessions"].as_object() {
            for (k, x) in obj {
                let a = &x["agent_id"];
                sessions.insert(
                    k.clone(),
                    SessionRecord {
                        agent_id: AgentId {
                            tool: a["tool"].as_str().unwrap_or("").into(),
                            id: a["id"].as_str().unwrap_or("").into(),
                            model: a["model"].as_str().unwrap_or("").into(),
                        },
                        human_author: x["human_author"].as_str().map(str::to_owned),
                        custom_attributes: HashMap::new(),
                    },
                );
            }
        }
        let mut humans = HashMap::new();
        if let Some(obj) = v["humans"].as_object() {
            for (k, x) in obj {
                humans.insert(
                    k.clone(),
                    HumanRecord {
                        author: x["author"].as_str().unwrap_or("").into(),
                    },
                );
            }
        }
        GitAIMetadata {
            schema_version: v["schema_version"].as_str().unwrap_or("3.0.0").into(),
            base_commit_sha: v["base_commit_sha"].as_str().unwrap_or("").into(),
            git_ai_version: v["git_ai_version"].as_str().unwrap_or("").into(),
            sessions,
            humans,
        }
    }
}
