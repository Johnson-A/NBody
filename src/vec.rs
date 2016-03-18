use num::{Num, traits};
use std::num::Zero;
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, DivAssign, MulAssign};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Vec2<T: Num + Copy>(pub T, pub T);

impl<T: Num + Copy + traits::Float> Vec2<T> {
    pub fn dot(&self, rhs: Vec2<T>) -> T {
        self.0 * rhs.0 + self.1 * rhs.1
    }

    pub fn inf_norm(&self) -> T {
        if self.0.abs() > self.1.abs() { self.0.abs() } else { self.1.abs() }
    }
}

impl<T: Num + Copy> Zero for Vec2<T> {
    fn zero() -> Self {
        Vec2(T::zero(), T::zero())
    }
}

impl<T: Num + Copy> Add for Vec2<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Num + Copy + AddAssign> AddAssign for Vec2<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: Num + Copy> Sub for Vec2<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: Num + Copy> Mul<T> for Vec2<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Vec2(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: Num + Copy> Div<T> for Vec2<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self {
        Vec2(self.0 / rhs, self.1 / rhs)
    }
}

impl<T: Num + Copy + MulAssign> MulAssign<T> for Vec2<T> {
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs;
    }
}

impl<T: Num + Copy> DivAssign<T> for Vec2<T> {
    fn div_assign(&mut self, rhs: T) {
        *self = *self * (T::one() / rhs);
    }
}

impl Mul<Vec2<f64>> for f64 {
    type Output = Vec2<f64>;
    fn mul(self, rhs: Vec2<f64>) -> Vec2<f64> {
        rhs * self
    }
}

impl Div<Vec2<f64>> for f64 {
    type Output = Vec2<f64>;
    fn div(self, rhs: Vec2<f64>) -> Vec2<f64> {
        rhs * (1.0 / self)
    }
}

impl<T: Num + Copy> Neg for Vec2<T> {
    type Output = Self;
    fn neg(self) -> Self {
        self * (T::zero() - T::one())
    }
}
