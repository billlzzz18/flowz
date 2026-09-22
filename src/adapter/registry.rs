use super::markdown::MarkdownDefinition;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub trait DefinitionSource: Send + Sync {
    fn scan(&self) -> Result<Vec<MarkdownDefinition>>;
}

pub struct MarkdownRegistry {
    pub root: PathBuf,
}
impl MarkdownRegistry {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}
impl DefinitionSource for MarkdownRegistry {
    fn scan(&self) -> Result<Vec<MarkdownDefinition>> {
        let mut paths = Vec::new();
        collect_markdown(&self.root, &mut paths)?;
        let mut definitions = Vec::new();
        // ponytail: one malformed/unrelated .md must not kill the whole registry;
        // skip it with a warning, still load the rest. Directory-walk IO errors still propagate.
        for path in paths {
            match MarkdownDefinition::parse(&path) {
                Ok(def) => definitions.push(def),
                Err(crate::error::FlowzError::Validation(e)) => tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "skipping malformed markdown definition"
                ),
                Err(e) => return Err(e),
            }
        }
        Ok(definitions)
    }
}
fn collect_markdown(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    collect_markdown_internal(root, paths, &mut std::collections::HashSet::new())
}

fn collect_markdown_internal(
    root: &Path,
    paths: &mut Vec<PathBuf>,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if !visited.insert(canonical.clone()) {
        // Already visited this directory (symlink cycle)
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            // Skip symlinks to avoid cycles
            continue;
        }
        if meta.is_dir() {
            collect_markdown_internal(&path, paths, visited)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scans_nested_markdown_files() {
        let root = std::env::temp_dir().join(format!("flowz-registry-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::write(root.join("nested/job.md"), "---\nname: Job\ncron: '* * * * *'\n---\nrun")
            .unwrap();
        let definitions = MarkdownRegistry::new(&root).scan().unwrap();
        assert_eq!(definitions.len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }
}
