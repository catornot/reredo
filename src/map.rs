use std::iter;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext, LoadState},
    log,
    prelude::*,
    reflect::TypePath,
};

use crate::{
    door::{spawn_door, spawn_pressure_plate, DoorSprites, PressurePlateActivated},
    game_over::{spawn_exit, GameWinTrigger},
    snake::{spawn_snake_piece, CanMove, RewindCounter, SnakeIndex, SnakeSize},
    spike::spawn_spike,
    title::UiResources,
    wall::spawn_wall,
    AssetHolder, GameState,
};

pub const GRID_CELL_SIZE: Vec2 = Vec2::new(50., 50.);

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridPos(pub [usize; 2]);

impl GridPos {
    fn to_vec(self) -> Vec2 {
        Vec2::from_array(self.0.map(|i| i as f32))
    }
}

#[allow(clippy::from_over_into)]
impl Into<[usize; 2]> for GridPos {
    fn into(self) -> [usize; 2] {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct Tile {
    top: TopTileType,
    bottom: BottomTileType,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum TopTileType {
    Snake,
    Wall,
    Door(char),
    #[default]
    Nothing,
}

#[derive(Debug, Default, Clone)]
pub enum BottomTileType {
    PressurePlate(char),
    Exit,
    Spike,
    TextHint(Box<str>),
    #[default]
    Nothing,
}

impl Tile {
    pub fn new(top: Option<TopTileType>, bottom: Option<BottomTileType>) -> Self {
        Self {
            top: top.unwrap_or_default(),
            bottom: bottom.unwrap_or_default(),
        }
    }

    pub fn is_occupied(&self) -> bool {
        !matches!(self.top, TopTileType::Nothing)
    }

    pub fn snake_passes(&mut self, pos: [usize; 2], commands: &mut Commands) {
        self.top = TopTileType::Snake;

        if let BottomTileType::PressurePlate(channel) = self.bottom {
            commands.trigger(PressurePlateActivated(channel, pos))
        }

        if let BottomTileType::Exit = self.bottom {
            commands.trigger(GameWinTrigger)
        }
    }

    pub fn top_removed(&mut self) {
        self.top = TopTileType::Nothing;
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        door_sprites: &DoorSprites,
        ui_resources: &UiResources,
        pos: GridPos,
    ) {
        match self.top {
            TopTileType::Snake => {
                _ = spawn_snake_piece(
                    commands,
                    meshes,
                    materials,
                    Color::srgb_u8(50, 200, 50),
                    pos,
                    SnakeSize(Vec2::new(40., 40.)),
                )
                .insert(CanMove)
                .insert(SnakeIndex(0))
            }
            TopTileType::Wall => _ = spawn_wall(commands, meshes, materials, pos),
            TopTileType::Door(door_char) => spawn_door(commands, door_sprites, pos, door_char),
            TopTileType::Nothing => {}
        }

        match self.bottom {
            BottomTileType::Exit => spawn_exit(commands, meshes, materials, pos),
            BottomTileType::PressurePlate(_) => spawn_pressure_plate(commands, door_sprites, pos),
            BottomTileType::Spike => spawn_spike(commands, meshes, materials, pos),
            BottomTileType::Nothing => {}
            BottomTileType::TextHint(ref text) => {
                _ = commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            text.as_ref(),
                            TextStyle {
                                font: ui_resources.font.clone(),
                                font_size: 20.,
                                color: Color::WHITE,
                            },
                        ),
                        transform: Transform::from_xyz(0., 0., 150.),
                        ..default()
                    },
                    pos,
                    StateScoped(GameState::Gaming),
                ))
            }
        }
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct MapAsset(Box<[Box<[Tile]>]>);

#[derive(Debug, Resource)]
pub struct GameMap(pub Box<[Box<[Tile]>]>);

impl From<MapAsset> for GameMap {
    fn from(val: MapAsset) -> Self {
        GameMap(val.0)
    }
}

impl GameMap {
    pub fn get(&self, pos: impl Into<[usize; 2]>) -> Option<&Tile> {
        let pos = pos.into();
        self.0.get(pos[0]).and_then(|line| line.get(pos[1]))
    }

    pub fn get_mut(&mut self, pos: impl Into<[usize; 2]>) -> Option<&mut Tile> {
        let pos = pos.into();
        self.0.get_mut(pos[0]).and_then(|line| line.get_mut(pos[1]))
    }
}

#[derive(Debug, Resource)]
pub struct MapName(pub String);

#[derive(Default)]
pub struct MapAssetLoader;

impl AssetLoader for MapAssetLoader {
    type Asset = MapAsset;
    type Settings = ();
    type Error = String;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut map_str = String::new();
        reader
            .read_to_string(&mut map_str)
            .await
            .map_err(|err| err.to_string())?;

        let (map_board_str, map_text_str) = map_str
            .split_once("SPLIT_HERE")
            .map(|(l, r)| (l.trim(), r.trim()))
            .unwrap_or((map_str.as_str().trim(), ""));

        let text_values = map_text_str
            .lines()
            .filter_map(|line| line.split_once("::"))
            .filter_map(|(key, value)| Some((key.parse::<usize>().ok()?, value)))
            .fold(
                Vec::from_iter(iter::repeat(None).take(15)),
                |mut acc, (key, value)| {
                    acc[key] = Some(value);
                    acc
                },
            );

