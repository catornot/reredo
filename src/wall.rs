use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{
    map::{GridPos, GRID_CELL_SIZE},
    GameState,
};

#[derive(Debug, Component, PartialEq, Eq)]
pub struct Wall;

pub fn wall_plugin(_app: &mut App) {}

pub fn spawn_wall<'a>(
    commands: &'a mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    grid_pos: GridPos,
) -> EntityCommands<'a> {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::from_size(GRID_CELL_SIZE))),
            material: materials.add(Color::srgb_u8(10, 10, 10)),
            transform: Transform::from_xyz(0., 0., 30.),
            ..default()
        },
        Wall,
        grid_pos,
        StateScoped(GameState::Gaming),
    ))
}
