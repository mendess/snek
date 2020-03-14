use ggez::{
    conf,
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics::{self, DrawParam},
    Context, ContextBuilder, GameResult,
};
use rand::distributions::{Distribution, Uniform};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const MAP_SIZE: usize = 20;
const UPDATE_RATE: u64 = 100;
const CELL_SIZE: f32 = 10.0;

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("snek", "Mendess")
        .window_mode(conf::WindowMode::default().dimensions(
            MAP_SIZE as f32 * CELL_SIZE + 80.0,
            MAP_SIZE as f32 * CELL_SIZE + 30.0,
        ))
        .build()
        .expect("could not create ggez context!");

    let mut my_game = MyGame::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

struct MyGame {
    snek_body: VecDeque<Coord>,
    snek_dir: Direction,
    map: [[Tile; MAP_SIZE]; MAP_SIZE],
    state: State,
    last_update: Instant,
    score: usize,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        const HORIZONTAL_WALL: [Tile; MAP_SIZE] = [Tile::Wall; MAP_SIZE];
        const VERTICAL_WALL: [Tile; MAP_SIZE] = {
            let mut a = [Tile::Nothing; MAP_SIZE];
            a[0] = Tile::Wall;
            a[a.len() - 1] = Tile::Wall;
            a
        };
        const MAP: [[Tile; MAP_SIZE]; MAP_SIZE] = {
            let mut map = [VERTICAL_WALL; MAP_SIZE];
            map[0] = HORIZONTAL_WALL;
            map[map.len() - 1] = HORIZONTAL_WALL;
            map
        };
        let mut state = MyGame {
            snek_body: VecDeque::from(vec![(1, 1), (1, 2), (1, 3)]),
            snek_dir: Direction::Right,
            map: MAP,
            state: State::Playing,
            last_update: Instant::now(),
            score: 0,
        };
        state.swap_fruit((1, 1));
        state
    }

    fn swap_fruit(&mut self, (x, y): Coord) {
        let mut rng = rand::thread_rng();
        let x_axis = Uniform::from(1..MAP_SIZE - 1);
        let y_axis = Uniform::from(1..MAP_SIZE - 1);
        self.map[x][y] = Tile::Nothing;
        let (x, y) = loop {
            let (x, y) = (x_axis.sample(&mut rng), y_axis.sample(&mut rng));
            if !self.snek_body.contains(&(x, y)) {
                break (x, y);
            }
        };
        self.map[x][y] = Tile::Fruit;
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.state == State::Playing
            && Instant::now() - self.last_update >= Duration::from_millis(UPDATE_RATE)
        {
            let new_c = new_coord(
                self.snek_dir,
                *self.snek_body.back().expect("Snek is empty?"),
            );
            let (x, y) = new_c;
            if self.snek_body.contains(&(x, y)) {
                self.state = State::GameOver;
            }
            self.snek_body.push_back(new_c);
            if let Tile::Fruit = self.map[x][y] {
                self.score += 1;
                self.swap_fruit((x, y))
            } else {
                self.snek_body.pop_front();
            }
            if let Tile::Wall = self.map[x][y] {
                self.state = State::GameOver;
            }
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        for (x, y, t) in self
            .map
            .iter()
            .enumerate()
            .flat_map(|(x, line)| line.iter().enumerate().map(move |(y, t)| (x, y, t)))
            .filter(|(_, _, t)| **t != Tile::Nothing)
        {
            let rect = graphics::Rect::new(
                x as f32 * CELL_SIZE,
                y as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rect,
                if *t == Tile::Wall {
                    graphics::WHITE
                } else {
                    graphics::Color::new(0.0, 1.0, 0.0, 1.0)
                },
            )?;
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }
        for &(x, y) in self.snek_body.iter().take(self.snek_body.len() - 1) {
            let rect = graphics::Rect::new(
                x as f32 * CELL_SIZE,
                y as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rect,
                graphics::Color::new(1.0, 0.0, 0.0, 1.0),
            )?;
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }
        let head = *self.snek_body.back().unwrap();
        let rect = graphics::Rect::new(
            head.0 as f32 * CELL_SIZE,
            head.1 as f32 * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE,
        );
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            rect,
            graphics::Color::new(0.7, 0.0, 0.0, 1.0),
        )?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;
        if let Some(text) = match self.state {
            State::GameOver => Some("Game Over"),
            State::Pause => Some("Pause"),
            _ => None,
        } {
            let text = graphics::Text::new(
                graphics::TextFragment::new(text)
                    .scale(graphics::Scale::uniform(20.0))
                    .color(graphics::WHITE),
            );
            graphics::draw(
                ctx,
                &text,
                DrawParam::default().dest([0.0, MAP_SIZE as f32 * CELL_SIZE]),
            )?;
        }
        let text = graphics::Text::new(
            graphics::TextFragment::new("SCORE:")
                .scale(graphics::Scale::uniform(20.0))
                .color(graphics::WHITE),
        );
        graphics::draw(
            ctx,
            &text,
            DrawParam::default().dest([MAP_SIZE as f32 * CELL_SIZE, 0.0 * CELL_SIZE]),
        )?;
        let text = graphics::Text::new(
            graphics::TextFragment::new(self.score.to_string())
                .scale(graphics::Scale::uniform(20.0))
                .color(graphics::WHITE),
        );
        graphics::draw(
            ctx,
            &text,
            DrawParam::default().dest([MAP_SIZE as f32 * CELL_SIZE, 2.0 * CELL_SIZE]),
        )?;
        graphics::present(ctx)
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        let mut checked_assign = |d: Direction| {
            if !self.snek_dir.is_inverse(d) {
                self.snek_dir = d;
            }
        };
        match keycode {
            KeyCode::H | KeyCode::A | KeyCode::Left => {
                checked_assign(Direction::Left);
            }
            KeyCode::J | KeyCode::S | KeyCode::Down => {
                checked_assign(Direction::Down);
            }
            KeyCode::K | KeyCode::W | KeyCode::Up => {
                checked_assign(Direction::Up);
            }
            KeyCode::L | KeyCode::D | KeyCode::Right => {
                checked_assign(Direction::Right);
            }
            KeyCode::P => {
                self.state = match self.state {
                    State::Pause => State::Playing,
                    State::Playing => State::Pause,
                    State::GameOver => State::GameOver,
                }
            }
            KeyCode::Q => {
                std::process::exit(0);
            }
            _ => (),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Tile {
    Nothing,
    Wall,
    Fruit,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum State {
    Pause,
    GameOver,
    Playing,
}

type Coord = (usize, usize);

fn new_coord(dir: Direction, (x, y): Coord) -> Coord {
    use Direction::*;
    match dir {
        Up => (x, y - 1),
        Down => (x, y + 1),
        Left => (x - 1, y),
        Right => (x + 1, y),
    }
}

impl Direction {
    fn is_inverse(self, d: Direction) -> bool {
        use Direction::*;
        match (self, d) {
            (Up, Down) => true,
            (Down, Up) => true,
            (Left, Right) => true,
            (Right, Left) => true,
            _ => false,
        }
    }
}
