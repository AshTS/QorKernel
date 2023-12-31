#![allow(dead_code)]

use core::marker::PhantomData;

use alloc::{boxed::Box, sync::Arc};
use qor_core::memory::allocators::page::bitmap::PageBox;

use qor_riscv::{
    drivers::virtio::generic::{
        driver::VirtIOWrapper,
        structures::{
            Descriptor, DeviceID, Queue, VirtIOError, VIRTIO_DESC_F_NEXT, VIRTIO_DESC_F_WRITE,
            VIRTIO_RING_SIZE, VIRTIO_RING_SIZE_USIZE,
        },
    },
    memory::Page,
};

use crate::memory::get_page_bitmap_allocator;

use super::{BlockOperationFuture, Request, VIRTIO_BLK_T_IN, VIRTIO_BLK_T_OUT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIOBlockDeviceError {
    IOError,
    UnsupportedOperation,
}

#[allow(dead_code)]
pub struct VirtIOBlockDevice {
    inner: VirtIOWrapper,
    queue: Option<PageBox<'static, Page, Queue>>,
    index: u16,
    ack_used_index: u16,
}

fn alloc_request<'a>(
    request_type: u32,
    sector: u64,
    buffer: *mut u8,
    status: u8,
) -> Box<Request<'a>> {
    Box::new(Request {
        request_type,
        reserved: 0,
        sector,
        data: buffer,
        status: core::sync::atomic::AtomicU8::new(status),
        _marker: PhantomData,
    })
}

fn arc_alloc_request<'a>(
    request_type: u32,
    sector: u64,
    buffer: *mut u8,
    status: u8,
) -> Arc<Request<'a>> {
    Arc::new(Request {
        request_type,
        reserved: 0,
        sector,
        data: buffer,
        status: core::sync::atomic::AtomicU8::new(status),
        _marker: PhantomData,
    })
}

impl VirtIOBlockDevice {
    /// Creates a new [`VirtIOBlockDevice`].
    ///
    /// # Panics
    ///
    /// This function will panic if the device is not a block device.
    #[must_use]
    pub fn new(inner: VirtIOWrapper) -> Self {
        assert!(inner.device_id() == Ok(Some(DeviceID::BlockDevice)));

        Self {
            inner,
            queue: None,
            index: 0,
            ack_used_index: 0,
        }
    }

    /// Initialize this [`VirtIOBlockDevice`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the initialization failed.
    pub fn initialize(&mut self) -> Result<(), VirtIOError> {
        let max_queue_size = self.inner.maximum_queue_size()?;
        self.inner.set_queue_size(VIRTIO_RING_SIZE)?;

        if max_queue_size < VIRTIO_RING_SIZE {
            return Err(VirtIOError::BadQueueSize);
        }

        let queue = self.inner.add_queue(0, || {
            get_page_bitmap_allocator()
                .alloc_boxed(Queue::default())
                .expect("Couldn't allocate queue")
        })?;
        self.queue = Some(queue);

        self.inner.complete_setup()
    }

