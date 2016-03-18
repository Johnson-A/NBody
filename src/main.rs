#![feature(box_syntax, box_patterns, zero_one)]
extern crate rand;
extern crate num;
extern crate rayon;
extern crate time;
extern crate piston_window;

mod node;
mod vec;

use node::*;

use piston_window::*;
use rayon::prelude::*;

use std::thread;
use std::sync::{Arc, Mutex};
use time::precise_time_s;

const THETA: f64 = 0.3;
const THETA_SQUARED: f64 = THETA * THETA;
const N: usize = 100_000;
const DT: f64 = 1E-6;

fn main() {
    let (width, height) = (900, 900);

    let mut bodies = Body::generate(N);

    let positions = Arc::new(Mutex::new(
        bodies.iter().map(|b| b.x).collect::<Vec<_>>()
    ));
    let write_data = positions.clone();

    thread::spawn(move || {
        let mut step = 0;
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

            for (w, b) in write_data.lock().unwrap().iter_mut().zip(&bodies) {
                *w = b.x;
            }
        }
    });

    let window: PistonWindow =
                    WindowSettings::new("NBody", [width, height])
                    .exit_on_esc(true)
                    // .opengl(OpenGL::V4_1)
                    .vsync(true)
                    .build()
                    .unwrap();

    for e in window {
        e.draw_2d(|c, g| {
            clear(color::WHITE, g);

            for x in positions.lock().unwrap().iter() {
                rectangle(
                    [1.0, 0.0, 0.0, 1.0],
                    [x.0 * width as f64, x.1 * height as f64, 1.0, 1.0],
                    c.transform, g);
            }
        });
    }
}
