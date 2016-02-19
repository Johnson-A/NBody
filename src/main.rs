extern crate rand;
extern crate num;

use num::Num;

use std::mem;
use std::ops::{Add, Sub, Mul, Neg};

fn main() {
    println!("Vec3 {:?}", mem::size_of::<Vec3<f64>>());
    println!("Body {:?}", mem::size_of::<Body>());
    println!("Section {:?}", mem::size_of::<Section>());

    let n = 100;
    let mut bodies: Vec<Body> = Vec::with_capacity(n);
    for body in &mut bodies {
        *body = Body {
            x: Vec3(0.0, 0.0, 0.0),
            p: Vec3(0.0, 0.0, 0.0),
            m: 0.0,
        };
    }
}

#[derive(Copy, Clone)]
struct Vec3<T: Num + Copy>(T, T, T);

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

impl<T: Num + Copy> Neg for Vec3<T> {
    type Output = Self;
    fn neg(self) -> Self {
        self * (T::zero() - T::one())
    }
}

struct Body {
    x: Vec3<f64>,
    p: Vec3<f64>,
    m: f64,
}

impl Body {
    fn attract(&mut self, other: &mut Body) {}
}

impl Region for Body {
    fn is_leaf(&self) -> bool {
        true
    }

    fn center(&self) -> Vec3<f64> {
        self.x
    }

    fn add_to(&mut self, _: &Region) {
        unreachable!()
    }
}

trait Region {
    fn is_leaf(&self) -> bool;
    fn center(&self) -> Vec3<f64>;
    fn add_to(&mut self, body: &Region);
}

struct Section {
    center: Vec3<f64>,
    width: f64,
    sub: [*mut Region; 8],
}

impl Region for Section {
    fn is_leaf(&self) -> bool {
        false
    }

    fn center(&self) -> Vec3<f64> {
        unreachable!()
    }

    fn add_to(&mut self, body: &Region) {
        let body_x = body.center();
        let pos = if body_x.0 > self.center.0 { 1 } else { 0 } +
                  if body_x.1 > self.center.1 { 2 } else { 0 } +
                  if body_x.2 > self.center.2 { 4 } else { 0 };

        let mut sub_region = self.sub[pos];
        unsafe {
            if (*sub_region).is_leaf() {
                let old_body_ref = sub_region;
                let new_width = self.width / 2.0;

                let offset = Vec3((pos >> 0 & 0b1) as f64 * self.width - new_width,
                                  (pos >> 1 & 0b1) as f64 * self.width - new_width,
                                  (pos >> 2 & 0b1) as f64 * self.width - new_width);

                sub_region = &mut Section {
                    center: self.center + offset,
                    width: new_width,
                    sub: std::mem::uninitialized()
                };
                (*sub_region).add_to(&*old_body_ref);
            }

            (*sub_region).add_to(body);
        }
    }
}
