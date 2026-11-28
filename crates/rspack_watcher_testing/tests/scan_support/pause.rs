use std::{
  path::Path,
  sync::{Arc, Mutex},
};

use rspack_watcher::FsWatcherIgnored;
use tokio::sync::oneshot;

pub(crate) fn pause_at(
  path: &Path,
) -> (FsWatcherIgnored, oneshot::Receiver<()>, oneshot::Sender<()>) {
  let (entered, waiting) = oneshot::channel();
  let (release, resume) = oneshot::channel();
  let gate = Mutex::new(Some((entered, resume)));
  let target = path.to_owned();
  let ignored = FsWatcherIgnored::Function(Arc::new(move |path| {
    let gate = if Path::new(&path) == target {
      gate.lock().expect("lock the scan gate").take()
    } else {
      None
    };
    Box::pin(async move {
      if let Some((entered, resume)) = gate {
        entered.send(()).expect("signal the blocked scan");
        let _ = resume.await;
      }
      false
    })
  }));
  (ignored, waiting, release)
}
