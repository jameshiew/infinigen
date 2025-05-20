use bevy::asset::LoadedFolder;
use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct AssetFolders {
    #[cfg(not(target_family = "wasm"))]
    pub(crate) block_definitions: Handle<LoadedFolder>,
    pub(crate) block_textures: Handle<LoadedFolder>,
}
