use bevy::{
    ecs::system::EntityCommands,
    log,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::ops::{Div, Not, Sub};

use crate::{
    fade_out::FadeOutThisEnt,
    input::{KeyBuffer, MoveEvent},
    map::{GameMap, GridPos, GRID_CELL_SIZE},
    GameState, GameplaySet,
};

#[derive(Debug, Resource, Default)]
pub struct RewindCounter {
    pub total: isize,
    pub individual: isize,
}

#[derive(Debug, Component)]
pub struct Move(pub Vec2);

#[derive(Debug, Component)]
pub struct TaggedDeath;

#[derive(Debug, Component, Clone, Copy)]
pub struct SnakeIndex(pub usize);

#[derive(Debug, Component)]
struct SnakeColor(Color);

#[derive(Debug, Component, Clone, Copy)]
pub struct SnakeSize(pub Vec2);

#[derive(Debug, Component)]
pub struct CanMove;

pub fn snake_plugin(app: &mut App) {
    app.init_resource::<RewindCounter>().add_systems(
        Update,
        (
            consume_move,
            on_move_snake,
            move_snake,
            cycle_snake,
            camera_follow,
        )
            .in_set(GameplaySet::Behavior),
    );
}

pub fn spawn_snake_piece<'a>(
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
            transform: Transform::from_xyz(0., 0., 100.),
            ..default()
        },
        SnakeColor(color),
        size,
        grid_pos,
        StateScoped(GameState::Gaming),
    ))
}

fn camera_follow(
    snake_pieces: Query<(&SnakeIndex, &GridPos), (With<CanMove>, Without<Camera>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<CanMove>)>,
    time: Res<Time>,
    mut maybe_timer: Local<Option<Timer>>,
    mut last_know_pos: Local<Vec2>,
    mut last_know_camera_pos: Local<Vec2>,
) {
    let Some(head_pos) = snake_pieces
        .iter()
        .fold(None::<(&SnakeIndex, &GridPos)>, |max, piece| {
            Some(max.unwrap_or(piece)).map(|other| {
                if other.0 .0 < piece.0 .0 {
                    piece
                } else {
                    other
                }
            })
        })
        .map(|head| head.1.to_vec2() * GRID_CELL_SIZE)
    else {
        maybe_timer.take();
        return;
    };

    let mut camera_pos = camera
        .get_single_mut()
        .expect("only one camera should ever exists");

    if let Some(timer) = maybe_timer.as_mut() {
        timer.tick(time.elapsed());

        camera_pos.translation = last_know_pos
            .lerp(*last_know_pos, timer.elapsed_secs())
            .extend(0.)
    } else {
        maybe_timer.replace(Timer::from_seconds(1.2, TimerMode::Once));
        *last_know_pos = head_pos;
        *last_know_camera_pos = camera_pos.translation.truncate();
        return;
    }

    if maybe_timer
        .as_ref()
        .map(|timer| timer.finished())
        .unwrap_or_default()
    {
        maybe_timer.take();
    }
}

fn cycle_snake(
    mut cycle_buffer: ResMut<KeyBuffer>,
    snake_pieces: Query<(Entity, &mut SnakeIndex, &SnakeColor, &GridPos)>,
    mut map: ResMut<GameMap>,
    mut commands: Commands,
    mut rewinds: ResMut<RewindCounter>,
) {
    if !matches!(cycle_buffer.0.last(), Some(KeyCode::Enter)) {
        return;
    }

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

    if cycle_buffer.0.len() > 10 {
        cycle_buffer.0.clear();
        return;
    }

    let steps = if matches!(cycle_buffer.0.first(), Some(KeyCode::KeyC)) {
        if let Some(steps) = cycle_buffer
            .0
            .get(1..cycle_buffer.0.len().saturating_sub(1))
            .and_then(|slice| {
                slice
                    .iter()
                    .rev()
                    .map(|key| KEY_MAP.iter().position(|other_key| key == other_key))
                    .enumerate()
                    .fold(None::<usize>, |steps, (pos, step)| {
                        if pos == 0 {
                            step
                        } else {
                            Some(steps? + 10usize.pow(pos as u32).saturating_mul(step?))
                        }
                    })
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

    let mut snake_ordered = snake_pieces.iter().collect::<Vec<_>>();
    snake_ordered.sort_by(|other, piece| other.1 .0.cmp(&piece.1 .0));
    snake_ordered.reverse();

    (0..steps)
        .filter_map(|i| snake_ordered.get(i))
        .for_each(|piece| {
            if let Some(map_tile) = map.get_mut(*piece.3) {
                map_tile.top_removed();
            }

            // commands.entity(piece.0).despawn_recursive();
            commands.entity(piece.0).insert(FadeOutThisEnt(piece.2 .0));
        });

    if let Some(mut new_head) = snake_ordered
        .get(steps)
        .map(|new_head| new_head.0)
        .and_then(|new_head| commands.get_entity(new_head))
    {
        new_head.insert(CanMove);
    }

    cycle_buffer.0.clear();
    rewinds.total -= 1;
    rewinds.individual -= steps as isize;
}

fn consume_move(mut gizmos: Gizmos, snake_pieces: Query<(&GridPos, &SnakeSize), With<CanMove>>) {
    for (grid_pos, size) in snake_pieces.iter() {
        gizmos.rect_2d(
            Vec2::new(grid_pos.0[0] as f32, grid_pos.0[1] as f32) * GRID_CELL_SIZE,
            0.,
            size.0,
            Color::WHITE,
        );
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
            .and_then(|x| {
                Some([
                    x,
                    usize::try_from(grid_pos[1] as isize + move_dir.y as isize).ok()?,
                ])
            })
            .and_then(|pos| map.get(pos))
            .and_then(|tile| tile.is_occupied().not().then_some(()))
            .is_none()
        {
            return;
        }

        map.get_mut(*grid_pos)
            .expect("previous checks should have valided this one too")
            .snake_passes(*grid_pos, &mut commands);

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

        if let Some(mut ent) = commands.get_entity(ent) {
            ent.remove::<Move>().try_insert(CanMove);
        }

        dst.take();
    }
}
