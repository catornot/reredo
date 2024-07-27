use bevy::prelude::*;

use crate::GameState;

// #[derive(Debug, Component)]
// struct BufferText;

#[derive(Debug, Resource)]
pub struct UiResources {
    pub font: Handle<Font>,
    pub cat: Handle<Image>,
    pub looker: Handle<Image>,
}

impl FromWorld for UiResources {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        UiResources {
            font: asset_server.load("fonts/oswald.ttf"),
            cat: asset_server.load("pictures/cat.png"),
            looker: asset_server.load("pictures/looker.png"),
        }
    }
}

pub fn title_ui_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::TitleScreen), setup_ui)
        .init_resource::<UiResources>()
        .add_systems(
            Update,
            switch_to_main_menu.run_if(in_state(GameState::TitleScreen)),
        );
}

fn setup_ui(mut commands: Commands, ui_resources: Res<UiResources>) {
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "ReSnaked",
                TextStyle {
                    font: ui_resources.font.clone(),
                    font_size: 100.,
                    color: Color::srgba(0.99, 0.99, 0.99, 1.),
                },
            ),
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            ..default()
        })
        .insert(StateScoped(GameState::TitleScreen));
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "A first bevy game by cat_or_not :)",
                TextStyle {
                    font: ui_resources.font.clone(),
                    font_size: 20.,
                    color: Color::srgba(0.99, 0.99, 0.99, 1.),
                },
            ),
            style: Style {
                align_self: AlignSelf::Center,
                top: Val::Vh(10.),
                margin: UiRect::all(Val::Auto),
                position_type: PositionType::Relative,
                ..default()
            },
            ..default()
        })
        .insert(StateScoped(GameState::TitleScreen));
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "press ANY key to skip this",
                TextStyle {
                    font: ui_resources.font.clone(),
                    font_size: 20.,
                    color: Color::srgba(0.99, 0.99, 0.99, 1.),
                },
            ),
            style: Style {
                align_self: AlignSelf::Center,
                top: Val::Vh(15.),
                margin: UiRect::all(Val::Auto),
                position_type: PositionType::Relative,
                ..default()
            },
            ..default()
        })
        .insert(StateScoped(GameState::TitleScreen));
}

fn switch_to_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut start_time: Local<f32>,
) {
    if *start_time == 0. {
        *start_time = time.elapsed_seconds();
        return;
    }

    if *start_time + 10. < time.elapsed_seconds() || keys.get_just_pressed().next().is_some() {
        next_state.set(GameState::MainMenu);
    }
}
