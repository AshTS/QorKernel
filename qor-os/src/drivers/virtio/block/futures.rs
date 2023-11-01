use alloc::sync::Arc;

use super::{Request, VirtIOBlockDeviceError};

#[derive(Clone)]
pub struct BlockOperationFuture<'a> {
    original_value: u8,
    request: Arc<Request<'a>>,
}

impl<'a> BlockOperationFuture<'a> {
    pub fn new(original_value: u8, request: Arc<Request<'a>>) -> Self {
        Self {
            original_value,
            request,
        }
    }
}

impl<'a> core::future::Future for BlockOperationFuture<'a> {
    type Output = Result<(), VirtIOBlockDeviceError>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let value = self
            .request
            .status
            .load(core::sync::atomic::Ordering::Acquire);

        if value == self.original_value {
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(match value {
                0 => Ok(()),
                1 => Err(VirtIOBlockDeviceError::IOError),
                2 => Err(VirtIOBlockDeviceError::UnsupportedOperation),
                _ => unimplemented!(),
            })
        }
    }
}
