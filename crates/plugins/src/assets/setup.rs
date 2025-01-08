use ahash::AHashMap;
use bevy::asset::{AssetServer, Assets, LoadedFolder};
use bevy::color::Color;
use bevy::image::Image;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{default, AlphaMode, NextState, Res, ResMut, TextureAtlasBuilder};
use infinigen_common::blocks::{BlockVisibility, Face};
use infinigen_common::mesh::textures::{BlockAppearances, FaceAppearance};
use linearize::static_copy_map;
use strum::IntoEnumIterator;

use crate::assets::blocks::{BlockDefinition, BlockRegistry};
use crate::assets::loading::AssetFolders;
use crate::assets::DefaultBlockTypes;
use crate::AppState;

#[allow(clippy::too_many_arguments)]
pub fn setup(
    mut next_state: ResMut<NextState<AppState>>,
    mut registry: ResMut<BlockRegistry>,
    folders: Res<AssetFolders>,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    settings: Res<DefaultBlockTypes>,
    block_definitions: Res<Assets<BlockDefinition>>,
) {
    // block textures
    let mut block_texture_handles_by_name = AHashMap::default();
    let mut block_tatlas_builder = TextureAtlasBuilder::default();
    let (atlas_sources, atlas_layout, texture_atlas) = if let Some(block_texture_folder) =
        loaded_folders.get(&folders.block_textures)
    {
        for handle in block_texture_folder.handles.iter() {
            let handle = handle.clone_weak().typed();
            let path = asset_server.get_path(handle.id());
            if let Some(texture) = textures.get(&handle) {
                tracing::debug!(?path, "Texture found");
                let path = path.unwrap();
                let name = path.path().file_name().unwrap().to_str().unwrap();
                let name = name.trim_end_matches(".png");

                block_texture_handles_by_name.insert(name.to_owned(), handle.clone_weak());
                block_tatlas_builder.add_texture(Some(handle.id()), texture);
            } else {
                tracing::error!("{:?} did not resolve to an `Image` asset.", path,);
                panic!();
            };
        }
        let (atlas_layout, atlas_sources, texture_atlas) = block_tatlas_builder.build().unwrap();
        tracing::debug!(?atlas_layout.size, ?atlas_layout.textures, "Stitched texture atlas");
        let texture_atlas = textures.add(texture_atlas);
        (Some(atlas_sources), Some(atlas_layout), Some(texture_atlas))
    } else {
        tracing::warn!("Block textures were not loaded, falling back to colours");
        (None, None, None)
    };

    let mut block_textures = BlockAppearances::default();
    if let Some(atlas_layout) = atlas_layout.as_ref() {
        block_textures.size = [atlas_layout.size[0] as usize, atlas_layout.size[1] as usize];
    }

    let mut block_definitions: Vec<_> = block_definitions
        .iter()
        .map(|(_handle, block_definition)| block_definition)
        .cloned()
        .collect();
    if block_definitions.is_empty() {
        tracing::warn!("No block definition files found, falling back to default definitions");
        let DefaultBlockTypes(default_block_types) = &*settings;
        block_definitions = default_block_types
            .iter()
            .map(|bt| BlockDefinition::from(bt.clone()))
            .collect();
    }
    // map block definitions in alphabetical order by ID
    // so for the same set of block definitions, we should get the same mapping
    block_definitions.sort();

    for block_definition in block_definitions {
        tracing::debug!(?block_definition, "Block definition found");
        // default to color in case texture is missing
        let color = FaceAppearance::Color {
            r: block_definition.0.color[0] as f32 / 256.,
            g: block_definition.0.color[1] as f32 / 256.,
            b: block_definition.0.color[2] as f32 / 256.,
            a: block_definition.0.color[3] as f32 / 256.,
        };
        let mut appearances = static_copy_map! {
            Face::Top => color,
            Face::Bottom => color,
            Face::Front => color,
            Face::Back => color,
            Face::Left => color,
            Face::Right => color,
        };
        if let Some(ref texture_paths) = block_definition.0.textures {
            let atlas_sources = atlas_sources.as_ref().unwrap();
            let atlas_layout = atlas_layout.as_ref().unwrap();
            for face in Face::iter() {
                // TODO: don't unwrap here
                let texture_handle = block_texture_handles_by_name
                    .get(texture_paths.get(&face).unwrap())
                    .unwrap();
                tracing::debug!(?face, block_id = ?block_definition.0.id, "Found specific texture");
                let tidx = atlas_sources.texture_index(texture_handle).unwrap();
                let tidx = FaceAppearance::Texture {
                    coords: [
                        atlas_layout.textures[tidx].min[0] as usize,
                        atlas_layout.textures[tidx].min[1] as usize,
                    ],
                };
                appearances[face] = tidx;
            }
        };

        let mapped_id = registry.definitions.add(block_definition);
        block_textures.add(
            mapped_id.expect("should have been able to map all block IDs"),
            appearances,
        );
    }
    registry.appearances = block_textures;

    tracing::debug!("Registered all block textures: {:#?}", registry.appearances);

    registry.materials[BlockVisibility::Opaque as usize] = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.75,
        reflectance: 0.25,
        base_color_texture: texture_atlas,
        ..default()
    });
    registry.materials[BlockVisibility::Translucent as usize] = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    next_state.set(AppState::InitializingWorld);
}
