use crate::math::Vec3;
use crate::object::{Hit, Face};

#[derive(Copy, Clone)]
pub struct Material {
	pub method: ScatteringMethod,

	// How much light is preserved in each channel
	pub albedo: Vec3
}

#[derive(Copy, Clone)]
pub enum ScatteringMethod {
	// Diffuse
	Lambertian,
	Spherical,
	Hemispherical,

	// Accepts a fuzziness parameter (how "foggy" it is)
	Reflective(f64),

	// Accepts a refractory index (air ~= 1.0, glass ~= 1.5)
	Refractive(f64)
}

impl ScatteringMethod {
	// Schlick's approximation uses the cosine of the incidence angle and the refraction index
	fn schlick(cos: f64, index: f64) -> f64 {
		let mut r0 = (1.0 - index) / (1.0 + index);
		r0 *= r0;
		r0 + (1.0 - r0) * (1.0 - cos).powi(5)
	}

	/// Returns the direction of the newly scattered ray
	pub(crate) fn scatter(self, hit: Hit) -> Option<Vec3> {
		match self {
			Self::Lambertian => Some((hit.normal + Vec3::random_on_unit_sphere()).unit()),
			Self::Spherical => Some((hit.normal + Vec3::random_in_unit_sphere()).unit()),

			Self::Hemispherical => {
				let direction = Vec3::random_in_unit_sphere();

				Some((hit.normal + if direction.dot(hit.normal) >= 0.0 {
					direction
				} else {
					-direction
				}).unit())
			}

			Self::Reflective(fuzziness) => {
				let r = hit.ray.direction;
				let n = hit.normal;

				let direction = (r - n * r.dot(n) * 2.0 + Vec3::random_in_unit_sphere() * fuzziness).unit();

				if direction.dot(n) > 0.0 {
					Some(direction)
				} else {
					None
				}
			}

			Self::Refractive(index) => {
				let r = hit.ray.direction;
				let n = hit.normal;

				let cos = -r.dot(n);

				let sin_ratio = match hit.face {
					Face::Front => 1.0 / index,
					Face::Back => index
				};

				let cos_ratio = (1.0 - sin_ratio * sin_ratio * (1.0 - cos * cos)).sqrt();

				if cos_ratio.is_nan() || rand::random::<f64>() < Self::schlick(cos, index) {
					ScatteringMethod::Reflective(0.0).scatter(hit)
				} else {
					Some((r + n * cos) * sin_ratio - n * cos_ratio)
				}
			}
		}
	}
}
