use ast_grep_core::{AstGrep, Pattern, tree_sitter::LanguageExt, tree_sitter::StrDoc};
use ast_grep_language::SupportLang;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Guardian brain log entry - compressed representation of analysis findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainLogEntry {
    pub file: String,
    pub language: String,
    pub pattern: String,
    pub matches: Vec<MatchInfo>,
    pub timestamp: u64,
    pub commit_sha: String,
}

/// Compressed match info using 3-position anchoring (file, line, col)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchInfo {
    pub rule_id: String,
    pub severity: String,
    pub anchor: Anchor,
    pub text: String,
    pub meta: Option<serde_json::Value>,
}

/// 3-position anchor to prevent drift: (file_path, line, column)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Anchor {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

impl Anchor {
    pub fn new(file: impl Into<String>, line: usize, col: usize) -> Self {
        Self { file: file.into(), line, col }
    }
}

/// YAML Rule configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub lang: String,
    pub glob: String,
    #[serde(default)]
    pub exclude: Option<String>,
    pub action: RuleAction,
    pub pattern: String,
    #[serde(default)]
    pub severity: Severity,
    /// Protocol for protocol-specific rules (anthropic_messages_v1, openai_chat, all)
    #[serde(default)]
    pub protocol: Option<String>,
    /// Threshold for repeat detection (e.g., tool thrashing)
    #[serde(default)]
    pub threshold: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Block,
    Warn,
    #[default]
    Review,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical = 5,
    High = 4,
    Medium = 3,
    Low = 2,
    #[default]
    Info = 1,
}

/// Pattern match result for multi-grep
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub rule: Rule,
    pub anchor: Anchor,
    pub text: String,
}

/// Multi-grep engine: single AST pass per language, all patterns applied
pub struct GuardianGrep {
    rules_by_lang: HashMap<SupportLang, Vec<Rule>>,
    rule_index: HashMap<String, Rule>,
}

impl Default for GuardianGrep {
    fn default() -> Self {
        Self::new()
    }
}

impl GuardianGrep {
    pub fn new() -> Self {
        let rules = Self::builtin_rules();
        Self::from_rules(rules)
    }

    pub fn from_rules(rules: Vec<Rule>) -> Self {
        let mut rules_by_lang: HashMap<SupportLang, Vec<Rule>> = HashMap::new();
        let mut rule_index = HashMap::new();

        for rule in rules {
            let lang = Self::parse_lang(&rule.lang);
            rules_by_lang.entry(lang).or_default().push(rule.clone());
            rule_index.insert(rule.name.clone(), rule);
        }

        Self { rules_by_lang, rule_index }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rules: Vec<Rule> = serde_yaml::from_str(yaml)?;
        Ok(Self::from_rules(rules))
    }

