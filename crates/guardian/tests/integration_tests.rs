use guardian::{
    core::git_ai_extractor::GitAIExtractor,
    lsp::GuardianLspAnalyzer,
};
#[cfg(unix)]
use guardian::integration::git_hooks::GitHooksManager;
use std::fs;

fn test_temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("guardian-test")
}
#[test]
fn parses_single_and_range_lines() {
    assert_eq!(
        GitAIExtractor::parse_ranges("1-3,8,10-12").unwrap(),
        vec![(1, 3), (8, 8), (10, 12)]
    );
    assert!(GitAIExtractor::parse_ranges("3-1").is_err());
    assert!(GitAIExtractor::parse_ranges("0").is_err())
}
#[test]
fn parses_attestation_only_for_requested_file() {
    let note = "src/lib.rs\n  sess::trace 1-2,5\nsrc/main.rs\n  other::x 1\n---\n{}";
    let a = GitAIExtractor::parse_attestations(note, "src/lib.rs").unwrap();
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].ai_lines, vec![(1, 2), (5, 5)]);
    assert_eq!(a[0].session_id, "sess")
}
#[test]
fn empty_file_percentage_is_zero() {
    let dir = test_temp_dir();
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("guardian-empty.rs");
    fs::write(&p, "").unwrap();
    assert_eq!(GitAIExtractor::get_ai_percentage(p.to_str().unwrap(), Some("missing")).unwrap_or(0.0), 0.0);
    fs::remove_file(&p).unwrap();
}
#[test]
fn lsp_analyzer_maps_detector_issues() {
    let r = GuardianLspAnalyzer::new().analyze("x.unwrap();");
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].line, 1);
    assert_eq!(r[0].code, "unwrap_001");
    assert!(GuardianLspAnalyzer::ranges_overlap((1, 2), (2, 4)));
    assert!(!GuardianLspAnalyzer::ranges_overlap((1, 1), (2, 4)))
}
#[cfg(unix)]
#[test]
fn hooks_are_written_executable_and_fail_for_non_repo() {
    let dir = test_temp_dir();
    let root = dir.join("guardian-hook-test");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".git/hooks")).unwrap();
    GitHooksManager::install_all_hooks(root.to_str().unwrap()).unwrap();
    for n in ["pre-commit", "post-commit", "commit-msg"] {
        let p = root.join(".git/hooks").join(n);
        assert!(p.exists());
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o111, 0o111)
    }
    assert!(GitHooksManager::install_all_hooks("/tmp/not-a-repo").is_err());
    fs::remove_dir_all(&root).unwrap();
}
