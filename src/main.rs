#![feature(box_syntax, box_patterns)]
extern crate rand;
extern crate num;
extern crate crossbeam;
extern crate num_cpus;

use num::Num;

use rand::Rng;
use std::{mem, ptr};
use std::ops::{Add, Sub, Mul, Neg};


fn main() {
    println!("Vec3 {:?}", mem::size_of::<Vec3<f64>>());
    println!("Body {:?}", mem::size_of::<Body>());
    println!("Section {:?}", mem::size_of::<Section>());

    let n = 100_000;
    let dt = 0.00000001;
    let mut bodies: Vec<Body> = Vec::with_capacity(n);
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        bodies.push(Body {
            x: Vec3(rng.gen(), rng.gen(), rng.gen()),
            // v: Vec3(rng.gen(), rng.gen(), rng.gen()),
            v: Vec3(0.0, 0.0, 0.0),
            m: 1.0,
        })
    }

    let mut t = 0.0;
    let mut forces = vec![Vec3(0.0,0.0,0.0); n];

    loop {
        t += dt;

        {
            let mut parent = Section {
                center: Vec3(0.5, 0.5, 0.5),
                width: 1.0,
                total_mass: 0.0,
                sub: None,
                body: None,
            };

            for body in &bodies {
                parent.add(body);
            }

            parent.compute(&bodies, 0.6, &mut forces);
        }

        for (b,&f) in bodies.iter_mut().zip(&forces) {
            b.v = b.v + f * dt;
            b.x = b.x + b.v * dt;
        }
        println!("{}", t)
    }

    // println!("{:?} {:?} {}", parent.body, parent.sub, parent.total_mass());
}

#[derive(Copy, Clone, Debug)]
struct Vec3<T: Num + Copy>(T, T, T);

impl<T: Num + Copy> Vec3<T> {
    fn dot(&self, rhs: Vec3<T>) -> T {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
}

impl<T: Num + Copy> Add for Vec3<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: Num + Copy> Sub for Vec3<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: Num + Copy> Mul<T> for Vec3<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;
    fn mul(self, rhs: Vec3<f64>) -> Vec3<f64> {
        rhs * self
    }
}

impl<T: Num + Copy> Neg for Vec3<T> {
    type Output = Self;
    fn neg(self) -> Self {
        self * (T::zero() - T::one())
    }
}

#[derive(Debug)]
struct Body {
    x: Vec3<f64>,
    v: Vec3<f64>,
    m: f64,
}

impl Body {
    fn attract(&self, other: &Body) -> Vec3<f64> {
        let dx = self.x - other.x;
        let inv_dist_sq = 1.0 / dx.dot(dx);
        let inv_dist = inv_dist_sq.sqrt();
        dx * inv_dist_sq * inv_dist
    }
}

#[derive(Debug)]
struct Section<'a> {
    center: Vec3<f64>,
    width: f64,
    total_mass: f64,
    sub: Option<Box<[Section<'a>; 8]>>,
    body: Option<&'a Body>
}

impl<'a> Section<'a> {
    fn compute(&self, bodies: &Vec<Body>, theta: f64, forces: &mut Vec<Vec3<f64>>) {
        let split = forces.len() / num_cpus::get();
        let iter = forces.chunks_mut(split).zip(bodies.chunks(split));

        crossbeam::scope(|scope|
            for (f,b) in iter {
                scope.spawn(move || {
                    for (f1,b1) in f.iter_mut().zip(b) {
                        *f1 = self.force(b1, theta);
                    }
                });
            });
        // for (f,b) in forces.iter_mut().zip(bodies) {
        //     *f = self.force(b, theta);
        // }
    }

    fn force(&self, body: &Body, theta: f64) -> Vec3<f64> {
        let dx = self.center - body.x;
        let inv_dist_sq = 1.0 / dx.dot(dx);
        let inv_dist = inv_dist_sq.sqrt();

        let width = if self.body.is_some() {
            0.0
        } else {
            self.width
        };

        if width * inv_dist <= theta {
            dx * self.total_mass * inv_dist_sq * inv_dist
        } else if let Some(ref sub) = self.sub {
            sub.iter().fold(Vec3(0.0, 0.0, 0.0), |sum, sr| sum + sr.force(body, theta))
        } else {
            Vec3(0.0, 0.0, 0.0) // TODO: don't even calculate
            // unreachable!("{:?} {:?} {}", self.body, self.sub, width)
        }
    }

    fn total_mass(&self) -> f64 {
        if let Some(body) = self.body {
            body.m
        } else if let Some(ref sub) = self.sub {
            sub.iter().fold(0.0, |sum, b| sum + b.total_mass())
        } else {
            0.0
        }
    }

    fn add(&mut self, body: &'a Body) {
        let position = self.position(body);
        self.total_mass += body.m;

        if let Some(leaf_body) = self.body {
            self.body = None;
            let mut new_sections = self.divide();
            new_sections[position].add(body);
            new_sections[self.position(leaf_body)].add(leaf_body);
            self.sub = Some(box new_sections);
        } else if let Some(ref mut sub) = self.sub {
            sub[position].add(body);
        } else {
            self.body = Some(body);
        }
    }

    fn position(&self, body: &Body) -> usize {
        (if body.x.0 > self.center.0 { 1 } else { 0 } +
         if body.x.1 > self.center.1 { 2 } else { 0 } +
         if body.x.2 > self.center.2 { 4 } else { 0 })
    }

    fn divide(&self) -> [Section<'a>; 8] {
        let mut new_regions: [Section<'a>; 8];
        let new_width = self.width / 2.0;
        let offset = [-new_width / 2.0, new_width / 2.0];
        let mut index = 0;

        unsafe {
            new_regions = mem::uninitialized();

            for &k in &offset {
                for &j in &offset {
                    for &i in &offset {
                        ptr::write(&mut new_regions[index], Section {
                            width: new_width,
                            center: self.center + Vec3(i,j,k),
                            total_mass: 0.0,
                            sub: None,
                            body: None,
                        });

                        index += 1;
                    }
                }
            }
        }
        new_regions
    }
}
