// borrow the ideas from turbo_tasks https://github.com/vercel/next.js/blob/678ef8b5650871a730ca14c480c762ca53716575/turbopack/crates/turbo-tasks/src/manager.rs#L1
// which creates a implicit compiler context to support isolated parallel compiler state
use std::{
  any::Any,
  cell::RefCell,
  collections::VecDeque,
  ffi::c_void,
  fmt,
  future::Future,
  panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
  pin::Pin,
  sync::{
    Arc, Condvar, LazyLock, Mutex, MutexGuard,
    atomic::{AtomicBool, AtomicPtr, AtomicU8, AtomicU32, Ordering},
  },
  task::{Context, Poll, Wake, Waker},
  thread,
  time::Duration,
};

type ReadyTask = Arc<dyn Runnable>;
type BlockingTask = Box<dyn FnOnce() + Send + 'static>;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(Runtime::new);
static BLOCKING_POOL: LazyLock<BlockingPool> = LazyLock::new(BlockingPool::new);

thread_local! {
  static CURRENT_CONTEXT: RefCell<Option<Arc<CompilerContext>>> = const { RefCell::new(None) };
}

// don't overuse this and put everything here, it's mostly used for store isolated id generator
#[derive(Debug)]
pub struct CompilerContext {
  dependenc_id_generator: AtomicU32,
  exports_info_artifact_ptr: AtomicPtr<c_void>,
}

pub struct CompilerContextLocal;

pub static CURRENT_COMPILER_CONTEXT: CompilerContextLocal = CompilerContextLocal;

#[derive(Debug)]
pub struct TryGetCompilerContextError;

pub struct CompilerContextFuture<F> {
  compiler_context: Arc<CompilerContext>,
  future: F,
}

impl CompilerContextLocal {
  pub fn get(&self) -> Arc<CompilerContext> {
    self.try_get().expect("CURRENT_COMPILER_CONTEXT is not set")
  }

  pub fn try_get(&self) -> Result<Arc<CompilerContext>, TryGetCompilerContextError> {
    CURRENT_CONTEXT.with(|current| current.borrow().clone().ok_or(TryGetCompilerContextError))
  }

  pub fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&Arc<CompilerContext>) -> R,
  {
    self
      .try_with(f)
      .expect("CURRENT_COMPILER_CONTEXT is not set")
  }

  pub fn try_with<F, R>(&self, f: F) -> Result<R, TryGetCompilerContextError>
  where
    F: FnOnce(&Arc<CompilerContext>) -> R,
  {
    CURRENT_CONTEXT.with(|current| {
      current
        .borrow()
        .as_ref()
        .map(f)
        .ok_or(TryGetCompilerContextError)
    })
  }

  pub fn scope<F>(
    &self,
    compiler_context: Arc<CompilerContext>,
    future: F,
  ) -> CompilerContextFuture<F>
  where
    F: Future,
  {
    CompilerContextFuture {
      compiler_context,
      future,
    }
  }

  pub fn sync_scope<F, R>(&self, compiler_context: Arc<CompilerContext>, f: F) -> R
  where
    F: FnOnce() -> R,
  {
    struct RestoreContext(Option<Arc<CompilerContext>>);

    impl Drop for RestoreContext {
      fn drop(&mut self) {
        let previous = self.0.take();
        CURRENT_CONTEXT.with(|current| {
          *current.borrow_mut() = previous;
        });
      }
    }

    let previous = CURRENT_CONTEXT.with(|current| current.replace(Some(compiler_context)));
    let _restore_context = RestoreContext(previous);
    f()
  }
}

impl<F> Future for CompilerContextFuture<F>
where
  F: Future,
{
  type Output = F::Output;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = unsafe { self.get_unchecked_mut() };
    CURRENT_COMPILER_CONTEXT.sync_scope(this.compiler_context.clone(), || {
      unsafe { Pin::new_unchecked(&mut this.future) }.poll(cx)
    })
  }
}

#[allow(clippy::new_without_default)]
impl CompilerContext {
  pub fn new() -> Self {
    Self {
      dependenc_id_generator: AtomicU32::new(0),
      exports_info_artifact_ptr: AtomicPtr::new(std::ptr::null_mut()),
    }
  }
  pub fn fetch_new_dependency_id(&self) -> u32 {
    self
      .dependenc_id_generator
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  }
  pub fn dependency_id(&self) -> u32 {
    self
      .dependenc_id_generator
      .load(std::sync::atomic::Ordering::SeqCst)
  }
  pub fn set_dependency_id(&self, id: u32) {
    self
      .dependenc_id_generator
      .store(id, std::sync::atomic::Ordering::SeqCst);
  }

