pub mod ai_behavior;
pub mod core;
pub mod detectors;
pub mod guardian_grep;
pub mod integration;
pub mod learning;
pub mod lsp;
pub mod report;

/// High-level API for embedding Guardian in other projects
pub mod api {
    use crate::{
        core::{
            metrics::{AIBehaviorMetrics, AIMetrics, CodeMetrics},
            metrics_db::{MetricsDb, default_db_path},
        },
        detectors::{
            command_log_detector::{CommandDetectorConfig, CommandLogDetector},
            minimal_check_detector::{MinimalCheckDetector, MinimalCheckReport},
            over_engineer_detector::{OverEngineerDetector, OverEngineeringReport},
            ponytail_comment_detector::{PonytailCommentDetector, PonytailCommentReport},
            slop_detector::{SlopDetector, SlopIssue},
            yagni_detector::{YAGNIDetector, YAGNIReport},
        },
    };
    use std::path::Path;

    /// Quick analysis result for a single file
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct FileAnalysis {
        pub file: String,
        pub quality_score: f64,
        pub grade: String,
        pub slop_issues: Vec<SlopIssue>,
        pub yagni_report: YAGNIReport,
        pub over_engineering: OverEngineeringReport,
        pub ponytail_marks: PonytailCommentReport,
        pub minimal_checks: MinimalCheckReport,
    }

    /// Guardian configuration
    #[derive(Debug, Clone)]
    pub struct GuardianConfig {
        pub db_path: Option<String>,
        pub slop_threshold: usize,
        pub loop_threshold: usize,
    }

    impl Default for GuardianConfig {
        fn default() -> Self {
            Self {
                db_path: None,
                slop_threshold: 10,
                loop_threshold: 3,
            }
        }
    }

    /// Main Guardian API
    pub struct Guardian {
        config: GuardianConfig,
    }

    impl Guardian {
        pub fn new(config: GuardianConfig) -> Self {
            Self { config }
        }

        pub fn default() -> Self {
            Self::new(GuardianConfig::default())
        }

        /// Analyze a single file
        pub fn analyze_file(
            &self,
            path: &Path,
        ) -> Result<FileAnalysis, Box<dyn std::error::Error>> {
            let code = std::fs::read_to_string(path)?;

            let issues = SlopDetector::new().detect(&code);
            let yd = YAGNIDetector::new();
            let y = yd.analyze(&code);
            let over = OverEngineerDetector::analyze(&code);
            let ponytail_marks = PonytailCommentDetector::analyze(&code);
            let minimal_checks = MinimalCheckDetector::analyze(&code);

            let slop_score = issues.len() as f64 / 10.0;
            let yagni_violations = y.unused_functions.len() + y.unused_variables.len() + y.unused_imports.len();
            let m = AIMetrics {
                commit_sha: "unknown".into(),
                file_path: path.to_string_lossy().into(),
                timestamp: chrono::Utc::now(),
                ai_lines: vec![],
                metrics: CodeMetrics {
                    slop_score,
                    yagni_violations,
                    dead_code_lines: y.unused_functions.len(),
                    over_engineering_score: over.score,
                    inefficient_patterns: over.findings.len(),
                    ..Default::default()
                },
                ai_behavior: AIBehaviorMetrics::default(),
                quality_score: 0.0,
            };
            let quality_score = m.calculate_overall_score();

            Ok(FileAnalysis {
                file: path.to_string_lossy().into(),
                quality_score,
                grade: Self::grade(quality_score),
                slop_issues: issues,
                yagni_report: y,
                over_engineering: over,
                ponytail_marks,
                minimal_checks,
            })
        }

        /// Analyze multiple files
        pub fn analyze_files(
            &self,
            paths: &[&Path],
        ) -> Vec<Result<FileAnalysis, Box<dyn std::error::Error>>> {
            paths.iter().map(|p| self.analyze_file(p)).collect()
        }

        /// Get database connection
        pub fn db(&self) -> Result<MetricsDb, Box<dyn std::error::Error>> {
            let path = self.config.db_path.clone().unwrap_or_else(|| {
                default_db_path().to_string_lossy().into_owned()
            });
            if let Some(parent) = std::path::Path::new(&path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            Ok(MetricsDb::open(&path)?)
        }

        /// Import command log and analyze
        pub fn import_and_analyze_commands(
            &self,
            log_path: &Path,
        ) -> Result<
            crate::detectors::command_log_detector::CommandLogReport,
            Box<dyn std::error::Error>,
        > {
            let db = self.db()?;
            let content = std::fs::read_to_string(log_path)?;
            db.import_command_log(&content, None)?;
            let commands = db.recent_commands(1000)?;
            let detector = CommandLogDetector::new(CommandDetectorConfig {
                loop_threshold: self.config.loop_threshold,
            });
            Ok(detector.analyze(&commands))
        }

        fn grade(score: f64) -> String {
            if score >= 90.0 {
                "A+"
            } else if score >= 80.0 {
                "A"
            } else if score >= 70.0 {
                "B"
            } else if score >= 60.0 {
                "C"
            } else {
                "F"
            }
            .into()
        }
    }
}

// Re-export commonly used types
pub use api::{FileAnalysis, Guardian, GuardianConfig};
pub use core::metrics_db::MetricsDb;
pub use detectors::{
    command_log_detector::{CommandDetectorConfig, CommandLogDetector, CommandLogReport},
    minimal_check_detector::{MinimalCheckDetector, MinimalCheckReport},
    over_engineer_detector::{OverEngineerDetector, OverEngineeringKind, OverEngineeringReport},
    ponytail_comment_detector::{PonytailCommentDetector, PonytailCommentReport},
    slop_detector::{SeverityLevel, SlopCategory, SlopDetector, SlopIssue},
    yagni_detector::{YAGNIDetector, YAGNIReport},
};
pub use guardian_grep::{BrainLogEntry, GuardianGrep, MatchInfo, Severity};
