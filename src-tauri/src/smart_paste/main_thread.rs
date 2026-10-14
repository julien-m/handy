//! Short, bounded dispatches of AppKit and Text Input Sources work to the
//! macOS main thread through the GCD main queue.
//!
//! The smart-paste coordinator runs on a worker thread and never blocks the
//! main thread: it hands it one small closure at a time and waits for the
//! result with a timeout. If the main thread is busy past the timeout, the
//! closure is abandoned atomically and will do nothing if it runs later, so
//! no late pasteboard write can happen behind the caller's back.

use std::ffi::c_void;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

const PENDING: u8 = 0;
const RUNNING: u8 = 1;
const ABANDONED: u8 = 2;

#[repr(C)]
struct DispatchQueue {
    _opaque: [u8; 0],
}

unsafe extern "C" {
    // `dispatch_get_main_queue()` is a macro returning `&_dispatch_main_q`.
    static _dispatch_main_q: DispatchQueue;
    fn dispatch_async_f(
        queue: *const DispatchQueue,
        context: *mut c_void,
        work: extern "C" fn(*mut c_void),
    );
    fn pthread_main_np() -> i32;
}

/// Errors of a main-thread dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainThreadError {
    /// The main thread did not start the work in time; it was abandoned.
    Timeout,
    /// The work was dropped without producing a result.
    Dropped,
}

type Job = Box<dyn FnOnce() + Send>;

extern "C" fn run_job(context: *mut c_void) {
    // SAFETY: `context` was produced by `Box::into_raw(Box<Job>)` in
    // `dispatch_job` and GCD hands it to this function exactly once.
    let job = unsafe { Box::from_raw(context.cast::<Job>()) };
    job();
}

fn dispatch_job(job: Job) {
    let context = Box::into_raw(Box::new(job)).cast::<c_void>();
    // SAFETY: the main queue is a process-lifetime static; `run_job` takes
    // ownership of `context` when GCD invokes it on the main thread.
    unsafe { dispatch_async_f(&raw const _dispatch_main_q, context, run_job) };
}

/// Whether the caller is the process main thread.
pub fn is_main_thread() -> bool {
    // SAFETY: pthread_main_np has no preconditions.
    unsafe { pthread_main_np() != 0 }
}

/// Runs `work` on the main thread and waits up to `timeout` for it to start.
///
/// On the main thread itself, `work` runs inline to avoid a self-deadlock.
/// Once started, the work is awaited to completion: it must stay short.
pub fn run_on_main<R: Send + 'static>(
    work: impl FnOnce() -> R + Send + 'static,
    timeout: Duration,
) -> Result<R, MainThreadError> {
    if is_main_thread() {
        return Ok(work());
    }
    let state = Arc::new(AtomicU8::new(PENDING));
    let (sender, receiver) = mpsc::channel();
    let job_state = Arc::clone(&state);
    dispatch_job(Box::new(move || {
        if job_state
            .compare_exchange(PENDING, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            // The receiver may be gone only if the caller already returned,
            // which cannot happen once RUNNING was claimed; ignore defensively.
            let _ = sender.send(work());
        }
    }));
    await_job(&state, &receiver, timeout)
}

fn await_job<R>(
    state: &AtomicU8,
    receiver: &mpsc::Receiver<R>,
    timeout: Duration,
) -> Result<R, MainThreadError> {
    match receiver.recv_timeout(timeout) {
        Ok(result) => return Ok(result),
        Err(mpsc::RecvTimeoutError::Disconnected) => return Err(MainThreadError::Dropped),
        Err(mpsc::RecvTimeoutError::Timeout) => {}
    }
    // Abandon atomically; if the main thread claimed the job just now, its
    // short work is already running and its result is awaited.
    match state.compare_exchange(PENDING, ABANDONED, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Err(MainThreadError::Timeout),
        Err(_) => receiver.recv().map_err(|_| MainThreadError::Dropped),
    }
}
