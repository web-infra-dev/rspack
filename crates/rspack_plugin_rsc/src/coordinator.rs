use std::sync::Arc;

use derive_more::Debug;
use rspack_core::CompilerId;
use rspack_error::{Error, Result};
use rustc_hash::FxHashMap;
use tokio::sync::{Mutex, Notify};

use crate::plugin_state::ActionIdNamePair;

pub(crate) type ActionsByEntry = FxHashMap<Arc<str>, FxHashMap<String, Vec<ActionIdNamePair>>>;

#[derive(Debug)]
enum State {
  Idle,
  BuildingServerEntries,
  ClientEntriesReady(#[debug(skip)] Vec<Arc<str>>),
  BuildingClientEntries,
  ServerActionsReady(#[debug(skip)] ActionsByEntry),
  BuildingServerActions,
  AwaitingClientManifest,
  ServerFinishing,
}

/// Coordinates the handoff between the RSC server and client compilers.
///
/// ```text
/// Idle -> BuildingServerEntries -> ClientEntriesReady
/// ClientEntriesReady -> BuildingClientEntries -> ServerActionsReady
/// ServerActionsReady -> BuildingServerActions
/// BuildingServerActions -> ClientEntriesReady      (entry plan changed)
/// BuildingServerActions -> AwaitingClientManifest  (entry plan stable)
/// AwaitingClientManifest -> ServerFinishing -> Idle
/// ```
///
/// Each compiler takes ownership of its batch under the state lock when it
/// moves from a ready state to a building state. In `AwaitingClientManifest`,
/// the client can seal. The server waits until the client publishes the manifest
/// and moves the state to `ServerFinishing`. The first fatal error replaces any
/// state and wakes both compilers.
pub struct Coordinator {
  state: Mutex<Result<State>>,
  state_notify: Notify,
  server_compiler_id: Mutex<Option<CompilerId>>,
}

impl Default for Coordinator {
  fn default() -> Self {
    Self::new()
  }
}

impl Coordinator {
  pub fn new() -> Self {
    Self {
      state: Mutex::new(Ok(State::Idle)),
      state_notify: Default::default(),
      server_compiler_id: Default::default(),
    }
  }

  pub async fn get_server_compiler_id(&self) -> Result<CompilerId> {
    (*self.server_compiler_id.lock().await).ok_or_else(|| {
      rspack_error::error!(
        "Server compiler ID is not registered in the RSC coordinator. \\
         Ensure RscServerPlugin starts compiling before RscClientPlugin."
      )
    })
  }

  async fn wait_for<T>(&self, mut ready: impl FnMut(&mut State) -> Option<T>) -> Result<T> {
    loop {
      // Register before reading the state so a concurrent notification cannot be missed.
      let notified = self.state_notify.notified();
      {
        let mut state = self.state.lock().await;
        if let Some(result) = ready(state.as_mut().map_err(|error| error.clone())?) {
          return Ok(result);
        }
      }
      notified.await;
    }
  }

  async fn transition(
    &self,
    expected: impl FnOnce(&State) -> bool,
    next: State,
    context: &'static str,
  ) -> Result<()> {
    let mut state = self.state.lock().await;
    let current = state.as_ref().map_err(Clone::clone)?;
    if !expected(current) {
      return Err(rspack_error::error!(
        "Invalid RSC coordinator state in {context}: {current:?}"
      ));
    }
    *state = Ok(next);
    self.state_notify.notify_waiters();
    Ok(())
  }

  pub(crate) async fn start_server_entries(&self, compiler_id: CompilerId) -> Result<()> {
    loop {
      {
        let mut state = self.state.lock().await;
        if matches!(state.as_ref().map_err(Clone::clone)?, State::Idle) {
          *self.server_compiler_id.lock().await = Some(compiler_id);
          *state = Ok(State::BuildingServerEntries);
          self.state_notify.notify_waiters();
          return Ok(());
        }
      }
      self
        .wait_for(|state| matches!(state, State::Idle).then_some(()))
        .await?;
    }
  }

  pub(crate) async fn server_entries_done(&self, entries: Vec<Arc<str>>) -> Result<()> {
    self
      .transition(
        |state| matches!(state, State::BuildingServerEntries),
        State::ClientEntriesReady(entries),
        "server_entries_done",
      )
      .await
  }

  /// Claim the next client entry batch. Returns `None` when the client can seal.
  pub(crate) async fn next_client_entries(&self) -> Result<Option<Vec<Arc<str>>>> {
    self
      .wait_for(|state| match state {
        State::ClientEntriesReady(entries) => {
          let entries = std::mem::take(entries);
          *state = State::BuildingClientEntries;
          Some(Some(entries))
        }
        State::AwaitingClientManifest => Some(None),
        _ => None,
      })
      .await
  }

  pub(crate) async fn client_round_done(&self, actions: ActionsByEntry) -> Result<()> {
    self
      .transition(
        |state| matches!(state, State::BuildingClientEntries),
        State::ServerActionsReady(actions),
        "client_round_done",
      )
      .await
  }

  /// Claim the next server action batch. Returns `None` when the client manifest is ready.
  pub(crate) async fn next_server_actions(&self) -> Result<Option<ActionsByEntry>> {
    self
      .wait_for(|state| match state {
        State::ServerActionsReady(actions) => {
          let actions = std::mem::take(actions);
          *state = State::BuildingServerActions;
          Some(Some(actions))
        }
        State::ServerFinishing => Some(None),
        _ => None,
      })
      .await
  }

  pub(crate) async fn server_round_done(&self, updated_entries: Vec<Arc<str>>) -> Result<()> {
    let next = if updated_entries.is_empty() {
      State::AwaitingClientManifest
    } else {
      State::ClientEntriesReady(updated_entries)
    };
    self
      .transition(
        |state| matches!(state, State::BuildingServerActions),
        next,
        "server_round_done",
      )
      .await
  }

  pub(crate) async fn client_manifest_ready(&self) -> Result<()> {
    self
      .transition(
        |state| matches!(state, State::AwaitingClientManifest),
        State::ServerFinishing,
        "client_manifest_ready",
      )
      .await
  }

  pub(crate) async fn server_done(&self) -> Result<()> {
    self
      .transition(
        |state| matches!(state, State::ServerFinishing),
        State::Idle,
        "server_done",
      )
      .await
  }

  pub async fn failed(&self, error: &Error) {
    let mut state = self.state.lock().await;
    // Keep the original error when the other compiler fails in response.
    if state.is_ok() {
      *state = Err(error.clone());
      self.state_notify.notify_waiters();
    }
  }
}
