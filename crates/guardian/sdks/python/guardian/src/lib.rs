use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::path::Path;

#[pyclass]
#[derive(Clone)]
pub struct GuardianConfig {
    #[pyo3(get, set)]
    pub db_path: Option<String>,
    #[pyo3(get, set)]
    pub slop_threshold: usize,
    #[pyo3(get, set)]
    pub loop_threshold: usize,
}

#[pymethods]
impl GuardianConfig {
    #[new]
    #[pyo3(signature = (db_path=None, slop_threshold=10, loop_threshold=3))]
    fn new(db_path: Option<String>, slop_threshold: usize, loop_threshold: usize) -> Self {
        Self {
            db_path,
            slop_threshold,
            loop_threshold,
        }
    }
}

#[pyclass]
pub struct FileAnalysis {
    #[pyo3(get)]
    pub file: String,
    #[pyo3(get)]
    pub quality_score: f64,
    #[pyo3(get)]
    pub grade: String,
    #[pyo3(get)]
    pub slop_issues: Vec<SlopIssue>,
    #[pyo3(get)]
    pub yagni_report: YAGNIReport,
    #[pyo3(get)]
    pub over_engineering: OverEngineeringReport,
    #[pyo3(get)]
    pub ponytail_marks: PonytailCommentReport,
    #[pyo3(get)]
    pub minimal_checks: MinimalCheckReport,
}

#[pyclass]
#[derive(Clone)]
pub struct SlopIssue {
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub pattern_id: String,
    #[pyo3(get)]
    pub pattern_name: String,
    #[pyo3(get)]
    pub severity: String,
    #[pyo3(get)]
    pub suggestion: String,
    #[pyo3(get)]
    pub category: String,
    #[pyo3(get)]
    pub code: String,
}

#[pyclass]
#[derive(Clone)]
pub struct YAGNIReport {
    #[pyo3(get)]
    pub unused_functions: usize,
    #[pyo3(get)]
    pub unused_variables: usize,
    #[pyo3(get)]
    pub unused_imports: usize,
    #[pyo3(get)]
    pub over_engineering_score: f64,
}

#[pyclass]
#[derive(Clone)]
pub struct OverEngineeringReport {
    #[pyo3(get)]
    pub findings: Vec<OverEngineeringFinding>,
    #[pyo3(get)]
    pub net_deletable_lines: usize,
    #[pyo3(get)]
    pub score: f64,
}

#[pyclass]
#[derive(Clone)]
pub struct OverEngineeringFinding {
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub suggestion: String,
    #[pyo3(get)]
    pub deletable_lines: usize,
}

#[pyclass]
#[derive(Clone)]
pub struct PonytailCommentReport {
    #[pyo3(get)]
    pub marks: Vec<PonytailMark>,
    #[pyo3(get)]
    pub total_shortcuts: usize,
}

#[pyclass]
#[derive(Clone)]
pub struct PonytailMark {
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub shortcut: String,
    #[pyo3(get)]
    pub upgrade_path: Option<String>,
}

#[pyclass]
#[derive(Clone)]
pub struct MinimalCheckReport {
    #[pyo3(get)]
    pub unchecked: Vec<UncheckedFunction>,
    #[pyo3(get)]
    pub total_nontrivial: usize,
    #[pyo3(get)]
    pub coverage_pct: f64,
}

#[pyclass]
#[derive(Clone)]
pub struct UncheckedFunction {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub has_branches: bool,
    #[pyo3(get)]
    pub has_loops: bool,
}

#[pyclass]
pub struct Guardian {
    config: GuardianConfig,
    inner: guardian::api::Guardian,
}

