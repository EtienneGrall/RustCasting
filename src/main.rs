//! The simplest possible example that does something.

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

const PI: f32 = std::f32::consts::PI;

const GRID_CELL_SIZE: (f32, f32) = (50.0, 50.0);
const GRID_SIZE: (u16, u16) = (10, 10);
const CELL_NUMBER: usize = (GRID_SIZE.0 * GRID_SIZE.1) as usize;

const WINDOW_HEIGHT: f32 = GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1;

const GRID_WINDOW_SIZE: (f32, f32) = (GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0, WINDOW_HEIGHT);
const MAX_RAY_LENGTH: f32 = 700.0;

const WORLD_WINDOW_WIDTH: f32 = 1000.0;

const PLAYER_SIZE: f32 = 10.0;
const PLAYER_ROTATION_SPEED: f32 = 0.1;
const PLAYER_MOVE_SPEED: f32 = 3.0;

const RAYS_CONE_ANGLE: f32 = PI / 6.0;
const NUMBER_OF_RAYS: u16 = 150;

#[rustfmt::skip]
const MAP: [i32; CELL_NUMBER] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 1, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 1, 1, 0, 0, 1,
    1, 0, 1, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 1, 1, 1, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 1, 0, 0, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn get_normalized_angle(angle: f32) -> f32 {
    let rem = angle % (2.0 * PI);
    if rem < 0.0 {
        rem + 2.0 * PI
    } else {
        rem
    }
}

fn get_cell(i: u16, j: u16, grid: [Cell; CELL_NUMBER]) -> Cell {
    grid[(j * GRID_SIZE.0 + i) as usize]
}

fn get_column_index(position_x: f32) -> u16 {
    (position_x / GRID_CELL_SIZE.0).floor() as u16
}

fn get_row_index(position_y: f32) -> u16 {
    (position_y / GRID_CELL_SIZE.1).floor() as u16
}

fn get_norm(x: f32, y: f32) -> f32 {
    (x.powi(2) + y.powi(2)).sqrt()
}

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
            [1.0, 0.0, 0.0, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        let rectangle = graphics::Mesh::new_rectangle(ctx, mode, bounds, color.into())?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }
}

#[derive(Copy, Clone)]
struct Ray {
    x: f32,            // x position of the player
    y: f32,            // y position of the player
    orientation: f32,  // Orientation of the player. Same for all rays
    angle_offset: f32, // Offset for this specific ray
    length: f32,
    hit_vertical: bool,
}

impl Ray {
    pub fn default() -> Self {
        Ray {
            x: 0.0,
            y: 0.0,
            angle_offset: 0.0,
            orientation: 0.0,
            length: 0.0,
            hit_vertical: false,
        }
    }