  pub fn exports_info_artifact_ptr(&self) -> Option<*mut c_void> {
    let ptr = self
      .exports_info_artifact_ptr
      .load(std::sync::atomic::Ordering::SeqCst);
    (!ptr.is_null()).then_some(ptr)
  }

  pub fn set_exports_info_artifact_ptr(&self, ptr: Option<*mut c_void>) {
    self.exports_info_artifact_ptr.store(
      ptr.unwrap_or(std::ptr::null_mut()),
      std::sync::atomic::Ordering::SeqCst,
    );
  }
}

pub fn fetch_new_dependency_id() -> u32 {
  CURRENT_COMPILER_CONTEXT.get().fetch_new_dependency_id()
}
pub fn get_current_dependency_id() -> u32 {
  CURRENT_COMPILER_CONTEXT.get().dependency_id()
}
pub fn set_current_dependency_id(id: u32) {
  CURRENT_COMPILER_CONTEXT.get().set_dependency_id(id);
}

pub fn within_compiler_context<F>(
  compiler_context: Arc<CompilerContext>,
  f: F,
) -> CompilerContextFuture<F>
where
  F: Future,
{
  CURRENT_COMPILER_CONTEXT.scope(compiler_context, f)
}
pub fn within_compiler_context_sync<F, R>(compiler_context: Arc<CompilerContext>, f: F) -> R
where
  F: FnOnce() -> R,
{
  CURRENT_COMPILER_CONTEXT.sync_scope(compiler_context, f)
}

// this is only used for testing rust builder api, we need to find better api in the future
/// For test use only.
pub fn within_compiler_context_for_testing_sync<F, R>(f: F) -> R
where
  F: FnOnce() -> R,
{
  CURRENT_COMPILER_CONTEXT.sync_scope(Arc::new(CompilerContext::new()), f)
}
/// For test use only.
pub fn within_compiler_context_for_testing<F>(f: F) -> CompilerContextFuture<F>
where
  F: Future,
{
  CURRENT_COMPILER_CONTEXT.scope(Arc::new(CompilerContext::new()), f)
}

pub fn spawn_in_compiler_context<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let compiler_context = CURRENT_COMPILER_CONTEXT.get();
  spawn(CURRENT_COMPILER_CONTEXT.scope(compiler_context, future))
}

/// Like [`spawn_in_compiler_context`], but falls back to a plain spawn when
/// there is no active compiler context (e.g. in unit tests or utility code
/// that is not driven by a compiler).
pub fn spawn_in_context<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  match CURRENT_COMPILER_CONTEXT.try_get() {
    Ok(compiler_context) => spawn(CURRENT_COMPILER_CONTEXT.scope(compiler_context, future)),
    Err(_) => spawn(future),
  }
}

pub fn ensure_runtime() {
  let _ = LazyLock::force(&RUNTIME);
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let task = Task::new(future);
  let join_task: Arc<dyn Joinable<F::Output>> = task.clone();
  task.schedule();
  JoinHandle { task: join_task }
}

pub fn spawn_blocking<F, T>(f: F) -> JoinHandle<T>
where
  F: FnOnce() -> T + Send + 'static,
  T: Send + 'static,
{
  let shared = Arc::new(BlockingShared {
    result: Mutex::new(None),
    waker: Mutex::new(None),
  });
  let shared_for_task = Arc::clone(&shared);
  blocking_pool().schedule(Box::new(move || {
    let result = catch_unwind(AssertUnwindSafe(f));
    *lock(&shared_for_task.result) = Some(result);
    if let Some(waker) = lock(&shared_for_task.waker).take() {
      waker.wake();
    }
  }));

  spawn(BlockingFuture { shared })
}

pub fn block_on<F: Future>(future: F) -> F::Output {
  ensure_runtime();

  let current_thread = thread::current();
  let waker = Waker::from(Arc::new(ThreadWaker { current_thread }));
  let mut cx = Context::from_waker(&waker);
  let mut future = std::pin::pin!(future);

  loop {
    if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
      return value;
    }

    if runtime().try_run_one() {
      continue;
    }

    thread::park();
  }
}

