#![feature(box_syntax, box_patterns, zero_one)]
extern crate rand;
extern crate num;
extern crate rayon;
extern crate time;
extern crate piston_window;
extern crate itertools;

#[macro_use]
pub mod util;

pub mod node;
pub mod vec;

use node::*;

use piston_window::*;
use rayon::prelude::*;

use std::thread;
use std::sync::{Arc, Mutex};
use time::precise_time_s;

const THETA: f64 = 0.3;
const THETA_SQUARED: f64 = THETA * THETA;
const N: usize = 1_000_000;
const DT: f64 = 1E-5;

fn main() {
    let (width, height) = (700, 700);

    let mut bodies = Body::generate_collision(N);

    let positions = Arc::new(Mutex::new(vec![]));
    let write_data = positions.clone();

    thread::spawn(move || {
        let mut step: u64 = 0;
        let start = precise_time_s();

        loop {
            step += 1;

            let mut parent = Section::containing(&bodies);
            parent.parallel_add(&bodies);
            parent.compute(&mut bodies);

            bodies.par_iter_mut().for_each(Body::advance);

            if step % 1 == 0 {
                println!("{:.3}", step as f64 / (precise_time_s() - start))
            }

            *write_data.lock().unwrap() = parent.render(parent.density(), parent.total_mass);
        }
    });

    let mut window: PistonWindow = WindowSettings::new("NBody", [width, height])
        .exit_on_esc(true)
        // .opengl(OpenGL::V4_4)
        .vsync(true)
        .build()
        .unwrap();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear(color::WHITE, g);

            for &(upp_left, size, color) in positions.lock().unwrap().iter() {
                // println!("{:?} {} {}", upp_left, size, color);
                rectangle([1.0, 0.0, 0.0, color as f32],
                          [upp_left.0 * width as f64,
                           upp_left.1 * height as f64,
                           size * width as f64,
                           size * height as f64],
                          c.transform, g);
            }
        });
    }
}
