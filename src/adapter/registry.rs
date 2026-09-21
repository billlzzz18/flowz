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
        paths
            .into_iter()
            .map(|path| MarkdownDefinition::parse(&path))
            .collect()
    }
}
fn collect_markdown(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if std::fs::symlink_metadata(&path)?.is_dir() {
            collect_markdown(&path, paths)?;
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
