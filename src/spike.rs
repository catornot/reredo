use bevy::prelude::*;

use crate::{map::GridPos, snake::SnakeSize, sounds::SoundEvent, GameState};
#[derive(Debug, Resource)]
pub struct SpikeSprites {
    spike: Handle<Image>,
    spike_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for SpikeSprites {
    fn from_world(world: &mut World) -> Self {
        let spike_layout =
            TextureAtlasLayout::from_grid(UVec2::new(50, 50), 4, 1, Some(UVec2::new(1, 1)), None);

        let handle = {
            let mut texture_atlases = world
                .get_resource_mut::<Assets<TextureAtlasLayout>>()
                .expect("texture atlases assets should exist");
            texture_atlases.add(spike_layout)
        };

        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        Self {
            spike: asset_server.load("dev/spike.png"),
            spike_layout: handle,
        }
    }
}

#[derive(Debug, Component, PartialEq, Eq)]
pub struct Spike;

#[derive(Debug, Resource)]
struct SpikeTimer(Timer);

pub fn spike_plugin(app: &mut App) {
    app.insert_resource(SpikeTimer(Timer::from_seconds(3.5, TimerMode::Repeating)))
        .init_resource::<SpikeSprites>()
        .add_systems(
            Update,
            activate_spike
                .run_if(in_state(GameState::Gaming))
                .run_if(|spikes: Query<(), With<Spike>>| spikes.iter().next().is_some()),
        );
}

pub fn spawn_spike(
    commands: &mut Commands<'_, '_>,
    spike_sprites: &SpikeSprites,
    grid_pos: GridPos,
) {
    commands.spawn((
        SpriteBundle {
            texture: spike_sprites.spike.clone(),
            transform: Transform::from_xyz(0., 0., 30.),
            ..default()
        },
        TextureAtlas::from(spike_sprites.spike_layout.clone()),
        Spike,
        grid_pos,
        StateScoped(GameState::Gaming),
    ));
}

fn activate_spike(
    mut spikes: Query<(&mut TextureAtlas, &GridPos), With<Spike>>,
    snakes: Query<(Entity, &GridPos), (With<SnakeSize>, Without<Spike>)>,
    mut commands: Commands,
    mut timer: ResMut<SpikeTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    for mut atlas in spikes.iter_mut().map(|(atlas, _)| atlas) {
        if timer.0.elapsed_secs() <= 0.5 && atlas.index != 2 {
            if atlas.index > 2 {
                atlas.index = 0;
            } else {
                atlas.index += 1;
            }
        } else if timer.0.elapsed_secs() > 0.5 && atlas.index != 0 {
            if atlas.index < 3 {
                atlas.index += 1;
            } else {
                atlas.index = 0;
            }
        }
    }

    if timer.0.elapsed_secs() <= 0.5 {
        if timer.0.times_finished_this_tick() > 0 {
            commands.trigger(SoundEvent::Spike);
        }

        for dead_pieces in spikes.iter().filter_map(|(_, spike_pos)| {
            snakes
                .iter()
                .find_map(|(ent, pos)| (*spike_pos == *pos).then_some(ent))
        }) {
            commands.entity(dead_pieces).despawn_recursive();
        }
    }
}
