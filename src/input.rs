use bevy::prelude::*;

use crate::{snake::CanMove, GameState};

#[derive(Debug, Event)]
pub struct MoveEvent(pub Vec2);

#[derive(Debug, Resource)]
pub struct KeyBuffer(pub Vec<KeyCode>);

pub fn input_plugin(app: &mut App) {
    app.insert_resource(KeyBuffer(Vec::new()))
        .add_event::<MoveEvent>()
        .add_systems(Update, handle_keys.run_if(in_state(GameState::Gaming)));
}

fn handle_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut move_event: EventWriter<MoveEvent>,
    mut cycle_buffer: ResMut<KeyBuffer>,
    snake_pieces: Query<Entity, With<CanMove>>,
) {
    if snake_pieces.iter().next().is_none() {
        return;
    }

    let move_dir = [
        keys.pressed(KeyCode::ArrowRight) as i32 as f32
            - keys.pressed(KeyCode::ArrowLeft) as i32 as f32,
        keys.pressed(KeyCode::ArrowUp) as i32 as f32
            - keys.pressed(KeyCode::ArrowDown) as i32 as f32,
    ];

    if !(move_dir[0] == 0. && move_dir[1] == 0.) && move_dir[0] != move_dir[1] {
        move_event.send(MoveEvent(Vec2::from_array(move_dir)));
    }

    cycle_buffer.0.extend(
        keys.get_just_pressed()
            .map(|key| key.to_owned())
            .filter(|key| {
                matches!(
                    *key,
                    KeyCode::Digit0
                        | KeyCode::Digit1
                        | KeyCode::Digit2
                        | KeyCode::Digit3
                        | KeyCode::Digit4
                        | KeyCode::Digit5
                        | KeyCode::Digit6
                        | KeyCode::Digit7
                        | KeyCode::Digit8
                        | KeyCode::Digit9
                        | KeyCode::KeyC
                        | KeyCode::KeyW
                        | KeyCode::KeyU
                        | KeyCode::Enter
                )
            }),
    )
}