#[pymethods]
impl Guardian {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<GuardianConfig>) -> PyResult<Self> {
        let config = config.unwrap_or_default();
        let inner = guardian::api::Guardian::new(guardian::api::GuardianConfig {
            db_path: config.db_path.clone(),
            slop_threshold: config.slop_threshold,
            loop_threshold: config.loop_threshold,
        });
        Ok(Self { config, inner })
    }

    fn analyze_file(&self, path: &str) -> PyResult<FileAnalysis> {
        let result = self
            .inner
            .analyze_file(Path::new(path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self::convert_file_analysis(result))
    }

    fn analyze_files(&self, paths: Vec<String>) -> PyResult<Vec<FileAnalysis>> {
        let paths: Vec<&Path> = paths.iter().map(|p| Path::new(p)).collect();
        let results = self.inner.analyze_files(&paths);
        results
            .into_iter()
            .map(|r| {
                r.map(|a| Self::convert_file_analysis(a))
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
            })
            .collect()
    }

    fn import_and_analyze_commands(&self, log_path: &str) -> PyResult<CommandLogReport> {
        let result = self
            .inner
            .import_and_analyze_commands(Path::new(log_path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self::convert_command_report(result))
    }
}

impl Guardian {
    fn convert_file_analysis(a: guardian::api::FileAnalysis) -> FileAnalysis {
        FileAnalysis {
            file: a.file,
            quality_score: a.quality_score,
            grade: a.grade,
            slop_issues: a.slop_issues.into_iter().map(Self::convert_slop_issue).collect(),
            yagni_report: Self::convert_yagni(a.yagni_report),
            over_engineering: Self::convert_over_engineering(a.over_engineering),
            ponytail_marks: Self::convert_ponytail(a.ponytail_marks),
            minimal_checks: Self::convert_minimal_check(a.minimal_checks),
        }
    }

    fn convert_slop_issue(i: guardian::detectors::slop_detector::SlopIssue) -> SlopIssue {
        SlopIssue {
            line: i.line,
            pattern_id: i.pattern.id,
            pattern_name: i.pattern.name,
            severity: format!("{:?}", i.pattern.severity),
            suggestion: i.pattern.suggestion,
            category: format!("{:?}", i.pattern.category),
            code: i.code,
        }
    }

    fn convert_yagni(y: guardian::detectors::yagni_detector::YAGNIReport) -> YAGNIReport {
        YAGNIReport {
            unused_functions: y.unused_functions.len(),
            unused_variables: y.unused_variables.len(),
            unused_imports: y.unused_imports.len(),
            over_engineering_score: y.over_engineering_score,
        }
    }

    fn convert_over_engineering(
        o: guardian::detectors::over_engineer_detector::OverEngineeringReport,
    ) -> OverEngineeringReport {
        OverEngineeringReport {
            findings: o.findings.into_iter().map(Self::convert_over_finding).collect(),
            net_deletable_lines: o.net_deletable_lines,
            score: o.score,
        }
    }

    fn convert_over_finding(
        f: guardian::detectors::over_engineer_detector::OverEngineeringFinding,
    ) -> OverEngineeringFinding {
        OverEngineeringFinding {
            line: f.line,
            kind: format!("{:?}", f.kind),
            description: f.description,
            suggestion: f.suggestion,
            deletable_lines: f.deletable_lines,
        }
    }

    fn convert_ponytail(
        p: guardian::detectors::ponytail_comment_detector::PonytailCommentReport,
    ) -> PonytailCommentReport {
        PonytailCommentReport {
            marks: p.marks.into_iter().map(Self::convert_mark).collect(),
            total_shortcuts: p.total_shortcuts,
        }
    }

    fn convert_mark(m: guardian::detectors::ponytail_comment_detector::PonytailMark) -> PonytailMark {
        PonytailMark {
            line: m.line,
            text: m.text,
            shortcut: m.shortcut,
            upgrade_path: m.upgrade_path,
        }
    }

    fn convert_minimal_check(
        m: guardian::detectors::minimal_check_detector::MinimalCheckReport,
    ) -> MinimalCheckReport {
        MinimalCheckReport {
            unchecked: m.unchecked.into_iter().map(Self::convert_unchecked).collect(),
            total_nontrivial: m.total_nontrivial,
            coverage_pct: m.coverage_pct,
        }
    }

    fn convert_unchecked(u: guardian::detectors::minimal_check_detector::UncheckedFunction) -> UncheckedFunction {
        UncheckedFunction {
            name: u.name,
            line: u.line,
            has_branches: u.has_branches,
            has_loops: u.has_loops,
        }
    }

    fn convert_command_report(
        r: guardian::detectors::command_log_detector::CommandLogReport,
    ) -> CommandLogReport {
        CommandLogReport {
            scanned: r.scanned,
            findings: r.findings.into_iter().map(Self::convert_cmd_finding).collect(),
            command_frequency: r.command_frequency,
        }
    }

    fn convert_cmd_finding(
        f: guardian::detectors::command_log_detector::CommandFinding,
    ) -> CommandFinding {
        CommandFinding {
            kind: format!("{:?}", f.kind),
            index: f.index,
            severity: f.severity,
            message: f.message,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct CommandLogReport {
    #[pyo3(get)]
    pub scanned: usize,
    #[pyo3(get)]
    pub findings: Vec<CommandFinding>,
    #[pyo3(get)]
    pub command_frequency: std::collections::HashMap<String, usize>,
}

#[pyclass]
#[derive(Clone)]
pub struct CommandFinding {
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub severity: u8,
    #[pyo3(get)]
    pub message: String,
}

#[pymodule]
fn guardian(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GuardianConfig>()?;
    m.add_class::<Guardian>()?;
    m.add_class::<FileAnalysis>()?;
    m.add_class::<SlopIssue>()?;
    m.add_class::<YAGNIReport>()?;
    m.add_class::<OverEngineeringReport>()?;
    m.add_class::<OverEngineeringFinding>()?;
    m.add_class::<PonytailCommentReport>()?;
    m.add_class::<PonytailMark>()?;
    m.add_class::<MinimalCheckReport>()?;
    m.add_class::<UncheckedFunction>()?;
    m.add_class::<CommandLogReport>()?;
    m.add_class::<CommandFinding>()?;
    Ok(())
}