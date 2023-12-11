use alloc::{boxed::Box, sync::Arc};
use qor_core::interfaces::fs::{
    INodeReference, MountableFileSystem, ParentFileSystem, VirtualFileSystem,
};
use spin::RwLock;

pub type InnerGlobalFS = RwLock<Box<dyn ParentFileSystem + Send + Sync>>;

pub static GLOBAL_FILE_SYSTEM: RwLock<Option<Arc<InnerGlobalFS>>> = RwLock::new(None);

pub fn initialize_file_system() {
    let fs = VirtualFileSystem::new();
    GLOBAL_FILE_SYSTEM
        .write()
        .replace(Arc::new(RwLock::new(Box::new(fs))));

    info!("Initialized empty fs");
}

#[allow(clippy::module_name_repetitions)]
pub fn global_fs() -> Arc<RwLock<Box<dyn ParentFileSystem + Send + Sync>>> {
    GLOBAL_FILE_SYSTEM.read().as_ref().unwrap().clone()
}

#[allow(clippy::module_name_repetitions)]
pub fn mount_fs(
    inode: INodeReference,
    device: alloc::sync::Arc<dyn MountableFileSystem + Send + Sync + 'static>,
) {
    global_fs().write().as_mut().mount_filesystem(inode, device);
}
