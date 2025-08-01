use std::sync::{Condvar, Mutex};

/// A synchronized stack of work items/jobs for use by a collection of
/// concurrent workers that are both producers and consumers of jobs.
///
/// After the stack is initialized with some starting jobs, each worker calls
/// `handle_job()` (likely in a loop) or `handle_many_jobs()` with a function
/// that takes a job and returns more jobs, which are pushed on top of the
/// stack.  All work terminates when (a) all jobs have completed successfully,
/// (b) a job has returned `Err`, or (c) `shutdown()` is called.
#[derive(Debug)]
pub struct JobStack<T> {
    data: Mutex<JobStackData<T>>,
    cond: Condvar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JobStackData<T> {
    stack: Vec<T>,
    jobs: usize,
    shutdown: bool,
}

impl<T> JobStack<T> {
    pub fn new<I: IntoIterator<Item = T>>(items: I) -> Self {
        let stack = Vec::from_iter(items);
        let jobs = stack.len();
        JobStack {
            data: Mutex::new(JobStackData {
                stack,
                jobs,
                shutdown: false,
            }),
            cond: Condvar::new(),
        }
    }

    pub fn handle_job<F, I, E>(&self, f: F) -> Result<bool, E>
    where
        F: FnOnce(T) -> Result<I, E>,
        I: IntoIterator<Item = T>,
    {
        let Some(value) = self.pop() else {
            return Ok(false);
        };
        match f(value) {
            Ok(iter) => {
                self.extend(iter);
                self.job_done();
                Ok(true)
            }
            Err(e) => {
                self.job_done();
                self.shutdown();
                Err(e)
            }
        }
    }

    pub fn handle_many_jobs<F, I, E>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(T) -> Result<I, E>,
        I: IntoIterator<Item = T>,
    {
        while let Some(value) = self.pop() {
            match f(value) {
                Ok(iter) => {
                    self.extend(iter);
                    self.job_done();
                }
                Err(e) => {
                    self.job_done();
                    self.shutdown();
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        let Ok(mut data) = self.data.lock() else {
            unreachable!("Mutex should not have been poisoned");
        };
        if !data.shutdown {
            data.jobs -= data.stack.len();
            data.stack.clear();
            data.shutdown = true;
            self.cond.notify_all();
        }
    }

    pub fn is_shutdown(&self) -> bool {
        let Ok(data) = self.data.lock() else {
            unreachable!("Mutex should not have been poisoned");
        };
        data.shutdown
    }

    fn pop(&self) -> Option<T> {
        let Ok(mut data) = self.data.lock() else {
            unreachable!("Mutex should not have been poisoned");
        };
        loop {
            if data.jobs == 0 || data.shutdown {
                return None;
            }
            if let value @ Some(_) = data.stack.pop() {
                return value;
            } else {
                let Ok(data2) = self.cond.wait(data) else {
                    unreachable!("Mutex should not have been poisoned");
                };
                data = data2;
            }
        }
    }

    fn job_done(&self) {
        let Ok(mut data) = self.data.lock() else {
            unreachable!("Mutex should not have been poisoned");
        };
        data.jobs -= 1;
        if data.jobs == 0 {
            self.cond.notify_all();
        }
    }

    // We can't impl Extend, as that requires the receiver to be mut
    fn extend<I: IntoIterator<Item = T>>(&self, iter: I) {
        let Ok(mut data) = self.data.lock() else {
            unreachable!("Mutex should not have been poisoned");
        };
        if !data.shutdown {
            let prelen = data.stack.len();
            data.stack.extend(iter);
            data.jobs += data.stack.len() - prelen;
            self.cond.notify_all();
        }
    }
}
