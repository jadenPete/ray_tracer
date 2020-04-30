use rand::Rng;

use std::{
	iter::Sum,
	ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}
};

#[derive(Copy, Clone, Debug)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Vec3 {
	pub fn dot(self, v: Self) -> f64 {
		self.0 * v.0 + self.1 * v.1 + self.2 * v.2
	}

	pub fn cross(self, v: Self) -> Self {
		Self(self.1 * v.2 - self.2 * v.1,
		     self.2 * v.0 - self.0 * v.2,
		     self.0 * v.1 - self.1 * v.0)
	}

	pub fn len(self) -> f64 {
		self.dot(self).sqrt()
	}

	pub fn unit(self) -> Self {
		self / self.len()
	}

	pub fn random_in_cube(min: f64, max: f64) -> Self {
		let mut rng = rand::thread_rng();

		Vec3(rng.gen_range(min, max),
		     rng.gen_range(min, max),
		     rng.gen_range(min, max))
	}

	pub fn random_in_unit_cube() -> Self {
		let mut rng = rand::thread_rng();
		Vec3(rng.gen(), rng.gen(), rng.gen())
	}

	pub fn random_on_unit_sphere() -> Self {
		let mut rng = rand::thread_rng();

		let a = rng.gen_range(0.0, std::f64::consts::PI * 2.0);
		let b = rng.gen::<f64>();
		let c = (1.0 - b * b).sqrt();

		Vec3(c * a.cos(), c * a.sin(), b)
	}

	pub fn random_in_unit_sphere() -> Self {
		Self::random_on_unit_sphere() * rand::random::<f64>().powf(1.0 / 3.0)
	}

	pub fn random_in_unit_disk() -> Self {
		let mut rng = rand::thread_rng();

		let angle = rng.gen_range(0.0, std::f64::consts::PI * 2.0);
		let radius = rng.gen::<f64>().sqrt();

		Vec3(angle.cos(), angle.sin(), 0.0) * radius
	}
}

impl Neg for Vec3 {
	type Output = Self;

	fn neg(self) -> Self {
		Self(-self.0, -self.1, -self.2)
	}
}

impl Add for Vec3 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
	}
}

impl AddAssign for Vec3 {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl Sub for Vec3 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
	}
}

impl SubAssign for Vec3 {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl Mul<f64> for Vec3 {
	type Output = Self;

	fn mul(self, rhs: f64) -> Self {
		Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
	}
}

impl MulAssign<f64> for Vec3 {
	fn mul_assign(&mut self, rhs: f64) {
		self.0 *= rhs;
		self.1 *= rhs;
		self.2 *= rhs;
	}
}

impl Mul for Vec3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		Self(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
	}
}

impl MulAssign for Vec3 {
	fn mul_assign(&mut self, rhs: Vec3) {
		self.0 *= rhs.0;
		self.1 *= rhs.1;
		self.2 *= rhs.2;
	}
}

impl Div<f64> for Vec3 {
	type Output = Self;

	fn div(self, rhs: f64) -> Self {
		Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
	}
}

impl DivAssign<f64> for Vec3 {
	fn div_assign(&mut self, rhs: f64) {
		self.0 /= rhs;
		self.1 /= rhs;
		self.2 /= rhs;
	}
}

impl Sum for Vec3 {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(Self(0.0, 0.0, 0.0), Add::add)
	}
}

#[derive(Copy, Clone)]
pub struct Ray {
	pub origin: Vec3,
	pub direction: Vec3,
	pub time: f64
}

impl Ray {
	pub fn at(self, distance: f64) -> Vec3 {
		self.origin + self.direction * distance
	}
}

/// A vector moving over time
#[derive(Copy, Clone)]
pub enum Path {
	Constant(Vec3),
	Linear(Vec3, Vec3, f64, f64)
}

impl Path {
	pub fn at(self, time: f64) -> Vec3 {
		match self {
			Self::Constant(v) => v,

			Self::Linear(v0, v1, time0, time1) => {
				v0 + (v1 - v0) * (time - time0) / (time1 - time0)
			}
		}
	}
}
