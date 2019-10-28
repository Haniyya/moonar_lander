use ggez::{
    event::EventHandler, graphics::*, input::keyboard::KeyCode, timer, Context, GameResult,
};
use nalgebra as na;
use std::collections::HashSet;
use std::time::Duration;

type D = f32;
type Vector = na::Vector2<D>;
type Point = na::Point2<D>;

static MOON_G: Force = Force(0., 8.);
// Thruster force when lander is pointing to the right
static THRUSTER: Force = Force(30., 0.);

fn white() -> Color {
    Color::from_rgb(255, 255, 255)
}

fn stroke() -> DrawMode {
    DrawMode::Stroke(StrokeOptions::default())
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    E = 0,
    NE = 1,
    N = 2,
    NW = 3,
    W = 4,
    SW = 5,
    S = 6,
    SE = 7,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::E
    }
}

impl From<&HashSet<KeyCode>> for Direction {
    fn from(s: &HashSet<KeyCode>) -> Self {
        if s.contains(&KeyCode::Up) {
            if s.contains(&KeyCode::Left) {
                Direction::NW
            } else if s.contains(&KeyCode::Right) {
                Direction::NE
            } else {
                Direction::N
            }
        } else if s.contains(&KeyCode::Down) {
            if s.contains(&KeyCode::Left) {
                Direction::SW
            } else if s.contains(&KeyCode::Right) {
                Direction::SE
            } else {
                Direction::S
            }
        } else {
            if s.contains(&KeyCode::Left) {
                Direction::W
            } else {
                Direction::E
            }
        }
    }
}

/// Translate a direciton into an angle
impl Into<D> for Direction {
    fn into(self) -> D {
        use std::f32::consts::FRAC_PI_8 as frac;
        let index = self as i32;
        frac * 2.0 * (index as D)
    }
}

impl Into<na::Rotation2<D>> for Direction {
    fn into(self) -> na::Rotation2<D> {
        na::Rotation2::new(self.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Force(pub f32, pub f32);

impl Into<Vector> for Force {
    fn into(self) -> Vector {
        Vector::new(self.0, self.1)
    }
}

impl Force {
    pub fn to_velocity(self, d: Duration) -> Vector {
        let scale = timer::duration_to_f64(d) as f32;
        let acceleration: Vector = self.into();
        acceleration.scale(scale)
    }

    pub fn per_second(self) -> Vector {
        self.to_velocity(Duration::from_secs(1))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Lander {
    dir: Direction,
    coordinates: Point,
    velocity: Vector,
}

impl Lander {
    fn dir(&mut self, d: Direction) {
        self.dir = d;
    }
}

impl Default for Lander {
    fn default() -> Self {
        Lander {
            dir: Direction::default(),
            coordinates: Point::new(100., 100.),
            velocity: Vector::new(0., 0.),
        }
    }
}

impl EventHandler for Lander {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        use ggez::input::keyboard::*;
        let arrow_keys = [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];
        if pressed_keys(ctx).iter().any(|k| arrow_keys.contains(k)) {
            self.dir(pressed_keys(ctx).into())
        }
        let delta_seconds = timer::duration_to_f64(timer::delta(&ctx)) as f32;
        let mut delta_v = MOON_G.per_second().scale(delta_seconds);
        if is_key_pressed(ctx, KeyCode::Space) {
            let new_force = THRUSTER.per_second().scale(delta_seconds);
            let rotation: na::Rotation2<D> = na::Rotation2::new(self.dir.into());
            delta_v += rotation * new_force;
        }
        self.velocity += delta_v;
        self.coordinates += self.velocity.scale(delta_seconds);
        GameResult::Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (width, height) = (15., 30.);
        let poly = [
            Point::new(height / 2., 0.),
            Point::new(-height / 2., width / 2.),
            Point::new(-height / 2., -width / 2.),
        ];
        let params = DrawParam::default()
            .dest(self.coordinates)
            .rotation(self.dir.into());
        MeshBuilder::new()
            .polygon(stroke(), &poly, white())?
            .build(ctx)?
            .draw(ctx, params)?;
        GameResult::Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
struct Moonar {
    lander: Lander,
    heightmap: Vec<i32>,
    score: u16,
}

impl EventHandler for Moonar {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.lander.update(ctx).expect("Unable to update lander.");
        GameResult::Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        ggez::graphics::clear(ctx, Color::from_rgb(0, 0, 0));
        self.lander.draw(ctx)?;
        ggez::graphics::present(ctx)
    }
}

fn main() -> GameResult {
    let mut game = Moonar::default();
    let (mut ctx, mut ev_loop) = ggez::ContextBuilder::new("moonar", "Paul Martensen")
        .build()
        .unwrap();
    println!("{}", ggez::graphics::renderer_info(&ctx)?);
    ggez::event::run(&mut ctx, &mut ev_loop, &mut game)
}