pub fn sleep(duration: Duration) -> Sleep {
  let shared = Arc::new(SleepShared {
    done: AtomicBool::new(false),
    waker: Mutex::new(None),
  });
  let shared_for_thread = Arc::clone(&shared);
  thread::Builder::new()
    .name("rspack-async-sleep".to_string())
    .spawn(move || {
      thread::sleep(duration);
      shared_for_thread.done.store(true, Ordering::Release);
      if let Some(waker) = lock(&shared_for_thread.waker).take() {
        waker.wake();
      }
    })
    .expect("Create rspack async sleep thread failed");
  Sleep { shared }
}

pub struct Sleep {
  shared: Arc<SleepShared>,
}

struct SleepShared {
  done: AtomicBool,
  waker: Mutex<Option<Waker>>,
}

impl Future for Sleep {
  type Output = ();

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if self.shared.done.load(Ordering::Acquire) {
      Poll::Ready(())
    } else {
      *lock(&self.shared.waker) = Some(cx.waker().clone());
      if self.shared.done.load(Ordering::Acquire) {
        Poll::Ready(())
      } else {
        Poll::Pending
      }
    }
  }
}

struct BlockingFuture<T> {
  shared: Arc<BlockingShared<T>>,
}

struct BlockingShared<T> {
  result: Mutex<Option<std::thread::Result<T>>>,
  waker: Mutex<Option<Waker>>,
}

impl<T> Future for BlockingFuture<T> {
  type Output = T;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if let Some(result) = lock(&self.shared.result).take() {
      return match result {
        Ok(value) => Poll::Ready(value),
        Err(payload) => resume_unwind(payload),
      };
    }

    *lock(&self.shared.waker) = Some(cx.waker().clone());
    if let Some(result) = lock(&self.shared.result).take() {
      match result {
        Ok(value) => Poll::Ready(value),
        Err(payload) => resume_unwind(payload),
      }
    } else {
      Poll::Pending
    }
  }
}

pub struct JoinHandle<T> {
  task: Arc<dyn Joinable<T>>,
}

impl<T> JoinHandle<T> {
  pub fn abort(&self) {
    self.task.abort();
  }
}

impl<T> Future for JoinHandle<T> {
  type Output = std::result::Result<T, JoinError>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.task.poll_join(cx)
  }
}

pub struct JoinError {
  kind: JoinErrorKind,
}

enum JoinErrorKind {
  Panic(Box<dyn Any + Send + 'static>),
  Cancelled,
}

impl JoinError {
  fn panic(payload: Box<dyn Any + Send + 'static>) -> Self {
    Self {
      kind: JoinErrorKind::Panic(payload),
    }
  }

  fn cancelled() -> Self {
    Self {
      kind: JoinErrorKind::Cancelled,
    }
  }

  pub fn is_cancelled(&self) -> bool {
    matches!(self.kind, JoinErrorKind::Cancelled)
  }

  pub fn is_panic(&self) -> bool {
    matches!(self.kind, JoinErrorKind::Panic(_))
  }

  pub fn into_panic(self) -> Box<dyn Any + Send + 'static> {
    match self.kind {
      JoinErrorKind::Panic(payload) => payload,
      JoinErrorKind::Cancelled => panic!("called into_panic on a cancelled task"),
    }
  }
}

impl fmt::Debug for JoinError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JoinError")
      .field("is_cancelled", &self.is_cancelled())
      .field("is_panic", &self.is_panic())
      .finish()
  }
}

impl fmt::Display for JoinError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.is_cancelled() {
      f.write_str("task was cancelled")
    } else {
      f.write_str("task panicked")
    }
  }
}

impl std::error::Error for JoinError {}

struct Runtime {
  inner: Arc<RuntimeInner>,
}

struct RuntimeInner {
  queue: Mutex<VecDeque<ReadyTask>>,
  available: Condvar,
}

struct BlockingPool {
  inner: Arc<BlockingPoolInner>,
}

struct BlockingPoolInner {
  queue: Mutex<VecDeque<BlockingTask>>,
  available: Condvar,
}

impl Runtime {
  fn new() -> Self {
    let inner = Arc::new(RuntimeInner {
      queue: Mutex::new(VecDeque::new()),
      available: Condvar::new(),
    });

    let threads = runtime_threads();
    for index in 0..threads {
      let inner = Arc::clone(&inner);
      thread::Builder::new()
        .name(format!("rspack-async-{index}"))
        .spawn(move || worker_loop(inner))
        .expect("Create rspack async runtime thread failed");
    }

    Self { inner }
  }

  fn schedule(&self, task: ReadyTask) {
    self.inner.schedule(task);
  }