    fn new(x: f32, y: f32, angle_offset: f32, orientation: f32) -> Self {
        Ray {
            x,
            y,
            angle_offset,
            orientation,
            length: 0.0,
            hit_vertical: false,
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let points: Vec<ggez::mint::Point2<f32>> = vec![
            ggez::mint::Point2 {
                x: self.x,
                y: self.y,
            },
            ggez::mint::Point2 {
                x: self.x + (self.angle_offset + self.orientation).cos() * self.length,
                y: self.y + (self.angle_offset + self.orientation).sin() * self.length,
            },
        ];

        let line = graphics::Mesh::new_line(ctx, &points, 0.1, Color::WHITE)?;
        graphics::draw(ctx, &line, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }

    fn draw_wall(&self, ctx: &mut Context) -> GameResult<()> {
        // We multiply the cos to prevent fisheye effect
        // https://www.permadi.com/tutorial/raycast/rayc8.html
        let wall_height = WINDOW_HEIGHT
            - 1.5 * WINDOW_HEIGHT * self.length * self.angle_offset.cos() / MAX_RAY_LENGTH;
        let bounds = graphics::Rect {
            x: GRID_WINDOW_SIZE.0
                + (WORLD_WINDOW_WIDTH / 2.0)
                + WORLD_WINDOW_WIDTH * self.angle_offset / RAYS_CONE_ANGLE,
            y: (WINDOW_HEIGHT - wall_height) / 2.0,
            w: WORLD_WINDOW_WIDTH / NUMBER_OF_RAYS as f32,
            h: wall_height,
        };

        let mode = graphics::DrawMode::fill();
        let color = if self.hit_vertical {
            [0.8, 0.8, 0.8, 1.0]
        } else {
            [0.7, 0.7, 0.7, 1.0]
        };

        let rectangle = graphics::Mesh::new_rectangle(ctx, mode, bounds, color.into())?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }

    fn is_facing_up(&mut self) -> bool {
        get_normalized_angle(self.orientation + self.angle_offset) > PI
    }

    fn is_facing_down(&mut self) -> bool {
        get_normalized_angle(self.orientation + self.angle_offset) < PI
    }

    fn is_facing_left(&mut self) -> bool {
        let full_angle = get_normalized_angle(self.orientation + self.angle_offset);
        full_angle > PI / 2.0 && full_angle < 3.0 * PI / 2.0
    }

    fn is_facing_right(&mut self) -> bool {
        let full_angle = get_normalized_angle(self.orientation + self.angle_offset);
        full_angle < PI / 2.0 || full_angle > 3.0 * PI / 2.0
    }

    fn is_horizontal(&mut self) -> bool {
        let full_angle = get_normalized_angle(self.orientation + self.angle_offset);
        full_angle == 0.0 || full_angle == PI
    }

    fn is_vertical(&mut self) -> bool {
        let full_angle = get_normalized_angle(self.orientation + self.angle_offset);
        full_angle == PI / 2.0 || full_angle == 3.0 * PI / 2.0
    }

    fn get_x0(&mut self) -> f32 {
        if self.is_facing_left() {
            -self.x % GRID_CELL_SIZE.0
        } else if self.is_facing_right() {
            GRID_CELL_SIZE.0 - self.x % GRID_CELL_SIZE.0
        } else {
            MAX_RAY_LENGTH
        }
    }

    fn get_y0(&mut self) -> f32 {
        if self.is_facing_up() {
            -self.y % GRID_CELL_SIZE.1
        } else if self.is_facing_down() {
            GRID_CELL_SIZE.1 - self.y % GRID_CELL_SIZE.1
        } else {
            MAX_RAY_LENGTH
        }
    }

    fn get_length_horizontal_collision(&mut self, grid: &mut [Cell; CELL_NUMBER]) -> f32 {
        let tan = get_normalized_angle(self.orientation + self.angle_offset).tan();
        let delta_y = if self.is_facing_up() {
            -GRID_CELL_SIZE.1
        } else {
            GRID_CELL_SIZE.1
        };
        let delta_x = delta_y / tan;

        // Row index where the ray is starting
        let m_y_0 = get_row_index(self.y);

        let y0 = self.get_y0();

        // Compute the sequence of intersection positions between the ray and horizontal lines
        if self.is_horizontal() {
            return MAX_RAY_LENGTH;
        }

        let mut horizontal_intersection_x = self.x + y0 / tan;
        let mut horizontal_intersection_y = self.y + y0;
        let mut m_y = m_y_0;

        loop {
            // The ray is out of bound and it didn't meet any wall
            if horizontal_intersection_x < 0.0
                || horizontal_intersection_y < 0.0
                || horizontal_intersection_x > GRID_WINDOW_SIZE.0
                || horizontal_intersection_y > GRID_WINDOW_SIZE.1
            {
                return MAX_RAY_LENGTH;
            }

            let m_x = get_column_index(horizontal_intersection_x);
            m_y = if self.is_facing_up() {
                m_y - 1
            } else {
                m_y + 1
            };

            let cell_to_check = get_cell(m_x, m_y, *grid);

            if cell_to_check.is_wall {
                return get_norm(
                    horizontal_intersection_x - self.x,
                    horizontal_intersection_y - self.y,
                );
            }

            horizontal_intersection_x = horizontal_intersection_x + delta_x;
            horizontal_intersection_y = horizontal_intersection_y + delta_y;
        }
    }

    fn get_length_vertical_collision(&mut self, grid: &mut [Cell; CELL_NUMBER]) -> f32 {
        let tan = get_normalized_angle(self.orientation + self.angle_offset).tan();
        let delta_x = if self.is_facing_left() {
            -GRID_CELL_SIZE.0
        } else {
            GRID_CELL_SIZE.0
        };
        let delta_y = delta_x * tan;

        // Col index where the ray is starting
        let m_x_0 = get_column_index(self.x);

        let x0 = self.get_x0();

        // Compute the sequence of intersection positions between the ray and horizontal lines
        if self.is_vertical() {
            return MAX_RAY_LENGTH;
        }

        let mut horizontal_intersection_x = self.x + x0;
        let mut horizontal_intersection_y = self.y + x0 * tan;
        let mut m_x = m_x_0;

        loop {
            // The ray is out of bound and it didn't meet any wall
            if horizontal_intersection_x < 0.0
                || horizontal_intersection_y < 0.0
                || horizontal_intersection_x > GRID_WINDOW_SIZE.0
                || horizontal_intersection_y > GRID_WINDOW_SIZE.1
            {
                return MAX_RAY_LENGTH;
            }

            m_x = if self.is_facing_left() {
                m_x - 1
            } else {
                m_x + 1
            };
            let m_y = get_row_index(horizontal_intersection_y);

            let cell_to_check = get_cell(m_x, m_y, *grid);

            if cell_to_check.is_wall {
                return get_norm(
                    horizontal_intersection_x - self.x,
                    horizontal_intersection_y - self.y,
                );
            }

            horizontal_intersection_x = horizontal_intersection_x + delta_x;
            horizontal_intersection_y = horizontal_intersection_y + delta_y;
        }
    }

    fn update_length(&mut self, grid: &mut [Cell; CELL_NUMBER]) {
        let vertical = self.get_length_vertical_collision(grid);
        let horizontal = self.get_length_horizontal_collision(grid);
        if vertical < horizontal {
            self.length = vertical;
            self.hit_vertical = true;
        } else {
            self.length = horizontal;
            self.hit_vertical = false;
        }
    }

    fn update_position(&mut self, player: &mut Player) -> GameResult {
        self.x = player.x;
        self.y = player.y;
        self.orientation = (player.orientation + self.angle_offset) % (2.0 * PI);
        Ok(())
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
            x: GRID_WINDOW_SIZE.0 / 2.0,
            y: GRID_WINDOW_SIZE.1 / 2.0,
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
            Color::WHITE,
        )?;
        graphics::draw(ctx, &triangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }

    fn rotate_right(&mut self) {
        self.orientation = get_normalized_angle(self.orientation + PLAYER_ROTATION_SPEED);
    }

    fn rotate_left(&mut self) {
        self.orientation = get_normalized_angle(self.orientation - PLAYER_ROTATION_SPEED);
    }

    fn move_forward(&mut self) {
        self.x += self.orientation.cos() * PLAYER_MOVE_SPEED;
        self.y += self.orientation.sin() * PLAYER_MOVE_SPEED;
    }
}

struct MainState {
    cells: [Cell; CELL_NUMBER],
    player: Player,
    rays: [Ray; NUMBER_OF_RAYS as usize],
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let mut cells: [Cell; CELL_NUMBER] = [Cell::default(); CELL_NUMBER];
        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let cell_index = (i * GRID_SIZE.0 + j) as usize;
                cells[cell_index] = Cell::new(i, j, MAP[cell_index] == 1);
            }
        }
        let player = Player::new();

        let mut rays: [Ray; NUMBER_OF_RAYS as usize] = [Ray::default(); NUMBER_OF_RAYS as usize];
        for i in 0..NUMBER_OF_RAYS {
            let offset = -(RAYS_CONE_ANGLE / 2.0)
                + RAYS_CONE_ANGLE * (i as f32 / (NUMBER_OF_RAYS - 1) as f32);
            rays[i as usize] = Ray::new(player.x, player.y, offset, player.orientation);
        }

        let s = MainState {
            cells,
            player,
            rays,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let rays = &mut self.rays;
        for ray in rays {
            ray.update_position(&mut self.player)?;
            ray.update_length(&mut self.cells);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for cell in &self.cells {
            cell.draw(ctx)?;
        }
        self.player.draw(ctx)?;

        for ray in &self.rays {
            ray.draw(ctx)?;
        }

        for ray in &self.rays {
            ray.draw_wall(ctx)?;
        }

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
        ggez::conf::WindowMode::default().dimensions(
            GRID_WINDOW_SIZE.0 + 1.0 + GRID_WINDOW_SIZE.0 * 2.0,
            GRID_WINDOW_SIZE.1 + 1.0,
        ),
    );
    let (ctx, event_loop) = cb.build()?;

    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
