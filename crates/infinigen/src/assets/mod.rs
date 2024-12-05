use crate::{settings::Config, world::World, AppState};
use bevy::asset::{LoadedFolder, RecursiveDependencyLoadState};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use blocks::{default_block_definitions, BlockDefinition, BlockRegistry, MaterialType};
use infinigen_common::mesh::textures::{Face, FaceAppearance, TextureMap};
use std::{collections::HashMap, sync::Arc};
use strum::IntoEnumIterator;

pub mod blocks;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing assets plugin");
        app.add_plugins((RonAssetPlugin::<BlockDefinition>::new(&["block.ron"]),))
            .init_resource::<BlockRegistry>()
            .add_systems(OnEnter(AppState::InitializingRegistry), setup);

        #[cfg(not(target_family = "wasm"))]
        app.add_systems(OnEnter(AppState::LoadingAssets), load_assets)
            .add_systems(
                Update,
                (check_assets.run_if(in_state(AppState::LoadingAssets)),),
            );

        #[cfg(target_family = "wasm")]
        app.add_systems(OnEnter(AppState::LoadingAssets), skip_loading_assets);
    }
}

/// For targets not yet supported for loading assets.
pub fn skip_loading_assets(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InitializingRegistry);
}

pub fn load_assets(mut registry: ResMut<BlockRegistry>, asset_server: Res<AssetServer>) {
    registry.block_texture_folder = asset_server.load_folder("blocks/textures/");
    registry.block_definition_folder = asset_server.load_folder("blocks/types/");
    tracing::info!(
        "Loading assets: textures: {}, block definitions: {}",
        registry.block_texture_folder.path().unwrap(),
        registry.block_definition_folder.path().unwrap(),
    );
}

pub fn check_assets(
    mut next_state: ResMut<NextState<AppState>>,
    registry: ResMut<BlockRegistry>,
    asset_server: Res<AssetServer>,
) {
    let mut block_definitions_loaded = false;
    let blockdef_load_state =
        asset_server.get_recursive_dependency_load_state(&registry.block_definition_folder);
    if let Some(RecursiveDependencyLoadState::Loaded) = blockdef_load_state {
        tracing::info!("Finished loading block definitions");
        block_definitions_loaded = true;
    }

    let mut block_textures_loaded = false;
    let blocktex_load_state =
        asset_server.get_recursive_dependency_load_state(&registry.block_texture_folder);
    if let Some(RecursiveDependencyLoadState::Loaded) = blocktex_load_state {
        tracing::info!("Finished loading block textures");
        block_textures_loaded = true;
    }

    if block_definitions_loaded && block_textures_loaded {
        next_state.set(AppState::InitializingRegistry);
    } else {
        tracing::info!(
            "Loading block definitions: {:?}, block textures: {:?}",
            blockdef_load_state,
            blocktex_load_state
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup(
    mut next_state: ResMut<NextState<AppState>>,
    mut registry: ResMut<BlockRegistry>,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    block_definitions: Res<Assets<BlockDefinition>>,
    mut world: ResMut<World>,
    config: Res<Config>,
) {
    // block textures
    let mut block_texture_handles_by_name = HashMap::new();
    let mut block_tatlas_builder = TextureAtlasBuilder::default();
    let (atlas_sources, atlas_layout, texture_atlas) = if let Some(block_texture_folder) =
        loaded_folders.get(&registry.block_texture_folder)
    {
        for handle in block_texture_folder.handles.iter() {
            let handle = handle.clone_weak().typed();
            let path = asset_server.get_path(handle.id());
            if let Some(texture) = textures.get(&handle) {
                tracing::info!(?path, "Texture found");
                let path = path.unwrap();
                let name = path.path().file_name().unwrap().to_str().unwrap();
                let name = name.trim_end_matches(".png");

                block_texture_handles_by_name.insert(name.to_owned(), handle.clone_weak());
                block_tatlas_builder.add_texture(Some(handle.id()), texture);
            } else {
                tracing::warn!("{:?} did not resolve to an `Image` asset.", path,);
                panic!();
            };
        }
        let (atlas_layout, atlas_sources, texture_atlas) = block_tatlas_builder.build().unwrap();
        tracing::info!(?atlas_layout.size, ?atlas_layout.textures, "Stitched texture atlas");
        let texture_atlas = textures.add(texture_atlas);
        (Some(atlas_sources), Some(atlas_layout), Some(texture_atlas))
    } else {
        tracing::warn!("Block textures were not loaded");
        (None, None, None)
    };

    let mut block_textures = TextureMap::default();
    if let Some(atlas_layout) = atlas_layout.as_ref() {
        block_textures.size = [atlas_layout.size[0] as usize, atlas_layout.size[1] as usize];
    }

    let mut block_definitions: Vec<_> = block_definitions
        .iter()
        .map(|(_handle, block_definition)| block_definition)
        .cloned()
        .collect();
    if block_definitions.is_empty() {
        tracing::warn!("No block definition files found, falling back to defaults");
        block_definitions = default_block_definitions();
    }
    // map block definitions in alphabetical order by ID
    // so for the same set of block definitions, we should get the same mapping
    block_definitions.sort();

    for block_definition in block_definitions {
        tracing::info!(?block_definition, "Block definition found");
        // default to color in case texture is missing
        let mut faces = [FaceAppearance::Color {
            r: block_definition.color[0] as f32 / 256.,
            g: block_definition.color[1] as f32 / 256.,
            b: block_definition.color[2] as f32 / 256.,
            a: block_definition.color[3] as f32 / 256.,
        }; 6];
        if let Some(ref texture_file_names) = block_definition.textures {
            let atlas_sources = atlas_sources.as_ref().unwrap();
            let atlas_layout = atlas_layout.as_ref().unwrap().clone();
            for face in Face::iter() {
                // TODO: don't unwrap here
                let texture_handle = block_texture_handles_by_name
                    .get(texture_file_names.get(&face).unwrap())
                    .unwrap();
                tracing::info!(?face, ?block_definition.id, "Found specific texture");
                let tidx = atlas_sources.texture_index(texture_handle).unwrap();
                let tidx = FaceAppearance::Texture {
                    coords: [
                        atlas_layout.textures[tidx].min[0] as usize,
                        atlas_layout.textures[tidx].min[1] as usize,
                    ],
                };
                faces[face as usize] = tidx;
            }
        };

        let mapped_id = registry.block_mappings.add(block_definition);
        block_textures.add(mapped_id, faces);
    }
    registry.block_textures = block_textures;

    tracing::debug!(
        "Registered all block textures: {:#?}",
        registry.block_textures
    );

    registry.materials[MaterialType::DenseOpaque as usize] = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.75,
        reflectance: 0.25,
        base_color_texture: texture_atlas,
        ..default()
    });
    registry.materials[MaterialType::Translucent as usize] = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mut worldgen: Box<dyn infinigen_common::world::WorldGen + Send + Sync> =
        config.world.into();
    worldgen.initialize((&registry.block_mappings).into());
    world.generator = Arc::new(worldgen);
    next_state.set(AppState::MainGame);
}
