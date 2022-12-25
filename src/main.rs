use bevy::{
    pbr::{AmbientLight, PointLight, PointLightBundle},
    prelude::{
        shape, App, Assets, BuildChildren, Camera3dBundle, ClearColor, Color, Commands, Component,
        Entity, Mesh, Msaa, PbrBundle, Plugin, Query, Res, ResMut, Resource, SpatialBundle,
        StandardMaterial, SystemSet, Transform, Vec2, Vec3, Visibility,
    },
    time::FixedTimestep,
    utils::HashMap,
    DefaultPlugins,
};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

const TIME_STEP: f64 = 0.5;
const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;
const GRID_DEPTH: i32 = 20;

// Candidates for the Game of Life in Three Dimensions, Carter Bays
// Department of Computer Science, University of South Carolina, Columbia, SC 29208, USA
// URL: https://content.wolfram.com/uploads/sites/13/2018/02/01-3-1.pdf
const EB: i32 = 4;
const FB: i32 = 5;
const EH: i32 = 5;
const FH: i32 = 5;

#[derive(Debug, Eq, PartialEq)]
enum CellState {
    Alive,
    Dead,
}

#[derive(Component, Debug)]
struct Cell {
    state: CellState,
}

#[derive(Component, Debug, Eq, PartialEq, Hash, Clone)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

impl Position {
    fn new(x: i32, y: i32, z: i32) -> Position {
        Position { x, y, z }
    }
}

impl Cell {
    fn new(state: CellState) -> Cell {
        Cell { state }
    }

    fn is_alive(&self) -> bool {
        self.state == CellState::Alive
    }
}

pub struct GameOfLife;

type CellGrid = HashMap<Position, Cell>;

#[derive(Resource, Default)]
struct Grid {
    cells: CellGrid,
}

impl Grid {
    fn new(width: i32, height: i32, depth: i32) -> Grid {
        let mut cells = CellGrid::new();
        for x in 0..width {
            for y in 0..height {
                for z in 0..depth {
                    let state = if rand::random() {
                        CellState::Alive
                    } else {
                        CellState::Dead
                    };
                    let position = Position::new(x, y, z);
                    cells.insert(position, Cell::new(state));
                }
            }
        }
        Grid { cells }
    }

    fn get_cell_mut(&mut self, position: &Position) -> Option<&mut Cell> {
        self.cells.get_mut(position)
    }

    fn get_cell(&self, position: &Position) -> Option<&Cell> {
        self.cells.get(position)
    }
}
impl Plugin for GameOfLife {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(Grid::new(GRID_WIDTH, GRID_HEIGHT, GRID_DEPTH))
            .add_startup_system(setup_game)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(TIME_STEP))
                    .with_system(print_position_system),
            );
    }
}

fn live_neighbors(grid: &Grid, position: &Position) -> i32 {
    let mut alives = 0;
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                let key = Position::new(position.x + x, position.y + y, position.z + z);
                if let Some(cell) = grid.get_cell(&key) {
                    alives += if cell.is_alive() { 1 } else { 0 }
                }
            }
        }
    }
    alives
}

fn print_position_system(mut grid: ResMut<Grid>, mut query: Query<(&Position, &mut Visibility)>) {
    for (position, mut visibility) in query.iter_mut() {
        let alive = live_neighbors(&grid, position);
        let cell = grid.get_cell_mut(position).unwrap();

        match cell.state {
            CellState::Alive => {
                if EB <= alive && alive <= EH {
                    cell.state = CellState::Alive;
                } else {
                    cell.state = CellState::Dead;
                }
            }
            CellState::Dead => {
                if FB <= alive && alive <= FH {
                    cell.state = CellState::Alive;
                } else {
                    cell.state = CellState::Dead;
                }
            }
        }

        visibility.is_visible = cell.is_alive();
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: Res<Grid>,
) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_translate_sensitivity: Vec2::new(4.0, 4.0),
                ..OrbitCameraController::default()
            },
            Vec3::new(
                GRID_WIDTH as f32 * 2.0,
                GRID_HEIGHT as f32 * 2.0,
                GRID_DEPTH as f32 * 2.0,
            ),
            Vec3::ZERO,
        ));

    for x in 0..=2 {
        for y in 0..=2 {
            for z in 0..=2 {
                commands.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 1000.0,
                        ..PointLight::default()
                    },
                    transform: Transform::from_xyz(
                        -(GRID_WIDTH as f32 / 2.0) + (GRID_WIDTH as f32 * x as f32),
                        -(GRID_HEIGHT as f32 / 2.0) + (GRID_HEIGHT as f32 * y as f32),
                        -(GRID_DEPTH as f32 / 2.0) + (GRID_DEPTH as f32 * z as f32),
                    ),
                    ..PointLightBundle::default()
                });
            }
        }
    }

    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.98, 0.95, 0.92),
        brightness: 0.2,
    });

    let parent = commands
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(
                -(GRID_WIDTH as f32 / 2.0),
                -(GRID_HEIGHT as f32 / 2.0),
                -(GRID_DEPTH as f32 / 2.0),
            ),
            ..SpatialBundle::default()
        })
        .id();

    let children: Vec<Entity> = grid
        .cells
        .iter()
        .map(|(position, cell)| {
            commands
                .spawn((
                    position.clone(),
                    PbrBundle {
                        visibility: Visibility {
                            is_visible: cell.state == CellState::Alive,
                        },
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.02, 0.8, 0.08).into()),
                        transform: Transform::from_xyz(
                            position.x as f32 + 5.0,
                            position.y as f32 + 5.0,
                            position.z as f32 + 5.0,
                        ),
                        ..PbrBundle::default()
                    },
                ))
                .id()
        })
        .collect();

    commands.entity(parent).push_children(&children);
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(GameOfLife)
        .run();
}
