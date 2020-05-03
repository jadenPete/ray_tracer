use crate::{
	material::Material,
	math::{Aabb, Curve, Ray, Vec3}
};

/// An object is a field in the scene that can intercept rays and return a record containing
/// information about the interaction, thus allowing rays to bounce between different objects.
///
/// While I originally defined an Object trait, I have since used an enum to avoid the lifetime and
/// performance issues commonly associated with trait objects.
#[derive(Copy, Clone)]
pub enum Object {
	Sphere {
		center: Curve,
		radius: f64,
		material: Material
	}
}

impl Object {
	pub(crate) fn aabb(self, time0: f64, time1: f64) -> Aabb {
		match self {
			Self::Sphere { center, radius, material: _ } => {
				let corner = Vec3(radius, radius, radius);

				// Explicitly handling constant curves is faster
				match center {
					Curve::Constant(v) => Aabb {
						min: v - corner,
						max: v + corner
					},

					_ => Aabb {
						min: center.at(time0) - corner,
						max: center.at(time0) + corner
					}.merge(Aabb {
						min: center.at(time1) - corner,
						max: center.at(time1) + corner
					})
				}
			}
		}
	}

	/// Returns a hit record if the object intercepts a ray at a distance within
	/// [min_distance, max_distance) from the ray's origin
	pub(crate) fn hit(self, ray: Ray, min_distance: f64, max_distance: f64) -> Option<Hit> {
		match self {
			Self::Sphere { center, radius, material } => {
				let center = center.at(ray.time);

				// We use the quadratic formula
				let a = ray.direction.dot(ray.direction);
				let half_b = ray.direction.dot(ray.origin - center);
				let c = (ray.origin - center).dot(ray.origin - center) - radius * radius;

				let discriminant = half_b * half_b - a * c;

				// If we didn't hit it
				if discriminant < 0.0 {
					return None;
				}

				let mut distance = (-half_b - discriminant.sqrt()) / a;

				// If the first root is beyond the maximum distance, the second one will be too
				if distance >= max_distance {
					return None;
				}

				// If it's too close, check the second root
				if distance < min_distance {
					distance = (-half_b + discriminant.sqrt()) / a;

					if distance < min_distance || distance >= max_distance {
						return None;
					}
				}

				let point = ray.at(distance);
				let normal = (point - center) / radius;

				Some(Hit::new(ray, distance, point, normal, material))
			}
		}
	}
}

#[derive(Copy, Clone)]
pub(crate) struct Hit {
	pub ray: Ray,
	pub distance: f64,
	pub point: Vec3,
	pub normal: Vec3,
	pub material: Material,
	pub face: Face,
}

impl Hit {
	fn new(
		ray: Ray,
		distance: f64,
		point: Vec3,
		normal: Vec3,
		material: Material) -> Hit
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
pub(crate) enum Face {
	Front, // Outside
	Back   // Inside
}
