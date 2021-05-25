//! The simplest possible example that does something.

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

const PI: f32 = std::f32::consts::PI;

const GRID_CELL_SIZE: (f32, f32) = (50.0, 50.0);

const GRID_SIZE: (u16, u16) = (10, 10);

const WINDOW_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1,
);

const PLAYER_SIZE: f32 = 10.0;
const PLAYER_ROTATION_SPEED: f32 = 0.20;
const PLAYER_MOVE_SPEED: f32 = 1.50;

#[rustfmt::skip]
const MAP: [i32; (GRID_SIZE.0 * GRID_SIZE.1) as usize] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 1, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 1, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 1, 0, 0, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

#[derive(Copy, Clone)]
struct Cell {
    row: u16,
    col: u16,
    is_wall: bool,
}

impl Cell {
    pub fn default() -> Self {
        Cell {
            row: 0,
            col: 0,
            is_wall: false,
        }
    }

    pub fn new(row: u16, col: u16, is_wall: bool) -> Self {
        Cell { row, col, is_wall }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let bounds = graphics::Rect {
            x: Into::<f32>::into(self.col) * GRID_CELL_SIZE.0 + 1.0,
            y: Into::<f32>::into(self.row) * GRID_CELL_SIZE.1 + 1.0,
            w: GRID_CELL_SIZE.0,
            h: GRID_CELL_SIZE.1,
        };

        let mode = if self.is_wall {
            graphics::DrawMode::fill()
        } else {
            graphics::DrawMode::stroke(1.0)
        };
        let color = if self.is_wall {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            [1.0, 0.0, 0.0, 1.0]
        };

        let rectangle = graphics::Mesh::new_rectangle(ctx, mode, bounds, color.into())?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }
}

struct Player {
    x: f32,
    y: f32,
    orientation: f32,
}

impl Player {
    fn new() -> Self {
        Player {
            x: WINDOW_SIZE.0 / 2.0,
            y: WINDOW_SIZE.1 / 2.0,
            orientation: 0.0,
        }
    }
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let points: Vec<ggez::mint::Point2<f32>> = vec![
            ggez::mint::Point2 {
                x: self.x + self.orientation.cos() * PLAYER_SIZE * 2.0,
                y: self.y + self.orientation.sin() * PLAYER_SIZE * 2.0,
            },
            ggez::mint::Point2 {
                x: self.x + (self.orientation + 2.0 * PI / 3.0).cos() * PLAYER_SIZE,
                y: self.y + (self.orientation + 2.0 * PI / 3.0).sin() * PLAYER_SIZE,
            },
            ggez::mint::Point2 {
                x: self.x + (self.orientation - 2.0 * PI / 3.0).cos() * PLAYER_SIZE,
                y: self.y + (self.orientation - 2.0 * PI / 3.0).sin() * PLAYER_SIZE,
            },
        ];

        let triangle = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::stroke(1.0),
            &points,
            Color::BLACK,
        )?;
        graphics::draw(ctx, &triangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }

    fn rotate_right(&mut self) {
        self.orientation = (self.orientation + PLAYER_ROTATION_SPEED) % (2.0 * PI);
    }

    fn rotate_left(&mut self) {
        self.orientation = (self.orientation - PLAYER_ROTATION_SPEED) % (2.0 * PI);
    }

    fn move_forward(&mut self) {
        self.x += self.orientation.cos() * PLAYER_MOVE_SPEED;
        self.y += self.orientation.sin() * PLAYER_MOVE_SPEED;
    }
}

struct MainState {
    cells: [Cell; GRID_SIZE.0 as usize * GRID_SIZE.1 as usize],
    player: Player,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let mut cells: [Cell; GRID_SIZE.0 as usize * GRID_SIZE.1 as usize] =
            [Cell::default(); GRID_SIZE.0 as usize * GRID_SIZE.1 as usize];
        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let cell_index = (i * GRID_SIZE.0 + j) as usize;
                cells[cell_index] = Cell::new(i, j, MAP[cell_index] == 0);
            }
        }
        let player = Player::new();
        let s = MainState { cells, player };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for cell in &self.cells {
            cell.draw(ctx)?;
        }
        self.player.draw(ctx)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Up => {
                self.player.move_forward();
            }
            KeyCode::Left => {
                self.player.rotate_left();
            }
            KeyCode::Right => {
                self.player.rotate_right();
            }
            KeyCode::Escape => event::quit(ctx),
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("Raycasting", "ggez").window_mode(
        ggez::conf::WindowMode::default().dimensions(WINDOW_SIZE.0 + 1.0, WINDOW_SIZE.1 + 1.0),
    );
    let (ctx, event_loop) = cb.build()?;

    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
