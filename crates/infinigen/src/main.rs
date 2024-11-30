use std::process::ExitCode;

use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use config::Config;

use infinigen::AppPlugin;

const APP_NAME: &str = "infinigen";
const CONFIG_PREFIX: &str = "infinigen_";
const LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=info";

fn main() -> ExitCode {
    let cfg = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix(CONFIG_PREFIX))
        .build();
    let cfg: infinigen::settings::Config = match cfg {
        Ok(settings) => match settings.try_deserialize() {
            Ok(settings) => settings,
            Err(err) => {
                eprintln!("Couldn't deserialize settings, exiting: {}", err);
                return ExitCode::FAILURE;
            }
        },
        Err(err) => {
            eprintln!(
                "Couldn't load settings, using default configuration: {:?}",
                err
            );
            infinigen::settings::Config::default()
        }
    };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: LOG_FILTER.into(),
                    level: bevy::log::Level::DEBUG,
                    custom_layer: |_| None,
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
        .run();
    ExitCode::SUCCESS
}
