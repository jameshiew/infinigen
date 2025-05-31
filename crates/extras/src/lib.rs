#![allow(clippy::tuple_array_conversions)] // gives false positives

//! Default world generators and block definitions.
use std::str::FromStr;

use bevy::prelude::*;
use infinigen_plugins::assets::DefaultBlockTypes;
use infinigen_plugins::world::WorldInitializer;
use worldgen::WorldGenTypes;

pub mod blocks;
pub mod worldgen;

pub struct ExtrasPlugin;

impl Plugin for ExtrasPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing extras plugin");
        app.insert_resource(DefaultBlockTypes(crate::blocks::block_types().collect()))
            .insert_resource(WorldInitializer(Box::new(
                move |world_gen_name: &str, seed, palette| {
                    let world_gen_type =
                        WorldGenTypes::from_str(world_gen_name).unwrap_or_else(|_| {
                            panic!("couldn't parse world gen type from {world_gen_name}")
                        });
                    world_gen_type.as_world_gen(seed, palette)
                },
            )));
    }
}
