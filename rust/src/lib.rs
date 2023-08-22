use std::{net::ToSocketAddrs, sync::Mutex};

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool},
};
use widestring::{U16CStr, U16CString, Utf16Str};

struct AppContainer {
    pub app: App,
    pub app_exit_event_reader: ManualEventReader<AppExit>,
}

impl Drop for AppContainer {
    fn drop(&mut self) {
        println!("AppContainer is being dropped");
    }
}

static APP: Mutex<Option<AppContainer>> = Mutex::new(None);

type ErrorHandlerFn = extern "C" fn(*const u16);

static ERROR_HANDLER: Mutex<ErrorHandlerFn> = Mutex::new(default_error_handler);

#[derive(Resource, Clone, Copy, Default)]
#[repr(C)]
pub struct MovementDelta {
    x: f32,
    y: f32,
}

extern "C" fn default_error_handler(error: *const u16) {
    let string = unsafe { U16CStr::from_ptr_str(error) };
    let display = string.display();
    panic!("{display}");
}

#[no_mangle]
pub extern "C" fn set_error_handler(handler: ErrorHandlerFn) {
    *ERROR_HANDLER.lock().unwrap() = handler;
}

fn handle_error(error: &str) {
    let error = error.replace('\0', "�");
    // SAFETY: We just replaced all null bytes in the string, so this is always safe
    let utf16 = unsafe { U16CString::from_str_unchecked(error) };
    let handler = ERROR_HANDLER.lock().unwrap();
    // SAFETY: The CLR copies strings when marshalling, so the C# code never actually sees this pointer
    handler(utf16.as_ptr());
}

#[no_mangle]
pub extern "C" fn init_app() {
    println!("App init");

    App::new()
        .add_plugins(MinimalPlugins.build().disable::<ScheduleRunnerPlugin>())
        .insert_resource(MovementDelta::default())
        .add_systems(Update, do_movement)
        .set_runner(move |mut app: App| {
            while !app.ready() {
                bevy::tasks::tick_global_task_pools_on_main_thread();
            }
            app.finish();
            app.cleanup();

            // app.update();

            *APP.lock().unwrap() = Some(AppContainer {
                app,
                app_exit_event_reader: default(),
            });
        })
        .run();
}

#[no_mangle]
pub extern "C" fn update_app() -> bool {
    if let Some(ref mut container) = *APP.lock().unwrap() {
        println!("App update");
        container.app.update();

        if let Some(app_exit_events) = container.app.world.get_resource_mut::<Events<AppExit>>() {
            if container
                .app_exit_event_reader
                .iter(&app_exit_events)
                .last()
                .is_some()
            {
                println!("App requesting exit");
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn destroy_app() {
    println!("Destroying app");
    *APP.lock().unwrap() = None;
}

fn do_movement(mut delta: ResMut<MovementDelta>, time: Res<Time>) {
    delta.x = time.elapsed_seconds().sin();
    delta.y = time.elapsed_seconds().cos();
}

#[no_mangle]
pub extern "C" fn query_movement_delta() -> MovementDelta {
    APP.lock()
        .unwrap()
        .as_ref()
        .and_then(|container| container.app.world.get_resource())
        .copied()
        .unwrap_or_default()
}

/// # Safety
///
/// The caller must ensure the given pointer points to a null-terminated, UTF-16 encoded string
#[no_mangle]
pub unsafe extern "C" fn connect_to_server(address: *const u16, port: u16) {
    let address = U16CStr::from_ptr_str(address);
    let address = Utf16Str::from_ucstr_unchecked(address);
    let address = address.to_string();

    if let Err(e) = try_connect_to_server(&address, port) {
        handle_error(&format!(
            "Cannot connect to address '{address}' on port {port}:\n\n{e}"
        ));
    }
}

fn try_connect_to_server(address: &str, port: u16) -> anyhow::Result<()> {
    println!("[Rust] Connecting to address: {address} on port: {port}");

    let addresses = (address, port).to_socket_addrs()?;

    let len = addresses.len();
    println!("[Rust] Resolved {len} addresses:");

    for (i, address) in addresses.enumerate() {
        println!("[Rust] #{i}: {address}");
    }

    Ok(())
}

#[no_mangle]
pub extern "C" fn destroy_static_taskpools() {
    println!("[Rust] Destroying static taskpools");

    if let Some(pool) = ComputeTaskPool::try_get() {
        pool.terminate_all_threads()
    }

    if let Some(pool) = AsyncComputeTaskPool::try_get() {
        pool.terminate_all_threads()
    }

    if let Some(pool) = IoTaskPool::try_get() {
        pool.terminate_all_threads()
    }
}
