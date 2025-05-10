#![deny(unstable_features)]
#![deny(unused_features)]
use std::process::ExitCode;

use bevy::DefaultPlugins;
use bevy::app::TaskPoolThreadAssignmentPolicy;
use bevy::log::LogPlugin;
use bevy::prelude::*;
#[cfg(all(feature = "remote", not(target_family = "wasm")))]
use bevy::remote::RemotePlugin;
#[cfg(all(feature = "remote", not(target_family = "wasm")))]
use bevy::remote::http::RemoteHttpPlugin;
use bevy::tasks::available_parallelism;
use bevy::window::{Window, WindowPlugin};
use clap::{Parser, command};
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
#[cfg(not(target_family = "wasm"))]
const CONFIG_PREFIX: &str = "INFINIGEN";
const DEFAULT_LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=info";

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(long, help = "Path to a configuration file")]
    config: Option<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let cfg = match cli.config {
        Some(config_path) => Config::builder().add_source(config::File::with_name(&config_path)),
        None => Config::builder().add_source(infinigen::settings::AppSettings::default()),
    };
    #[cfg(not(target_family = "wasm"))]
    let cfg = cfg.add_source(
        config::Environment::with_prefix(CONFIG_PREFIX)
            .prefix_separator("_")
            .separator("__")
            .try_parsing(true),
    );
    let cfg: infinigen::settings::AppSettings = match cfg.build() {
        Ok(settings) => match settings.try_deserialize() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("Couldn't parse settings: {}", err);
                return ExitCode::from(78);
            }
        },
        Err(err) => {
            eprintln!("Couldn't load settings: {}", err);
            return ExitCode::from(78);
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
                            on_thread_spawn: None,
                            on_thread_destroy: None,
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
                })
                .set(AssetPlugin {
                    file_path: "../extras/assets".into(),
                    ..default()
                }),
            bevy_framepace::FramepacePlugin,
            #[cfg(all(feature = "remote", not(target_family = "wasm")))]
            RemotePlugin::default(),
            #[cfg(all(feature = "remote", not(target_family = "wasm")))]
            RemoteHttpPlugin::default(),
        ))
        .add_plugins((AppPlugin::new(cfg),))
        .add_plugins(infinigen_extras::ExtrasPlugin)
        .run()
    {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}
