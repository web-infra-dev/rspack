use rspack_core::CompilerId;
use rspack_error::{Error, Result};
use tokio::sync::{Mutex, Notify};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
  Idle,
  ServerEntriesCompiling,
  ServerEntriesDone,
  ClientEntriesCompiling,
  ClientEntriesDone,
  ServerActionsCompiling,
  ServerActionsDone,
  ClientManifestDone,
}

/// Coordinates the compilation sequence between Server Compiler and Client Compiler.
///
/// The compilers alternate between building client references and the server actions
/// they import. Once the server has no new client entries, the client emits the
/// manifest that the server needs to finish.
///
/// The coordinator manages state transitions and synchronization between compilers
/// to maintain the correct build sequence for React Server Components.
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
        "Server compiler ID is not registered in the RSC coordinator. \
         Ensure RscServerPlugin starts compiling before RscClientPlugin."
      )
    })
  }

  async fn wait_for(&self, predicate: impl Fn(State) -> bool) -> Result<()> {
    loop {
      // Register before reading the state so a concurrent notification cannot be missed.
      let notified = self.state_notify.notified();
      let state = *self.state.lock().await.as_ref().map_err(Clone::clone)?;
      if predicate(state) {
        return Ok(());
      }
      notified.await;
    }
  }

  async fn transition(&self, expected: State, next: State, context: &'static str) -> Result<()> {
    let mut state = self.state.lock().await;
    let current = *state.as_ref().map_err(Clone::clone)?;
    if current != expected {
      return Err(rspack_error::error!(
        "Invalid state transition in {}: expected {:?}, got {:?}",
        context,
        expected,
        current
      ));
    }
    *state = Ok(next);
    self.state_notify.notify_waiters();
    Ok(())
  }

  async fn set_if_current(&self, current: State, next: State) -> Result<bool> {
    let mut state = self.state.lock().await;
    if *state.as_ref().map_err(Clone::clone)? == current {
      *state = Ok(next);
      self.state_notify.notify_waiters();
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub async fn idle(&self) -> Result<()> {
    self
      .transition(State::ClientManifestDone, State::Idle, "idle")
      .await
  }

  async fn wait_idle(&self) -> Result<()> {
    self.wait_for(|s| s == State::Idle).await
  }

  pub async fn start_server_entries_compilation(&self, compiler_id: CompilerId) -> Result<()> {
    loop {
      {
        let mut state = self.state.lock().await;
        if *state.as_ref().map_err(Clone::clone)? == State::Idle {
          *self.server_compiler_id.lock().await = Some(compiler_id);
          *state = Ok(State::ServerEntriesCompiling);
          self.state_notify.notify_waiters();
          return Ok(());
        }
      }
      self.wait_idle().await?;
    }
  }

  pub async fn complete_server_entries_compilation(&self) -> Result<()> {
    self
      .transition(
        State::ServerEntriesCompiling,
        State::ServerEntriesDone,
        "complete_server_entries_compilation",
      )
      .await
  }

  async fn wait_server_entries_compiled(&self) -> Result<()> {
    self.wait_for(|s| s == State::ServerEntriesDone).await
  }

  pub async fn start_client_entries_compilation(&self) -> Result<()> {
    loop {
      if self
        .set_if_current(State::ServerEntriesDone, State::ClientEntriesCompiling)
        .await?
      {
        return Ok(());
      }
      self.wait_server_entries_compiled().await?;
    }
  }

  pub async fn complete_client_entries_compilation(&self) -> Result<()> {
    self
      .transition(
        State::ClientEntriesCompiling,
        State::ClientEntriesDone,
        "complete_client_entries_compilation",
      )
      .await
  }

  async fn wait_client_entries_compiled(&self) -> Result<()> {
    self.wait_for(|s| s == State::ClientEntriesDone).await
  }

  pub async fn start_server_actions_compilation(&self) -> Result<()> {
    loop {
      if self
        .set_if_current(State::ClientEntriesDone, State::ServerActionsCompiling)
        .await?
      {
        return Ok(());
      }

      {
        let state = *self.state.lock().await.as_ref().map_err(Clone::clone)?;
        match state {
          State::ServerEntriesDone | State::ClientEntriesCompiling | State::ClientEntriesDone => {
            // fallthrough to wait below
          }
          _ => {
            return Err(rspack_error::error!(
              "Invalid state transition in start_server_actions_compilation: expected {:?}/{:?}/{:?}, got {:?}",
              State::ServerEntriesDone,
              State::ClientEntriesCompiling,
              State::ClientEntriesDone,
              state
            ));
          }
        }
      }

      self.wait_client_entries_compiled().await?;
    }
  }

  pub async fn complete_server_actions_compilation(&self) -> Result<()> {
    self
      .transition(
        State::ServerActionsCompiling,
        State::ServerActionsDone,
        "complete_server_actions_compilation",
      )
      .await
  }

  pub async fn publish_updated_client_entries(&self) -> Result<()> {
    self
      .transition(
        State::ServerActionsCompiling,
        State::ServerEntriesDone,
        "publish_updated_client_entries",
      )
      .await
  }

  pub async fn start_updated_client_entries_compilation(&self) -> Result<bool> {
    self
      .wait_for(|state| matches!(state, State::ServerEntriesDone | State::ServerActionsDone))
      .await?;
    self
      .set_if_current(State::ServerEntriesDone, State::ClientEntriesCompiling)
      .await
  }

  pub async fn complete_client_manifest(&self) -> Result<()> {
    self
      .transition(
        State::ServerActionsDone,
        State::ClientManifestDone,
        "complete_client_manifest",
      )
      .await
  }

  pub async fn wait_for_client_manifest(&self) -> Result<()> {
    self.wait_for(|s| s == State::ClientManifestDone).await
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
