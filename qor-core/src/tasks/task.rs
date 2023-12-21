use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

/// Kernel task object used to enable async execution on the kernel
pub struct Task<'a> {
    future: Pin<Box<dyn Future<Output = ()> + 'a>>,
}

async fn ignore<'a, T>(future: impl Future<Output = T> + 'a + Send) {
    future.await;
}

impl<'a> Task<'a> {
    /// Construct a task around a future
    pub fn new(future: impl Future<Output = ()> + 'a) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    /// Construct a task around a future
    pub fn ignore_result<T: 'a>(future: impl Future<Output = T> + 'a + Send) -> Self {
        Self {
            future: Box::pin(ignore(future)),
        }
    }

    /// Poll the wrapped future
    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
