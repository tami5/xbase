use crate::broadcast::{self, Broadcast};
use crate::constants::{State, DAEMON_STATE};
use crate::util::log_request;
use crate::watch::{Event, Watchable};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use xbase_proto::BuildRequest;

/// Handle build Request
pub async fn handle(req: BuildRequest) -> Result<()> {
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;
    let client = &req.client;
    let root = &req.client.root;
    let broadcast = state.broadcasters.get(&client.root)?;
    let target = &req.settings.target;
    let args = &req.settings.to_string();

    log_request!("Build", root, req);

    if req.ops.is_once() {
        req.trigger(state, &Event::default(), &broadcast).await?;
        return Ok(());
    }

    if req.ops.is_watch() {
        broadcast::notify_info!(broadcast, "[{target}] Watching  with '{args}'")?;
        state.watcher.get_mut(&req.client.root)?.add(req)?;
    } else {
        state
            .watcher
            .get_mut(&req.client.root)?
            .remove(&req.to_string())?;
    }

    Ok(())
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(
        &self,
        state: &MutexGuard<State>,
        _event: &Event,
        broadcast: &Arc<Broadcast>,
    ) -> Result<()> {
        let is_once = self.ops.is_once();
        let config = &self.settings;
        let root = &self.client.root;
        let target = &self.settings.target;
        let project = state.projects.get(root)?;

        if is_once {
            broadcast::notify_info!(broadcast, "[{target}] Building ⚙")?;
        }
        let (args, mut recv) = project.build(&config, None, broadcast)?;

        if !recv.recv().await.unwrap_or_default() {
            let verb = if is_once { "building" } else { "Rebuilding" };
            broadcast::notify_error!(broadcast, "[{target}] {verb} Failed, checkout logs")?;
            broadcast::log_error!(broadcast, "[{target}] ran args {}", args.join(" "))?;
        } else {
            broadcast::notify_info!(broadcast, "[{target}] Built ")?;
        };

        Ok(())
    }

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, _state: &MutexGuard<State>, event: &Event) -> bool {
        event.is_content_update_event()
            || event.is_rename_event()
            || event.is_create_event()
            || event.is_remove_event()
            || !(event.path().exists() || event.is_seen())
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _state: &MutexGuard<State>, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self, _state: &MutexGuard<State>) -> Result<()> {
        Ok(())
    }
}