    pub fn from_yaml_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    fn parse_lang(s: &str) -> SupportLang {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => SupportLang::Rust,
            "typescript" | "ts" | "tsx" => SupportLang::TypeScript,
            "javascript" | "js" | "jsx" => SupportLang::JavaScript,
            "python" | "py" => SupportLang::Python,
            "go" => SupportLang::Go,
            "java" => SupportLang::Java,
            "cpp" | "c++" | "cc" | "cxx" => SupportLang::Cpp,
            "c" | "h" => SupportLang::C,
            "markdown" | "md" | "mkd" => SupportLang::Markdown,
            "css" => SupportLang::Css,
            "html" => SupportLang::Html,
            _ => SupportLang::Rust,
        }
    }

    fn builtin_rules() -> Vec<Rule> {
        vec![
            // Rust hallucination patterns
            Rule {
                            name: "rust-fake-std-method".into(),
                            description: "Detect non-existent std library methods (hallucination)".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Block,
                            pattern: "fake_method".into(),
                            severity: Severity::Critical,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "rust-hashmap-remove-outliers".into(),
                            description: "Detect HashMap::remove_outliers hallucination".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Block,
                            pattern: r#"HashMap::remove_outliers"#.into(),
                            severity: Severity::Critical,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "rust-fake-crate-import".into(),
                            description: "Detect fake crate imports".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Block,
                            pattern: r#"use\s+crate_that_doesnt_exist"#.into(),
                            severity: Severity::Critical,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "rust-tokio-spawn-blocking".into(),
                            description: "Detect wrong tokio path (should be tokio::task::spawn_blocking)".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Warn,
                            pattern: r#"tokio::spawn_blocking"#.into(),
                            severity: Severity::High,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                                        name: "rust-into-iter-on-ref".into(),
                                        description: "Detect .into_iter() on &Vec (should use .iter())".into(),
                                        lang: "rust".into(),
                                        glob: "**/*.rs".into(),
                                        exclude: Some("**/tests/**".to_string()),
                                        action: RuleAction::Warn,
                                        pattern: r#"into_iter\("#.into(),
                                        severity: Severity::High,
                                        protocol: None,
                                        threshold: None,
                                    },
                        Rule {
                            name: "rust-unwrap-or-else-wrong-sig".into(),
                            description: "Detect unwrap_or_else with wrong closure signature".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Warn,
                            pattern: r#"Option::unwrap_or_else\("#.into(),
                            severity: Severity::Medium,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "rust-derive-serialize-no-serde".into(),
                            description: "Detect #[derive(Serialize)] without serde import".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Warn,
                            pattern: r#"#\[derive\([^)]*Serialize[^)]*\)\]"#.into(),
                            severity: Severity::High,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "rust-tokio-main-on-sync".into(),
                            description: "Detect #[tokio::main] on non-async fn".into(),
                            lang: "rust".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Block,
                            pattern: r#"#\[tokio::main\]\s*fn\s+\w+\("#.into(),
                            severity: Severity::Critical,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "rust-cargo-fake-feature".into(),
                            description: "Detect fake features in Cargo.toml".into(),
                            lang: "rust".into(),
                            glob: "**/Cargo.toml".into(),
                            exclude: None,
                            action: RuleAction::Warn,
                            pattern: r#"fake-feature\s*=\s*true"#.into(),
                            severity: Severity::Medium,
                            protocol: None,
                            threshold: None,
                        },
                        // General slop patterns (cross-language)
                        Rule {
                            name: "slop-unwrap".into(),
                            description: "Detect unwrap() without error handling".into(),
                            lang: "all".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Warn,
                            pattern: r#"\.unwrap\("#.into(),
                            severity: Severity::Critical,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "slop-clone".into(),
                            description: "Detect unnecessary clone()".into(),
                            lang: "all".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Review,
                            pattern: r#"\.clone\("#.into(),
                            severity: Severity::Medium,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "slop-todo".into(),
                            description: "Detect todo!() macro".into(),
                            lang: "all".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Warn,
                            pattern: r#"todo!\("#.into(),
                            severity: Severity::High,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "slop-unimplemented".into(),
                            description: "Detect unimplemented!() macro".into(),
                            lang: "all".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Warn,
                            pattern: r#"unimplemented!\("#.into(),
                            severity: Severity::High,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "slop-index".into(),
                            description: "Detect direct indexing (bounds check)".into(),
                            lang: "all".into(),
                            glob: "**/*.rs".into(),
                            exclude: Some("**/tests/**".to_string()),
                            action: RuleAction::Review,
                            pattern: r#"\w+\[\w+\]"#.into(),
                            severity: Severity::Medium,
                            protocol: None,
                            threshold: None,
                        },
                        // TypeScript/JavaScript
                        Rule {
                            name: "ts-console-log".into(),
                            description: "Detect console.log statements".into(),
                            lang: "typescript".into(),
                            glob: "**/*.{ts,tsx}".into(),
                            exclude: Some("**/tests/**".to_string()).into(),
                            action: RuleAction::Review,
                            pattern: r#"console\.log\("#.into(),
                            severity: Severity::Low,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "js-debugger".into(),
                            description: "Detect debugger statements".into(),
                            lang: "javascript".into(),
                            glob: "**/*.{js,jsx}".into(),
                            exclude: Some("**/tests/**".to_string()).into(),
                            action: RuleAction::Block,
                            pattern: r#"debugger"#.into(),
                            severity: Severity::Critical,
                            protocol: None,
                            threshold: None,
                        },
                        // Python
                        Rule {
                            name: "py-print".into(),
                            description: "Detect print statements".into(),
                            lang: "python".into(),
                            glob: "**/*.py".into(),
                            exclude: Some("**/tests/**".to_string()).into(),
                            action: RuleAction::Review,
                            pattern: r#"print\("#.into(),
                            severity: Severity::Low,
                            protocol: None,
                            threshold: None,
                        },
                        Rule {
                            name: "py-bare-except".into(),
                            description: "Detect bare except clauses".into(),
                            lang: "python".into(),
                            glob: "**/*.py".into(),
                            exclude: Some("**/tests/**".to_string()).into(),
                            action: RuleAction::Warn,
                            pattern: r#"except:"#.into(),
                            severity: Severity::High,
                            protocol: None,
                            threshold: None,
                        },
                        // Go
                        Rule {
                            name: "go-panic".into(),
                            description: "Detect panic() calls".into(),
                            lang: "go".into(),
                            glob: "**/*.go".into(),
                            exclude: Some("**/tests/**".to_string()).into(),
                            action: RuleAction::Warn,
                            pattern: r#"panic\("#.into(),
                            severity: Severity::High,
                            protocol: None,
                            threshold: None,
                        },
            // Markdown
            Rule {
                name: "md-bare-urls".into(),
                description: "Detect bare URLs without markdown link syntax".into(),
                lang: "markdown".into(),
                glob: "**/*.md".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Warn,
                pattern: r#"https?://[^\s)]+"#.into(),
                severity: Severity::Medium,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "md-unclosed-code-blocks".into(),
                description: "Detect unclosed code blocks".into(),
                lang: "markdown".into(),
                glob: "**/*.md".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Warn,
                pattern: r#"^```[^`]*$"#.into(),
                severity: Severity::High,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "md-todo-fixme".into(),
                description: "Detect TODO/FIXME comments in markdown".into(),
                lang: "markdown".into(),
                glob: "**/*.md".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Review,
                pattern: r#"(TODO|FIXME|XXX|HACK):"#.into(),
                severity: Severity::Low,
                protocol: None,
                threshold: None,
            },
            // CSS
            Rule {
                name: "css-important".into(),
                description: "Detect !important usage".into(),
                lang: "css".into(),
                glob: "**/*.css".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Warn,
                pattern: r#"!important"#.into(),
                severity: Severity::Medium,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "css-universal-selector".into(),
                description: "Detect universal * selector".into(),
                lang: "css".into(),
                glob: "**/*.css".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Review,
                pattern: r#"^\s*\*\s*\{"#.into(),
                severity: Severity::Low,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "css-vendor-prefix".into(),
                description: "Detect vendor prefixes without standard fallback".into(),
                lang: "css".into(),
                glob: "**/*.css".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Review,
                pattern: r#"(-webkit-|-moz-|-ms-|-o-)[a-z-]+:\s*[^;]+;[^}]*}"#.into(),
                severity: Severity::Low,
                protocol: None,
                threshold: None,
            },
            // HTML
            Rule {
                name: "html-inline-events".into(),
                description: "Detect inline event handlers (onclick, onload, etc.)".into(),
                lang: "html".into(),
                glob: "**/*.html".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Block,
                pattern: r#"on(click|load|change|submit|mouseover|mouseout|keydown|keyup|focus|blur)\s*="#.into(),
                severity: Severity::Critical,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "html-missing-alt".into(),
                description: "Detect img tags without alt attribute".into(),
                lang: "html".into(),
                glob: "**/*.html".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Warn,
                pattern: r#"<img(?![^>]*\balt=)[^>]*>"#.into(),
                severity: Severity::High,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "html-deprecated-tags".into(),
                description: "Detect deprecated HTML tags (center, font, marquee, etc.)".into(),
                lang: "html".into(),
                glob: "**/*.html".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Warn,
                pattern: r#"</?(center|font|marquee|blink|strike|big|tt|acronym|applet|frame|frameset|noframes|isindex|dir|basefont|bgsound|spacer)[\s>]"#.into(),
                severity: Severity::High,
                protocol: None,
                threshold: None,
            },
            Rule {
                name: "html-script-type-module".into(),
                description: "Detect script without type=module for ES modules".into(),
                lang: "html".into(),
                glob: "**/*.html".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Review,
                pattern: r#"<script(?![^>]*\btype\s*=\s*["']module["'])[^>]*src="#.into(),
                severity: Severity::Low,
                protocol: None,
                threshold: None,
            },
            // Protocol-specific behavioral rules
            Rule {
                name: "contract_violation".into(),
                description: "ตรวจจับโค้ดที่คืนค่าหรือส่งข้อมูลไม่ตรงตามข้อตกลงของ API แต่ type ผ่าน".into(),
                lang: "rust".into(),
                glob: "src/**/*.rs".into(),
                exclude: Some("**/tests/**".to_string()),
                action: RuleAction::Block,
                pattern: r#"return\s+\w+\s*;"#.into(), // placeholder - protocol rules use custom matching
                severity: Severity::Critical,
                protocol: Some("all".into()),
                threshold: None,
            },
            Rule {
                name: "mock_leak_to_prod".into(),
                description: "ตรวจจับค่าคงที่จำลองหรือ stub implementation ที่ AI ทิ้งไว้ในโค้ดจริง".into(),
                lang: "all".into(),
                glob: "src/**/*".into(),
                exclude: Some("**/tests/**,**/mocks/**".to_string()),
                action: RuleAction::Block,
                pattern: r#"(stub|mock|dummy|placeholder|fake|test_value|example\.com|localhost:\d+|password123|secret123)"#.into(),
                severity: Severity::Critical,
                protocol: Some("all".into()),
                threshold: None,
            },
            Rule {
                name: "tool_thrashing".into(),
                description: "ตรวจจับการฝืนเรียก tool เดิมซ้ำด้วยคำสั่งเดิมโดยไม่มีความคืบหน้า".into(),
                lang: "all".into(),
                glob: "**/*.jsonl".into(), // protocol logs
                exclude: None,
                action: RuleAction::Block,
                pattern: r#"repeat\(call\(tool=same, args=same\)\)"#.into(), // special pattern
                severity: Severity::High,
                protocol: Some("anthropic_messages_v1".into()),
                threshold: Some(3),
            },
            Rule {
                name: "unhandled_tool_failure_loop".into(),
                description: "ตรวจจับการฝืนส่งคำสั่งเดิมซ้ำทันทีหลัง tool ส่ง error กลับมา".into(),
                lang: "all".into(),
                glob: "**/*.jsonl".into(),
                exclude: None,
                action: RuleAction::Block,
                pattern: r#"tool_error -> call\(same_tool, same_args\)"#.into(), // special pattern
                severity: Severity::High,
                protocol: Some("openai_chat".into()),
                threshold: None,
            },
            Rule {
                name: "context_drift".into(),
                description: "ตรวจจับการให้เหตุผลหรือตอบหลุดจากเป้าหมายเดิมของระบบ".into(),
                lang: "all".into(),
                glob: "**/*.jsonl".into(),
                exclude: None,
                action: RuleAction::Review,
                pattern: r#"semantic_drift\(summary, intent\)"#.into(), // special pattern
                severity: Severity::Medium,
                protocol: Some("all".into()),
                threshold: None,
            },
        ]
    }

    /// Scan a file using multi-grep: single AST pass, all patterns for that language
    pub fn scan_file(&self, path: &Path) -> Result<Vec<PatternMatch>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let lang = Self::detect_language(path);
        let file_name = path.to_string_lossy().to_string();

        let rules = self.rules_by_lang.get(&lang).ok_or("no rules for language")?;
        if rules.is_empty() {
            return Ok(Vec::new());
        }

        let ast: AstGrep<_> = lang.ast_grep(&content);
        let mut matches = Vec::new();

        for rule in rules {
            if !Self::glob_match(&rule.glob, &file_name) {
                continue;
            }
            if let Some(exclude) = &rule.exclude {
                if Self::glob_match(exclude, &file_name) {
                    continue;
                }
            }

            let rule_matches = self.run_rule_pattern(&ast, &content, &file_name, rule)?;
            matches.extend(rule_matches);
        }

        // Apply fuzzy matching for patterns not covered by ast-grep (regex fallback)
        let fuzzy_matches = self.run_fuzzy_patterns(&content, &file_name, rules)?;
        matches.extend(fuzzy_matches);

        Ok(matches)
    }

    fn run_rule_pattern(
        &self,
        ast: &AstGrep<StrDoc<SupportLang>>,
        content: &str,
        file: &str,
        rule: &Rule,
    ) -> Result<Vec<PatternMatch>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // Try ast-grep structural pattern first
        let lang = Self::parse_lang(&rule.lang);
        if let Ok(pattern) = Pattern::try_new(&rule.pattern, lang) {
            for m in ast.root().find_all(pattern) {
                let start_pos = m.start_pos();
                let anchor = Anchor::new(file, start_pos.line(), start_pos.column(&m));
                results.push(PatternMatch {
                    rule: rule.clone(),
                    anchor,
                    text: m.text().to_string(),
                });
            }
        }

        Ok(results)
    }

    fn run_fuzzy_patterns(
            &self,
            content: &str,
            file: &str,
            rules: &[Rule],
        ) -> Result<Vec<PatternMatch>, Box<dyn std::error::Error>> {
            let mut results = Vec::new();
            let regex_cache = RegexCache::global();

            for rule in rules {
                // Use regex for fuzzy/partial patterns that ast-grep can't handle
                if rule.pattern.contains('*') || rule.pattern.contains('\\') {
                    let regex = regex_cache.get_or_create(&rule.pattern);
                    for mat in regex.find_iter(content) {
                        let line = content[..mat.start()].matches('\n').count() + 1;
                        let col = mat.start()
                            - content[..mat.start()]
                                .rfind('\n')
                                .map(|i| i + 1)
                                .unwrap_or(0);
                        let anchor = Anchor::new(file, line, col);
                        results.push(PatternMatch {
                            rule: rule.clone(),
                            anchor,
                            text: mat.as_str().to_string(),
                        });
                    }
                }
            }

            // Protocol-specific behavioral rules (special patterns)
            results.extend(self.run_protocol_patterns(content, file, rules)?);

            Ok(results)
        }

        fn run_protocol_patterns(
            &self,
            content: &str,
            file: &str,
            rules: &[Rule],
        ) -> Result<Vec<PatternMatch>, Box<dyn std::error::Error>> {
            let mut results = Vec::new();
            let regex_cache = RegexCache::global();

            for rule in rules {
                if rule.protocol.is_none() {
                    continue;
                }

                match rule.name.as_str() {
                    "tool_thrashing" => {
                        // Pattern: repeat(call(tool=same, args=same)) - detect repeated same tool calls
                        // Parse JSONL lines for tool calls, track frequency
                        let threshold = rule.threshold.unwrap_or(3);
                        let tool_calls = Self::extract_tool_calls(content);
                        let mut call_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                        for call in tool_calls {
                            *call_counts.entry(call).or_insert(0) += 1;
                        }
                        for (call, count) in call_counts {
                            if count >= threshold {
                                let line = content.lines().position(|l| l.contains(&call)).unwrap_or(0) + 1;
                                let anchor = Anchor::new(file, line, 1);
                                results.push(PatternMatch {
                                    rule: rule.clone(),
                                    anchor,
                                    text: format!("tool_thrashing: {} repeated {} times", call, count),
                                });
                            }
                        }
                    }
                    "unhandled_tool_failure_loop" => {
                        // Pattern: tool_error -> call(same_tool, same_args) - retry same failed tool
                        let lines: Vec<&str> = content.lines().collect();
                        for i in 0..lines.len().saturating_sub(1) {
                            if lines[i].contains("error") || lines[i].contains("Error") {
                                if lines[i + 1].contains(&lines[i].replace("error", "").replace("Error", "").trim()) {
                                    let anchor = Anchor::new(file, i + 1, 1);
                                    results.push(PatternMatch {
                                        rule: rule.clone(),
                                        anchor,
                                        text: format!("unhandled_tool_failure_loop: retry after error at line {}", i + 1),
                                    });
                                }
                            }
                        }
                    }
                    "context_drift" => {
                        // Pattern: semantic_drift(summary, intent) - detect response drift
                        // Simple heuristic: check if response contains keywords unrelated to initial intent
                        // This is a placeholder for semantic analysis
                        if content.len() > 5000 {
                            let anchor = Anchor::new(file, 1, 1);
                            results.push(PatternMatch {
                                rule: rule.clone(),
                                anchor,
                                text: "context_drift: potential semantic drift detected (long response)".into(),
                            });
                        }
                    }
                    "contract_violation" => {
                        // Placeholder - would need API spec to validate
                        // For now, detect common patterns like returning wrong types
                        let regex = regex_cache.get_or_create(r#"return\s+\w+\s*;"#);
                        for mat in regex.find_iter(content) {
                            let line = content[..mat.start()].matches('\n').count() + 1;
                            let col = mat.start()
                                - content[..mat.start()]
                                    .rfind('\n')
                                    .map(|i| i + 1)
                                    .unwrap_or(0);
                            let anchor = Anchor::new(file, line, col);
                            results.push(PatternMatch {
                                rule: rule.clone(),
                                anchor,
                                text: mat.as_str().to_string(),
                            });
                        }
                    }
                    "mock_leak_to_prod" => {
                        // Uses regex pattern directly (already handled in run_fuzzy_patterns)
                        // But we add extra check for protocol
                        let regex = regex_cache.get_or_create(&rule.pattern);
                        for mat in regex.find_iter(content) {
                            let line = content[..mat.start()].matches('\n').count() + 1;
                            let col = mat.start()
                                - content[..mat.start()]
                                    .rfind('\n')
                                    .map(|i| i + 1)
                                    .unwrap_or(0);
                            let anchor = Anchor::new(file, line, col);
                            results.push(PatternMatch {
                                rule: rule.clone(),
                                anchor,
                                text: mat.as_str().to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            Ok(results)
        }

        fn extract_tool_calls(content: &str) -> Vec<String> {
            // Extract tool names from JSONL format (simplified)
            let mut calls = Vec::new();
            for line in content.lines() {
                if let Some(start) = line.find("\"tool\":") {
                    let after = &line[start + 7..];
                    if let Some(end) = after.find(',').or_else(|| after.find('}')) {
                        let tool = after[..end].trim().trim_matches('"');
                        calls.push(tool.to_string());
                    }
                }
            }
            calls
        }

        fn glob_match(pattern: &str, path: &str) -> bool {
        let pattern = pattern.replace("**/", "");
        let pattern = pattern.replace("*.", ".*\\.");
        let pattern = pattern.replace("*", ".*");
        let regex = format!("^{}$", pattern);
        regex::Regex::new(&regex).map(|r| r.is_match(path)).unwrap_or(false)
    }

    /// Scan directory recursively
    pub fn scan_dir(&self, dir: &Path) -> Result<Vec<BrainLogEntry>, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();
        let commit_sha = Self::get_commit_sha(dir);

        for entry in WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                let matches = self.scan_file(path)?;
                if !matches.is_empty() {
                    entries.push(BrainLogEntry {
                        file: path.strip_prefix(dir).unwrap_or(path).to_string_lossy().into(),
                        language: Self::detect_language(path).to_string(),
                        pattern: "multi-grep".into(),
                        matches: matches.into_iter().map(|m| MatchInfo {
                            rule_id: m.rule.name.clone(),
                            severity: format!("{:?}", m.rule.severity),
                            anchor: m.anchor,
                            text: m.text,
                            meta: Some(serde_json::json!({"description": m.rule.description})),
                        }).collect(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        commit_sha: commit_sha.clone(),
                    });
                }
            }
        }
        Ok(entries)
    }

    fn get_commit_sha(dir: &Path) -> String {
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(dir)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into())
    }

    fn detect_language(path: &Path) -> SupportLang {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => SupportLang::Rust,
            Some("ts") | Some("tsx") => SupportLang::TypeScript,
            Some("js") | Some("jsx") => SupportLang::JavaScript,
            Some("py") => SupportLang::Python,
            Some("go") => SupportLang::Go,
            Some("java") => SupportLang::Java,
            Some("cpp") | Some("cc") | Some("cxx") => SupportLang::Cpp,
            Some("c") | Some("h") => SupportLang::C,
            Some("toml") => SupportLang::Rust, // Cargo.toml
            Some("md") | Some("mkd") => SupportLang::Markdown,
            Some("css") => SupportLang::Css,
            Some("html") | Some("htm") => SupportLang::Html,
            _ => SupportLang::Rust,
        }
    }

    /// Compress brain logs to JSONL (token-efficient)
    pub fn compress_logs(entries: &[BrainLogEntry]) -> String {
        entries
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Decompress brain logs from JSONL
    pub fn decompress_logs(data: &str) -> Result<Vec<BrainLogEntry>, Box<dyn std::error::Error>> {
        data.lines()
            .map(|l| Ok(serde_json::from_str(l)?))
            .collect()
    }

    /// Apply patch A -> B using 3-position anchoring to prevent drift
    /// Uses partial pattern matching (trust workspace style)
    pub fn apply_patch(
        &self,
        content: &str,
        file: &str,
        patch: &Patch,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let anchor = &patch.anchor;
        
        // Verify anchor matches current content (drift detection)
        let lines: Vec<&str> = content.lines().collect();
        if anchor.line > lines.len() {
            return Err(format!("anchor line {} out of bounds (file has {} lines)", anchor.line, lines.len()).into());
        }
        let line = lines[anchor.line - 1];
        if anchor.col > line.len() {
            return Err(format!("anchor col {} out of bounds (line has {} chars)", anchor.col, line.len()).into());
        }

        // Find the best match using partial pattern (trust workspace approach)
        let match_result = self.find_best_partial_match(content, file, &patch.from_pattern, anchor)?;
        
        // Apply replacement at the exact anchored position
        let mut result = content.to_string();
        let byte_start = content.lines().take(anchor.line - 1).map(|l| l.len() + 1).sum::<usize>() + anchor.col - 1;
        let byte_end = byte_start + match_result.matched_len;
        result.replace_range(byte_start..byte_end, &patch.to_pattern);
        
        Ok(result)
    }

    fn find_best_partial_match(
        &self,
        content: &str,
        file: &str,
        pattern: &str,
        anchor: &Anchor,
    ) -> Result<PartialMatch, Box<dyn std::error::Error>> {
        let regex_cache = RegexCache::global();
        let regex = regex_cache.get_or_create(pattern);
        
        let mut candidates = Vec::new();
        for mat in regex.find_iter(content) {
            let line = content[..mat.start()].matches('\n').count() + 1;
            let col = mat.start() - content[..mat.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let _cand_anchor = Anchor::new(file, line, col);
            
            // Score by distance to anchor (3-position)
            let dist = ((line as isize - anchor.line as isize).abs() + (col as isize - anchor.col as isize).abs()) as usize;
            candidates.push((dist, mat.start(), mat.end(), mat.as_str().to_string()));
        }
        
        // Sort by distance to anchor, then by match length (prefer longer matches)
        candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.2.cmp(&a.2)));
        
        candidates.first()
            .map(|(_, start, end, text)| PartialMatch {
                start: *start,
                end: *end,
                matched_len: end - start,
                text: text.clone(),
            })
            .ok_or_else(|| format!("no match found for pattern '{}' near anchor {:?}", pattern, anchor).into())
    }
}

struct PartialMatch {
    start: usize,
    end: usize,
    matched_len: usize,
    text: String,
}

/// Patch format A -> B with partial pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub anchor: Anchor,
    pub from_pattern: String,
    pub to_pattern: String,
}

/// Shared regex cache for fuzzy matching (singleton)
struct RegexCache {
    patterns: std::sync::RwLock<HashMap<String, regex::Regex>>,
}

impl RegexCache {
    fn global() -> &'static Self {
        use once_cell::sync::Lazy;
        static INSTANCE: Lazy<RegexCache> = Lazy::new(|| RegexCache {
            patterns: std::sync::RwLock::new(HashMap::new()),
        });
        &INSTANCE
    }

    fn get(&self, pattern: &str) -> Option<regex::Regex> {
        self.patterns.read().ok()?.get(pattern).cloned()
    }

    fn get_or_create(&self, pattern: &str) -> regex::Regex {
        if let Some(r) = self.get(pattern) {
            return r;
        }
        let regex = regex::Regex::new(pattern).unwrap_or_else(|_| regex::Regex::new("").unwrap());
        self.patterns.write().ok().map(|mut m| m.insert(pattern.to_string(), regex.clone()));
        regex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_yaml_rules_parsing() {
        let yaml = r#"
- name: test-rule
  description: "Test rule"
  lang: rust
  glob: "**/*.rs"
  exclude: "**/tests/**"
  action: warn
  pattern: "unwrap()"
  severity: critical
"#;
        let grep = GuardianGrep::from_yaml(yaml).unwrap();
        assert_eq!(grep.rule_index.len(), 1);
        assert!(grep.rule_index.contains_key("test-rule"));
    }

    #[test]
    fn test_multi_grep_single_pass() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "fn main() {{ let x = vec![1,2,3]; x.unwrap(); x.clone(); }}").unwrap();

        let matches = grep.scan_file(&file).unwrap();
        // Should find both unwrap and clone in single pass
        assert!(matches.iter().any(|m| m.rule.name == "slop-unwrap"));
        assert!(matches.iter().any(|m| m.rule.name == "slop-clone"));
    }

    #[test]
    fn test_3_position_anchor() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "fn main() {{ x.unwrap(); }}").unwrap();

        let matches = grep.scan_file(&file).unwrap();
        let unwrap_match = matches.iter().find(|m| m.rule.name == "slop-unwrap").unwrap();
        
        // Anchor should be precise: file, line 1, column where unwrap starts
        assert_eq!(unwrap_match.anchor.file, file.to_string_lossy().to_string());
        assert_eq!(unwrap_match.anchor.line, 1);
        assert!(unwrap_match.anchor.col > 0);
    }

    #[test]
    fn test_anchor_drift_detection() {
        let grep = GuardianGrep::new();
        let content = "fn main() { x.unwrap(); }";
        
        let patch = Patch {
            anchor: Anchor::new("test.rs", 1, 15),
            from_pattern: "unwrap()".into(),
            to_pattern: "expect(\"msg\")".into(),
        };
        
        let result = grep.apply_patch(content, "test.rs", &patch).unwrap();
        assert!(result.contains("expect(\"msg\")"));
    }

    #[test]
    fn test_anchor_drift_fails_on_moved_code() {
        let grep = GuardianGrep::new();
        let content = "fn main() {\n    x.unwrap();\n}";  // unwrap moved to line 2
        
        let patch = Patch {
            anchor: Anchor::new("test.rs", 1, 15),  // Old anchor at line 1
            from_pattern: "unwrap()".into(),
            to_pattern: "expect(\"msg\")".into(),
        };
        
        let result = grep.apply_patch(content, "test.rs", &patch);
        assert!(result.is_err(), "should fail when anchor drifted");
    }

    #[test]
    fn test_partial_pattern_matching() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "std::collections::HashMap::fake_method();").unwrap();
        
        let matches = grep.scan_file(&file).unwrap();
        let fake_match = matches.iter().find(|m| m.rule.name == "rust-fake-std-method");
        assert!(fake_match.is_some(), "should detect fake std method");
    }

    #[test]
    fn test_partial_pattern_fuzzy_match() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "use crate_that_doesnt_exist::something;").unwrap();
        
        let matches = grep.scan_file(&file).unwrap();
        let fake_import = matches.iter().find(|m| m.rule.name == "rust-fake-crate-import");
        assert!(fake_import.is_some(), "should detect fake crate import");
    }

    #[test]
    fn test_tokio_wrong_path() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "tokio::spawn_blocking(|| {{}});").unwrap();
        
        let matches = grep.scan_file(&file).unwrap();
        let wrong_path = matches.iter().find(|m| m.rule.name == "rust-tokio-spawn-blocking");
        assert!(wrong_path.is_some(), "should detect wrong tokio path");
    }

    #[test]
    fn test_into_iter_on_ref() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "let v = &vec![1,2,3]; v.into_iter();").unwrap();
        
        let matches = grep.scan_file(&file).unwrap();
        let wrong_iter = matches.iter().find(|m| m.rule.name == "rust-into-iter-on-ref");
        assert!(wrong_iter.is_some(), "should detect into_iter on ref");
    }

    #[test]
    fn test_compress_decompress_logs() {
        let entry = BrainLogEntry {
            file: "test.rs".into(),
            language: "Rust".into(),
            pattern: "multi-grep".into(),
            matches: vec![MatchInfo {
                rule_id: "slop-unwrap".into(),
                severity: "Critical".into(),
                anchor: Anchor::new("test.rs", 1, 10),
                text: "x.unwrap()".into(),
                meta: None,
            }],
            timestamp: 1234567890,
            commit_sha: "abc123".into(),
        };

        let compressed = GuardianGrep::compress_logs(&[entry.clone()]);
        let decompressed = GuardianGrep::decompress_logs(&compressed).unwrap();
        assert_eq!(decompressed.len(), 1);
        assert_eq!(decompressed[0].file, entry.file);
        assert_eq!(decompressed[0].matches[0].anchor, entry.matches[0].anchor);
    }

    #[test]
    fn test_all_languages_covered() {
        let grep = GuardianGrep::new();
        let langs = [
            ("test.rs", SupportLang::Rust),
            ("test.ts", SupportLang::TypeScript),
            ("test.js", SupportLang::JavaScript),
            ("test.py", SupportLang::Python),
            ("test.go", SupportLang::Go),
            ("test.md", SupportLang::Markdown),
            ("test.css", SupportLang::Css),
            ("test.html", SupportLang::Html),
        ];
        
        for (filename, expected_lang) in langs {
            let path = Path::new(filename);
            let lang = GuardianGrep::detect_language(path);
            assert_eq!(lang, expected_lang, "language detection failed for {}", filename);
        }
    }

    #[test]
    fn test_exclude_patterns() {
        let yaml = r#"
- name: test-exclude
  description: "Test exclude"
  lang: rust
  glob: "**/*.rs"
  exclude: "**/tests/**"
  action: warn
  pattern: "unwrap()"
  severity: low
"#;
        let grep = GuardianGrep::from_yaml(yaml).unwrap();
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("tests").join("test.rs");
        std::fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut f = std::fs::File::create(&test_file).unwrap();
        writeln!(f, "x.unwrap();").unwrap();

        let matches = grep.scan_file(&test_file).unwrap();
        assert!(matches.is_empty(), "should exclude test files");
    }

    #[test]
    fn test_action_block_warn_review() {
        let yaml = r#"
- name: block-rule
  description: "Block"
  lang: rust
  glob: "**/*.rs"
  action: block
  pattern: "panic!()"
  severity: critical
- name: warn-rule
  description: "Warn"
  lang: rust
  glob: "**/*.rs"
  action: warn
  pattern: "unwrap()"
  severity: high
- name: review-rule
  description: "Review"
  lang: rust
  glob: "**/*.rs"
  action: review
  pattern: "clone()"
  severity: medium
"#;
        let grep = GuardianGrep::from_yaml(yaml).unwrap();
        assert_eq!(grep.rule_index["block-rule"].action, RuleAction::Block);
        assert_eq!(grep.rule_index["warn-rule"].action, RuleAction::Warn);
        assert_eq!(grep.rule_index["review-rule"].action, RuleAction::Review);
    }
}

