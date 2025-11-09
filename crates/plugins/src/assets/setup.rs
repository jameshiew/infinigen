use ahash::AHashMap;
use bevy::asset::{AssetPath, AssetServer, Assets};
use bevy::prelude::{
    AlphaMode, BevyError, Color, Handle, Image, NextState, Res, ResMut, Result, StandardMaterial,
    TextureAtlasBuilder, TextureAtlasLayout, TextureAtlasSources, default,
};
use infinigen_common::blocks::{BlockVisibility, Face};
use infinigen_common::mesh::textures::{BlockAppearances, FaceAppearance};
use linearize::{StaticCopyMap, static_copy_map};
use strum::IntoEnumIterator;

use crate::AppState;
use crate::assets::{BlockAssets, DefaultBlockTypes};
use crate::registry::{BlockDefinition, BlockRegistry};

pub fn initialize_block_assets(
    mut next_state: ResMut<NextState<AppState>>,
    mut registry: ResMut<BlockRegistry>,
    block_assets: Option<Res<BlockAssets>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    settings: Res<DefaultBlockTypes>,
    block_definitions: Res<Assets<BlockDefinition>>,
) -> Result {
    let block_assets = block_assets.as_deref();
    let prepared_textures =
        prepare_block_textures(block_assets, asset_server.as_ref(), textures.as_mut())?;

    let mut block_textures = BlockAppearances::default();
    if let Some(prepared) = prepared_textures.as_ref() {
        block_textures.size = [
            prepared.atlas_layout.size.x as usize,
            prepared.atlas_layout.size.y as usize,
        ];
    }

    let block_definitions =
        load_block_definitions(block_assets, block_definitions.as_ref(), settings.as_ref());

    register_blocks(
        block_definitions,
        registry.as_mut(),
        &mut block_textures,
        prepared_textures.as_ref(),
    );

    registry.appearances = block_textures;
    tracing::debug!("Registered all block textures: {:#?}", registry.appearances);

    configure_materials(
        registry.as_mut(),
        materials.as_mut(),
        prepared_textures
            .as_ref()
            .map(|prepared| prepared.atlas_texture.clone()),
    );

    next_state.set(AppState::InitializingWorld);
    Ok(())
}

#[derive(Debug)]
struct PreparedTextures {
    handles_by_name: AHashMap<String, Handle<Image>>,
    atlas_sources: TextureAtlasSources,
    atlas_layout: TextureAtlasLayout,
    atlas_texture: Handle<Image>,
}

impl PreparedTextures {
    fn coords_for_name(&self, texture_name: &str) -> Option<[usize; 2]> {
        let handle = self.handles_by_name.get(texture_name)?;
        let index = self.atlas_sources.texture_index(handle)?;
        let rect = self.atlas_layout.textures.get(index)?;
        Some([rect.min.x as usize, rect.min.y as usize])
    }
}

fn prepare_block_textures(
    block_assets: Option<&BlockAssets>,
    asset_server: &AssetServer,
    textures: &mut Assets<Image>,
) -> Result<Option<PreparedTextures>> {
    let Some(block_assets) = block_assets else {
        tracing::warn!("Block textures were not loaded, falling back to colours");
        return Ok(None);
    };

    if block_assets.block_textures.is_empty() {
        tracing::warn!("Block textures were not loaded, falling back to colours");
        return Ok(None);
    }

    let mut handles_by_name = AHashMap::with_capacity(block_assets.block_textures.len());
    let mut atlas_builder = TextureAtlasBuilder::default();

    for handle in &block_assets.block_textures {
        let Some(asset_path) = asset_server.get_path(handle.id()) else {
            tracing::warn!(?handle, "Skipping texture without a known path");
            continue;
        };

        let Some(texture) = textures.get(handle) else {
            return Err(BevyError::from(format!(
                "{asset_path:?} did not resolve to an `Image` asset.",
            )));
        };

        let Some(texture_name) = texture_name_from_path(&asset_path) else {
            tracing::warn!(path = ?asset_path, "Skipping texture without a valid filename");
            continue;
        };

        tracing::debug!(path = ?asset_path, "Texture found");
        handles_by_name.insert(texture_name, handle.clone());
        atlas_builder.add_texture(Some(handle.id()), texture);
    }

    if handles_by_name.is_empty() {
        tracing::warn!("Block textures were not loaded, falling back to colours");
        return Ok(None);
    }

    let (atlas_layout, atlas_sources, atlas_image) = atlas_builder.build()?;
    tracing::debug!(
        size = ?atlas_layout.size,
        textures = ?atlas_layout.textures,
        "Stitched texture atlas"
    );
    let atlas_texture = textures.add(atlas_image);

    Ok(Some(PreparedTextures {
        handles_by_name,
        atlas_sources,
        atlas_layout,
        atlas_texture,
    }))
}

