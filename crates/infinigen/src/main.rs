#![deny(unstable_features)]
#![deny(unused_features)]
use std::process::ExitCode;

use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::log::LogPlugin;
use bevy::prelude::*;
#[cfg(all(feature = "remote", not(target_family = "wasm")))]
use bevy::remote::http::RemoteHttpPlugin;
#[cfg(all(feature = "remote", not(target_family = "wasm")))]
use bevy::remote::RemotePlugin;
use bevy::tasks::available_parallelism;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use clap::{command, Parser};
use config::Config;
use infinigen_plugins::AppPlugin;
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

#[derive(Parser)]
#[command(version)]
struct Cli;

fn main() -> ExitCode {
    let _cli = Cli::parse();
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
        .add_plugins((
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
            #[cfg(all(feature = "remote", not(target_family = "wasm")))]
            RemotePlugin::default(),
            #[cfg(all(feature = "remote", not(target_family = "wasm")))]
            RemoteHttpPlugin::default(),
        ))
        .add_plugins((AppPlugin::new(cfg),))
        .run()
    {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}
