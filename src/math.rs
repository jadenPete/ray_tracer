use rand::Rng;

use std::{
	iter::Sum,
	mem,
	ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign}
};

#[derive(Copy, Clone, Debug)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Vec3 {
	pub fn cross(self, v: Self) -> Self {
		Self(self.1 * v.2 - self.2 * v.1,
		     self.2 * v.0 - self.0 * v.2,
		     self.0 * v.1 - self.1 * v.0)
	}

	pub fn dot(self, v: Self) -> f64 {
		self.0 * v.0 + self.1 * v.1 + self.2 * v.2
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

impl Index<u8> for Vec3 {
	type Output = f64;

	fn index(&self, index: u8) -> &f64 {
		match index {
			0 => &self.0,
			1 => &self.1,
			2 => &self.2,

			_ => panic!()
		}
	}
}

impl IndexMut<u8> for Vec3 {
	fn index_mut(&mut self, index: u8) -> &mut f64 {
		match index {
			0 => &mut self.0,
			1 => &mut self.1,
			2 => &mut self.2,

			_ => panic!()
		}
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

impl Neg for Vec3 {
	type Output = Self;

	fn neg(self) -> Self {
		Self(-self.0, -self.1, -self.2)
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

impl Sum for Vec3 {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(Self(0.0, 0.0, 0.0), Add::add)
	}
}

/// A ray begins at an "origin" point and continues infinitely in a "direction" unit vector
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

/// A curve is a continuous set of points accessible over a time interval
#[derive(Copy, Clone)]
pub enum Curve {
	Constant(Vec3),
	Linear(Vec3, Vec3, f64, f64)
}

impl Curve {
	pub(crate) fn at(self, time: f64) -> Vec3 {
		match self {
			Self::Constant(v) => v,

			Self::Linear(v0, v1, time0, time1) => {
				v0 + (v1 - v0) * (time - time0) / (time1 - time0)
			}
		}
	}
}

/// An axis-aligned minimum bounding box (AABB)
#[derive(Copy, Clone)]
pub struct Aabb {
	pub min: Vec3,
	pub max: Vec3
}

impl Aabb {
	pub fn hit(self, ray: Ray, mut min_distance: f64, mut max_distance: f64) -> bool {
		for i in 0..3 {
			let mut t0 = (self.min[i] - ray.origin[i]) / ray.direction[i];
			let mut t1 = (self.max[i] - ray.origin[i]) / ray.direction[i];

			// If negative, t0 > t1
			if ray.direction[i] < 0.0 {
				mem::swap(&mut t0, &mut t1);
			}

			// If the intervals [t0, t1] and [min_distance, max_distance] don't overlap
			min_distance = t0.max(min_distance);
			max_distance = t1.min(max_distance);

			if min_distance > max_distance {
				return false;
			}
		}

		true
	}

	pub fn merge(self, aabb: Self) -> Self {
		Self {
			min: Vec3(self.min.0.min(aabb.min.0),
			          self.min.1.min(aabb.min.1),
			          self.min.2.min(aabb.min.2)),

			max: Vec3(self.max.0.max(aabb.max.0),
			          self.max.1.max(aabb.max.1),
			          self.max.2.max(aabb.max.2))
		}
	}
}
