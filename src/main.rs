#![feature(box_syntax, box_patterns, zero_one)]
extern crate rand;
extern crate num;
extern crate rayon;
extern crate time;
extern crate core;

use node::*;
use vec::Vec3;
use core::num::Zero;

use rayon::prelude::*;
use time::precise_time_s;

use rand::{SeedableRng, StdRng, Rng};
use std::mem;

mod vec;
mod node;

const THETA: f64 = 0.3;
const THETA_SQUARED: f64 = THETA * THETA;
const N: usize = 1_000_000;
const DT: f64 = 1E-15;

fn main() {
    println!("Vec3 {:?}", mem::size_of::<Vec3<f64>>());
    println!("Body {:?}", mem::size_of::<Body>());
    println!("Section {:?}", mem::size_of::<Section>());

    let mut bodies: Vec<Body> = Vec::with_capacity(N);
    let seed: &[_] = &[1,2,3];
    let mut rng: StdRng = SeedableRng::from_seed(seed);;
    for _ in 0..N {
        bodies.push(Body {
            x: Vec3(rng.gen(), rng.gen(), rng.gen()),
            v: Vec3(0.0, 0.0, 0.0),
            a: Vec3::zero(),
            m: 1.0,
        })
    }

    let mut step = 0;
    let start = precise_time_s();

    loop {
        step += 1;
        let mut parent = Section::containing(&bodies);

        // for body in &bodies {
        //     parent.add(body.x, body.m);
        // }
        // parent.aggregate();
        parent.parallel_add(&bodies);

        // println!("{}", parent.total_mass);

        parent.compute(&mut bodies);
        bodies.par_iter_mut().for_each(|b| {
            b.v = b.v + b.a * DT;
            b.x = b.x + b.v * DT;
        });
        println!("{}", step as f64 / (precise_time_s() - start))
    }

    // println!("{:?} {:?} {}", parent.body, parent.sub, parent.total_mass());
}
