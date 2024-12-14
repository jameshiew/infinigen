use bevy::asset::LoadedFolder;
use bevy::prelude::*;

#[derive(Default, Resource)]
pub(crate) struct AssetFolders {
    pub(crate) block_definitions: Handle<LoadedFolder>,
    pub(crate) block_textures: Handle<LoadedFolder>,
}