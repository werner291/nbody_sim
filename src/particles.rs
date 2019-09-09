extern crate image;
extern crate nalgebra;
extern crate rand;

extern crate ggez;
use ggez::graphics::{DrawMode, Point2 as GGPoint2, Color, Rect};
use ggez::*;
use nalgebra::{Point2,Vector2,distance_squared};
use rand::{Rng};

struct Particle {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    acceleration : Vector2<f32>,
    trace: Vec<Point2<f32>>,
}

struct SimState {
    particles : Vec<Particle>
}

impl event::EventHandler for SimState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {

        const TIMESTEP:f32 = 0.1;
        const G:f32 = 1000.0;

        for idx in 0..self.particles.len() {

            let _m1 = 1.0;

            let (left, right) = self.particles.split_at_mut(idx);
            let (par, right) = right.split_first_mut().expect("expected there to be at least one item on the right");

            let mut grav_accel = Vector2::new(0.0f32,0.0);

            for par2 in left.iter().chain(right.iter()) {
                let m2 = 1.0;

                let grav_dist_factor = G / distance_squared(&par.position, &par2.position);

                let mut dir = Vector2::new(par2.position.x - par.position.x,
                                           par2.position.y - par.position.y);
                dir.normalize_mut();

                grav_accel.x += dir.x * grav_dist_factor * m2 as f32;
                grav_accel.y += dir.y * grav_dist_factor * m2 as f32;
            }

            par.acceleration = grav_accel;
        }

        for pt in &mut self.particles {

            pt.velocity.x += pt.acceleration.x * TIMESTEP;
            pt.velocity.y += pt.acceleration.y * TIMESTEP;

            pt.position.x += pt.velocity.x * TIMESTEP;
            pt.position.y += pt.velocity.y * TIMESTEP;

            pt.trace.push(pt.position.clone());
            if pt.trace.len() > 100 as usize {
                pt.trace.remove(0);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let min : f32 = -300.0; // Technically wrong, but origin kinda always visible.
        let max : f32 = 300.0;

//        for pt in &self.particles {
//            min = min.min(pt.position.x);
//            max = max.max(pt.position.x);
//            min = min.min(pt.position.y);
//            max = max.max(pt.position.y);
//        }

        graphics::set_screen_coordinates(ctx, Rect::new(min,min,max-min, max-min));

        for pt in &self.particles {

            graphics::set_color(ctx, Color::from_rgb(255,255,255))?;

            graphics::circle(
                ctx,
                DrawMode::Fill,
                GGPoint2::new(pt.position.x,pt.position.y),
                2.0,
                2.0,
            )?;

            graphics::set_color(ctx, Color::from_rgb(0,0,255))?;

            if pt.trace.len() >= 2 {
                let gg_trace: Vec<GGPoint2> = pt.trace.iter().map(|point| { GGPoint2::new(point.x, point.y) }).collect();

                graphics::line(
                    ctx,
                    &gg_trace,
                    1.0,
                )?;
            }
        }

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;

        graphics::present(ctx);
        Ok(())
    }
}

fn main() {

//    dists = dist.cdist(positions, positions)
//    massProds = masses[xs] * masses[ys]
//    massProds = massProds.reshape(massProds.shape[:2])
//    gravF = G * massProds / dists ** 2
//    gravA = gravF / masses[xs].reshape(xs.shape[:2])
//    np.fill_diagonal(gravA, 0)
//    dirs = (positions[ys] - positions[xs]) / np.stack([dists, dists], axis=2)
//    np.fill_diagonal(dirs[:, :, 0], 0)
//    np.fill_diagonal(dirs[:, :, 1], 0)
//    accel = (dirs * np.stack([gravA, gravA], axis=2)).sum(axis=0)
//    v = velocities + accel
//    p = v + positions

    const START_NUM_PARTICLES: usize = 150;
    const START_RADIUS : f32 = 100.0;

    let mut rng = rand::thread_rng();

    let mut state = SimState {
        particles: Vec::new(),
    };

    for _ in 0..START_NUM_PARTICLES {

        let pos = Point2::new(rng.gen_range(- START_RADIUS, START_RADIUS),
                              rng.gen_range(- START_RADIUS, START_RADIUS));

        let rad = pos.coords.norm();

        state.particles.push(
            Particle {
                position: pos,
                velocity: Vector2::new(pos.y, -pos.x).normalize() * rad.sqrt(), // turn 90 degrees
                acceleration : Vector2::new(0.0,0.0),
                trace : vec![]
            }
        );
    }

    // Displaying

    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    event::run(ctx, &mut state).unwrap();
}