use bevy::prelude::*;

use crate::{
    map::{GameMap, GridPos},
    snake::{CanMove, Move, SnakeIndex, SnakeSize, TaggedDeath},
    sounds::SoundEvent,
    title::UiResources,
    GameState,
};

#[derive(Debug, Resource)]
pub struct DoorSprites {
    door: Handle<Image>,
    pressure_plate_layout: Handle<TextureAtlasLayout>,
    pressure_plate: Handle<Image>,
}

impl FromWorld for DoorSprites {
    fn from_world(world: &mut World) -> Self {
        let pressure_plate_layout =
            TextureAtlasLayout::from_grid(UVec2::new(50, 50), 1, 2, Some(UVec2::new(1, 1)), None);

        let handle = {
            let mut texture_atlases = world
                .get_resource_mut::<Assets<TextureAtlasLayout>>()
                .expect("texture atlases assets should exist");
            texture_atlases.add(pressure_plate_layout)
        };

        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        Self {
            door: asset_server.load("dev/door.png"),
            pressure_plate_layout: handle,
            pressure_plate: asset_server.load("dev/pressure_plate.png"),
        }
    }
}

#[derive(Debug, Event)]
pub struct PressurePlateActivated(pub char, pub [usize; 2]);

#[derive(Debug, Component)]
pub struct PressurePlate;

#[derive(Debug, Component, PartialEq, Eq)]
pub struct Door(char);

#[derive(Debug, Component, PartialEq, Eq)]
pub struct AntiDoor(char);

pub fn door_plugin(app: &mut App) {
    app.init_resource::<DoorSprites>()
        .observe(on_plate_activated);
}

fn on_plate_activated(
    trigger: Trigger<PressurePlateActivated>,
    mut commands: Commands,
    doors: Query<(Entity, &Door), Without<AntiDoor>>,
    mut anti_doors: Query<(Entity, &AntiDoor, &GridPos, &mut Sprite), Without<Door>>,
    mut pressure_plates: Query<(&GridPos, &mut TextureAtlas), (Without<Door>, With<PressurePlate>)>,
    mut snakes: Query<
        (Entity, &GridPos, &mut Visibility),
        (
            With<SnakeSize>,
            Without<Door>,
            Without<AntiDoor>,
            Without<PressurePlate>,
        ),
    >,
    mut map: ResMut<GameMap>,
) {
    let index = &mut pressure_plates
        .iter_mut()
        .find(|(pos, _)| pos.0 == trigger.event().1)
        .expect("trigger pos should be a valid pressure plate")
        .1
        .index;

    if *index == 1 {
        return;
    }
    *index = 1;

    for ent in doors
        .iter()
        .filter_map(|door| door.1 .0.eq(&trigger.event().0).then_some(door.0))
    {
        commands.entity(ent).despawn_recursive();
    }

    for (ent, anti_door, grid_pos, mut sprite) in anti_doors.iter_mut().filter_map(|door| {
        door.1
             .0
            .eq(&trigger.event().0)
            .then_some((door.0, door.1, door.2, door.3))
    }) {
        commands
            .entity(ent)
            .remove::<AntiDoor>()
            .insert(Door(anti_door.0));

        map.get_mut(*grid_pos)
            .expect("should be on a valid tile")
            .upgrade_to_door(anti_door.0);

        sprite.color = Color::default();
        sprite.flip_y = false;
        sprite.flip_x = false;

        if let Some((dead_piece, mut visibility)) = snakes
            .iter_mut()
            .find_map(|(ent, pos, visibility)| (*grid_pos == *pos).then_some((ent, visibility)))
        {
            // hack
            commands
                .entity(dead_piece)
                .remove::<SnakeSize>()
                .remove::<Move>()
                .remove::<CanMove>()
                .remove::<Sprite>()
                .remove::<Handle<Image>>()
                .remove::<SnakeIndex>()
                .try_insert(TaggedDeath);

            *visibility = Visibility::Hidden;
        }
    }

    commands.trigger(SoundEvent::PressurePlate);

    pressure_plates
        .iter_mut()
        .find(|(pos, _)| pos.0 == trigger.event().1)
        .expect("trigger pos should be a valid pressure plate")
        .1
        .index = 1;
}

pub fn spawn_door(
    commands: &mut Commands<'_, '_>,
    door_sprites: &DoorSprites,
    ui_resources: &UiResources,
    grid_pos: GridPos,
    door_char: char,
) {
    commands
        .spawn((
            SpriteBundle {
                texture: door_sprites.door.clone(),
                transform: Transform::from_xyz(0., 0., 30.),
                ..default()
            },
            Door(door_char),
            grid_pos,
            StateScoped(GameState::Gaming),
        ))
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    door_char,
                    TextStyle {
                        font: ui_resources.font.clone(),
                        font_size: 20.,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_xyz(0., 0., 32.),
                ..default()
            });
        });
}

pub fn spawn_anti_door(
    commands: &mut Commands<'_, '_>,
    door_sprites: &DoorSprites,
    ui_resources: &UiResources,
    grid_pos: GridPos,
    door_char: char,
) {
    commands
        .spawn((
            SpriteBundle {
                texture: door_sprites.door.clone(),
                transform: Transform::from_xyz(0., 0., 30.),
                sprite: Sprite {
                    color: Color::srgba(0., 0., 1., 1.),
                    flip_y: true,
                    flip_x: true,
                    ..default()
                },
                ..default()
            },
            AntiDoor(door_char),
            grid_pos,
            StateScoped(GameState::Gaming),
        ))
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    door_char,
                    TextStyle {
                        font: ui_resources.font.clone(),
                        font_size: 20.,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_xyz(0., 0., 32.),
                ..default()
            });
        });
}

pub fn spawn_pressure_plate(
    commands: &mut Commands<'_, '_>,
    door_sprites: &DoorSprites,
    ui_resources: &UiResources,
    grid_pos: GridPos,
    door_char: char,
) {
    commands
        .spawn((
            SpriteBundle {
                texture: door_sprites.pressure_plate.clone(),
                transform: Transform::from_xyz(0., 0., 30.),
                ..default()
            },
            TextureAtlas::from(door_sprites.pressure_plate_layout.clone()),
            PressurePlate,
            grid_pos,
            StateScoped(GameState::Gaming),
        ))
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    door_char,
                    TextStyle {
                        font: ui_resources.font.clone(),
                        font_size: 20.,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_xyz(0., 0., 32.),
                ..default()
            });
        });
}
