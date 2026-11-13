use rspack_error::{Result, error};
use tokio::sync::oneshot;

pub struct Task<Payload, Output> {
  state: Option<(Payload, oneshot::Sender<Result<Output>>)>,
  cancellation_message: &'static str,
}

impl<Payload, Output> Task<Payload, Output> {
  pub fn new(
    payload: Payload,
    sender: oneshot::Sender<Result<Output>>,
    cancellation_message: &'static str,
  ) -> Self {
    Self {
      state: Some((payload, sender)),
      cancellation_message,
    }
  }

  pub fn payload(&self) -> &Payload {
    &self.state.as_ref().expect("unfinished task").0
  }

  pub fn payload_mut(&mut self) -> &mut Payload {
    &mut self.state.as_mut().expect("unfinished task").0
  }

  pub fn finish(mut self, result: Result<Output>) {
    self.send(result);
  }

  fn send(&mut self, result: Result<Output>) {
    if let Some((payload, sender)) = self.state.take() {
      // Release payload-owned references before waking the receiver.
      drop(payload);
      let _ = sender.send(result);
    }
  }
}

impl<Payload, Output> Drop for Task<Payload, Output> {
  fn drop(&mut self) {
    if self.state.is_some() {
      self.send(Err(error!("{}", self.cancellation_message)));
    }
  }
}
