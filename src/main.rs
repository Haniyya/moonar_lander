use ggez::{event::EventHandler, graphics::*, timer, Context, GameResult};
use nalgebra as na;
use rand;
use rand::*;
use std::collections::LinkedList;
use std::time::Duration;

type D = f32;
type Vector = na::Vector2<D>;
type Point = na::Point2<D>;

static MOON_G: Force = Force(0., 8.);
// Thruster force when lander is pointing to the right
static THRUSTER: Force = Force(50., 0.);
static FULL_TURN_MILLIS: u64 = 3000;
static TURN_TIME: Duration = Duration::from_millis(FULL_TURN_MILLIS / (Lander::dir_count() as u64));

fn white() -> Color {
    Color::from_rgb(255, 255, 255)
}

fn stroke() -> DrawMode {
    DrawMode::Stroke(StrokeOptions::default())
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
    dir: u8,
    turn_cooldown: Duration,
    coordinates: Point,
    velocity: Vector,
}

impl Lander {
    const fn dir_count() -> u8 {
        32
    }

    fn dir(&mut self, d: u8) {
        self.dir = d % Self::dir_count();
    }

    fn change_dir(&mut self, d: i8, delta: Duration) {
        if self.turn_cooldown <= delta {
            self.turn_cooldown = TURN_TIME;
            self.dir(((self.dir as i8) + d) as u8)
        }
    }

    fn angle(&self) -> D {
        use std::f32::consts::FRAC_PI_8 as frac;

        frac * (self.dir as f32)
    }
}

impl Default for Lander {
    fn default() -> Self {
        Lander {
            dir: 0,
            turn_cooldown: Duration::from_secs(0),
            coordinates: Point::new(100., 100.),
            velocity: Vector::new(0., 0.),
        }
    }
}

impl EventHandler for Lander {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        use ggez::input::keyboard::*;
        let delta = timer::delta(ctx);
        let delta_seconds = timer::duration_to_f64(delta) as f32;
        let mut apply_change = |key| {
            if is_key_pressed(ctx, key) {
                let dir = match key {
                    KeyCode::Left => -1,
                    KeyCode::Right => 1,
                    _ => 0,
                };
                self.change_dir(dir, timer::delta(ctx))
            }
        };
        apply_change(KeyCode::Left);
        apply_change(KeyCode::Right);
        self.turn_cooldown = self
            .turn_cooldown
            .checked_sub(delta)
            .unwrap_or(Duration::from_micros(0));
        let mut delta_v = MOON_G.per_second().scale(delta_seconds);
        if is_key_pressed(ctx, KeyCode::Space) || is_key_pressed(ctx, KeyCode::Up) {
            let new_force = THRUSTER.per_second().scale(delta_seconds);
            let rotation: na::Rotation2<D> = na::Rotation2::new(self.angle());
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
            .rotation(self.angle());
        MeshBuilder::new()
            .polygon(stroke(), &poly, white())?
            .build(ctx)?
            .draw(ctx, params)?;
        GameResult::Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Moonar {
    lander: Lander,
    heightmap: LinkedList<u32>,
    score: u16,
}

impl Default for Moonar {
    fn default() -> Self {
        Moonar {
            lander: Lander::default(),
            heightmap: Self::generate_heightmap(),
            score: 0,
        }
    }
}

impl Moonar {
    const fn max_height() -> u32 {
        120
    }

    const fn max_degree() -> u32 {
        80
    }

    const fn map_length() -> usize {
        50
    }

    fn generate_heightmap() -> LinkedList<u32> {
        let mut rng = rand::thread_rng();
        let mut vector = LinkedList::new();
        let mut last = 0u32;
        for _ in 0..=Self::map_length() {
            let next = (last as i32)
                .checked_add(rng.gen::<i32>() % (Self::max_degree() as i32))
                .unwrap_or(10);
            last = (next as u32).max(Self::max_height());
            vector.push_back(last);
        }
        vector
    }

    fn draw_map(&self, ctx: &mut Context) -> GameResult {
        let (width, w_height) = ggez::graphics::drawable_size(ctx);
        let segment_width = width / (Self::map_length() as f32);
        let points: Vec<Point> = self
            .heightmap
            .iter()
            .enumerate()
            .map(|(index, &height)| Point::new((index as f32) * segment_width, -(height as f32)))
            .collect();
        MeshBuilder::new()
            .line(&points, 1., white())?
            .build(ctx)?
            .draw(
                ctx,
                DrawParam::default().dest(Point::new(0., w_height * 1.1)),
            )
    }
}

impl EventHandler for Moonar {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.lander.update(ctx).expect("Unable to update lander.");
        GameResult::Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        ggez::graphics::clear(ctx, Color::from_rgb(0, 0, 0));
        self.draw_map(ctx)?;
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
