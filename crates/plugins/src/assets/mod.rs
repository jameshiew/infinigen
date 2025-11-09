use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use infinigen_common::blocks::BlockType;

use crate::AppState;
use crate::registry::BlockDefinition;

mod setup;

pub struct AssetsPlugin;

#[derive(Resource)]
pub struct DefaultBlockTypes(pub Vec<BlockType>);

#[derive(AssetCollection, Resource, Default)]
pub struct BlockAssets {
    #[asset(path = "blocks/types", collection(typed))]
    pub block_definitions: Vec<Handle<BlockDefinition>>,
    #[asset(path = "blocks/textures", collection(typed))]
    pub block_textures: Vec<Handle<Image>>,
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing assets plugin");
        app.add_plugins((RonAssetPlugin::<BlockDefinition>::new(&["block.ron"]),))
            .add_systems(
                OnEnter(AppState::InitializingRegistry),
                setup::initialize_block_assets,
            );

        register_loading_flow(app);
    }
}

#[cfg(not(target_family = "wasm"))]
fn register_loading_flow(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(AppState::LoadingAssets)
            .continue_to_state(AppState::InitializingRegistry)
            .on_failure_continue_to_state(AppState::InitializingRegistry)
            .load_collection::<BlockAssets>(),
    );
}

#[cfg(target_family = "wasm")]
fn register_loading_flow(app: &mut App) {
    app.add_systems(OnEnter(AppState::LoadingAssets), skip_loading_assets);
}

/// For targets not yet supported for loading assets.
#[cfg(target_family = "wasm")]
pub fn skip_loading_assets(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InitializingRegistry);
}