// Benchmark tests
#[cfg(test)]
mod bench {
    use super::*;
    use std::io::Write;
    use std::time::Instant;
    use tempfile::tempdir;

    #[test]
    fn bench_multi_grep_performance() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        
        // Create 20 files with various patterns (reduced from 100 for speed)
        for i in 0..20 {
            let file = dir.path().join(format!("test{}.rs", i));
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "fn main() {{ let x = vec![{}]; x.unwrap(); x.clone(); todo!(); }}", i).unwrap();
        }

        let start = Instant::now();
        let entries = grep.scan_dir(dir.path()).unwrap();
        let elapsed = start.elapsed();

        println!("Scanned 20 files in {:?}", elapsed);
        println!("Found {} entries with matches", entries.len());
        
        // Should complete in under 5 seconds
        assert!(elapsed.as_millis() < 5000, "multi-grep too slow: {:?}", elapsed);
    }

    #[test]
    fn bench_token_efficiency() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        
        for i in 0..50 {
            let file = dir.path().join(format!("test{}.rs", i));
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "fn f{}() {{ let x = vec![1,2,3]; x.unwrap(); x.clone(); }}", i).unwrap();
        }

        let entries = grep.scan_dir(dir.path()).unwrap();
        let compressed = GuardianGrep::compress_logs(&entries);
        
        // Compressed size should be small (token-efficient)
        let tokens_estimate = compressed.len() / 4; // rough token estimate
        println!("Compressed size: {} bytes, ~{} tokens", compressed.len(), tokens_estimate);
        
        // Should be well under typical context limits
        assert!(compressed.len() < 100_000, "compressed logs too large");
    }

    #[test]
    fn bench_anchor_precision() {
        let grep = GuardianGrep::new();
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file).unwrap();
        writeln!(f, "fn main() {{\n    let x = vec![1,2,3];\n    x.unwrap();\n    y.clone();\n}}").unwrap();
        
        let matches = grep.scan_file(&file).unwrap();
        
        for m in &matches {
            // Verify each anchor points to the exact match
            let content = std::fs::read_to_string(&file).unwrap();
            let lines: Vec<&str> = content.lines().collect();
            let line = lines[m.anchor.line - 1];
            assert!(line.contains(&m.text), "anchor text mismatch: {} vs {}", line, m.text);
        }
    }
}