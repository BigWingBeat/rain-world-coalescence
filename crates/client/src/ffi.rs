use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use anyhow::anyhow;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, Task},
};
use multiplayer_mvp_net::AppEndpoint;
use widestring::{U16CStr, U16CString, Utf16Str};

use crate::app::{configure_logging, create_endpoint, AppContainer};

/// A `Box`, but only for `Sized` types, so guranteed to always be 'thin', i.e. always 1 `usize`.
/// Pointers to unsized types are 'fat', i.e. 2 `usize`s. The second `usize` is for len/vtable/etc.
/// This is needed because pointers get marshalled to C#'s `IntPtr` type, which is always 1 `usize`.
/// See https://doc.rust-lang.org/std/boxed/index.html#memory-layout
#[repr(transparent)]
#[derive(Debug)]
pub struct ThinBox<T: Sized>(Box<T>);

impl<T> ThinBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn into_raw(b: Self) -> *mut T {
        Box::into_raw(b.0)
    }

    /// # Safety
    ///
    /// See [`Box::from_raw`]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Self(Box::from_raw(raw))
    }

    pub fn into_inner(b: Self) -> T {
        *b.0
    }
}

impl<T> Deref for ThinBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ThinBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct ConnectionTask(Task<anyhow::Result<()>>);

#[no_mangle]
pub extern "C" fn new_app() -> *mut AppContainer {
    ThinBox::into_raw(ThinBox::new(AppContainer::new()))
}

/// Returns whether or not the given app requests to exit, or false if the pointer is null
///
/// # Safety
///
/// The given pointer must be [valid]
///
/// [valid]: https://doc.rust-lang.org/std/ptr/index.html#safety
#[no_mangle]
pub unsafe extern "C" fn update_app(app: *mut AppContainer) -> bool {
    if app.is_null() {
        warn!("Cannot update null app pointer");
        false
    } else {
        (*app).update().is_some()
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum AppConnectToServerResult {
    Ok(*mut ConnectionTask),
    Err(anyhow::Error),
}

/// # Safety
///
/// The given pointers must be [valid], and `address` must point to a null-terminated, UTF-16 encoded string
///
/// [valid]: https://doc.rust-lang.org/std/ptr/index.html#safety
#[no_mangle]
pub unsafe extern "C" fn app_connect_to_server(
    app: *mut AppContainer,
    address: *const u16,
    port: u16,
) -> AppConnectToServerResult {
    if app.is_null() {
        AppConnectToServerResult::Err(anyhow!("Given app pointer is null"))
    } else {
        let endpoint = match (*app).world.get_resource::<AppEndpoint>() {
            Some(AppEndpoint(endpoint)) => endpoint.clone(),
            None => match create_endpoint() {
                Ok(endpoint) => {
                    (*app).insert_resource(AppEndpoint(endpoint.clone()));
                    endpoint
                }
                Err(e) => return AppConnectToServerResult::Err(e.into()),
            },
        };

        let address = U16CStr::from_ptr_str(address);
        let address = Utf16Str::from_ucstr_unchecked(address);
        let address = address.to_string();
        let task = IoTaskPool::get()
            .spawn(async move { AppContainer::connect_to_server(&endpoint, &address, port).await });
        AppConnectToServerResult::Ok(ThinBox::into_raw(ThinBox::new(ConnectionTask(task))))
    }
}

/// # Safety
///
/// See [`Box::from_raw`]
#[no_mangle]
pub unsafe extern "C" fn drop_app(app: *mut AppContainer) {
    if !app.is_null() {
        drop(ThinBox::from_raw(app));
    }
}

/// # Safety
///
/// The given pointer must be an [`anyhow::Error`]
#[no_mangle]
pub unsafe extern "C" fn format_error(error: anyhow::Error) -> *mut u16 {
    // FFI values should only be dropped by the corresponding `drop_xxx` functions, so use ManuallyDrop to
    // avoid dropping the error when we're just formatting it
    let error = ManuallyDrop::new(error);
    let message = format!("{:#}", *error);
    let message = message.replace('\0', "�");
    // SAFETY: We just replaced all null bytes in the string, so this is always safe
    let utf16 = unsafe { U16CString::from_str_unchecked(message) };
    utf16.into_raw()
}

/// # Safety
///
/// The given pointer must be an [`anyhow::Error`]
#[no_mangle]
pub unsafe extern "C" fn drop_error(error: anyhow::Error) {
    drop(error);
}

/// # Safety
///
/// See [`U16CString::from_raw`]
#[no_mangle]
pub unsafe extern "C" fn drop_string(string: *mut u16) {
    if !string.is_null() {
        drop(U16CString::from_raw(string));
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum PollConnectionTaskResult {
    Pending,
    Ok,
    Err(anyhow::Error),
}

/// # Safety
///
/// The given pointer must be [valid]
///
/// [valid]: https://doc.rust-lang.org/std/ptr/index.html#safety
#[no_mangle]
pub unsafe extern "C" fn poll_connection_task(
    task: *mut ConnectionTask,
) -> PollConnectionTaskResult {
    if task.is_null() {
        PollConnectionTaskResult::Err(anyhow!("Given task pointer is null"))
    } else {
        match futures_lite::future::block_on(futures_lite::future::poll_once(&mut (*task).0)) {
            Some(Ok(())) => PollConnectionTaskResult::Ok,
            Some(Err(e)) => PollConnectionTaskResult::Err(e),
            None => PollConnectionTaskResult::Pending,
        }
    }
}

/// # Safety
///
/// See [`Box::from_raw`]
#[no_mangle]
pub unsafe extern "C" fn drop_connection_task(task: *mut ConnectionTask) {
    if !task.is_null() {
        let task = ThinBox::into_inner(ThinBox::from_raw(task));
        futures_lite::future::block_on(task.0.cancel());
    }
}

/// Configures native logging permanently for the whole application. Calling this more than once will panic.
#[no_mangle]
pub extern "C" fn configure_native_logging() {
    configure_logging()
}

#[no_mangle]
pub extern "C" fn terminate_taskpool_threads() {
    if let Some(pool) = ComputeTaskPool::try_get() {
        info!("Terminating ComputeTaskPool threads!");
        pool.terminate_all_threads();
    }

    if let Some(pool) = AsyncComputeTaskPool::try_get() {
        info!("Terminating AsyncComputeTaskPool threads!");
        pool.terminate_all_threads();
    }

    if let Some(pool) = IoTaskPool::try_get() {
        info!("Terminating IoTaskPool threads!");
        pool.terminate_all_threads();
    }
}
