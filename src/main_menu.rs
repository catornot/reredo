use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const PRESSED_BUTTON: Color = Color::srgb(0.30, 0.30, 0.30);

#[derive(Debug, Component)]
struct GamingButton;

#[derive(Debug, Component)]
struct CatButton;

#[derive(Debug, Component)]
struct CreditsButton;

#[derive(Debug, Component)]
struct GoBackButton;

#[derive(Debug, Component)]
struct QuitButton;

#[derive(Debug, Component)]
struct ToLevelSeLectionButton;

#[derive(Debug, Component)]
struct SelectLevelButton(usize);

#[derive(Debug, SubStates, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[source(GameState = GameState::MainMenu)]
pub enum MainMenuState {
    #[default]
    Main,
    Creadits,
    Cat,
    ToGaming,
    Quit,
    LevelSelection,
}

use crate::{map::MapName, sounds::SoundEvent, title::UiResources, GameState};

pub fn main_menu_ui_plugin(app: &mut App) {
    app.add_sub_state::<MainMenuState>()
        .enable_state_scoped_entities::<MainMenuState>()
        .add_systems(
            OnEnter(GameState::MainMenu),
            setup_menu_builder(setup_main_ui),
        )
        .add_systems(
            OnEnter(MainMenuState::Main),
            setup_menu_builder(setup_main_ui),
        )
        .add_systems(
            OnEnter(MainMenuState::Cat),
            setup_menu_builder(setup_cat_ui),
        )
        .add_systems(
            OnEnter(MainMenuState::Creadits),
            setup_menu_builder(setup_credits_ui),
        )
        .add_systems(
            OnEnter(MainMenuState::LevelSelection),
            setup_menu_builder(setup_level_selection),
        )
        .add_systems(
            OnEnter(MainMenuState::ToGaming),
            |mut next_state: ResMut<NextState<GameState>>| next_state.set(GameState::Loading),
        )
        .add_systems(
            OnEnter(MainMenuState::ToGaming),
            |mut next_state: ResMut<NextState<MainMenuState>>| next_state.set(MainMenuState::Main),
        )
        .add_systems(
            OnEnter(MainMenuState::Quit),
            |mut exit: EventWriter<AppExit>| _ = exit.send(AppExit::Success),
        )
        .add_systems(
            Update,
            (
                transition_to_builder::<GamingButton>(MainMenuState::ToGaming),
                transition_to_builder::<CreditsButton>(MainMenuState::Creadits),
                transition_to_builder::<CatButton>(MainMenuState::Cat),
                transition_to_builder::<GoBackButton>(MainMenuState::Main),
                transition_to_builder::<QuitButton>(MainMenuState::Quit),
                transition_to_builder::<ToLevelSeLectionButton>(MainMenuState::LevelSelection),
                skip_to_play,
                select_level_button,
            )
                .run_if(in_state(GameState::MainMenu)),
        );
}

fn skip_to_play(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<MainMenuState>>) {
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        next_state.set(MainMenuState::ToGaming);
    }
}

fn setup_main_ui(parent: &mut ChildBuilder, _ui_resources: &UiResources, button_style: TextStyle) {
    let title_style = TextStyle {
        font_size: 100.,
        ..button_style.clone()
    };

    parent
        .spawn(
            TextBundle {
                text: Text::from_section("ReSnaked", title_style),

                ..default()
            }
            .with_style(Style {
                justify_self: JustifySelf::Center,
                margin: UiRect::left(Val::Auto).with_right(Val::Auto),
                ..default()
            }),
        )
        .insert(StateScoped(MainMenuState::Main))
        .insert(StateScoped(GameState::MainMenu));

    create_button(parent, "Play", button_style.clone(), GamingButton)
        .insert(StateScoped(MainMenuState::Main));
    create_button(
        parent,
        "Level Selection",
        button_style.clone(),
        ToLevelSeLectionButton,
    )
    .insert(StateScoped(MainMenuState::Main));
    create_button(parent, "Credits", button_style.clone(), CreditsButton)
        .insert(StateScoped(MainMenuState::Main));
    create_button(parent, "Cat", button_style.clone(), CatButton)
        .insert(StateScoped(MainMenuState::Main));
    create_button(parent, "Quit", button_style, QuitButton)
        .insert(StateScoped(MainMenuState::Main));
}

fn setup_credits_ui(
    parent: &mut ChildBuilder,
    _ui_resources: &UiResources,
    button_style: TextStyle,
) {
    let title_style = TextStyle {
        font_size: 100.,
        ..button_style.clone()
    };

    parent
        .spawn(
            TextBundle {
                text: Text::from_section("The Credits", title_style),

                ..default()
            }
            .with_style(Style {
                justify_self: JustifySelf::Center,
                margin: UiRect::left(Val::Auto).with_right(Val::Auto),
                ..default()
            }),
        )
        .insert(StateScoped(MainMenuState::Creadits))
        .insert(StateScoped(GameState::MainMenu));

    parent
        .spawn(TextBundle {
            text: Text::from_section(
                "I made the game and like got the oswald font from vernnobile's github\nalso bevy",
                button_style.clone(),
            ),
            ..default()
        })
        .insert(StateScoped(MainMenuState::Creadits));

    parent.spawn(TextBundle {
        text: Text::from_section(
            r#"Pressure Plate by proolsen -- https://freesound.org/s/466272/ -- License: Creative Commons 0"#,
            button_style.clone(),
        ),
        ..default()
    }).insert(StateScoped(MainMenuState::Creadits));

    create_button(parent, "Go Back", button_style, GoBackButton)
        .insert(StateScoped(MainMenuState::Creadits));
}

