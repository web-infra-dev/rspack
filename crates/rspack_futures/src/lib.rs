pub mod scope;
use std::{
  cell::UnsafeCell,
  future::Future,
  iter::ExactSizeIterator,
  mem::{ManuallyDrop, MaybeUninit},
};

pub use scope::scope;

/// par_iter then collect into vec
///
/// This is a wrapper around `scope`, but allows non-'static return values.
///
/// # Safety
///
/// Its safety assumptions are the same as `scope`.
///
/// # Example
///
/// ```rust
/// # #[tokio::test]
/// # async fn foo() {
/// async fn handle(s: &str) -> (usize, &str) {
///   (s.len(), s)
/// }
///
/// let data: Vec<String> = vec!["hello".into(), "world".into(), "!".into()];
/// let tasks = data.iter().map(|s| handle(s));
///
/// let list = unsafe { par_iter_then_collect(tasks) };
///
/// assert_eq!(list, vec![(5, "hello"), (5, "world"), (1, "!")]);
/// # }
/// ```
pub async unsafe fn par_iter_then_collect<I, F, O>(iter: I) -> Vec<O>
where
  I: IntoIterator<Item = F>,
  I::IntoIter: ExactSizeIterator,
  F: Future<Output = O> + Send + Sync,
  O: Send + Sync,
{
  // TODO use `std::cell::SyncUnsafeCell`
  //
  // see https://github.com/rust-lang/rust/issues/95439
  #[repr(transparent)]
  struct SyncUnsafeCell<T: ?Sized>(UnsafeCell<T>);

  // # Safety
  //
  // We guarantee that `SyncUnsafeCell` will never be accesse parallel
  unsafe impl<T: ?Sized + Sync> Sync for SyncUnsafeCell<T> {}

  let iter = iter.into_iter();
  let output: Box<[MaybeUninit<SyncUnsafeCell<O>>]> = Box::new_uninit_slice(iter.len());

  scope(|token| {
    for (i, f) in iter.enumerate() {
      // # Safety
      //
      // The caller needs to ensure that the task is legally consumed
      let spawner = unsafe { token.used((f, &output)) };

      spawner.spawn(move |(f, output)| async move {
        let result = f.await;

        // # Safety
        //
        // This assumes that the length provided by the `ExactSizeIterator` is correct,
        // and will abort if it is not.
        let slot = &output[i];

        // # Safety
        //
        // because transparent repr
        let slot = slot.as_ptr().cast::<UnsafeCell<O>>();

        // # Safety
        //
        // This slot is exclusive to the thread and
        // will not be accessed by other threads at the same time.
        unsafe {
          UnsafeCell::raw_get(slot).write(result);
        }
      });
    }
  })
  .await;

  // # Safety
  //
  // `scope` ensures that all slots are initialized after completion
  let output = unsafe { output.assume_init() };
  let output = Vec::from(output);

  unsafe {
    // TODO use into_raw_parts
    //
    // see https://github.com/rust-lang/rust/issues/65816
    let mut output = ManuallyDrop::new(output);
    let ptr = output.as_mut_ptr();
    let len = output.len();
    let cap = output.capacity();

    // # Safety
    //
    // because transparent repr
    let ptr = ptr.cast::<O>();
    Vec::from_raw_parts(ptr, len, cap)
  }
}
