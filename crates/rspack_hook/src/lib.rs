#![feature(slice_group_by)]

mod async_parallel;
mod async_series;
mod async_series_bail;
mod interceptor;
mod sync_bail;
mod util;

pub use async_parallel::{AsyncParallel, AsyncParallelHook};
pub use async_series::{AsyncSeries, AsyncSeries2, AsyncSeries2Hook, AsyncSeriesHook};
pub use async_series_bail::{AsyncSeriesBail, AsyncSeriesBailHook};
pub use interceptor::{Hook, Interceptor};
pub use sync_bail::{SyncBail, SyncBailHook, SyncBailHookMap};

// pub trait Plugin<HookContainer> {
//   fn apply(&self, hook_container: &mut HookContainer);
// }
