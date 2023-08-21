use std::sync::Mutex;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool},
};

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

#[derive(Resource, Clone, Copy, Default)]
#[repr(C)]
pub struct MovementDelta {
    x: f32,
    y: f32,
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
