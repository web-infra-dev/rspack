use std::{mem, thread};

/// Fast set `src` into the referenced `dest`, and drop the old value in other thread
///
/// This method is used when the dropping time is long
pub fn fast_set<T>(dest: &mut T, src: T)
where
  T: Send + 'static,
{
  let old = mem::replace(dest, src);
  thread::Builder::new()
    .name("fast_set".to_string())
    .spawn(move || {
      mem::drop(old);
    })
    .expect("spawn fast_set thread failed");
}

pub fn fast_drop<T>(src: T)
where
  T: Send + 'static,
{
  thread::Builder::new()
    .name("fast_drop".to_string())
    .spawn(move || {
      mem::drop(src);
    })
    .expect("spawn fast_drop thread failed");
}
