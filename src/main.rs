use std::mem;
use std::ops::{Add, Sub, Mul, Div, Neg};

trait Num: Sized + Copy + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> + Neg<Output=Self> {}
impl Num for f64 {}

fn main() {
    println!("Pointer {:?}", mem::size_of::<&Node>());
    println!("Node {:?}", mem::size_of::<Node>());
    println!("Vec3 {:?}", mem::size_of::<Vec3<f64>>());
    println!("Body {:?}", mem::size_of::<Body<f64>>());
}

struct Vec3<T: Num>(T, T, T);

impl<T: Num> Add for Vec3<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: Num> Sub for Vec3<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: Num> Mul<T> for Vec3<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl<T: Num> Neg for Vec3<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Vec3(-self.0, -self.1, -self.2)
    }
}

struct Body<T: Num> {
    x: Vec3<T>,
    p: Vec3<T>,
    m: T
}

impl<T: Num> Body<T> {
    fn attract(&mut self, other: &mut Self) {
        
    }
}

trait Region<T: Num> {
    fn is_leaf(&self) -> bool;
    fn add_to(&mut self, body: Body<T>) -> bool;
}

struct Node {
    of: Box<Region<f64>>,
}
