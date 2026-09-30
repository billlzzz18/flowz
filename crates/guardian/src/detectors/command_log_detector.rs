use crate::core::metrics_db::CommandEntry;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandFindingKind {
    /// Same command repeated N times in sequence.
    CommandLoop,
    /// Destructive command detected.
    Destructive,
    /// cd back and forth between dirs.
    CdOscillation,
    /// Command failed repeatedly.
    RepeatedFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFinding {
    pub kind: CommandFindingKind,
    pub index: usize,
    pub severity: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLogReport {
    pub scanned: usize,
    pub findings: Vec<CommandFinding>,
    pub command_frequency: HashMap<String, usize>,
}

pub struct CommandLogDetector {
    loop_threshold: usize,
    destructive_patterns: Vec<Regex>,
}

impl Default for CommandLogDetector {
    fn default() -> Self {
        Self::new(CommandDetectorConfig::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CommandDetectorConfig {
    pub loop_threshold: usize,
}

impl Default for CommandDetectorConfig {
    fn default() -> Self {
        Self { loop_threshold: 3 }
    }
}

impl CommandLogDetector {
    pub fn new(config: CommandDetectorConfig) -> Self {
        let destructive = vec![
            Regex::new(r"\brm\s+-rf?\b").expect("built-in regex"),
            Regex::new(r"\bgit\s+push\s+.*--force").expect("built-in regex"),
            Regex::new(r"\bgit\s+reset\s+--hard").expect("built-in regex"),
            Regex::new(r"\bgit\s+clean\s+-fd").expect("built-in regex"),
            Regex::new(r"\bsudo\s+rm\b").expect("built-in regex"),
            Regex::new(r">\s*/dev/sd").expect("built-in regex"),
        ];
        Self {
            loop_threshold: config.loop_threshold,
            destructive_patterns: destructive,
        }
    }

    pub fn analyze(&self, commands: &[CommandEntry]) -> CommandLogReport {
        let mut findings = Vec::new();
        let mut freq: HashMap<String, usize> = HashMap::new();
        let mut cd_history: Vec<String> = Vec::new();

        for (i, entry) in commands.iter().enumerate() {
            *freq.entry(entry.cmd.clone()).or_insert(0) += 1;

            // Command loop: same command repeated N times
            let key = entry.cmd.trim().to_string();
            let count = freq.get(&key).copied().unwrap_or(0);
            if count == self.loop_threshold {
                findings.push(CommandFinding {
                    kind: CommandFindingKind::CommandLoop,
                    index: i,
                    severity: 2,
                    message: format!("Command repeated {} times: {}", self.loop_threshold, key),
                });
            }

            // Destructive commands
            for pat in &self.destructive_patterns {
                if pat.is_match(&entry.cmd) {
                    findings.push(CommandFinding {
                        kind: CommandFindingKind::Destructive,
                        index: i,
                        severity: 3,
                        message: format!("Destructive command: {}", entry.cmd.trim()),
                    });
                    break;
                }
            }

            // CD oscillation
            if entry.cmd.trim().starts_with("cd ") {
                let dir = entry.cmd.trim()[3..].trim().to_string();
                if let Some(last) = cd_history.last() {
                    if last == &dir {
                        // Same dir twice — not oscillation, skip
                    } else if cd_history.len() >= 2 && cd_history[cd_history.len() - 2] == dir {
                        findings.push(CommandFinding {
                            kind: CommandFindingKind::CdOscillation,
                            index: i,
                            severity: 1,
                            message: format!(
                                "CD oscillation: {} -> {} -> {}",
                                cd_history[cd_history.len() - 2],
                                last,
                                dir
                            ),
                        });
                    }
                }
                cd_history.push(dir);
            }

            // Repeated failures
            if let Some(code) = entry.exit_code {
                if code != 0 && i > 0 {
                    if let Some(prev) = commands.get(i - 1) {
                        if prev.exit_code.is_some_and(|c| c != 0) && prev.cmd == entry.cmd {
                            findings.push(CommandFinding {
                                kind: CommandFindingKind::RepeatedFailure,
                                index: i,
                                severity: 2,
                                message: format!("Command failed twice: {}", entry.cmd.trim()),
                            });
                        }
                    }
                }
            }
        }

        CommandLogReport {
            scanned: commands.len(),
            findings,
            command_frequency: freq,
        }
    }
}
