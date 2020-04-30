use crate::math::{Ray, Path, Vec3};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub mod math;

/// An object is a field in the scene that can intercept rays and return a record containing
/// information about the interaction, thus allowing rays to bounce between different objects.
///
/// While I originally defined an Object trait, I have since used an enum to avoid the lifetime and
/// performance issues commonly associated with trait objects.

#[derive(Copy, Clone)]
pub enum Object {
	Sphere {
		center: Path,
		radius: f64,
		material: Material
	}
}

impl Object {
	/// Returns a hit record if the object intercepts a ray at a distance within
	/// [min_distance, max_distance) from the ray's origin
	fn hit(&self, ray: Ray, min_distance: f64, max_distance: f64) -> Option<Hit> {
		match self {
			Self::Sphere { center, radius, material } => {
				let center = center.at(ray.time);

				// We use the quadratic formula
				let a = ray.direction.dot(ray.direction);
				let half_b = ray.direction.dot(ray.origin - center);
				let c = (ray.origin - center).dot(ray.origin - center) - *radius * *radius;

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
				let normal = (point - center) / *radius;

				Some(Hit::new(ray, distance, point, normal, *material))
			}
		}
	}
}

#[derive(Copy, Clone)]
struct Hit {
	ray: Ray,
	distance: f64,
	point: Vec3,
	normal: Vec3,
	material: Material,
	face: Face,
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
enum Face {
	Front, // Outside
	Back   // Inside
}

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
	fn scatter(self, hit: Hit) -> Option<Vec3> {
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

#[derive(Copy, Clone)]
pub struct Camera {
	origin: Vec3,
	upper_left: Vec3,
	horizontal_unit: Vec3,
	horizontal: Vec3,
	vertical_unit: Vec3,
	vertical: Vec3,
	lens_radius: f64,
	time0: f64,
	time1: f64
}

impl Camera {
	/// Defines a camera at "origin", looking at "target" (whose magnitude is irrelevant)
	///
	/// Viewport:
	///   "aspect" is the ratio of the viewport's width to its height
	///   "fov" is the angle between the left and right sides of the screen and the origin
	///   "rotation" is the angle (clockwise) between the top of the viewport and directly up
	///
	/// Depth of field:
	///   "aperture" is the diameter of the lens from which rays are sent
	///   "focal_length" is the distance from the camera at which objects are in focus
	///
	/// Motion blur:
	///   "time0" and "time1" are the times between which the shutter is open
	pub fn new(
		origin: Vec3,
		target: Vec3,
		aspect: f64,
		fov: f64,
		rotation: f64,
		aperture: f64,
		focal_length: f64,
		time0: f64,
		time1: f64) -> Self
	{
		let direction = (target - origin).unit();
		let rotation = rotation.to_radians();

		let width = (fov.to_radians() / 2.0).tan() * 2.0;
		let height = width / aspect;

		const UP: Vec3 = Vec3(0.0, 1.0, 0.0);

		// Find the unit vector perpendicular to the "direction" and "up" unit vectors,
		// then rotate it by scaling it by the cosine and moving it down by the sin
		let horizontal_unit = direction.cross(UP).unit() * rotation.cos() - UP * rotation.sin();

		// Find the unit vector perpendicular to the "direction" and "horizontal" unit vectors
		let vertical_unit = direction.cross(horizontal_unit);

		// Used to generate random points on the lens
		let horizontal = horizontal_unit * width * focal_length;
		let vertical = vertical_unit * height * focal_length;

		// Now, finding the upper left corner of the viewport crelative to the origin is trivial
		let upper_left = direction * focal_length - horizontal / 2.0 - vertical / 2.0;

		Self {
			origin: origin,
			upper_left: upper_left,
			horizontal_unit: horizontal_unit,
			horizontal: horizontal,
			vertical_unit: vertical_unit,
			vertical: vertical,
			lens_radius: aperture / 2.0,
			time0,
			time1
		}
	}

	fn ray(&self, u: f64, v: f64) -> Ray {
		let w = Vec3::random_in_unit_disk() * self.lens_radius;
		let offset = self.horizontal_unit * w.0 + self.vertical_unit * w.1;

		Ray {
			origin: self.origin + offset,
			direction: (self.upper_left + self.horizontal * u + self.vertical * v - offset).unit(),

			time: if self.time0 == self.time1 {
				self.time0
			} else {
				rand::thread_rng().gen_range(self.time0, self.time1)
			}
		}
	}
}

pub struct Scene {
	objects: Vec<Object>
}

impl Scene {
	pub fn new() -> Self {
		Self {
			objects: Vec::new()
		}
	}

	pub fn add(&mut self, object: Object) {
		self.objects.push(object);
	}

	fn color(&self, mut ray: Ray, min_distance: f64, max_distance: f64, max_depth: u32) -> Vec3 {
		let mut color = Vec3(1.0, 1.0, 1.0);

		for _ in 0..max_depth {
			match self.objects
				.iter()
				.filter_map(|object| object.hit(ray, min_distance, max_distance))
				.min_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap())
			{
				// If we hit the object
				Some(hit) => match hit.material.method.scatter(hit) {
					// If the ray scattered
					Some(direction) => {
						ray = Ray {
							origin: hit.point,
							direction: direction,
							time: ray.time
						};

						color *= hit.material.albedo;
					}

					// If the object absorbed the ray
					None => break
				}

				// If we didn't hit the object
				None => {
					let t = (ray.direction.1 + 1.0) / 2.0;

					const COLOR1: Vec3 = Vec3(0.5, 0.7, 1.0);
					const COLOR2: Vec3 = Vec3(1.0, 1.0, 1.0);

					return color * (COLOR1 * t + COLOR2 * (1.0 - t));
				}
			}
		}

		Vec3(0.0, 0.0, 0.0)
	}

	pub fn render(&self,
		width: u32,
		height: u32,
		camera: Camera,
		min_distance: f64,
		max_distance: f64,
		samples_per_pixel: u32,
		max_depth: u32) -> Vec<Vec3>
	{
		let pb = &ProgressBar::new((width * height) as u64);

		pb.set_draw_delta(width as u64);

		pb.set_style(ProgressStyle::default_bar()
			.template("{elapsed_precise} / {eta_precise} {wide_bar} {pos} / {len} pixels")
			.progress_chars("=> "));

		let image = (0..height).into_par_iter().flat_map(|y| {
			(0..width).into_par_iter().map(move |x| {
				let color = (0..samples_per_pixel).into_par_iter().map(|_| {
					let mut rng = rand::thread_rng();

					let u = (x as f64 + rng.gen::<f64>()) / width as f64;
					let v = (y as f64 + rng.gen::<f64>()) / height as f64;

					self.color(camera.ray(u, v), min_distance, max_distance, max_depth)
				}).sum::<Vec3>() / samples_per_pixel as f64;

				pb.inc(1);
				color
			})
		}).collect();

		pb.finish();
		image
	}
}
