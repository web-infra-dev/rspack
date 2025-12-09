use std::sync::Arc;

use futures::future::BoxFuture;
use rspack_core::CompilerId;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use tokio::sync::{Mutex, MutexGuard, Notify, OwnedMutexGuard};

use crate::{
  loaders::client_entry_loader::ClientEntry, plugin_state::PLUGIN_STATE_BY_COMPILER_ID,
  utils::GetServerCompilerId,
};

// Server 启动（只有 js 侧拿到最准确的状态）
// Client 启动（只有 js 侧拿到最准确的状态）

#[derive(Debug)]
enum State {
  Idle,
  ServerEntriesCompiling,
  ServerEntriesCompiled,
  ClientEntriesCompiling,
  ClientEntriesCompiled,
  ServerActionsCompiling,
  ServerActionsCompiled,
}

type InvalidateFn = Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct Coordinator {
  state: Mutex<State>,
  state_notify: Notify,

  invalidate_server_compiler: InvalidateFn,
  invalidate_client_compiler: InvalidateFn,
  get_server_compiler_id: GetServerCompilerId,
}

impl Coordinator {
  pub fn new(
    invalidate_server_compiler: InvalidateFn,
    invalidate_client_compiler: InvalidateFn,
    get_server_compiler_id: GetServerCompilerId,
  ) -> Self {
    Self {
      state: Mutex::new(State::Idle),
      state_notify: Default::default(),
      invalidate_server_compiler,
      invalidate_client_compiler,
      get_server_compiler_id,
    }
  }

  pub async fn get_server_compiler_id(&self) -> Result<CompilerId> {
    (self.get_server_compiler_id)().await
  }

  pub async fn idle(&self) -> Result<()> {
    let mut state = self.state.lock().await;
    if !matches!(*state, State::ServerActionsCompiled) {
      return Err(rspack_error::error!(
        "Invalid state transition: expected ServerActionsCompiled before idle, got {:?}",
        *state
      ));
    }
    *state = State::Idle;
    self.state_notify.notify_waiters();
    Ok(())
  }

  pub async fn wait_idle(&self) -> Result<()> {
    loop {
      {
        let state = self.state.lock().await;
        if matches!(*state, State::Idle) {
          return Ok(());
        }
      }
      self.state_notify.notified().await;
    }
  }

  pub async fn start_server_entries_compilation(&self) -> Result<()> {
    println!("start_server_entries_compilation");
    loop {
      {
        let mut state = self.state.lock().await;
        match *state {
          State::Idle => {
            *state = State::ServerEntriesCompiling;
            self.state_notify.notify_waiters();
            (self.invalidate_client_compiler)().await?;
            return Ok(());
          }
          _ => {}
        }
      }
      self.wait_idle().await?;
    }
  }

  pub async fn complete_server_entries_compilation(&self) -> Result<()> {
    println!("complete_server_entries_compilation");
    let mut state = self.state.lock().await;
    if !matches!(*state, State::ServerEntriesCompiling) {
      return Err(rspack_error::error!(
        "Invalid state transition: expected ServerEntriesCompiling before completing server entries, got {:?}",
        *state
      ));
    }
    *state = State::ServerEntriesCompiled;
    println!("self.state_notify.notify_waiters()");
    self.state_notify.notify_waiters();
    Ok(())
  }

  pub async fn wait_server_entries_compiled(&self) -> Result<()> {
    loop {
      {
        let state = self.state.lock().await;
        println!("wait_server_entries_compiled {:#?}", *state);
        if matches!(*state, State::ServerEntriesCompiled) {
          return Ok(());
        }
      }
      self.state_notify.notified().await;
    }
  }

  pub async fn start_client_entries_compilation(&self) -> Result<()> {
    println!("start_client_entries_compilation");
    loop {
      {
        let mut state = self.state.lock().await;
        match *state {
          State::ServerEntriesCompiling => {}
          State::ServerEntriesCompiled => {
            *state = State::ClientEntriesCompiling;
            self.state_notify.notify_waiters();
            return Ok(());
          }
          State::ClientEntriesCompiling => {
            panic!()
          }
          State::ClientEntriesCompiled => {
            panic!()
          }
          State::Idle | State::ServerActionsCompiling | State::ServerActionsCompiled => {
            println!("self.invalidate_server_compiler");
            (self.invalidate_server_compiler)().await?;
            println!("self.invalidate_server_compiler end");
          }
        }
      }
      self.wait_server_entries_compiled().await?;
    }
  }

  pub async fn complete_client_entries_compilation(&self) -> Result<()> {
    println!("complete_client_entries_compilation");
    let mut state = self.state.lock().await;
    if !matches!(*state, State::ClientEntriesCompiling) {
      return Err(rspack_error::error!(
        "Invalid state transition: expected ClientEntriesCompiling before completing server entries, got {:?}",
        *state
      ));
    }
    *state = State::ClientEntriesCompiled;
    self.state_notify.notify_waiters();
    Ok(())
  }

  pub async fn wait_client_entries_compiled(&self) -> Result<()> {
    loop {
      {
        let state = self.state.lock().await;
        if matches!(*state, State::ClientEntriesCompiled) {
          return Ok(());
        }
      }
      self.state_notify.notified().await;
    }
  }

  pub async fn start_server_actions_compilation(&self) -> Result<()> {
    println!("start_server_actions_compilation");
    loop {
      {
        let mut state = self.state.lock().await;
        match *state {
          State::ClientEntriesCompiled => {
            *state = State::ServerActionsCompiling;
            self.state_notify.notify_waiters();
            return Ok(());
          }
          State::ServerEntriesCompiled | State::ClientEntriesCompiling => {
            drop(state);
            self.wait_client_entries_compiled().await?;
          }
          _ => {
            return Err(rspack_error::error!(
              "Invalid state transition: expected ServerEntriesCompiled/ClientEntriesCompiled/ClientEntriesCompiling before starting server actions, got {:?}",
              *state
            ));
          }
        }
      }
    }
  }

  pub async fn complete_server_actions_compilation(&self) -> Result<()> {
    println!("complete_server_actions_compilation");
    let mut state = self.state.lock().await;
    if !matches!(*state, State::ServerActionsCompiling) {
      return Err(rspack_error::error!(
        "Invalid state transition: expected ServerActionsCompiling before completing server entries, got {:?}",
        *state
      ));
    }
    *state = State::ServerActionsCompiled;
    self.state_notify.notify_waiters();
    Ok(())
  }
}
