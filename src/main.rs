#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::{
    array,
    ops::{Div, Sub},
};

use bevy::{
    asset::AssetMetaCheck,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    ecs::system::EntityCommands,
    log::{self, Level, LogPlugin},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

const GRID_CELL_SIZE: Vec2 = Vec2::new(50., 50.);

#[derive(Debug, Event)]
struct MoveEvent(Vec2);

#[derive(Debug, Resource)]
struct KeyBuffer(Vec<KeyCode>);

#[derive(Debug, Resource)]
struct GameMap(Box<[Box<[bool]>]>);

#[derive(Debug, Component, Clone, Copy)]
struct GridPos([usize; 2]);

#[derive(Debug, Component)]
struct Move(Vec2);

#[derive(Debug, Component, Clone, Copy)]
struct SnakeIndex(usize);

#[derive(Debug, Component)]
struct SnakeColor(Color);

#[derive(Debug, Component, Clone, Copy)]
struct SnakeSize(Vec2);

#[derive(Debug, Component)]
struct CanMove;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
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
            }),))
        .insert_resource(ClearColor(Color::srgba(0.01, 0.01, 0.01, 1.)))
        .insert_resource(GameMap(Box::new(array::from_fn::<_, 100, _>(|_| {
            Box::new([false; 100]) as Box<[bool]>
        })) as Box<[Box<[bool]>]>))
        .insert_resource(KeyBuffer(Vec::new()))
        .add_event::<MoveEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_keys,
                consume_move.after(handle_keys),
                on_move_snake.after(handle_keys),
                move_snake.after(on_move_snake),
                cycle_snake.after(move_snake),
                camera_follow,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
    ));

    spawn_snake_piece(
        &mut commands,
        &mut meshes,
        &mut materials,
        Color::srgb_u8(50, 200, 50),
        GridPos([0, 0]),
        SnakeSize(Vec2::new(40., 40.)),
    )
    .insert(CanMove)
    .insert(SnakeIndex(0));
}

fn spawn_snake_piece<'a>(
    commands: &'a mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    color: Color,
    grid_pos: GridPos,
    size: SnakeSize,
) -> EntityCommands<'a> {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::from_size(size.0))),
            material: materials.add(color),
            transform: Transform::from_translation(
                (Vec2::from_array([grid_pos.0[0] as f32, grid_pos.0[1] as f32]) * GRID_CELL_SIZE)
                    .extend(0.),
            ),
            ..default()
        },
        SnakeColor(color),
        size,
        grid_pos,
    ))
}

fn camera_follow(
    snake_pieces: Query<(&SnakeIndex, &Transform), (With<CanMove>, Without<Camera>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<CanMove>)>,
    time: Res<Time>,
    mut start_time: Local<f32>,
    mut start_pos: Local<Vec2>,
) {
    let Some(head_pos) = snake_pieces
        .iter()
        .fold(None::<(&SnakeIndex, &Transform)>, |max, piece| {
            Some(max.unwrap_or(piece)).map(|other| {
                if other.0 .0 < piece.0 .0 {
                    piece
                } else {
                    other
                }
            })
        })
        .map(|head| head.1.translation.truncate())
    else {
        return;
    };

    let mut camera_pos = camera
        .get_single_mut()
        .expect("only one camera should ever exists");

    if *start_time + 0.3 < time.elapsed_seconds() {
        if camera_pos.translation.truncate() != head_pos {
            *start_time = time.elapsed_seconds();
            *start_pos = camera_pos.translation.truncate();
        }
        return;
    }

    camera_pos.translation = start_pos
        .lerp(head_pos, (time.elapsed_seconds() - *start_time).div(0.3))
        .extend(0.)
}

