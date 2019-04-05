use std::thread;

use crossbeam_channel::{
    bounded,
    select,
    Receiver,
    Select,
    Sender,
};

pub enum Never {}

/// A handle for some long-running computation off of the main thread.
/// It can be used, indirectly, to wait for the completion of said computation.
///
/// All `ConcurrentJob`s must eventually be stored in a `Jobs` table.
///
/// All concurrent activities, such as spawning a thread or pushing a work
/// item to a job queue, should be covered by `ConcurrentJob`.
/// This way, the `Jobs` table will give a complete overview of
/// concurrency in the system, and it will be possible to wait for all jobs to
/// finish, which helps tremendously with making tests deterministic.
///
/// `JobToken` is the worker-side counterpart of `ConcurrentJob`.  Dropping
/// a `JobToken` signals that the corresponding job has finished.
#[must_use]
pub struct ConcurrentJob {
    chan: Receiver<Never>,
}

impl ConcurrentJob {
    pub fn new() -> (Self, JobToken) {
        let (tx, rx) = bounded(0);
        let job = ConcurrentJob { chan: rx };
        let token = JobToken { _chan: tx };
        (job, token)
    }

    fn is_completed(&self) -> bool {
        is_closed(&self.chan)
    }
}

impl Drop for ConcurrentJob {
    fn drop(&mut self) {
        if self.is_completed() || thread::panicking() {
            return;
        }

        panic!("orphaned concurrent job");
    }
}

pub struct JobToken {
    _chan: Sender<Never>,
}

#[derive(Default)]
pub struct Jobs {
    jobs: Vec<ConcurrentJob>,
}

impl Jobs {
    pub fn add(&mut self, job: ConcurrentJob) {
        self.gc();
        self.jobs.push(job);
    }

    /// Blocks the current thread until all pending jobs are finished
    pub fn wait_for_all(&mut self) {
        while !self.jobs.is_empty() {
            let done_idx: usize = {
                let mut select = Select::new();
                for job in &self.jobs {
                    select.recv(&job.chan);
                }

                let oper = select.select();
                let oper_idx = oper.index();
                let chan = &self.jobs[oper_idx].chan;
                assert!(oper.recv(chan).is_err());
                oper_idx
            };

            drop(self.jobs.swap_remove(done_idx))
        }
    }

    fn gc(&mut self) {
        self.jobs.retain(|cj| !cj.is_completed());
    }
}

/// Non-blocking.
fn is_closed(chan: &Receiver<Never>) -> bool {
    select! {
        recv(chan) -> msg => match msg {
            Err(_) => true,
            Ok(never) => match never {},
        },
        default => false,
    }
}