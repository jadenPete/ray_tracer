use crate::{
	camera::Camera,
	math::{Ray, Vec3},
	object::Object
};

use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