fn setup_cat_ui(parent: &mut ChildBuilder, ui_resources: &UiResources, button_style: TextStyle) {
    parent
        .spawn(TextBundle {
            text: Text::from_section("CAT", button_style.clone()),
            ..default()
        })
        .insert(StateScoped(MainMenuState::Cat));

    parent
        .spawn(ImageBundle {
            image: UiImage::new(ui_resources.cat.clone()),
            style: Style {
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                width: Val::Percent(150.),
                height: Val::Percent(60.),
                margin: UiRect::axes(Val::Auto, Val::Px(10.)),
                ..default()
            },
            ..default()
        })
        .insert(StateScoped(MainMenuState::Cat));

    create_button(parent, "Go Back", button_style, GoBackButton)
        .insert(StateScoped(MainMenuState::Cat));
}

fn setup_level_selection(
    parent: &mut ChildBuilder,
    _ui_resources: &UiResources,
    button_style: TextStyle,
) {
    parent
        .spawn(TextBundle {
            text: Text::from_section(
                "SELECT LEVEL",
                TextStyle {
                    font_size: 50.,
                    ..button_style.clone()
                },
            ),
            ..default()
        })
        .insert(StateScoped(MainMenuState::LevelSelection));

    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(150.0),
                height: Val::Percent(60.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                justify_self: JustifySelf::Center,
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                grid_template_rows: RepeatedGridTrack::flex(5, 1.0),
                row_gap: Val::Px(60.0),
                column_gap: Val::Px(30.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // TODO: count levels
            (1..=15).for_each(|i| {
                _ = create_button(
                    parent,
                    &format!("level {i}"),
                    button_style.clone(),
                    SelectLevelButton(i),
                )
            })
        })
        .insert(StateScoped(MainMenuState::LevelSelection));

    create_button(parent, "Go Back", button_style, GoBackButton)
        .insert(StateScoped(MainMenuState::LevelSelection));
}
fn setup_menu(
    mut commands: Commands,
    ui_resources: Res<UiResources>,
    menu_func: impl Fn(&mut ChildBuilder, &UiResources, TextStyle),
) {
    let button_style = TextStyle {
        font_size: 30.,
        color: Color::srgba(0.99, 0.99, 0.99, 1.),
        font: ui_resources.font.clone(),
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(60.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                justify_self: JustifySelf::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| menu_func(parent, ui_resources.as_ref(), button_style))
        .insert(StateScoped(GameState::MainMenu));
}

fn setup_menu_builder<T: Fn(&mut ChildBuilder, &UiResources, TextStyle) + Clone>(
    menu_func: T,
) -> impl Fn(Commands, Res<UiResources>) {
    move |commands: Commands, ui_resources: Res<UiResources>| {
        setup_menu(commands, ui_resources, menu_func.clone())
    }
}

fn create_button<'a, T: Component>(
    parent: &'a mut ChildBuilder,
    text: &str,
    style: TextStyle,
    tag: T,
) -> EntityCommands<'a> {
    let mut ent = parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                justify_self: JustifySelf::Center,
                margin: UiRect::left(Val::Auto)
                    .with_right(Val::Auto)
                    .with_top(Val::Px(20.)),
                ..default()
            },
            border_color: NORMAL_BUTTON.into(),
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        tag,
        StateScoped(GameState::MainMenu),
    ));

    ent.with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text::from_section(text, style),
            focus_policy: FocusPolicy::Pass,
            ..default()
        });
    });

    ent
}

fn select_level_button(
    mut commands: Commands,
    mut button: Query<
        (
            &mut BorderColor,
            &mut BackgroundColor,
            &Interaction,
            &SelectLevelButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<MainMenuState>>,
    mut map_name: ResMut<MapName>,
) {
    for (mut border_color, mut color, interaction, level) in button.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::BLACK;

                map_name.0 = format!("maps/map_{}.game_map", level.0);
                next_state.set(MainMenuState::ToGaming);
                commands.trigger(SoundEvent::Select);
            }
            Interaction::Hovered => {
                border_color.0 = PRESSED_BUTTON;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn transition_to<T: Component>(
    mut commands: Commands,
    mut button: Query<
        (&mut BorderColor, &mut BackgroundColor, &Interaction),
        (Changed<Interaction>, With<Button>, With<T>),
    >,
    mut next_state: ResMut<NextState<MainMenuState>>,
    to_state: MainMenuState,
) {
    for (mut border_color, mut color, interaction) in button.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::BLACK;
                next_state.set(to_state);
                commands.trigger(SoundEvent::Select);
            }
            Interaction::Hovered => {
                border_color.0 = PRESSED_BUTTON;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn transition_to_builder<T: Component>(
    to_state: MainMenuState,
) -> impl Fn(
    Commands,
    Query<
        (&mut BorderColor, &mut BackgroundColor, &Interaction),
        (Changed<Interaction>, With<Button>, With<T>),
    >,
    ResMut<NextState<MainMenuState>>,
) {
    move |commands: Commands,
          button: Query<
        (&mut BorderColor, &mut BackgroundColor, &Interaction),
        (Changed<Interaction>, With<Button>, With<T>),
    >,
          next_state: ResMut<NextState<MainMenuState>>| {
        transition_to::<T>(commands, button, next_state, to_state)
    }
}
