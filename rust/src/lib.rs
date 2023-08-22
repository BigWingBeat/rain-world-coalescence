use std::{net::ToSocketAddrs, sync::Mutex};

use anyhow::bail;
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool},
};
use widestring::{u16cstr, U16CStr, U16CString};

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

extern "C" fn default_error_handler(error: *const u16) {}

#[no_mangle]
pub extern "C" fn set_error_handler(handler: ErrorHandlerFn) {
    *ERROR_HANDLER.lock().unwrap() = handler;
}

fn handle_error(error: impl AsRef<str>) {
    let handler = ERROR_HANDLER.lock().unwrap();
    match U16CString::from_str(error.as_ref()) {
        Ok(utf16) => handler(utf16.into_raw()),
        Err(e) => {
            let string = error.as_ref();
            println!("[Rust] Could not utf16-encode error message '{string}': {e}");
            handler(u16cstr!("Encountered an error while attempting to display a previous error. See console log for details.").as_ptr());
        }
    }
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

#[no_mangle]
pub unsafe extern "C" fn connect_to_server(address: *const u16, port: u16) {
    if let Err(e) = try_connect_to_server(address, port) {
        let address = U16CStr::from_ptr_str(address);
        let display = address.display();
        handle_error(format!(
            "Cannot connect to address '{display}' on port {port}:\n\n{e}"
        ));
    }
}

unsafe fn try_connect_to_server(address: *const u16, port: u16) -> anyhow::Result<()> {
    let address = U16CStr::from_ptr_str(address);

    if address.is_empty() {
        bail!("Address is empty.");
    }

    let address = address.to_string()?;

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