        let y_len = map_board_str.lines().count();
        let x_len = map_board_str
            .lines()
            .next()
            .map(|line| line.len())
            .unwrap_or_default();
        let lines = (map_board_str.lines().next().is_some()
            && map_board_str
                .lines()
                .try_fold((), |_, line| line.len().eq(&x_len).then_some(()))
                .is_some())
        .then(|| map_board_str.lines().rev().map(|line| line.chars()))
        .ok_or("map isn't consitent")?;

        let mut map = (0..x_len)
            .map(|_| {
                (0..y_len)
                    .map(|_| Tile::default())
                    .collect::<Vec<Tile>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        for (pos, tile) in lines
            .enumerate()
            .flat_map(|(y, tiles)| tiles.enumerate().map(move |(x, t)| ([x, y], t)))
        {
            let tile = match tile {
                'z' | 'x' | 'c' | 'v' | 'b' | 'n' | 'm' => Tile::new(
                    None,
                    Some(BottomTileType::PressurePlate(tile.to_ascii_uppercase())),
                ),
                'Z' | 'X' | 'C' | 'V' | 'B' | 'N' | 'M'
                    if tile.is_ascii_alphabetic() && tile.is_uppercase() =>
                {
                    Tile::new(Some(TopTileType::Door(tile)), None)
                }
                tile if tile.is_ascii_digit() => tile
                    .to_digit(10)
                    .and_then(|index| text_values.get(index as usize).map(|s| s.as_ref()))
                    .flatten()
                    .copied()
                    .map(|value| {
                        Tile::new(
                            Some(TopTileType::Wall),
                            Some(BottomTileType::TextHint(value.into())),
                        )
                    })
                    .unwrap_or_else(|| {
                        log::warn!("missing text value");
                        Tile::default()
                    }),
                '#' => Tile::new(Some(TopTileType::Wall), None),
                '%' => Tile::new(Some(TopTileType::Snake), None),
                '$' => Tile::new(None, Some(BottomTileType::Spike)),
                '|' => Tile::new(None, Some(BottomTileType::Exit)),
                _ => Tile::default(),
            };

            map[pos[0]][pos[1]] = tile;
        }

        Ok(MapAsset(map))
    }

    fn extensions(&self) -> &[&str] {
        &["game_map"]
    }
}

pub fn map_plugin(app: &mut App) {
    app.register_asset_loader(MapAssetLoader)
        .init_asset::<MapAsset>()
        .insert_resource(MapName("maps/map_1.game_map".to_string()))
        .observe(on_grid_added)
        .observe(on_grid_removed)
        .add_systems(OnEnter(GameState::Loading), start_map_load)
        .add_systems(OnEnter(GameState::Gaming), init_rewinds)
        .add_systems(Update, on_map_loaded.run_if(in_state(GameState::Loading)));
}

fn on_grid_added(
    trigger: Trigger<OnAdd, GridPos>,
    mut grid_ents: Query<(&mut Transform, &GridPos)>,
) {
    let Ok(mut grid_ent) = grid_ents.get_mut(trigger.entity()) else {
        log::warn!("trigger ent isn't in query");
        return;
    };

    grid_ent.0.translation =
        (GRID_CELL_SIZE * grid_ent.1.to_vec()).extend(grid_ent.0.translation.z);
}

fn on_grid_removed(
    trigger: Trigger<OnRemove, GridPos>,
    grid_ents: Query<&GridPos>,
    mut map: ResMut<GameMap>,
) {
    let grid_ent = grid_ents
        .get(trigger.entity())
        .expect("triggered ent isn't in query!");

    map.get_mut(*grid_ent)
        .expect("should be valid grid pos")
        .top_removed();
}

pub fn start_map_load(
    mut commands: Commands,
    map_name: Res<MapName>,
    assset_server: Res<AssetServer>,
) {
    commands.insert_resource(AssetHolder::<MapAsset>(
        assset_server.load(map_name.0.clone()),
    ));
}

pub fn on_map_loaded(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    door_sprites: Res<DoorSprites>,
    ui_resources: Res<UiResources>,
    assset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<MapAsset>>,
    asset_map: Res<AssetHolder<MapAsset>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if matches!(
        assset_server.load_state(asset_map.0.id()),
        LoadState::Failed(_)
    ) {
        next_state.set(GameState::MainMenu)
    }

    if assset_server.load_state(asset_map.0.id()) != LoadState::Loaded {
        return;
    }

    let Some(MapAsset(map)) = assets.remove(asset_map.0.id()) else {
        log::warn!("missing map asset after it's loaded");
        return;
    };
    commands.remove_resource::<AssetHolder<MapAsset>>();

    for (pos, tile) in map
        .iter()
        .enumerate()
        .flat_map(|(x, line)| line.iter().enumerate().map(move |(y, tile)| ([x, y], tile)))
    {
        tile.spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            door_sprites.as_ref(),
            ui_resources.as_ref(),
            GridPos(pos),
        )
    }

    commands.insert_resource(GameMap(map));

    next_state.set(GameState::Gaming)
}

fn init_rewinds(mut rewinds: ResMut<RewindCounter>, map: Res<MapName>) {
    let (total, individual) = match map.0.as_str() {
        "maps/map_1.game_map" => (2, 1),
        "maps/map_2.game_map" => (100, 100),
        "maps/map_3.game_map" => (100, 100),
        "maps/map_4.game_map" => (100, 100),
        "maps/map_5.game_map" => (100, 100),
        _ => (100, 100),
    };

    rewinds.total = total;
    rewinds.individual = individual;
}
