use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};

use crate::{
    map::{GridPos, GRID_CELL_SIZE},
    snake::SnakeSize,
    GameState,
};

#[derive(Debug, Component, PartialEq, Eq)]
pub struct Spike;

pub fn spike_plugin(app: &mut App) {
    app.add_systems(
        Update,
        activate_spike
            .run_if(on_timer(Duration::from_secs(3)))
            .run_if(in_state(GameState::Gaming)),
    );
}

pub fn spawn_spike(
    commands: &mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    grid_pos: GridPos,
) {
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::from_size(GRID_CELL_SIZE))),
                material: materials.add(Color::srgb_u8(50, 50, 50)),
                transform: Transform::from_xyz(0., 0., 50.),
                ..default()
            },
            Spike,
            grid_pos,
            StateScoped(GameState::Gaming),
        ))
        .with_children(|parent| {
            for [x, y] in [[-1., 1.], [1., -1.], [-1., -1.], [1., 1.]] {
                parent.spawn((
                    {
                        MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(meshes.add(Circle::new(GRID_CELL_SIZE.x / 8.))),
                            material: materials.add(Color::srgba_u8(60, 60, 60, 255)),
                            transform: Transform::from_xyz(
                                x * GRID_CELL_SIZE.x / 5.,
                                y * GRID_CELL_SIZE.y / 5.,
                                50.,
                            ),
                            ..default()
                        }
                    },
                    Spike,
                ));
            }
        });
}

fn activate_spike(
    spikes: Query<(&Children, &GridPos), (With<Spike>, Without<Parent>)>,
    mut children: Query<&mut Transform, (With<Parent>, With<Spike>, Without<Children>)>,
    snakes: Query<(Entity, &GridPos), (With<SnakeSize>, Without<Spike>)>,
    mut commands: Commands,
    mut is_up: Local<bool>,
) {
    *is_up = !*is_up;
    let is_up = *is_up;

    for child in spikes.iter().flat_map(|(children, _)| children.iter()) {
        if let Ok(mut transform) = children.get_mut(*child) {
            if is_up {
                transform.scale.x = 1.5;
                transform.scale.y = 1.5;
            } else {
                transform.scale.x = 1.;
                transform.scale.y = 1.;
            }
        }
    }

    if is_up {
        for dead_pieces in spikes.iter().filter_map(|(_, spike_pos)| {
            snakes
                .iter()
                .find_map(|(ent, pos)| (*spike_pos == *pos).then_some(ent))
        }) {
            commands.entity(dead_pieces).despawn_recursive();
        }
    }
}
