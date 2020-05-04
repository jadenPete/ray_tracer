use crate::math::Vec3;
use crate::object::{Hit, Face};

pub trait Material: Send + Sync {
	// Returns a redirection record if the ray wasn't absorbed
	fn redirect(&self, hit: &Hit) -> Option<Redirection>;
}

pub struct Redirection {
	pub direction: Vec3,
	pub albedo: Vec3
}

pub struct Lambertian {
	pub albedo: Vec3
}

impl Material for Lambertian {
	fn redirect(&self, hit: &Hit) -> Option<Redirection> {
		Some(Redirection {
			direction: (hit.normal + Vec3::random_on_unit_sphere()).unit(),
			albedo: self.albedo
		})
	}
}

pub struct Spherical {
	pub albedo: Vec3
}

impl Material for Spherical {
	fn redirect(&self, hit: &Hit) -> Option<Redirection> {
		Some(Redirection {
			direction: (hit.normal + Vec3::random_in_unit_sphere()),
			albedo: self.albedo
		})
	}
}

pub struct Hemispherical {
	pub albedo: Vec3
}

impl Material for Hemispherical {
	fn redirect(&self, hit: &Hit) -> Option<Redirection> {
		Some(Redirection {
			direction: {
				let direction = Vec3::random_in_unit_sphere();

				(hit.normal + if direction.dot(hit.normal) >= 0.0 {
					direction
				} else {
					-direction
				}).unit()
			},

			albedo: self.albedo
		})
	}
}

pub struct Specular {
	pub albedo: Vec3,
	pub fuzziness: f64
}

impl Specular {
	/// Independent from the redirect function because it may be used during refraction
	fn reflect(hit: &Hit) -> Vec3 {
		hit.ray.direction - hit.normal * hit.ray.direction.dot(hit.normal) * 2.0
	}
}

impl Material for Specular {
	fn redirect(&self, hit: &Hit) -> Option<Redirection> {
		let direction = (Self::reflect(hit) + Vec3::random_in_unit_sphere() * self.fuzziness).unit();

		if direction.dot(hit.normal) > 0.0 {
			Some(Redirection {
				direction: direction,
				albedo: self.albedo
			})
		} else {
			None
		}
	}
}

pub struct Refractive {
	pub albedo: Vec3,
	pub index: f64
}

impl Refractive {
	fn schlick(&self, cos: f64) -> f64 {
		let mut r0 = (1.0 - self.index) / (1.0 + self.index);
		r0 *= r0;
		r0 + (1.0 - r0) * (1.0 - cos).powi(5)
	}
}

impl Material for Refractive {
	fn redirect(&self, hit: &Hit) -> Option<Redirection> {
		let cos = -hit.ray.direction.dot(hit.normal);

		let sin_ratio = match hit.face {
			Face::Front => 1.0 / self.index,
			Face::Back => self.index
		};

		let cos_ratio = (1.0 - sin_ratio * sin_ratio * (1.0 - cos * cos)).sqrt();

		Some(Redirection {
			direction: if cos_ratio.is_nan() || rand::random::<f64>() < self.schlick(cos) {
				Specular::reflect(hit)
			} else {
				(hit.ray.direction + hit.normal * cos) * sin_ratio - hit.normal * cos_ratio
			},

			albedo: self.albedo
		})

	}
}
