use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    prelude::*,
};

#[derive(Debug)]
pub struct AppContainer {
    pub app: App,
    pub app_exit_event_reader: ManualEventReader<AppExit>,
}

impl AppContainer {
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins.build().disable::<ScheduleRunnerPlugin>(),));
        // .insert_resource(MovementDelta::default())
        // .add_systems(Update, do_movement)

        while !app.ready() {
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();

        Self {
            app,
            app_exit_event_reader: default(),
        }
    }

    pub fn update(&mut self) -> Option<AppExit> {
        self.app.update();

        self.app
            .world
            .get_resource_mut::<Events<AppExit>>()
            .and_then(|app_exit_events| {
                self.app_exit_event_reader
                    .iter(&app_exit_events)
                    .last()
                    .cloned()
            })
    }
}

// #[no_mangle]
// extern "C" fn create_app() -> Box<AppContainer> {
//     Box::new(AppContainer::new())
// }

// #[no_mangle]
// extern "C" fn update_app(app: Option<&mut AppContainer>) {
//     let Some(app) = app else { return };

//     if app.update().is_some() {
//         println!("App requested exit");
//     }
// }

// #[no_mangle]
// extern "C" fn drop_app(_: Option<Box<AppContainer>>) {}
