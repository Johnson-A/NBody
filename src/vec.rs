use num::Num;
use core::num::Zero;
use std::ops::{Add, Sub, Mul, Neg, AddAssign, DivAssign, MulAssign};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Vec3<T: Num + Copy>(pub T, pub T, pub T);

impl Vec3<f64> {
    #[inline(always)]
    pub fn dist(&self, point: Vec3<f64>) -> (Vec3<f64>, f64, f64) {
        let dx = *self - point;
        let inv_dist_sq = 1.0 / dx.dot(dx);
        (dx, inv_dist_sq, inv_dist_sq.sqrt())
    }
}

impl<T: Num + Copy> Zero for Vec3<T> {
    fn zero() -> Self {
        Vec3(T::zero(), T::zero(), T::zero())
    }
}

impl<T: Num + Copy> Vec3<T> {
    pub fn dot(&self, rhs: Vec3<T>) -> T {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
}

impl<T: Num + Copy> Add for Vec3<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: Num + Copy + AddAssign> AddAssign for Vec3<T> {
    fn add_assign(&mut self, _rhs: Self) {
        *self = *self + _rhs;
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

impl<T: Num + Copy + MulAssign> MulAssign<T> for Vec3<T> {
    fn mul_assign(&mut self, _rhs: T) {
        *self = *self * _rhs;
    }
}

impl<T: Num + Copy> DivAssign<T> for Vec3<T> {
    fn div_assign(&mut self, _rhs: T) {
        *self = *self * (T::one() / _rhs);
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
