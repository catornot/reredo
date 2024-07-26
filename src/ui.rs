use bevy::prelude::*;

use crate::{input::KeyBuffer, title::UiResources, GameState};

#[derive(Debug, Component)]
struct BufferText;

#[derive(Debug, Component)]
struct CyclesText;

#[derive(Debug, Component)]
struct RewindsText;

pub fn game_ui_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gaming), setup_ui)
        .add_systems(Update, update_buffer.run_if(in_state(GameState::Gaming)));
}

fn setup_ui(mut commands: Commands, ui_resources: Res<UiResources>) {
    let text_style = TextStyle {
        font_size: 20.0,
        color: Color::srgba_u8(153, 153, 153, 255),
        font: ui_resources.font.clone(),
    };

    let text_style_red = TextStyle {
        color: Color::srgba_u8(255, 153, 153, 255),
        ..text_style.clone()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(400.),
                height: Val::Px(40.),
                border: UiRect::all(Val::Px(2.)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Start,
                top: Val::Px(0.),
                left: Val::Px(0.),
                ..default()
            },
            ..default()
        })
        .insert(StateScoped(GameState::Gaming))
        .with_children(|parent| {
            parent
                .spawn(
                    TextBundle::from_sections(vec![
                        TextSection::new("cyles left", text_style_red.clone()),
                        TextSection::new(" : ", text_style.clone()),
                        TextSection::new("2", text_style.clone()),
                    ])
                    .with_style(Style {
                        width: Val::Percent(100.),
                        top: Val::Px(5.),
                        ..default()
                    }),
                )
                .insert(CyclesText);
            parent
                .spawn(
                    TextBundle::from_sections(vec![
                        TextSection::new("recyles left", text_style_red),
                        TextSection::new(" : ", text_style.clone()),
                        TextSection::new("2", text_style.clone()),
                    ])
                    .with_style(Style {
                        width: Val::Percent(100.),
                        top: Val::Px(5.),
                        // left: Val::Px(50.),
                        ..default()
                    }),
                )
                .insert(RewindsText);
        });
    commands
        .spawn(
            TextBundle::from_section("", text_style)
                .with_text_justify(JustifyText::Left)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                }),
        )
        .insert(StateScoped(GameState::Gaming))
        .insert(BufferText);
}

fn update_buffer(
    mut text: Query<&mut Text, With<BufferText>>,
    key_buffer: Res<KeyBuffer>,
    mut last_len: Local<usize>,
) {
    if *last_len == key_buffer.0.len() {
        return;
    }
    *last_len = key_buffer.0.len();

    if let Some(mut text) = text.iter_mut().next() {
        text.sections[0].value = key_buffer.0.iter().copied().map(map_key_buffer).collect();
    }
}

fn map_key_buffer(key: KeyCode) -> char {
    match key {
        KeyCode::Digit0 => '0',
        KeyCode::Digit1 => '1',
        KeyCode::Digit2 => '2',
        KeyCode::Digit3 => '3',
        KeyCode::Digit4 => '4',
        KeyCode::Digit5 => '5',
        KeyCode::Digit6 => '6',
        KeyCode::Digit7 => '7',
        KeyCode::Digit8 => '8',
        KeyCode::Digit9 => '9',
        KeyCode::KeyC => 'c',
        KeyCode::KeyW => 'w',
        KeyCode::KeyU => 'u',
        KeyCode::Enter => ' ',
        _ => ' ',
    }
}
