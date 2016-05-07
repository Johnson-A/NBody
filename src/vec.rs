use node::Elem;
use num::{Num, traits};
use std::num::Zero;
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, DivAssign, MulAssign};

trait_alias!(pub NumCopy = Num + Copy);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Vec2<T: NumCopy>(pub T, pub T);

impl<T: Num + traits::Float> Vec2<T> {
    pub fn dot(&self, rhs: Vec2<T>) -> T {
        self.0 * rhs.0 + self.1 * rhs.1
    }

    pub fn inf_norm(&self) -> T {
        if self.0.abs() > self.1.abs() { self.0.abs() } else { self.1.abs() }
    }
}

impl<T: NumCopy> Zero for Vec2<T> {
    fn zero() -> Self {
        Vec2(T::zero(), T::zero())
    }
}

impl<T: NumCopy> Add for Vec2<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: NumCopy> AddAssign for Vec2<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: NumCopy> Sub for Vec2<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: NumCopy> Mul<T> for Vec2<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Vec2(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: NumCopy> Div<T> for Vec2<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self {
        Vec2(self.0 / rhs, self.1 / rhs)
    }
}

impl<T: NumCopy> MulAssign<T> for Vec2<T> {
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs;
    }
}

impl<T: NumCopy> DivAssign<T> for Vec2<T> {
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs;
    }
}

impl Mul<Vec2<Elem>> for Elem {
    type Output = Vec2<Elem>;
    fn mul(self, rhs: Vec2<Elem>) -> Vec2<Elem> {
        rhs * self
    }
}

impl Div<Vec2<Elem>> for Elem {
    type Output = Vec2<Elem>;
    fn div(self, rhs: Vec2<Elem>) -> Vec2<Elem> {
        rhs * (1.0 / self)
    }
}

impl<T: NumCopy> Neg for Vec2<T> {
    type Output = Self;
    fn neg(self) -> Self {
        self * (T::zero() - T::one())
    }
}
