#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    asset::AssetMetaCheck,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        tonemapping::Tonemapping,
    },
    log::{Level, LogPlugin},
    prelude::*,
};

use door::door_plugin;
use game_over::game_over_plugin;
use input::input_plugin;
use main_menu::main_menu_ui_plugin;
use map::map_plugin;
use snake::snake_plugin;
use spike::spike_plugin;
use title::title_ui_plugin;
use ui::game_ui_plugin;
use wall::wall_plugin;

mod door;
mod game_over;
mod input;
mod main_menu;
mod map;
mod snake;
mod spike;
mod title;
mod ui;
mod wall;

#[derive(Debug, Resource)]
pub struct AssetHolder<T: std::fmt::Debug + Asset>(pub Handle<T>);

#[derive(Debug, States, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum GameState {
    #[default]
    TitleScreen,
    MainMenu,
    Gaming,
    Loading,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Behavior,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=off,bevy_render=info,bevy_ecs=trace".to_string(),
                    custom_layer: |_| None,
                }),
            snake_plugin,
            door_plugin,
            map_plugin,
            wall_plugin,
            spike_plugin,
            game_ui_plugin,
            input_plugin,
            main_menu_ui_plugin,
            title_ui_plugin,
            game_over_plugin,
        ))
        .enable_state_scoped_entities::<GameState>()
        .configure_sets(
            Update,
            (
                GameplaySet::Input.run_if(in_state(GameState::Gaming)),
                GameplaySet::Behavior
                    .run_if(in_state(GameState::Gaming))
                    .after(GameplaySet::Input),
            ),
        )
        .insert_resource(ClearColor(Color::srgba(0.1, 0.1, 0.1, 1.)))
        .init_state::<GameState>()
        .add_systems(Startup, main_setup)
        .run();
}

fn main_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings {
            intensity: 0.15,
            low_frequency_boost: 0.7,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.0,
                threshold_softness: 0.0,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
        },
    ));
}
