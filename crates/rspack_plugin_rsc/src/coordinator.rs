use rspack_core::CompilerId;
use rspack_error::Result;
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
  Failed,
}

/// Coordinates the compilation sequence between Server Compiler and Client Compiler.
///
/// Ensures the following compilation order:
/// 1. Server Entries compilation（in Server Compiler）
/// 2. Client Entries compilation（in Client Compiler）
/// 3. Server Actions compilation（in Server Compiler）
///
/// The coordinator manages state transitions and synchronization between compilers
/// to maintain the correct build sequence for React Server Components.
pub struct Coordinator {
  state: Mutex<State>,
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
      state: Mutex::new(State::Idle),
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

  async fn wait_for(&self, mut predicate: impl FnMut(State) -> bool) -> Result<()> {
    loop {
      {
        let state = *self.state.lock().await;
        if predicate(state) {
          return Ok(());
        }
        if state == State::Failed {
          return Ok(());
        }
      }
      self.state_notify.notified().await;
    }
  }

  async fn transition(&self, expected: State, next: State, context: &'static str) -> Result<()> {
    let mut state = self.state.lock().await;
    if *state == State::Failed {
      return Ok(());
    }

    if *state != expected {
      return Err(rspack_error::error!(
        "Invalid state transition in {}: expected {:?}, got {:?}",
        context,
        expected,
        *state
      ));
    }
    *state = next;
    self.state_notify.notify_waiters();
    Ok(())
  }

  async fn set_if_current(&self, current: State, next: State) -> bool {
    let mut state = self.state.lock().await;
    if *state == current {
      *state = next;
      self.state_notify.notify_waiters();
      true
    } else {
      false
    }
  }

  pub async fn idle(&self) -> Result<()> {
    self
      .transition(State::ServerActionsDone, State::Idle, "idle")
      .await
  }

  async fn wait_idle(&self) -> Result<()> {
    self.wait_for(|s| s == State::Idle).await
  }

  pub async fn start_server_entries_compilation(&self, compiler_id: CompilerId) -> Result<()> {
    loop {
      {
        let mut state = self.state.lock().await;
        if *state == State::Idle {
          *self.server_compiler_id.lock().await = Some(compiler_id);
          *state = State::ServerEntriesCompiling;
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
        .await
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
        .await
      {
        return Ok(());
      }

      {
        let state = *self.state.lock().await;
        match state {
          State::ServerEntriesDone | State::ClientEntriesCompiling => {
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

  pub async fn failed(&self) -> Result<()> {
    let mut state = self.state.lock().await;
    *state = State::Failed;
    self.state_notify.notify_waiters();
    Ok(())
  }
}
