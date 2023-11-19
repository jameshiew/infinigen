/// Adapted from lines example of Bevy
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

use crate::common::chunks::CHUNK_SIZE_F32;

#[derive(Debug, Default, Eq, PartialEq, Resource)]
pub struct ChunkBordersState {
    pub show: bool,
}

#[derive(Component)]
pub struct ChunkBorder;

/// A list of lines with a start and end position. From lines bevy example.
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone, TypePath)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
pub struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

pub fn toggle(
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    mut chunk_borders: ResMut<ChunkBordersState>,
    existing_chunk_borders: Query<Entity, With<ChunkBorder>>,
) {
    for key in keys.get_just_pressed() {
        if key == &KeyCode::F9 {
            if chunk_borders.show {
                for eid in existing_chunk_borders.iter() {
                    commands.entity(eid).despawn();
                }
            } else {
                // TODO: disable shadows of cube chunk border lines
                // TODO: share mesh and material between entities
                // TODO: the lines for all chunks borders should be in one single mesh to be efficient
                // TODO: this only works for the origin
                let chunk_cube_lines = vec![
                    // XY face
                    (Vec3::ZERO, Vec3::new(CHUNK_SIZE_F32, 0.0, 0.0)),
                    (
                        Vec3::new(CHUNK_SIZE_F32, 0.0, 0.0),
                        Vec3::new(CHUNK_SIZE_F32, CHUNK_SIZE_F32, 0.0),
                    ),
                    (
                        Vec3::new(CHUNK_SIZE_F32, CHUNK_SIZE_F32, 0.0),
                        Vec3::new(0.0, CHUNK_SIZE_F32, 0.0),
                    ),
                    (Vec3::new(0.0, CHUNK_SIZE_F32, 0.0), Vec3::ZERO),
                    // opposite face
                    (
                        Vec3::new(0.0, 0.0, CHUNK_SIZE_F32),
                        Vec3::new(CHUNK_SIZE_F32, 0.0, CHUNK_SIZE_F32),
                    ),
                    (
                        Vec3::new(CHUNK_SIZE_F32, 0.0, CHUNK_SIZE_F32),
                        Vec3::new(CHUNK_SIZE_F32, CHUNK_SIZE_F32, CHUNK_SIZE_F32),
                    ),
                    (
                        Vec3::new(CHUNK_SIZE_F32, CHUNK_SIZE_F32, CHUNK_SIZE_F32),
                        Vec3::new(0.0, CHUNK_SIZE_F32, CHUNK_SIZE_F32),
                    ),
                    (
                        Vec3::new(0.0, CHUNK_SIZE_F32, CHUNK_SIZE_F32),
                        Vec3::new(0.0, 0.0, CHUNK_SIZE_F32),
                    ),
                    // the four lines connecting these two faces
                    (Vec3::ZERO, Vec3::new(0.0, 0.0, CHUNK_SIZE_F32)),
                    (
                        Vec3::new(CHUNK_SIZE_F32, 0.0, 0.0),
                        Vec3::new(CHUNK_SIZE_F32, 0.0, CHUNK_SIZE_F32),
                    ),
                    (
                        Vec3::new(CHUNK_SIZE_F32, CHUNK_SIZE_F32, 0.0),
                        Vec3::new(CHUNK_SIZE_F32, CHUNK_SIZE_F32, CHUNK_SIZE_F32),
                    ),
                    (
                        Vec3::new(0.0, CHUNK_SIZE_F32, 0.0),
                        Vec3::new(0.0, CHUNK_SIZE_F32, CHUNK_SIZE_F32),
                    ),
                ];
                for x in -3..3 {
                    for y in -3..3 {
                        for z in -3..3 {
                            let transform = Transform::from_xyz(
                                x as f32 * CHUNK_SIZE_F32,
                                y as f32 * CHUNK_SIZE_F32,
                                z as f32 * CHUNK_SIZE_F32,
                            );
                            let mesh = Mesh::from(LineList {
                                lines: chunk_cube_lines.clone(),
                            });
                            commands.spawn((
                                MaterialMeshBundle {
                                    mesh: meshes.add(mesh),
                                    transform,
                                    material: materials.add(LineMaterial {
                                        color: Color::WHITE,
                                    }),
                                    ..default()
                                },
                                ChunkBorder,
                            ));
                        }
                    }
                }
            }
            chunk_borders.show = !chunk_borders.show;
            tracing::info!(%chunk_borders.show, "Chunk borders toggled");
        }
    }
}
