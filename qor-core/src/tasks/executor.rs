use super::Task;
use alloc::collections::VecDeque;
use core::task::Context;
use core::task::Poll;
use core::task::RawWaker;
use core::task::RawWakerVTable;
use core::task::Waker;

/// Kernel executor
#[allow(clippy::module_name_repetitions)]
pub struct SimpleExecutor<'a> {
    task_queue: VecDeque<Task<'a>>,
}

impl<'a> SimpleExecutor<'a> {
    /// Construct a new empty executor
    #[must_use]
    pub const fn new() -> Self {
        Self {
            task_queue: VecDeque::new(),
        }
    }

    /// Add a new task to the spawn
    pub fn spawn(&mut self, task: Task<'a>) {
        self.task_queue.push_back(task);
    }

    /// Single step, returns true when there is at least one task in the queue
    pub fn step(&mut self) -> Option<bool> {
        if let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => Some(true),
                Poll::Pending => {
                    self.task_queue.push_back(task);
                    Some(false)
                }
            }
        } else {
            None
        }
    }

    /// Run to exhaustion
    pub fn run(&mut self) {
        while self.run_until_pending() {}
    }

    /// Run through the queue until all tasks are pending
    pub fn run_until_pending(&mut self) -> bool {
        if self.task_queue.is_empty() {
            return false;
        }

        'outer: loop {
            let remaining = self.task_queue.len();

            let mut flag = false;

            for _ in 0..remaining {
                if let Some(b) = self.step() {
                    flag |= b;
                } else {
                    break 'outer false;
                }
            }

            if !flag {
                break true;
            }
        }
    }
}

impl<'a> Default for SimpleExecutor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

fn dummy_raw_waker() -> RawWaker {
    const fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(core::ptr::null::<()>(), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
