#![feature(box_syntax, box_patterns, zero_one, iter_arith)]
extern crate rand;
extern crate num;
extern crate rayon;
extern crate time;
extern crate core;

use node::*;
use vec::Vec3;

use rayon::prelude::*;
use time::precise_time_s;

use rand::{SeedableRng, StdRng, Rng};
use std::mem;

mod vec;
mod node;

const THETA: f64 = 0.3;
const N: usize = 1_000_000;
const DT: f64 = 1E-9;

fn main() {
    println!("Vec3 {:?}", mem::size_of::<Vec3<f64>>());
    println!("Body {:?}", mem::size_of::<Body>());
    println!("Section {:?}", mem::size_of::<Section>());
    println!("Test {}", mem::size_of::<Node>());

    let mut bodies: Vec<Body> = Vec::with_capacity(N);
    let seed: &[_] = &[1,2,3];
    let mut rng: StdRng = SeedableRng::from_seed(seed);;
    for _ in 0..N {
        bodies.push(Body {
            x: Vec3(rng.gen(), rng.gen(), rng.gen()),
            v: Vec3(0.0, 0.0, 0.0),
            m: 1.0,
        })
    }

    let mut forces = vec![Vec3(0.0,0.0,0.0); N];
    let mut step = 0;
    let start = precise_time_s();

    loop {
        step += 1;
        {
            let mut parent = Section::containing(&bodies);
            parent.parallel_add(&bodies);

            // println!("{} {}", parent.count(), parent.total_mass);

            Node::Internal(Some(box parent)).compute(&bodies, &mut forces);
        }
        bodies.par_iter_mut().zip(&forces).for_each(|(b, &f)| {
            b.v = b.v + f * DT;
            b.x = b.x + b.v * DT;
        });
        println!("{}", step as f64 / (precise_time_s() - start))
    }

    // println!("{:?} {:?} {}", parent.body, parent.sub, parent.total_mass());
}