    /// Begin executing a block operation.
    fn execute_request<'b, 'a: 'b>(
        &'b mut self,
        request: &'b Request<'a>,
        length: u32,
        write: bool,
    ) -> &'b core::sync::atomic::AtomicU8 {
        let queue = self.queue.as_mut().expect("Queue not initialized");

        let descriptor = Descriptor {
            addr: request as *const _ as u64,
            len: 16,
            flags: VIRTIO_DESC_F_NEXT,
            next: 0,
        };
        let head_index = queue.add_descriptor(descriptor);

        let descriptor = Descriptor {
            addr: request.data as u64,
            len: length,
            flags: VIRTIO_DESC_F_NEXT | (if write { 0 } else { VIRTIO_DESC_F_WRITE }),
            next: 0,
        };
        queue.add_descriptor(descriptor);

        let descriptor = Descriptor {
            addr: core::ptr::addr_of!((request.status)) as u64,
            len: 1,
            flags: VIRTIO_DESC_F_WRITE,
            next: 0,
        };
        queue.add_descriptor(descriptor);

        let idx = queue.available.idx as usize % VIRTIO_RING_SIZE_USIZE;
        queue.available.ring[idx] = head_index;
        queue.available.idx = queue.available.idx.wrapping_add(1);

        self.inner.queue_notify(0);

        &request.status
    }

    /// Execute a blocking block operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    pub fn blocking_block_operation(
        &mut self,
        buffer: *mut u8,
        buffer_length: usize,
        block_index: usize,
        write: bool,
    ) -> Result<(), VirtIOBlockDeviceError> {
        let truncated_buffer_length =
            u32::try_from(buffer_length).expect("Length exceeds maximum buffer size");

        let request = alloc_request(
            if write {
                VIRTIO_BLK_T_OUT
            } else {
                VIRTIO_BLK_T_IN
            },
            block_index as u64,
            buffer,
            111,
        );
        let status_ptr = self.execute_request(&request, truncated_buffer_length, write);

        while status_ptr.load(core::sync::atomic::Ordering::Acquire) == 111 {}
        let value = status_ptr.load(core::sync::atomic::Ordering::Acquire);

        match value {
            0 => Ok(()),
            1 => Err(VirtIOBlockDeviceError::IOError),
            2 => Err(VirtIOBlockDeviceError::UnsupportedOperation),
            _ => unimplemented!(),
        }
    }

    unsafe fn non_blocking_block_operation<'b, 'a: 'b>(
        &'b mut self,
        buffer: *mut u8,
        buffer_length: usize,
        block_index: usize,
        write: bool,
        _phantom: PhantomData<&'a u8>,
    ) -> BlockOperationFuture<'a> {
        let truncated_buffer_length =
            u32::try_from(buffer_length).expect("Length exceeds maximum buffer size");

        let request = arc_alloc_request(
            if write {
                VIRTIO_BLK_T_OUT
            } else {
                VIRTIO_BLK_T_IN
            },
            block_index as u64,
            buffer,
            111,
        );
        self.execute_request(&request, truncated_buffer_length, write);

        BlockOperationFuture::new(111, request)
    }

    /// Execute a blocking read operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    pub fn blocking_read(
        &mut self,
        buffer: &mut [[u8; 512]],
        block_index: usize,
    ) -> Result<(), VirtIOBlockDeviceError> {
        self.blocking_block_operation(
            buffer.as_mut_ptr().cast(),
            buffer.len() * 512,
            block_index,
            false,
        )
    }

    /// Execute a non-blocking read operation.
    pub fn non_blocking_read<'b, 'a: 'b>(
        &'b mut self,
        buffer: &'a mut [[u8; 512]],
        block_index: usize,
    ) -> BlockOperationFuture {
        unsafe {
            self.non_blocking_block_operation(
                buffer.as_mut_ptr().cast(),
                buffer.len() * 512,
                block_index,
                false,
                PhantomData::<&'a u8>,
            )
        }
    }

    /// Execute a blocking write operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    pub fn blocking_write(
        &mut self,
        buffer: &[[u8; 512]],
        block_index: usize,
    ) -> Result<(), VirtIOBlockDeviceError> {
        #[allow(clippy::as_ptr_cast_mut)]
        self.blocking_block_operation(
            buffer.as_ptr() as *mut u8,
            buffer.len() * 512,
            block_index,
            true,
        )
    }

    /// Execute a non-blocking write operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    pub fn non_blocking_write<'a, 'b>(
        &'b mut self,
        buffer: &'a [[u8; 512]],
        block_index: usize,
    ) -> BlockOperationFuture<'a> {
        #[allow(clippy::as_ptr_cast_mut)]
        unsafe {
            self.non_blocking_block_operation(
                buffer.as_ptr() as *mut u8,
                buffer.len() * 512,
                block_index,
                true,
                PhantomData::<&'a u8>,
            )
        }
    }

    /// Clean up after the used ring, note that we don't actually free anything here, as individual requests are
    /// responsible for doing that themselves.
    pub fn clean_up(&mut self) {
        let queue = self.queue.as_mut().expect("Queue not initialized");

        while self.ack_used_index != queue.used.idx {
            // warn!("A");
            self.ack_used_index = self.ack_used_index.wrapping_add(1);
        }
    }
}