  fn try_run_one(&self) -> bool {
    let task = self.inner.pop_task();
    if let Some(task) = task {
      task.run();
      true
    } else {
      false
    }
  }
}

impl RuntimeInner {
  fn schedule(&self, task: ReadyTask) {
    lock(&self.queue).push_back(task);
    self.available.notify_one();
  }

  fn pop_task(&self) -> Option<ReadyTask> {
    lock(&self.queue).pop_front()
  }

  fn wait_for_task(&self) -> ReadyTask {
    let mut queue = lock(&self.queue);
    loop {
      if let Some(task) = queue.pop_front() {
        return task;
      }
      queue = self
        .available
        .wait(queue)
        .unwrap_or_else(|e| e.into_inner());
    }
  }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
  mutex.lock().unwrap_or_else(|e| e.into_inner())
}

fn worker_loop(inner: Arc<RuntimeInner>) -> ! {
  loop {
    inner.wait_for_task().run();
  }
}

impl BlockingPool {
  fn new() -> Self {
    let inner = Arc::new(BlockingPoolInner {
      queue: Mutex::new(VecDeque::new()),
      available: Condvar::new(),
    });

    let threads = blocking_threads();
    for index in 0..threads {
      let inner = Arc::clone(&inner);
      thread::Builder::new()
        .name(format!("rspack-blocking-{index}"))
        .spawn(move || blocking_worker_loop(inner))
        .expect("Create rspack blocking worker thread failed");
    }

    Self { inner }
  }

  fn schedule(&self, task: BlockingTask) {
    self.inner.schedule(task);
  }
}

impl BlockingPoolInner {
  fn schedule(&self, task: BlockingTask) {
    lock(&self.queue).push_back(task);
    self.available.notify_one();
  }

  fn wait_for_task(&self) -> BlockingTask {
    let mut queue = lock(&self.queue);
    loop {
      if let Some(task) = queue.pop_front() {
        return task;
      }
      queue = self
        .available
        .wait(queue)
        .unwrap_or_else(|e| e.into_inner());
    }
  }
}

fn blocking_worker_loop(inner: Arc<BlockingPoolInner>) -> ! {
  loop {
    let task = inner.wait_for_task();
    task();
  }
}

fn runtime() -> &'static Runtime {
  &RUNTIME
}

fn blocking_pool() -> &'static BlockingPool {
  &BLOCKING_POOL
}

fn runtime_threads() -> usize {
  #[cfg(target_family = "wasm")]
  {
    1
  }

  #[cfg(all(not(target_family = "wasm"), any(codspeed, feature = "codspeed")))]
  {
    1
  }

  #[cfg(all(not(target_family = "wasm"), not(any(codspeed, feature = "codspeed"))))]
  {
    thread::available_parallelism().map_or(4, |threads| threads.get())
  }
}

fn blocking_threads() -> usize {
  #[cfg(target_family = "wasm")]
  {
    1
  }

  #[cfg(all(not(target_family = "wasm"), any(codspeed, feature = "codspeed")))]
  {
    1
  }

  #[cfg(all(not(target_family = "wasm"), not(any(codspeed, feature = "codspeed"))))]
  {
    thread::available_parallelism()
      .map_or(4, |threads| threads.get().saturating_mul(2))
      .clamp(4, 32)
  }
}

trait Runnable: Send + Sync {
  fn run(self: Arc<Self>);
}

trait Joinable<T>: Send + Sync {
  fn poll_join(&self, cx: &mut Context<'_>) -> Poll<std::result::Result<T, JoinError>>;
  fn abort(&self);
}

struct TaskState {
  state: AtomicU8,
}

impl TaskState {
  const IDLE: u8 = 0;
  const SCHEDULED: u8 = 1;
  const RUNNING: u8 = 2;
  const NOTIFIED_WHILE_RUNNING: u8 = 3;
  const COMPLETE: u8 = 4;

  fn wake(&self) -> bool {
    self
      .state
      .fetch_update(Ordering::Release, Ordering::Relaxed, |state| match state {
        Self::SCHEDULED | Self::NOTIFIED_WHILE_RUNNING | Self::COMPLETE => None,
        Self::RUNNING => Some(Self::NOTIFIED_WHILE_RUNNING),
        Self::IDLE => Some(Self::SCHEDULED),
        _ => unreachable!("invalid task state"),
      })
      .is_ok_and(|state| state == Self::IDLE)
  }

