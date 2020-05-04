use crate::{
	math::{Ray, Vec3},
	object::Object
};

use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct Scene {
	objects: Vec<Box<dyn Object>>
}

impl Scene {
	pub fn new() -> Self {
		Self {
			objects: Vec::new()
		}
	}

	pub fn add<T: Object + 'static>(&mut self, object: T) {
		self.objects.push(Box::new(object));
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
				Some(hit) => match hit.material.redirect(&hit) {
					// If the ray scattered
					Some(redirection) => {
						ray = Ray {
							origin: hit.point,
							direction: redirection.direction,
							time: ray.time
						};

						color *= redirection.albedo;
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
			.template("{elapsed_precise} / {eta_precise} {wide_bar} {pos} / {len} pixels {per_sec}")
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