fn texture_name_from_path(path: &AssetPath) -> Option<String> {
    path.path()
        .file_name()
        .and_then(|file| file.to_str())
        .map(|name| name.trim_end_matches(".png").to_owned())
}

fn load_block_definitions(
    block_assets: Option<&BlockAssets>,
    block_definitions: &Assets<BlockDefinition>,
    defaults: &DefaultBlockTypes,
) -> Vec<BlockDefinition> {
    let mut loaded = block_assets
        .map(|assets| collect_block_definitions(assets, block_definitions))
        .unwrap_or_default();

    if loaded.is_empty() {
        tracing::warn!("No block definition files found, falling back to default definitions");
        loaded = defaults
            .0
            .iter()
            .cloned()
            .map(BlockDefinition::from)
            .collect();
    }

    loaded.sort();
    loaded
}

fn collect_block_definitions(
    block_assets: &BlockAssets,
    block_definitions: &Assets<BlockDefinition>,
) -> Vec<BlockDefinition> {
    let mut definitions = Vec::with_capacity(block_assets.block_definitions.len());
    for handle in &block_assets.block_definitions {
        match block_definitions.get(handle) {
            Some(definition) => definitions.push(definition.clone()),
            None => tracing::warn!(?handle, "Skipping missing block definition"),
        }
    }
    definitions
}

fn register_blocks(
    block_definitions: Vec<BlockDefinition>,
    registry: &mut BlockRegistry,
    block_textures: &mut BlockAppearances,
    prepared_textures: Option<&PreparedTextures>,
) {
    for block_definition in block_definitions {
        tracing::debug!(?block_definition, "Block definition found");
        let appearances = build_face_appearances(&block_definition, prepared_textures);
        let mapped_id = registry
            .definitions
            .add(block_definition)
            .expect("should have been able to map all block IDs");
        block_textures.add(mapped_id, appearances);
    }
}

fn build_face_appearances(
    block_definition: &BlockDefinition,
    prepared_textures: Option<&PreparedTextures>,
) -> StaticCopyMap<Face, FaceAppearance> {
    let mut appearances = default_face_appearances(block_definition);

    if let (Some(textures), Some(texture_paths)) =
        (prepared_textures, block_definition.0.textures.as_ref())
    {
        for face in Face::iter() {
            if let Some(texture_name) = texture_paths.get(&face)
                && let Some(coords) = textures.coords_for_name(texture_name.as_str())
            {
                tracing::debug!(
                    ?face,
                    block_id = ?block_definition.0.id,
                    "Found specific texture"
                );
                appearances[face] = FaceAppearance::Texture { coords };
            }
        }
    }

    appearances
}

fn default_face_appearances(
    block_definition: &BlockDefinition,
) -> StaticCopyMap<Face, FaceAppearance> {
    let color = block_color(block_definition);
    static_copy_map! {
        Face::Top => color,
        Face::Bottom => color,
        Face::Front => color,
        Face::Back => color,
        Face::Left => color,
        Face::Right => color,
    }
}

fn block_color(block_definition: &BlockDefinition) -> FaceAppearance {
    FaceAppearance::Color {
        r: block_definition.0.color[0] as f32 / 255.,
        g: block_definition.0.color[1] as f32 / 255.,
        b: block_definition.0.color[2] as f32 / 255.,
        a: block_definition.0.color[3] as f32 / 255.,
    }
}

fn configure_materials(
    registry: &mut BlockRegistry,
    materials: &mut Assets<StandardMaterial>,
    texture_atlas: Option<Handle<Image>>,
) {
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
}
