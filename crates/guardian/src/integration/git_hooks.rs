use std::{fs, io, path::Path};
pub struct GitHooksManager;
impl GitHooksManager {
    pub fn install_pre_commit_hook(repo: &str) -> io::Result<()> {
        Self::write(
            repo,
            "pre-commit",
            r#"#!/bin/sh
set -eu
files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)
[ -z "$files" ] && exit 0
for file in $files; do cargo run --quiet --bin guardian -- "$file" || exit 1; done
"#,
        )
    }
    pub fn install_post_commit_hook(repo: &str) -> io::Result<()> {
        Self::write(
            repo,
            "post-commit",
            r#"#!/bin/sh
set -eu
mkdir -p .guardian/metrics
sha=$(git rev-parse HEAD)
cargo run --quiet --bin guardian -- src/lib.rs > ".guardian/metrics/$sha.txt"
"#,
        )
    }
    pub fn install_commit_msg_hook(repo: &str) -> io::Result<()> {
        Self::write(
            repo,
            "commit-msg",
            r#"#!/bin/sh
set -eu
msg=$(cat "$1")
case "$msg" in *AI*|*ai*|*generated*|*assistant*) echo "AI-related commit: review generated code.";; esac
exit 0
"#,
        )
    }
    pub fn install_all_hooks(repo: &str) -> io::Result<()> {
        Self::install_pre_commit_hook(repo)?;
        Self::install_post_commit_hook(repo)?;
        Self::install_commit_msg_hook(repo)
    }
    fn write(repo: &str, name: &str, content: &str) -> io::Result<()> {
        let dir = Path::new(repo).join(".git/hooks");
        if !dir.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, ".git/hooks directory not found"));
        }
        let p = dir.join(name);
        if p.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("hook already exists: {}", p.display()),
            ));
        }
        fs::write(&p, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(p, fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    }
}
