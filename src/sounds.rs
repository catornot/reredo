use bevy::{audio::Volume, prelude::*};

use crate::snake::Move;

#[derive(Debug, Default)]
struct PitchLevels {
    index: usize,
}

impl PitchLevels {
    #[rustfmt::skip]    fn next(&mut self) -> f32 {
        const PITCHES: [f32;20] = [0.729,0.926,0.717,0.380,0.021,0.024,0.546,0.930,0.085,0.907,0.368,0.130,0.382,0.237,0.014,0.327,0.698,0.940,0.621,0.225];

        self.index = if self.index < PITCHES.len() - 1 { self.index + 1 } else { 0 };
        PITCHES[self.index]
    }
}

#[derive(Debug, Event)]
pub enum SoundEvent {
    SnakeMove,
    PressurePlate,
    Select,
    Exit,
    #[allow(dead_code)]
    Wind,
    Spike,
}

#[derive(Debug, Resource)]
struct AudioFiles {
    snake_move: Handle<AudioSource>,
    pressure_plate: Handle<AudioSource>,
    comfirm: Handle<AudioSource>,
    exit: Handle<AudioSource>,
    wind: Handle<AudioSource>,
    spike: Handle<AudioSource>,
}

impl FromWorld for AudioFiles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        AudioFiles {
            snake_move: asset_server.load("sounds/snek_move.wav"),
            pressure_plate: asset_server.load("sounds/plate.wav"),
            comfirm: asset_server.load("sounds/confirm.wav"),
            exit: asset_server.load("sounds/end.wav"),
            wind: asset_server.load("sounds/wind.wav"),
            spike: asset_server.load("sounds/hit.wav"),
        }
    }
}

pub fn sounds_plugin(app: &mut App) {
    app.add_event::<SoundEvent>()
        .init_resource::<AudioFiles>()
        .observe(on_sound)
        .observe(create_sound_on_new_snake);
}

fn create_sound_on_new_snake(_: Trigger<OnAdd, Move>, mut commands: Commands) {
    commands.trigger(SoundEvent::SnakeMove);
}

fn on_sound(
    trigger: Trigger<SoundEvent>,
    mut commands: Commands,
    source: Res<AudioFiles>,
    mut pitches: Local<PitchLevels>,
) {
    match trigger.event() {
        SoundEvent::SnakeMove => {
            _ = commands.spawn((AudioBundle {
                source: source.snake_move.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: Volume::new(0.4),
                    speed: 1.3 + pitches.next(),
                    ..default()
                },
            },))
        }
        SoundEvent::PressurePlate => {
            _ = commands.spawn(AudioBundle {
                source: source.pressure_plate.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..default()
                },
            })
        }
        SoundEvent::Select => {
            _ = commands.spawn(AudioBundle {
                source: source.comfirm.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: Volume::new(0.5),
                    speed: 2.0,
                    ..default()
                },
            })
        }
        SoundEvent::Exit => {
            _ = commands.spawn(AudioBundle {
                source: source.exit.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..default()
                },
            })
        }
        SoundEvent::Wind => {
            _ = commands.spawn(AudioBundle {
                source: source.wind.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..default()
                },
            })
        }
        SoundEvent::Spike => {
            _ = commands.spawn(AudioBundle {
                source: source.spike.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: Volume::new(0.6),
                    speed: 1.2,
                    ..default()
                },
            })
        }
    }
}
