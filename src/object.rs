use crate::{
	material::Material,
	math::{Aabb, Curve, Ray, Vec3}
};

use std::sync::Arc;

pub trait Object: Send + Sync {
	fn aabb(&self, time0: f64, time1: f64) -> Aabb;
	fn hit(&self, ray: Ray, min_distance: f64, max_distance: f64) -> Option<Hit>;
}

pub struct Sphere {
	pub center: Curve,
	pub radius: f64,
	pub material: Arc<dyn Material>
}

impl Object for Sphere {
	fn aabb(&self, time0: f64, time1: f64) -> Aabb {
		let corner = Vec3(self.radius, self.radius, self.radius);

		// Explicitly handling constant curves is faster
		match self.center {
			Curve::Constant(v) => Aabb {
				min: v - corner,
				max: v + corner
			},

			_ => Aabb {
				min: self.center.at(time0) - corner,
				max: self.center.at(time0) + corner
			}.merge(Aabb {
				min: self.center.at(time1) - corner,
				max: self.center.at(time1) + corner
			})
		}
	}

	fn hit(&self, ray: Ray, min_distance: f64, max_distance: f64) -> Option<Hit> {
		let center = self.center.at(ray.time);

		// We use the quadratic formula
		let oc = ray.origin - center;
		let a = ray.direction.dot(ray.direction);
		let b = ray.direction.dot(oc) * 2.0;
		let c = oc.dot(oc) - self.radius * self.radius;

		let discriminant = b * b - 4.0 * a * c;

		// If we didn't hit it
		if discriminant < 0.0 {
			return None;
		}

		let mut distance = (-b - discriminant.sqrt()) / (2.0 * a);

		// If the first root is beyond the maximum distance, the second one will be too
		if distance >= max_distance {
			return None;
		}

		// If it's too close, check the second root
		if distance < min_distance {
			distance = (-b + discriminant.sqrt()) / (2.0 * a);

			if distance < min_distance || distance >= max_distance {
				return None;
			}
		}

		let point = ray.at(distance);
		let normal = (point - center) / self.radius;

		Some(Hit::new(ray, distance, point, normal, self.material.clone()))
	}
}

pub struct Hit {
	pub ray: Ray,
	pub distance: f64,
	pub point: Vec3,
	pub normal: Vec3,
	pub material: Arc<dyn Material>,
	pub face: Face,
}

impl Hit {
	fn new(
		ray: Ray,
		distance: f64,
		point: Vec3,
		normal: Vec3,
		material: Arc<dyn Material>) -> Hit
	{
		let (normal, face) = if ray.direction.dot(normal) <= 0.0 {
			(normal, Face::Front)
		} else {
			(-normal, Face::Back)
		};

		Hit {
			ray: ray,
			distance: distance,
			point: point,
			normal: normal,
			material: material,
			face: face
		}
	}
}

#[derive(Copy, Clone)]
pub enum Face {
	Front, // Outside
	Back   // Inside
}
