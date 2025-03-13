use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use tokio::task::{JoinError, JoinHandle};

/// Scope Token
pub struct Token<'scope, 'spawner, O> {
  list: &'spawner RefCell<Vec<JoinHandle<O>>>,
  _phantom: PhantomData<&'scope mut &'scope ()>,
}

/// Scope Spawner
pub struct Spawner<'scope, 'spawner, T, O> {
  list: &'spawner RefCell<Vec<JoinHandle<O>>>,
  used: T,
  _phantom: PhantomData<&'scope mut &'scope ()>,
}

/// Async scope helper
///
/// This function helps you write unsafe
/// asynchronous structured concurrent code more easily.
/// but it is **still unsafe**, so need to be careful when using it.
///
/// To use it safely,
/// the user needs to ensure that the task is done within used reference lifetime.
/// Due to `std::mem::forget`, the Rust currently cannot guarantee it.
///
/// From a practical point of view, the following points need to be note
///
/// * `.await` as early as possible
/// * Don't put task into container unless you know what you are doing
/// * Don't call `std::mem::forget`
///
/// # Example
///
/// ```rust
/// # #[tokio::test]
/// # async fn foo() {
/// let list: Vec<u32> = vec![1, 2, 3, 4];
///
/// rspack_futures::scope(|token| {
///   for i in 0..list.len() {
///     let s = unsafe { token.used(&list) };
///
///     s.spawn(move |list| async move {
///       &list[i];
///     });
///   }
/// })
/// .await;
/// # }
/// ```
///
/// This doesn't compile
///
/// ```rust,compile_fail
/// # async fn foo() {
/// rspack_futures::scope(|token| {
///   let list: Vec<u32> = vec![1, 2, 3, 4];
///
///   for i in 0..list.len() {
///     let s = unsafe { token.used(&list) };
///
///     s.spawn(move |list| async move {
///       &list[i];
///     });
///   }
/// })
/// .await;
/// # }
/// ```
pub async fn scope<'scope, F, O>(f: F) -> Vec<Result<O, JoinError>>
where
  for<'spawner> F: FnOnce(Token<'scope, 'spawner, O>),
  O: Send + 'static,
{
  struct ScopeGuard(());

  impl ScopeGuard {
    fn forget(self) {
      std::mem::forget(self);
    }
  }

  impl Drop for ScopeGuard {
    fn drop(&mut self) {
      // avoid unsound caused by poll interruption
      std::process::abort();
    }
  }

  let guard = ScopeGuard(());
  let list = RefCell::new(Vec::new());

  let token = Token {
    list: &list,
    _phantom: PhantomData,
  };

  f(token);

  let list = RefCell::into_inner(list);
  let mut output = Vec::with_capacity(list.len());

  for j in list {
    output.push(j.await);
  }

  guard.forget();
  output
}

impl<'scope, 'spawner, O> Token<'scope, 'spawner, O> {
  /// Use references
  ///
  /// Specify the reference to use when spawning the task.
  ///
  /// # Safety
  ///
  /// This is not sound.
  ///
  /// the user must ensure that `scope` task is legally consumed,
  /// and assume that the runtime handles the task correctly.
  pub unsafe fn used<T: 'scope>(&self, used: T) -> Spawner<'scope, 'spawner, T, O> {
    Spawner {
      list: self.list,
      used,
      _phantom: PhantomData,
    }
  }
}

impl<'scope, T, O> Spawner<'scope, '_, T, O> {
  /// Spawn task from used reference
  pub fn spawn<F, Fut>(self, f: F)
  where
    // TODO Use AsyncFnOnce
    F: FnOnce(T) -> Fut + 'static,
    Fut: Future<Output = O> + Send + 'scope,
    T: Send + Sync + 'scope,
    O: Send + 'static,
  {
    let fut = f(self.used);
    let fut: Pin<Box<dyn Future<Output = O> + Send + 'scope>> = Box::pin(fut);

    // # Safety
    //
    // The safety guarantee here comes from `Token::used`.
    // The user needs to ensure that the task will done within used reference lifetime.
    let fut: Pin<Box<dyn Future<Output = O> + Send + 'static>> =
      unsafe { std::mem::transmute(fut) };

    let j = tokio::spawn(fut);
    self.list.borrow_mut().push(j);
  }
}
