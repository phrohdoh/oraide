use std::{
    sync::{
        Arc,
        Mutex,
        atomic::AtomicBool,
    },
};

use crate::{
    server::{
        Output,
    },
    concurrency::{
        ConcurrentJob,
        Jobs,
    },
};

pub enum Context {
    Init(InitContext),
    Uninit(UninitContext),
}

impl Context {
    /// Create a new, uninitialized, `Context`
    pub fn new() -> Self {
        Context::Uninit(UninitContext::new())
    }

    pub fn init<O: Output>(&mut self, _out: &O) -> Result<(), ()> {
        let ctx = match *self {
            Context::Uninit(ref uninit) => {
                InitContext::new(uninit.server_pid)
            },
            _ => return Err(()),
        };

        *self = Context::Init(ctx);
        Ok(())
    }

    /// Returns an initialized context or `Err(())` if not yet initialized via `init()`
    pub fn inited(&self) -> Result<InitContext, ()> {
        match *self {
            Context::Uninit(_) => Err(()),
            Context::Init(ref ctx) => Ok(ctx.clone()),
        }
    }
}

/// Persistent context shared across all requests and notifications after the
/// language server has been initialized
#[derive(Clone)]
pub struct InitContext {
    jobs: Arc<Mutex<Jobs>>,

    /// Whether the server is performing cleanup (after having received a
    /// 'shutdown' request) just before the final 'exit' request
    pub is_shutting_down: Arc<AtomicBool>,

    pub server_pid: u32,
}

impl InitContext {
    fn new(server_pid: u32) -> Self {
        Self {
            jobs: Arc::default(),
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            server_pid,
        }
    }

    pub fn add_job(&self, job: ConcurrentJob) {
        self.jobs.lock().unwrap().add(job);
    }

    pub fn wait_for_concurrent_jobs(&self) {
        self.jobs.lock().unwrap().wait_for_all();
    }
}

/// Persistent context shared across all requests and notifications before the
/// language server has been initialized
pub struct UninitContext {
    server_pid: u32,
}

impl UninitContext {
    fn new() -> Self {
        Self {
            server_pid: std::process::id(),
        }
    }
}