#![feature(box_syntax, box_patterns, zero_one)]
extern crate rand;
extern crate num;
extern crate rayon;
extern crate time;
extern crate crossbeam;
extern crate piston_window;
extern crate sdl2_window;
extern crate itertools;

pub mod node;
pub mod vec;

use node::*;

use piston_window::*;
use sdl2_window::Sdl2Window;
use rayon::prelude::*;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use time::precise_time_s;

const THETA: f64 = 0.3;
const THETA_SQUARED: f64 = THETA * THETA;
const N: usize = 1_000_000;
const DT: f64 = 1E-5;

fn main() {
    let (width, height) = (700, 700);

    let mut bodies = Body::generate_collision(N);

    let run_simulation = AtomicBool::new(true);
    let redraw = AtomicBool::new(true);
    let positions = Mutex::new(vec![]);

    crossbeam::scope(|scope| {
        scope.spawn(|| {
            let mut step: u64 = 0;
            let start = precise_time_s();

            while run_simulation.load(Ordering::Relaxed) {
                step += 1;

                let mut parent = Section::containing(&bodies);
                parent.parallel_add(&bodies);
                parent.compute(&mut bodies);

                bodies.par_iter_mut().for_each(Body::advance);

                if step % 10 == 0 {
                    println!("{:.3} steps / second", step as f64 / (precise_time_s() - start))
                }

                if !redraw.compare_and_swap(false, true, Ordering::Relaxed) {
                    println!("Overwriting render data");

                    *positions.lock().unwrap() = parent.render(parent.density(), parent.total_mass);
                    // *positions.lock().unwrap() = parent.render_simple();
                }
            }
        });

        let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("NBody", [width, height])
            .vsync(true)
            .build()
            .unwrap();

        window.set_swap_buffers(false);

        while let Some(e) = window.next() {
            match e {
                Event::Render(_args) => {
                    if redraw.compare_and_swap(true, false, Ordering::Relaxed) {
                        println!("Drawing");

                        window.draw_2d(&e, |c, g| {
                            clear(color::WHITE, g);

                            for &(upp_left, size, color) in positions.lock().unwrap().iter() {
                                rectangle([1.0, 0.0, 0.0, color as f32],
                                          [upp_left.0 * width as f64,
                                           upp_left.1 * height as f64,
                                           size * width as f64,
                                           size * height as f64],
                                          c.transform, g);
                            }
                        });

                        Window::swap_buffers(&mut window);
                    }
                }
                _ => (),
            };
        }

        // Stop the simulation when we close the window
        run_simulation.store(false, Ordering::Relaxed);
    });
}
