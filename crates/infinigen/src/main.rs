#![deny(unstable_features)]
#![deny(unused_features)]
use anyhow::{Context, Result};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use config::Config;
use infinigen::AppPlugin;
#[cfg(all(
    feature = "jemalloc",
    not(target_env = "msvc"),
    not(target_family = "wasm")
))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(
    feature = "jemalloc",
    not(target_env = "msvc"),
    not(target_family = "wasm")
))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const APP_NAME: &str = "infinigen";
const CONFIG_PREFIX: &str = "infinigen_";
const DEFAULT_LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=info";

fn main() -> Result<()> {
    let cfg = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix(CONFIG_PREFIX))
        .build();
    let cfg: infinigen::settings::Config = match cfg {
        Ok(settings) => settings
            .try_deserialize()
            .context("failed to deserialize config file")?,
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
                    filter: DEFAULT_LOG_FILTER.into(),
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
    Ok(())
}
