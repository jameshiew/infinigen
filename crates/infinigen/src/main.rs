#![deny(unstable_features)]
#![deny(unused_features)]
use std::process::ExitCode;

use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::tasks::available_parallelism;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use config::Config;
use infinigen::AppPlugin;

const APP_NAME: &str = "infinigen";
const CONFIG_PREFIX: &str = "infinigen_";
const DEFAULT_LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=info";

fn main() -> ExitCode {
    let cfg = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix(CONFIG_PREFIX))
        .build();
    let cfg: infinigen_plugins::settings::Config = match cfg {
        Ok(settings) => match settings.try_deserialize() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("Couldn't parse settings: {}", err);
                return ExitCode::from(78);
            }
        },
        Err(err) => {
            eprintln!(
                "Couldn't load settings, using default configuration: {:?}",
                err
            );
            infinigen_plugins::settings::Config::default()
        }
    };
    match App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: DEFAULT_LOG_FILTER.into(),
                    level: bevy::log::Level::DEBUG,
                    custom_layer: |_| None,
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions {
                        compute: TaskPoolThreadAssignmentPolicy {
                            min_threads: available_parallelism(),
                            max_threads: usize::MAX,
                            percent: 1.0,
                        },
                        ..default()
                    },
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: APP_NAME.into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((AppPlugin::new(cfg),))
        .run()
    {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}
