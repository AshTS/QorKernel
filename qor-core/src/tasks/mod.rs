pub mod executor;
pub use executor::*;

pub mod task;
pub use task::*;

pub mod timer;
pub use timer::*;

pub mod r#yield;
pub use r#yield::task_yield;

/// Spawn a new executor to run a task to completion synchronously
pub fn execute_task(task: Task) {
    let mut executor = SimpleExecutor::new();
    executor.spawn(task);
    executor.run();
}