extern crate sdl2;
extern crate ndarray;
extern crate chrono;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use ndarray::{Array2, Array3, Array4, s, arr2};
use sdl2::gfx::primitives::DrawRenderer;
use chrono::prelude::*;
use ndarray::Axis;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;
const DT: f32 = 1.0;

const VALUES_PER_CELL: usize = 9;
// Middle, 4 directions, 4 corners
const NEIGHBOUR_WEIGHTS: [[f32; 3]; 3] =
    [
        [0.25 / 9.0, 1.0 / 9.0, 0.25 / 9.0],
        [1.0 / 9.0,  4.0 / 9.0, 1.0 / 9.0],
        [0.25 / 9.0, 1.0 / 9.0, 0.25 / 9.0]
    ];

// Returns `value` clamped between `low` and `high`.
pub fn clamp<T: PartialOrd>(low: T, value: T, high: T) -> T {
    debug_assert!(low < high, "low is bigger than high!");
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

fn step_stream(density: &mut Array4<f32>) {
    let mut new_density: Array4<f32> = Array4::zeros(density.raw_dim());

    // Streaming.
    for i in 0..3 {
        for j in 0..3 {
            new_density.slice_mut(s![i..WIDTH, j..WIDTH,i,j]).assign(&density.slice(s![0..(WIDTH - i), 0..(WIDTH - j),2-i,2-j]));
        }
    }

    density.assign(&new_density);
}

pub fn main() {
    let mut pdf: Array4<f32> = Array4::zeros((WIDTH + 2, HEIGHT + 2, 3, 3));

    pdf.slice_mut(s![WIDTH/2 - 2..WIDTH/2 + 2, HEIGHT/2 - 2..HEIGHT/2 + 2,..,..]).fill(1.0);

    let space: Array2<u8> = Array2::zeros((WIDTH, HEIGHT));

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