fn cycle_snake(
    mut cycle_buffer: ResMut<KeyBuffer>,
    snake_pieces: Query<(Entity, &mut SnakeIndex, &GridPos)>,
    mut map: ResMut<GameMap>,
    mut commands: Commands,
) {
    if !matches!(cycle_buffer.0.last(), Some(KeyCode::Enter)) {
        return;
    }

    log::info!("uh: {:?}", cycle_buffer.0);

    if cycle_buffer.0.first_chunk() == Some(&[KeyCode::KeyU, KeyCode::KeyW, KeyCode::KeyU]) {
        // uwu
        log::info!("uwu");
    }

    const KEY_MAP: [KeyCode; 10] = [
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    let steps = if matches!(cycle_buffer.0.first(), Some(KeyCode::KeyC)) {
        if let Some(steps) = cycle_buffer
            .0
            .get(1..cycle_buffer.0.len().saturating_sub(1))
            .and_then(|slice| {
                dbg!(slice
                    .iter()
                    .rev()
                    .inspect(|key| log::info!("{key:?}"))
                    .map(|key| KEY_MAP.iter().position(|other_key| key == other_key))
                    .enumerate()
                    .fold(None::<usize>, |steps, (pos, step)| {
                        if pos == 0 {
                            step
                        } else {
                            Some(steps? + 10usize.pow(pos as u32) * step?)
                        }
                    }))
            })
        {
            steps
        } else {
            cycle_buffer.0.clear();
            return;
        }
    } else {
        cycle_buffer.0.clear();
        return;
    };

    log::info!("{steps}");

    let mut snake_ordered = snake_pieces.iter().collect::<Vec<_>>();
    snake_ordered.sort_by(|other, piece| other.1 .0.cmp(&piece.1 .0));
    snake_ordered.reverse();

    (0..steps)
        .filter_map(|i| snake_ordered.get(i))
        .for_each(|piece| {
            if let Some(map_tile) = map
                .0
                .get_mut(piece.2 .0[0])
                .and_then(|line| line.get_mut(piece.2 .0[1]))
            {
                *map_tile = false;
            }

            return commands.entity(piece.0).despawn_recursive();
        });

    if let Some(new_head) = snake_ordered.get(steps).map(|new_head| new_head.0) {
        commands.entity(new_head).insert(CanMove);
    }

    cycle_buffer.0.clear();
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

fn consume_move(
    mut move_event: EventReader<MoveEvent>,
    mut gizmos: Gizmos,
    snake_pieces: Query<(&GridPos, &SnakeSize), With<CanMove>>,
) {
    for (grid_pos, size) in snake_pieces.iter() {
        gizmos.rect_2d(
            Vec2::new(grid_pos.0[0] as f32, grid_pos.0[1] as f32) * GRID_CELL_SIZE,
            0.,
            size.0,
            Color::WHITE,
        );
    }

    if let Some(event) = move_event.read().last() {
        log::info!("{event:?}");
    }
}

fn on_move_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut move_event: EventReader<MoveEvent>,
    mut snake_pieces: Query<(
        Entity,
        &mut SnakeIndex,
        &mut GridPos,
        &SnakeSize,
        &SnakeColor,
        Option<&CanMove>,
    )>,
    mut map: ResMut<GameMap>,
) {
    assert!(snake_pieces.iter().filter_map(|piece| piece.5).count() <= 1);

    let Some(move_dir) = move_event.read().last().map(|move_event| move_event.0) else {
        return;
    };

    for (ent, mut snake_index, mut grid_pos, size, color) in
        snake_pieces.iter_mut().filter_map(|piece| {
            piece
                .5
                .map(|_| (piece.0, piece.1, piece.2, piece.3, piece.4))
        })
    {
        let grid_pos = &mut grid_pos.0;

        if usize::try_from(grid_pos[0] as isize + move_dir.x as isize)
            .ok()
            .and_then(|x| map.0.get(x))
            .and_then(|row| {
                usize::try_from(grid_pos[1] as isize + move_dir.y as isize)
                    .ok()
                    .and_then(|y| row.get(y))
            })
            .and_then(|is_occupied| (!is_occupied).then_some(()))
            .is_none()
        {
            return;
        }

        *map.0
            .get_mut(grid_pos[0])
            .and_then(|row| row.get_mut(grid_pos[1]))
            .expect("previous checks should have valided this one too") = true;

        spawn_snake_piece(
            &mut commands,
            &mut meshes,
            &mut materials,
            Color::Srgba(color.0.to_srgba()),
            GridPos(*grid_pos),
            *size,
        )
        .insert(*snake_index);
        snake_index.0 += 1;
        *grid_pos = [
            (grid_pos[0] as isize + move_dir.x as isize) as usize,
            (grid_pos[1] as isize + move_dir.y as isize) as usize,
        ];

        commands
            .entity(ent)
            .remove::<CanMove>()
            .insert(Move(move_dir));
    }
}

fn move_snake(
    mut commands: Commands,
    mut snake_pieces: Query<(Entity, &Move, &mut Transform)>,
    time: Res<Time>,
    mut dst: Local<Option<Vec2>>,
    mut elapsed: Local<f32>,
) {
    let Some((ent, move_dir, mut transform)) = snake_pieces.get_single_mut().ok() else {
        return;
    };

    if dst.is_none() {
        dst.replace(transform.translation.truncate() + move_dir.0 * GRID_CELL_SIZE);
        *elapsed = time.elapsed().as_secs_f32();
    };

    let dif = (time.elapsed().as_secs_f32() - *elapsed).div(0.3);
    let ddst = dst.unwrap();

    transform.translation = ddst
        .sub(move_dir.0 * GRID_CELL_SIZE)
        .lerp(ddst, dif)
        .extend(0.);

    if dif >= 1. {
        transform.translation = ddst.extend(0.);
        commands.entity(ent).remove::<Move>().insert(CanMove);
        dst.take();
    }
}
