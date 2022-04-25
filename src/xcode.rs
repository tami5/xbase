use anyhow::Result;
use serde_json::json;
use std::ffi;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

// https://github.com/Gordon-F/cargo-xcodebuild
/// run xcodebuild build with extra arguments
pub async fn build<P, I, S>(root: P, args: I) -> Result<Vec<String>>
where
    P: AsRef<Path> + Debug,
    I: IntoIterator<Item = S>,
    S: AsRef<ffi::OsStr>,
{
    tracing::info!("Building {:?}", root);
    let output = Command::new("/usr/bin/xcodebuild")
        .arg("build")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .current_dir(root)
        .spawn()?
        .wait_with_output()
        .await
        .map(|o| String::from_utf8(o.stdout))??
        .split("\n")
        .map(|s| s.to_string())
        .collect();

    // TODO: Check xcodebuild build output if it contains failure
    //
    // Command succeed (return 0 status) but the output contains failure! need to be handled
    // somehow as errror
    tracing::trace!(
        "xcodebuild output: \n{:#?}\n\n\n---------------------------------- end",
        output
    );
    Ok(output)
}

/// run xcodebuild clean with extra arguments
pub async fn clean<P, I, S>(root: P, args: I) -> Result<ExitStatus, std::io::Error>
where
    P: AsRef<Path> + Debug,
    I: IntoIterator<Item = S>,
    S: AsRef<ffi::OsStr>,
{
    tracing::info!("Cleaning {:?}", root);

    Command::new("/usr/bin/xcodebuild")
        .arg("clean")
        .args(args)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start xcodebuild clean")
        .wait()
        .await
}

pub async fn ensure_server_config_file(root: &PathBuf) -> Result<()> {
    let path = root.join("buildServer.json");
    if fs::File::open(&path).await.is_ok() {
        return Ok(());
    }

    tracing::info!("Creating {:?}", path);

    let mut file = fs::File::create(path).await?;
    let config = json! ({
        "name": "XcodeBase Server",
        // FIXME: Point to user xcode-build-server
        "argv": ["/Users/tami5/repos/neovim/XcodeBase.nvim/target/debug/xcodebase-server"],
        "version": "0.1",
        "bspVersion": "0.2",
        "languages": [
            "swift",
            "objective-c",
            "objective-cpp",
            "c",
            "cpp"
        ]
    });

    file.write_all(config.to_string().as_ref()).await?;
    file.sync_all().await?;
    file.shutdown().await?;

    Ok(())
}