  fn start_running(&self) {
    assert_eq!(self.state.load(Ordering::Acquire), Self::SCHEDULED);
    self.state.store(Self::RUNNING, Ordering::Relaxed);
  }

  fn finish_running(&self) -> bool {
    self
      .state
      .fetch_update(Ordering::Release, Ordering::Relaxed, |state| match state {
        Self::RUNNING => Some(Self::IDLE),
        Self::NOTIFIED_WHILE_RUNNING => Some(Self::SCHEDULED),
        Self::COMPLETE => None,
        _ => panic!("finish_running called on invalid task state"),
      })
      .is_ok_and(|old_state| old_state == Self::NOTIFIED_WHILE_RUNNING)
  }

  fn complete(&self) {
    self.state.store(Self::COMPLETE, Ordering::Release);
  }
}

impl Default for TaskState {
  fn default() -> Self {
    Self {
      state: AtomicU8::new(Self::IDLE),
    }
  }
}

enum TaskData<F: Future> {
  Polling(Pin<Box<F>>, Waker),
  Complete(Option<std::result::Result<F::Output, JoinError>>),
  Joined,
}

struct Task<F: Future> {
  state: TaskState,
  data: Mutex<TaskData<F>>,
  join_waker: Mutex<Option<Waker>>,
}

impl<F> Task<F>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  fn new(future: F) -> Arc<Self> {
    let state = TaskState::default();
    let waker = Waker::noop().clone();

    let task = Arc::new(Self {
      state,
      data: Mutex::new(TaskData::Polling(Box::pin(future), waker)),
      join_waker: Mutex::new(None),
    });

    let waker = Waker::from(Arc::clone(&task));
    {
      let mut data = lock(&task.data);
      let TaskData::Polling(_, stored_waker) = &mut *data else {
        unreachable!("new task should be pending");
      };
      *stored_waker = waker;
    }

    task
  }

  fn schedule(self: Arc<Self>) {
    if self.state.wake() {
      runtime().schedule(self);
    }
  }

  fn complete(&self, result: std::result::Result<F::Output, JoinError>) {
    let mut data = lock(&self.data);
    if !matches!(*data, TaskData::Polling(..)) {
      return;
    }
    *data = TaskData::Complete(Some(result));
    drop(data);
    self.state.complete();
    self.wake_joiner();
  }

  fn wake_joiner(&self) {
    if let Some(waker) = lock(&self.join_waker).take() {
      waker.wake();
    }
  }
}

impl<F> Wake for Task<F>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  fn wake(self: Arc<Self>) {
    self.schedule();
  }

  fn wake_by_ref(self: &Arc<Self>) {
    self.clone().wake();
  }
}

impl<F> Runnable for Task<F>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  fn run(self: Arc<Self>) {
    let poll_result = {
      let mut data = lock(&self.data);
      let TaskData::Polling(future, waker) = &mut *data else {
        return;
      };

      self.state.start_running();

      let mut cx = Context::from_waker(waker);
      catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(&mut cx)))
    };

    match poll_result {
      Ok(Poll::Pending) => {
        if self.state.finish_running() {
          runtime().schedule(self);
        }
      }
      Ok(Poll::Ready(value)) => {
        self.complete(Ok(value));
      }
      Err(payload) => {
        self.complete(Err(JoinError::panic(payload)));
      }
    }
  }
}

impl<F> Joinable<F::Output> for Task<F>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  fn poll_join(&self, cx: &mut Context<'_>) -> Poll<std::result::Result<F::Output, JoinError>> {
    *lock(&self.join_waker) = Some(cx.waker().clone());

    let mut data = lock(&self.data);
    let TaskData::Complete(result) = &mut *data else {
      return Poll::Pending;
    };

    let result = result.take().expect("join handle polled after completion");
    *data = TaskData::Joined;
    *lock(&self.join_waker) = None;
    Poll::Ready(result)
  }

  fn abort(&self) {
    let mut data = lock(&self.data);
    if matches!(*data, TaskData::Polling(..)) {
      *data = TaskData::Complete(Some(Err(JoinError::cancelled())));
      self.state.complete();
      drop(data);
      self.wake_joiner();
    }
  }
}

struct ThreadWaker {
  current_thread: thread::Thread,
}

impl Wake for ThreadWaker {
  fn wake(self: Arc<Self>) {
    self.current_thread.unpark();
  }

  fn wake_by_ref(self: &Arc<Self>) {
    self.current_thread.unpark();
  }
}
