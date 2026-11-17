//! Process-wide native queues. JS environments and loader context types belong
//! to the callers; this crate has no dependency on core, loader runner or N-API.
mod dispatcher;
mod task;
mod timing;

pub use dispatcher::{JsTaskDispatcher, JsTaskReceiver, MainTaskRegistration, js_task_dispatcher};
pub use task::{JsTask, LoaderTask, LoaderTaskResult};
pub use timing::{LoaderTaskTiming, TimingPoint};
