use std::process::ExitCode;

use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{Window, WindowPlugin, WindowResolution},
    DefaultPlugins,
};
use config::Config;
use infinigen::{extras::worldgen::WorldGenTypes, ClientPlugin};

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
            infinigen::settings::Config {
                hview_distance: 8,
                vview_distance: 8,
                world: WorldGenTypes::MountainIslands,
                wx: -1283.,
                wy: 140.,
                wz: -1752.,
                rotation_x: -0.08,
                rotation_y: -0.9,
                rotation_z: -0.4,
                rotation_w: 0.18,
                ..default()
            }
        }
    };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: LOG_FILTER.into(),
                    level: bevy::log::Level::DEBUG,
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1920., 1080.),
                        title: APP_NAME.into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugin(ClientPlugin::new(cfg))
        .run();
    ExitCode::SUCCESS
}
