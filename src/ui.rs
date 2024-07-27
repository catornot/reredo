use bevy::prelude::*;

use crate::{input::KeyBuffer, snake::RewindCounter, title::UiResources, GameState};

#[derive(Debug, Component)]
struct BufferText;

#[derive(Debug, Component)]
struct CyclesText;

#[derive(Debug, Component)]
struct RewindsText;

pub fn game_ui_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Gaming), setup_gaming_ui)
        .add_systems(OnEnter(GameState::Loading), setup_loading_ui)
        .add_systems(
            Update,
            (update_rewinds_count, update_buffer).run_if(in_state(GameState::Gaming)),
        );
}

fn setup_gaming_ui(mut commands: Commands, ui_resources: Res<UiResources>) {
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
                flex_direction: FlexDirection::Column,
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
                        TextSection::new("rewinds left", text_style_red.clone()),
                        TextSection::new(" : ", text_style.clone()),
                        TextSection::new("2", text_style.clone()),
                    ])
                    .with_style(Style {
                        top: Val::Px(5.),
                        left: Val::Px(5.),
                        ..default()
                    }),
                )
                .insert(RewindsText);
            parent
                .spawn(
                    TextBundle::from_sections(vec![
                        TextSection::new("total rewinds left", text_style_red),
                        TextSection::new(" : ", text_style.clone()),
                        TextSection::new("2", text_style.clone()),
                    ])
                    .with_style(Style {
                        top: Val::Px(5.),
                        left: Val::Px(5.),
                        ..default()
                    }),
                )
                .insert(CyclesText);
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

fn setup_loading_ui(mut commands: Commands, ui_resources: Res<UiResources>) {
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Please wait something is loading",
                TextStyle {
                    font: ui_resources.font.clone(),
                    font_size: 50.,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                margin: UiRect::all(Val::Auto),
                align_self: AlignSelf::Center,
                ..default()
            },
            z_index: ZIndex::Global(20),
            ..default()
        })
        .insert(StateScoped(GameState::Loading));

    commands
        .spawn(ImageBundle {
            image: UiImage::new(ui_resources.looker.clone()),
            style: Style {
                width: Val::Percent(150.),
                height: Val::Percent(60.),
                margin: UiRect::all(Val::Auto),
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        })
        .insert(StateScoped(GameState::Loading));
}

fn update_rewinds_count(
    mut text_cycles: Query<&mut Text, (With<RewindsText>, Without<CyclesText>)>,
    mut text_rewinds: Query<&mut Text, (With<CyclesText>, Without<RewindsText>)>,
    rewinds: Res<RewindCounter>,
) {
    if let Some(mut text) = text_cycles.iter_mut().next() {
        text.sections[2].value = rewinds.individual.to_string();
    }

    if let Some(mut text) = text_rewinds.iter_mut().next() {
        text.sections[2].value = rewinds.total.to_string();
    }
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
