use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tap::Pipe;
use tokio::process::Command;
// NOTE: use process-stream and log output from generators

#[derive(Debug, Deserialize, Serialize)]
pub enum ProjectGenerator {
    /// No Generator
    None,
    /// XCodeGen Generator
    XCodeGen,
    /// Tuist Generator
    Tuist,
}

impl Default for ProjectGenerator {
    fn default() -> Self {
        Self::None
    }
}

impl ProjectGenerator {
    /// Identify generator from project root
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref();
        if root.join("project.yml").exists() {
            Self::XCodeGen
        } else if root.join("Project.swift").exists() {
            Self::Tuist
        } else {
            Self::None
        }
    }

    /// Check if is a supported generator file
    pub fn is_genertor_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .file_name()
            .and_then(|f| f.to_str())
            .map(|name| name == "project.yml" || name == "Project.swift")
            .unwrap_or_default()
    }

    /// Regenerate project from given path
    /// TODO(regenerate): return Result<Option<Stream>>
    ///
    /// commands like tuist does network calls. Which makes very important to have logs for
    /// regeneration
    pub async fn regenerate(&self, root: &PathBuf) -> Result<bool> {
        match self {
            ProjectGenerator::None => Ok(false),
            ProjectGenerator::XCodeGen => Command::new(which::which("xcodegen")?)
                .current_dir(root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .arg("generate")
                .arg("-c")
                .spawn()?
                .wait()
                .await?
                .success()
                .pipe(Ok),
            // tuist is most likely installed in /usr/local/bin/tuist, but here to still use
            // which in cases tuist is install in some other location.
            ProjectGenerator::Tuist => Command::new(which::which("tuist")?)
                .current_dir(root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .arg("generate")
                .arg("--no-open") // prevent xcode from being opened
                .spawn()?
                .wait()
                .await?
                .success()
                .pipe(Ok),
        }
    }

    /// Returns `true` if the project generator is [`XCodeGen`].
    ///
    /// [`XCodeGen`]: ProjectGenerator::XCodeGen
    #[must_use]
    pub fn is_xcodegen(&self) -> bool {
        matches!(self, Self::XCodeGen)
    }

    /// Returns `true` if the project generator is [`Tuist`].
    ///
    /// [`Tuist`]: ProjectGenerator::Tuist
    #[must_use]
    pub fn is_tuist(&self) -> bool {
        matches!(self, Self::Tuist)
    }

    /// Returns `true` if the project generator is [`None`].
    ///
    /// [`None`]: ProjectGenerator::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
