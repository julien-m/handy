//! One dedicated thread owning a state that must never leave it (011's Windows UI Automation
//! client on its COM MTA thread), with bounded calls (plan "Windows adapter"): a call waits at
//! most its timeout and a late answer is discarded without effect; a call made while an earlier
//! one still occupies the thread is refused at once, so a hung cross-process call never queues
//! or blocks later dictations, the UI thread or the guard. Releases are posted without waiting.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::time::Duration;

type Job<S> = Box<dyn FnOnce(&mut S) + Send>;

/// Why a bounded call got no answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallError {
    /// An earlier call still occupies the thread: nothing was sent.
    Busy,
    /// No answer within the timeout: the job may still run later (a write may still apply).
    Expired,
    /// The thread is gone or its state could not be created.
    Stopped,
}

/// Marks the thread occupied until the call's job ends, or is dropped without running.
struct Occupied(Arc<AtomicBool>);
impl Drop for Occupied {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// The thread and its queue.
pub struct BoundedWorker<S> {
    jobs: Sender<Job<S>>,
    busy: Arc<AtomicBool>,
}

impl<S: 'static> BoundedWorker<S> {
    /// Start the thread; `init` builds the state on it (`None`: every call is `Stopped`).
    pub fn spawn(name: &str, init: impl FnOnce() -> Option<S> + Send + 'static) -> Self {
        let (jobs, queue) = channel::<Job<S>>();
        let spawned = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                let Some(mut state) = init() else {
                    return;
                };
                for job in queue {
                    job(&mut state);
                }
            });
        if let Err(error) = spawned {
            log::warn!("Could not start the {name} thread: {error}");
        }
        Self {
            jobs,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Run `job` on the thread and wait at most `timeout` for its answer.
    ///
    /// # Errors
    /// `Busy` (nothing sent), `Expired` (sent, unanswered in time) or `Stopped`.
    pub fn call<T: Send + 'static>(
        &self,
        timeout: Duration,
        job: impl FnOnce(&mut S) -> T + Send + 'static,
    ) -> Result<T, CallError> {
        // Business rule (plan): one answer-awaited call at a time; never queue behind a hang.
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(CallError::Busy);
        }
        let (answer, answered) = channel();
        let occupied = Occupied(Arc::clone(&self.busy));
        let sent = self.jobs.send(Box::new(move |state: &mut S| {
            let result = job(state);
            drop(occupied);
            // A dropped receiver means the caller gave up: the late answer is discarded.
            let _ = answer.send(result);
        }));
        if sent.is_err() {
            return Err(CallError::Stopped);
        }
        answered.recv_timeout(timeout).map_err(|error| match error {
            std::sync::mpsc::RecvTimeoutError::Timeout => CallError::Expired,
            std::sync::mpsc::RecvTimeoutError::Disconnected => CallError::Stopped,
        })
    }

    /// Queue `job` without waiting (releases of platform references).
    pub fn post(&self, job: impl FnOnce(&mut S) + Send + 'static) {
        // A stopped thread has already released everything it owned.
        let _ = self.jobs.send(Box::new(job));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHORT: Duration = Duration::from_millis(20);

    #[test]
    fn a_call_answers_from_the_thread_state() {
        let worker = BoundedWorker::spawn("test", || Some(41_u32));
        assert_eq!(worker.call(SHORT * 50, |state| *state + 1), Ok(42));
    }

    // @spec AC-007: Bounded UI Automation calls — .specs/features/011-dictation-field-return/spec.md#ac-007
    #[test]
    fn a_hung_call_expires_refuses_the_next_one_and_its_late_answer_is_dropped() {
        let worker = BoundedWorker::spawn("test", || Some(Vec::<&str>::new()));
        let (release, hold) = channel::<()>();
        let hung = worker.call(SHORT, move |state: &mut Vec<&str>| {
            let _ = hold.recv();
            state.push("late write applied");
        });
        assert_eq!(hung, Err(CallError::Expired), "the caller stops waiting");
        let next = worker.call(SHORT, |_| ());
        assert_eq!(next, Err(CallError::Busy), "refused at once, not queued");
        drop(release);
        let writes = poll(|| worker.call(SHORT, |state: &mut Vec<&str>| state.len()).ok());
        assert_eq!(
            writes,
            Some(1),
            "an expired write may still apply: unsettled"
        );
    }

    /// The first answer of `probe` within 100 tries 5 ms apart.
    fn poll<T>(probe: impl Fn() -> Option<T>) -> Option<T> {
        (0..100).find_map(|_| {
            let answer = probe();
            if answer.is_none() {
                std::thread::sleep(Duration::from_millis(5));
            }
            answer
        })
    }

    #[test]
    fn a_state_that_cannot_be_created_stops_every_call() {
        let worker = BoundedWorker::<u8>::spawn("test", || None);
        assert_eq!(worker.call(SHORT * 50, |_| ()), Err(CallError::Stopped));
        assert_eq!(
            worker.call(SHORT * 50, |_| ()),
            Err(CallError::Stopped),
            "never busy"
        );
    }
}
