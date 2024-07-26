use bevy::{
    log,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{
    map::{GridPos, MapName, GRID_CELL_SIZE},
    snake::{CanMove, Move},
    title::UiResources,
    GameState,
};

#[derive(Debug, Event)]
pub struct GameWinTrigger;

#[derive(Debug, Default, SubStates, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[source(GameState = GameState::Gaming)]
pub enum GameOverState {
    #[default]
    None,
    Death,
    Win,
}

#[derive(Debug, Component)]
pub struct Exit;

pub fn game_over_plugin(app: &mut App) {
    app.add_sub_state::<GameOverState>()
        .observe(on_game_won_reached)
        .add_systems(OnEnter(GameState::Gaming), reset_game_over)
        .add_systems(OnEnter(GameOverState::Death), display_right_ui)
        .add_systems(OnEnter(GameOverState::Win), display_right_ui)
        .add_systems(
            Update,
            (check_for_death, continue_from_state).run_if(in_state(GameState::Gaming)),
        );
}

pub fn spawn_exit(
    commands: &mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    grid_pos: GridPos,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::from_size(GRID_CELL_SIZE))),
            material: materials.add(Color::srgb_u8(20, 20, 200)),
            transform: Transform::from_xyz(0., 0., 50.),
            ..default()
        },
        Exit,
        grid_pos,
        StateScoped(GameState::Gaming),
    ));
}

fn reset_game_over(mut game_over: ResMut<NextState<GameOverState>>) {
    game_over.set(GameOverState::None);
}

fn on_game_won_reached(
    _: Trigger<GameWinTrigger>,
    mut game_over: ResMut<NextState<GameOverState>>,
) {
    game_over.set(GameOverState::Win);
}

fn check_for_death(
    snake: Query<Entity, Or<(With<CanMove>, With<Move>)>>,
    mut game_over: ResMut<NextState<GameOverState>>,
) {
    if snake.iter().next().is_none() {
        game_over.set(GameOverState::Death);
    }
}

fn display_right_ui(
    game_over: Res<State<GameOverState>>,
    mut commands: Commands,
    ui_resources: Res<UiResources>,
) {
    let (condition_text, tip, state) = match game_over.get() {
        GameOverState::None => return,
        GameOverState::Death => ("You Died", "Press ENTER to restart", GameOverState::Death),
        GameOverState::Win => (
            "You Won",
            "Press ENTER to go the next level",
            GameOverState::Win,
        ),
    };

    commands
        .spawn(TextBundle {
            text: Text::from_section(
                condition_text,
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
        .insert(StateScoped(GameState::Gaming))
        .insert(StateScoped(state));
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                tip,
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
        .insert(StateScoped(GameState::Gaming))
        .insert(StateScoped(state));
}

fn continue_from_state(
    keys: Res<ButtonInput<KeyCode>>,
    game_over: Res<State<GameOverState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut map_name: ResMut<MapName>,
) {
    if !keys.just_pressed(KeyCode::Enter) {
        return;
    }

    match game_over.get() {
        GameOverState::None => {}
        GameOverState::Death => next_state.set(GameState::Loading),
        GameOverState::Win => {
            if let Some(map_index) = map_name
                .0
                .split('_')
                .last()
                .and_then(|last| last.parse::<u32>().ok())
            {
                map_name
                    .0
                    .replace_range(.., &format!("map_{}", map_index + 1));

                next_state.set(GameState::Loading);
            } else {
                log::info!("Game ran out of maps or parsed incorrectly");
                next_state.set(GameState::MainMenu);
            }
        }
    }
}
