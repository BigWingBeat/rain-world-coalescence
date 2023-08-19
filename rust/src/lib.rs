use std::sync::Mutex;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool},
};

pub struct AppContainer {
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

#[derive(Resource)]
pub struct MovementCallback(extern "C" fn(f32, f32));

#[no_mangle]
pub extern "C" fn init_app(movement_callback: extern "C" fn(f32, f32)) {
    println!("App init");

    App::new()
        .add_plugins(MinimalPlugins.build().disable::<ScheduleRunnerPlugin>())
        .insert_resource(MovementDelta::default())
        .insert_resource(MovementCallback(movement_callback))
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
    println!("App try update");
    if let Some(ref mut container) = *APP.lock().unwrap() {
        println!("App real update");
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

pub fn do_movement(
    callback: ResMut<MovementCallback>,
    mut delta: ResMut<MovementDelta>,
    time: Res<Time>,
) {
    let diff_x = time.elapsed_seconds().sin();
    let diff_y = time.elapsed_seconds().cos();
    println!("Bevy doing movement: ({diff_x}, {diff_y})");
    delta.x = diff_x;
    delta.y = diff_y;
    callback.0(diff_x, diff_y);
}

// #[no_mangle]
// pub extern "C" fn query_movement_delta() -> MovementDelta {
//     if let Some(ref mut container) = *APP.lock().unwrap() {
//         *container.app.world.resource()
//     } else {
//         panic!("Cannot query when the app does not exist");
//     }
// }

#[no_mangle]
pub extern "C" fn destroy_static_taskpools() {
    println!("[Rust] Destroying static taskpools");
    ComputeTaskPool::get().terminate_all_threads();
    AsyncComputeTaskPool::get().terminate_all_threads();
    IoTaskPool::get().terminate_all_threads();
}
