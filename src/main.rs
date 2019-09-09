extern crate sdl2;
extern crate chrono;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::gfx::primitives::DrawRenderer;
use chrono::prelude::*;
use na::{Point2,distance_squared};
use std::iter::FromIterator;

#[derive(Copy, Clone)]
struct Body {
    position : Point2,
    mass : f32
}

fn newton_gravity_acceleration(subject_position: &Point2, grav_source_position: &Point2, grav_source_mass: f32) {

    const G : f32 = 1.0;

    let a = G * grav_source_mass / (subject_position.distance_squared(grav_source_position));

    return (grav_source_position - subject_position).normalized() * a;
}

enum Quadtree {
    Leaf { body: Optional<Body> },
    InternalNode { center_of_gravity: Vector2, combined_mass : f32, nw: Quadtree, ne: Quadtree, sw: Quadtree, se: Quadtree }
}

fn construct_tree(bodies: &Vec<Body>, x_range: Range<f32>, y_range: Range<f32>) -> Quadtree {

    if bodies.len() <= 1 {
        return Leaf { body : bodies.pop() };
    } else {
        let mut nw : Vec<Body>::new();
        let mut ne : Vec<Body>::new();
        let mut sw : Vec<Body>::new();
        let mut se : Vec<Body>::new();

        let x1_range = x_range.start .. x_range.start + x_range.end / 2.0;
        let x2_range = x_range.start + x_range.end / 2.0 .. x_range.end;

        let y1_range = y_range.start .. y_range.start + y_range.end / 2.0;
        let y2_range = y_range.start + y_range.end / 2.0 .. y_range.end;

        for b in bodies.iter() {
            if x1_range.contains(b.position.x) {
                if y1_range.contains(b.position.y) {
                    nw.push(b);
                } else {
                    sw.push(b);
                }
            } else {
                if y1_range.contains(b.position.y) {
                    ne.push(b);
                } else {
                    se.push(b);
                }
            }
        }

        let tree_nw = construct_tree(nw, x1_range.copy(), y1_range.copy());
        let tree_ne = construct_tree(ne, x2_range.copy(), y1_range);
        let tree_sw = construct_tree(sw, x1_range, y2_range.copy());
        let tree_se = construct_tree(se, x2_range, y2_range);

        let cog = tree_nw.position * tree_nw.mass
            + tree_ne.position * tree_ne.mass
            + tree_sw.position * tree_sw.mass
            + tree_se.position * tree_se.mass;

        let mass = tree_nw.mass + tree_ne.mass + tree_sw.mass + tree_se.mass;

        return IntenalNode {
            nw : tree_nw,
            ne : tree_ne,
            sw : tree_sw,
            se : tree_se,
            center_of_gravity : cog,
            combined_mass : mass
        };
    }
}

pub fn get_approx_gravity(body: Body, gravtree : &Quadtree, tree_node_size: f32) -> Vector2<f32> {

    match gravtree {
        Leaf { body : None } => { Vector2::new(0.0,0.0) }
        Leaf { body : Some(b) } => { newton_gravity_acceleration(body.position, b.position, b.mass) }
        InternalNode {center_of_gravity, combined_mass, nw, ne, sw, se} => {
            if treeNodeSize < body.position.distance_squared(center_of_gravity) {
                return get_approx_gravity(body, nw, tree_node_size /2.0) +
                    get_approx_gravity(body, ne, tree_node_size /2.0) +
                    get_approx_gravity(body, sw, tree_node_size /2.0) +
                    get_approx_gravity(body, se, tree_node_size /2.0);
            } else {
                return newton_gravity_acceleration(body.position, center_of_gravity, combined_mass);
            }

        }
    }

}

pub fn main() {

    const RADIUS : f32 = 1000.0;

    const DT = 1.0;

    let mut bodies :  Vec<Body>::new();

    for i in 0..1000 {
        init_bodies.push(Body { position: Vector2::new(0.0,0.0), mass: 1.0 });
    }


    let mut gravTree = construct_tree(init_bodies, -RADIUS..RADIUS, -RADIUS..RADIUS);

    for b in bodies.iter_mut() {

        b += get_approx_gravity(b, &gravTree, RADIUS * 2.0) * DT;

    }


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

//    canvas.set_draw_color(Color::RGB(0, 255, 255));
//    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        let start: DateTime<Utc> = Utc::now();       // e.g. `2014-11-28T12:45:59.324310806Z`

        // Streaming step.

        step_stream(&mut pdf);

        // Macroscopic density
        let density = pdf.sum_axis(Axis(2)).sum_axis(Axis(2));

        // Macroscopic velocity.
        // TODO check that thoroughly.
        let velocity_x = pdf.slice(s![..,..,..,0]).sum_axis(Axis(2)) - pdf.slice(s![..,..,..,3]).sum_axis(Axis(2));
        let velocity_y = pdf.slice(s![..,..,..,0]).sum_axis(Axis(2)) - pdf.slice(s![..,..,..,3]).sum_axis(Axis(2));

        let vel_sqr = (&velocity_x * &velocity_x + &velocity_y * &velocity_y);

        let pdf_eq : Array4<f32> = &density * &arr2(&NEIGHBOUR_WEIGHTS) * (1.0 + &density + 0.5 * &density * &density - 3.0 / 2.0 * usqr);

        pdf.assign(&pdf - (&pdf - &pdf_eq) * Reynolds_thingy); // # Collision step.

        let end = Utc::now();

        println!("Delta: {}", end - start);

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let m: f32 = density[[x, y]];
                let max = 1.0;
                let min = 0.0;
                let r: u8 = (255.0 * (m.min(max).max(min) - min) / (max - min)) as u8;

                canvas.pixel(x as i16, y as i16,
                             (r, r, r, 0xff)).expect("No pixel for you!");
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}