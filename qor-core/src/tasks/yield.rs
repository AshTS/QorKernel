use core::future::Future;

pub struct TaskYieldFuture(bool);

impl TaskYieldFuture {
    #[must_use]
    pub const fn new() -> Self {
        Self(false)
    }
}

impl Future for TaskYieldFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.0 {
            core::task::Poll::Ready(())
        } else {
            self.get_mut().0 = true;
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}

/// Yield the current task, allowing other tasks to run
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub const fn task_yield() -> TaskYieldFuture {
    TaskYieldFuture::new()
}
