use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use blocks::{BlockDefinition, BlockRegistry};
use infinigen_common::blocks::BlockType;
use loading::AssetFolders;

use crate::AppState;

pub mod blocks;
mod loading;
mod setup;

pub struct AssetsPlugin;

#[derive(Resource)]
pub struct AssetSettings {
    pub default_block_types: Vec<BlockType>,
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing assets plugin");
        app.add_plugins((RonAssetPlugin::<BlockDefinition>::new(&["block.ron"]),))
            .init_resource::<AssetFolders>()
            .init_resource::<BlockRegistry>()
            .add_systems(OnEnter(AppState::InitializingRegistry), setup::setup);

        #[cfg(not(target_family = "wasm"))]
        app.add_systems(OnEnter(AppState::LoadingAssets), load_assets)
            .add_systems(
                FixedUpdate,
                (check_assets.run_if(in_state(AppState::LoadingAssets)),),
            );

        #[cfg(target_family = "wasm")]
        app.add_systems(OnEnter(AppState::LoadingAssets), skip_loading_assets);
    }
}

/// For targets not yet supported for loading assets.
#[cfg(target_family = "wasm")]
pub fn skip_loading_assets(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InitializingRegistry);
}

#[cfg(not(target_family = "wasm"))]
fn load_assets(mut folders: ResMut<AssetFolders>, server: Res<AssetServer>) {
    folders.block_textures = server.load_folder("blocks/textures/");
    folders.block_definitions = server.load_folder("blocks/types/");
    tracing::debug!(
        "Loading assets: textures: {}, block definitions: {}",
        folders.block_textures.path().unwrap(),
        folders.block_definitions.path().unwrap(),
    );
}

#[cfg(not(target_family = "wasm"))]
fn check_assets(
    mut next_state: ResMut<NextState<AppState>>,
    folders: ResMut<AssetFolders>,
    asset_server: Res<AssetServer>,
) {
    use bevy::asset::RecursiveDependencyLoadState;

    let blockdef_load_state =
        asset_server.get_recursive_dependency_load_state(&folders.block_definitions);

    let block_definitions_loaded = match blockdef_load_state {
        Some(RecursiveDependencyLoadState::Loaded) => true,
        Some(RecursiveDependencyLoadState::Failed(_)) => {
            tracing::info!("Couldn't load block definitions, won't use assets");
            next_state.set(AppState::InitializingRegistry);
            return;
        }
        _ => false,
    };
    if block_definitions_loaded {
        tracing::debug!("Finished loading block definitions");
    }

    let blocktex_load_state =
        asset_server.get_recursive_dependency_load_state(&folders.block_textures);
    let block_textures_loaded = match blocktex_load_state {
        Some(RecursiveDependencyLoadState::Loaded) => true,
        Some(RecursiveDependencyLoadState::Failed(_)) => {
            tracing::info!("Couldn't load block textures, won't use assets");
            next_state.set(AppState::InitializingRegistry);
            return;
        }
        _ => false,
    };
    if block_textures_loaded {
        tracing::debug!("Finished loading block textures");
    }

    if block_definitions_loaded && block_textures_loaded {
        next_state.set(AppState::InitializingRegistry);
    } else {
        tracing::debug!(
            "Loading block definitions: {:?}, block textures: {:?}",
            blockdef_load_state,
            blocktex_load_state
        );
    }
}
