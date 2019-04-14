use std::{
    fmt,
    time::{
        Instant,
        Duration,
    },
    sync::{
        mpsc,
        Mutex
    },
};

/// Description of the work on the request work pool.  Equality implies two
/// pieces of work are the same kind of thing.  The `str` should be
/// human-readable for logging (e.g., the LSP request message name or similar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkDescription(pub &'static str);

impl fmt::Display for WorkDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Maximum concurrent working tasks of the same type (equal `WorkDescription`s).
/// Note: `2` allows a single task to run immediately after a similar task has
/// timed out.
/// Once multiple similar tasks have timed out but remain running we start
/// refusing to start new ones.
const MAX_SIMILAR_CONCURRENT_WORK: usize = 2;

lazy_static::lazy_static! {
    /// Maximum total concurrent working tasks
    static ref NUM_THREADS: usize = num_cpus::get();

    /// Duration of work after which we should warn something is taking a long time
    static ref WARN_TASK_DURATION: Duration = crate::dispatch::DEFAULT_REQUEST_TIMEOUT * 5;

    /// Current work descriptions active on the work pool
    static ref WORK: Mutex<Vec<WorkDescription>> = Mutex::new(vec![]);

    /// Thread pool for request execution allowing concurrent request processing
    static ref WORK_POOL: rayon::ThreadPool = rayon::ThreadPoolBuilder::new()
        .num_threads(*NUM_THREADS)
        .thread_name(|num| format!("request-worker-{}", num))
        .build()
        .unwrap();
}

pub fn receive_from_thread<T, F>(work_fn: F, description: WorkDescription) -> mpsc::Receiver<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + std::panic::UnwindSafe + 'static,
{
    let (tx, rx) = mpsc::channel();

    {
        let mut work = WORK.lock().unwrap();
        if work.len() >= *NUM_THREADS {
            // There are already N ongoing tasks, which may or may not have
            // timed out, don't add more to the queue.  Fail fast to allow the
            // work pool to recover.
            log::warn!("Refused to start `{}` as we are at work capacity, {:?} in progress", description, *work);
            return rx;
        }

        if work.iter().filter(|desc| *desc == &description).count() >= MAX_SIMILAR_CONCURRENT_WORK {
            // This type of work is already filling max proportion of the
            // work pool so fail new requests fo this kind until some/all of
            // the ongoing similar work finishes.
            log::info!("Refused to start `{}` as same work-type is filling capacity, {:?} in progress", description, *work);
            return rx;
        }

        work.push(description);
    }

    WORK_POOL.spawn(move || {
        let start = Instant::now();

        // Panic details will be written to stderr.  Ignore the work panic as
        // it will cause an mpsc disconnect-error and there is nothing else
        // for us to log.
        if let Ok(work_result) = std::panic::catch_unwind(work_fn) {
            let _ = tx.send(work_result);
        }

        let mut work = WORK.lock().unwrap();
        if let Some(idx) = work.iter().position(|desc| desc == &description) {
            work.swap_remove(idx);
        }

        let elapsed = start.elapsed();
        if elapsed >= *WARN_TASK_DURATION {
            let secs = elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) / 1_000_000_000_f64;
            log::warn!("`{}` took {:.1}s", description, secs);
        }
    });

    rx
}