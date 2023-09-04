use std::mem::ManuallyDrop;

use anyhow::anyhow;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, Task},
};
use multiplayer_mvp_net::AppEndpoint;
use widestring::{U16CStr, U16CString, Utf16Str};

use crate::app::AppContainer;

/// The `#[repr(u8)]` functions according to https://github.com/rust-lang/rfcs/blob/master/text/2195-really-tagged-unions.md
///
/// The C# type this gets marshalled to is:
/// ```csharp
/// [StructLayout(LayoutKind.Explicit)]
/// struct MarshalledResult
/// {
///     enum MarshalledResultTag : byte {
///         Ok,
///         Err
///     }
///     [FieldOffset(0)]
///     public MarshalledResultTag tag;
///     [FieldOffset(x)]
///     public T OkValue;
///     [FieldOffset(y)]
///     public E ErrValue;
/// }
/// ```
/// where `x` and `y` are the correct alignments for the given types `T` and `E`.
#[repr(u8)]
pub enum MarshalledResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> From<Result<T, E>> for MarshalledResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(t) => Self::Ok(t),
            Err(e) => Self::Err(e),
        }
    }
}

/// An opaque pointer to an error type, suitable for marshalling to C# as an `IntPtr`.
pub type MarshalledError = anyhow::Error;

/// A `Box`, but only for `Sized` types, so guranteed to always be 'thin', i.e. always 1 `usize`.
/// Pointers to unsized types are 'fat', i.e. 2 `usize`s. The second `usize` is for len/vtable/etc.
/// This is needed because pointers get marshalled to C#'s `IntPtr` type, which is always 1 `usize`.
#[repr(transparent)]
pub struct MarshalledBox<T: Sized>(pub Box<T>);

impl<T> From<Box<T>> for MarshalledBox<T> {
    fn from(value: Box<T>) -> Self {
        Self(value)
    }
}

type ConnectionTask = Task<anyhow::Result<()>>;

#[no_mangle]
pub extern "C" fn new_app() -> MarshalledResult<MarshalledBox<AppContainer>, MarshalledError> {
    AppContainer::new()
        .map(|app| Box::new(app).into())
        .map_err(MarshalledError::new)
        .into()
}

#[no_mangle]
pub extern "C" fn update_app(app: Option<&mut AppContainer>) -> bool {
    app.and_then(|app| app.update()).is_some()
}

/// # Safety
///
/// The caller must ensure `address` points to a null-terminated, UTF-16 encoded string
#[no_mangle]
pub unsafe extern "C" fn app_connect_to_server(
    app: Option<&mut AppContainer>,
    address: *const u16,
    port: u16,
) -> Option<Box<ConnectionTask>> {
    app.map(|app| {
        let address = U16CStr::from_ptr_str(address);
        let address = Utf16Str::from_ucstr_unchecked(address);
        let address = address.to_string();
        let endpoint = app.app.world.resource::<AppEndpoint>().0.clone();
        let task = IoTaskPool::get()
            .spawn(async move { AppContainer::connect_to_server(&endpoint, &address, port).await });
        Box::new(task)
    })
}

#[no_mangle]
pub extern "C" fn free_app(_: Option<Box<AppContainer>>) {}

#[no_mangle]
pub extern "C" fn format_error(error: MarshalledError) -> *mut u16 {
    // FFI values should only be dropped by the corresponding `free_xxx` functions, so use ManuallyDrop to
    // avoid dropping the error when we're just formatting it
    let error = ManuallyDrop::new(error);
    let message = format!("{:#}", *error);
    let message = message.replace('\0', "�");
    // SAFETY: We just replaced all null bytes in the string, so this is always safe
    let utf16 = unsafe { U16CString::from_str_unchecked(message) };
    utf16.into_raw()
}

#[no_mangle]
pub extern "C" fn free_error(_: MarshalledError) {}

#[no_mangle]
pub extern "C" fn poll_connection_task(
    task: Option<&mut ConnectionTask>,
) -> MarshalledResult<bool, MarshalledError> {
    task.ok_or_else(|| anyhow!("Cannot poll a null pointer"))
        .and_then(|task| {
            futures_lite::future::block_on(futures_lite::future::poll_once(task))
                .transpose()
                .map(|o| o.is_some())
        })
        .into()
}

#[no_mangle]
pub extern "C" fn free_connection_task(task: Option<Box<ConnectionTask>>) {
    if let Some(task) = task {
        futures_lite::future::block_on(task.cancel());
    }
}

#[no_mangle]
pub extern "C" fn default_port() -> u16 {
    multiplayer_mvp_net::DEFAULT_PORT
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
