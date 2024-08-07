use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use bevy::asset::{LoadedFolder, RecursiveDependencyLoadState};
use bevy::{
    prelude::{IntoSystemConfigs, *},
    reflect::TypePath,
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    common::{
        backends::file,
        world::{BlockId, BlockVisibility, ChunkBlockId},
    },
    fake_client::FakeClient,
    mesh::textures::{Face, FaceAppearance, TextureMap},
    settings::Config,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, EnumCount)]
pub enum MaterialType {
    DenseOpaque = 0,
    Translucent,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, TypePath, Asset,
)]
pub struct BlockDefinition {
    pub id: BlockId,
    #[serde(default)]
    pub visibility: BlockVisibility,
    #[serde(default = "default_block_color")]
    pub color: [u8; 4],
    pub textures: Option<BTreeMap<Face, String>>,
}

fn default_block_color() -> [u8; 4] {
    [255, 255, 255, 255]
}

/// Maps mapped block IDs (i.e. bytes) <-> block definitions.
// TODO: overflow checking, safety
#[derive(Debug, Default, Clone)]
pub struct BlockMappings {
    pub by_mapped_id: FxHashMap<ChunkBlockId, BlockDefinition>,
    by_block_id: FxHashMap<BlockId, ChunkBlockId>,
    next_free_mapped_id: ChunkBlockId,
}

impl From<&BlockMappings> for FxHashMap<BlockId, ChunkBlockId> {
    fn from(value: &BlockMappings) -> Self {
        value.by_block_id.clone()
    }
}

impl BlockMappings {
    pub fn get_by_mapped_id(&self, mapped_id: &ChunkBlockId) -> &BlockDefinition {
        self.by_mapped_id.get(mapped_id).unwrap()
    }

    pub fn get_by_block_id(&self, block_id: &BlockId) -> ChunkBlockId {
        *self.by_block_id.get(block_id).unwrap()
    }

    pub fn add(&mut self, block_definition: BlockDefinition) -> ChunkBlockId {
        let mapped_id = self.next_free_mapped_id;
        tracing::info!(?block_definition, ?mapped_id, "Mapping block");
        self.by_block_id
            .insert(block_definition.id.clone(), self.next_free_mapped_id);
        self.by_mapped_id
            .insert(self.next_free_mapped_id, block_definition);
        self.next_free_mapped_id += 1;
        mapped_id
    }
}

#[derive(Default, Resource)]
pub struct Registry {
    materials: [Handle<StandardMaterial>; MaterialType::COUNT],
    block_texture_folder: Handle<LoadedFolder>,
    block_definition_folder: Handle<LoadedFolder>,
    block_textures: TextureMap,
    pub block_mappings: BlockMappings,
}

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadAssets,
    RegisterAssets,
}

pub fn load_assets(mut registry: ResMut<Registry>, asset_server: Res<AssetServer>) {
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
    registry: ResMut<Registry>,
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
        next_state.set(AppState::RegisterAssets);
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
    mut registry: ResMut<Registry>,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    block_definitions: Res<Assets<BlockDefinition>>,
    mut client: ResMut<FakeClient>,
    config: Res<Config>,
) {
    // block textures
    let mut block_texture_handles_by_name = HashMap::new();
    let mut block_tatlas_builder = TextureAtlasBuilder::default();
    for handle in loaded_folders
        .get(&registry.block_texture_folder)
        .unwrap()
        .handles
        .iter()
    {
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
    let (atlas_layout, texture_atlas) = block_tatlas_builder.build().unwrap();
    tracing::info!(?atlas_layout.size, ?atlas_layout.textures, "Stitched texture atlas");
    let texture_atlas = textures.add(texture_atlas);

    let mut block_textures = TextureMap::default();
    block_textures.size = [atlas_layout.size[0] as usize, atlas_layout.size[1] as usize];

    // map block definitions in alphabetical order by ID
    // so for the same set of block definitions, we should get the same mapping
    let mut block_definitions: Vec<_> = block_definitions
        .iter()
        .map(|(_handle, block_definition)| block_definition)
        .cloned()
        .collect();
    block_definitions.sort();

    for block_definition in block_definitions {
        // fall back to color where texture isn't provided
        tracing::info!(?block_definition, "Block definition found");
        let mut faces = [FaceAppearance::Color {
            r: block_definition.color[0] as f32 / 256.,
            g: block_definition.color[1] as f32 / 256.,
            b: block_definition.color[2] as f32 / 256.,
            a: block_definition.color[3] as f32 / 256.,
        }; 6];
        if let Some(ref texture_file_names) = block_definition.textures {
            for face in Face::iter() {
                // TODO: don't unwrap here
                let texture_handle = block_texture_handles_by_name
                    .get(texture_file_names.get(&face).unwrap())
                    .unwrap();
                tracing::info!(?face, ?block_definition.id, "Found specific texture");
                let tidx = atlas_layout.get_texture_index(texture_handle).unwrap();
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
        base_color_texture: Some(texture_atlas),
        ..default()
    });
    registry.materials[MaterialType::Translucent as usize] = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mut worldgen: Box<dyn crate::common::world::WorldGen + Send + Sync> = config.world.into();
    worldgen.initialize((&registry.block_mappings).into());
    let worldgen = Arc::new(RwLock::new(worldgen));
    client.world = match &config.save_dir {
        Some(save_dir) => {
            let backend = file::Backend::new(save_dir.into());
            let worldgen = file::PersistentWorld::new(backend, worldgen);
            Arc::new(RwLock::new(Box::new(worldgen)))
        }
        None => worldgen,
    }
}

impl Registry {
    /// Returns a weak handle to a material.
    pub fn get_material(&self, material_type: MaterialType) -> Handle<StandardMaterial> {
        self.materials[material_type as usize].clone_weak()
    }

    /// Returns a weak handle to the block texture atlas.
    pub fn get_block_textures(&self) -> &TextureMap {
        &self.block_textures
    }
}

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing registry plugin");
        app.init_resource::<Registry>()
            .init_state::<AppState>()
            .add_systems(OnEnter(AppState::LoadAssets), load_assets)
            .add_systems(Update, check_assets.run_if(in_state(AppState::LoadAssets)))
            .add_systems(OnEnter(AppState::RegisterAssets), setup);
    }
}
