use rand::Rng;
use ray_tracer::{math::{Path, Vec3}, Camera, Object, Material, ScatteringMethod, Scene};
use std::{fs::File, io::BufWriter};

#[allow(dead_code)]
fn generate_wide_angle(width: u32, height: u32) -> (Scene, Camera) {
	let mut scene = Scene::new();

	let r = (std::f64::consts::PI / 4.0).cos();

	scene.add(Object::Sphere {
		center: Path::Constant(Vec3(-r, 0.0, -1.0)),
		radius: r,

		material: Material {
			method: ScatteringMethod::Lambertian,
			albedo: Vec3(0.0, 0.0, 1.0)
		}
	});

	scene.add(Object::Sphere {
		center: Path::Constant(Vec3(r, 0.0, -1.0)),
		radius: r,

		material: Material {
			method: ScatteringMethod::Lambertian,
			albedo: Vec3(1.0, 0.0, 0.0)
		}
	});

	const ORIGIN: Vec3 = Vec3(0.0, 0.0, 0.0);
	const TARGET: Vec3 = Vec3(0.0, 0.0, -1.0);
	let aspect = width as f64 / height as f64;
	let fov = ((45.0f64.to_radians().tan() * 2.0).atan() * 2.0).to_degrees();
	const ROTATION: f64 = 0.0;
	const APERATURE: f64 = 0.0;
	const FOCAL_LENGTH: f64 = 1.0;
	const TIME0: f64 = 0.0;
	const TIME1: f64 = 0.0;

	let camera = Camera::new(
		ORIGIN, TARGET, aspect, fov, ROTATION,
		APERATURE, FOCAL_LENGTH,
		TIME0, TIME1);

	(scene, camera)
}

#[allow(dead_code)]
fn generate_cover(width: u32, height: u32) -> (Scene, Camera) {
	let mut scene = Scene::new();

	scene.add(Object::Sphere {
		center: Path::Constant(Vec3(0.0, -1000.0, 0.0)),
		radius: 1000.0,

		material: Material {
			method: ScatteringMethod::Lambertian,
			albedo: Vec3(0.5, 0.5, 0.5)
		}
	});

	let mut rng = rand::thread_rng();

	for i in -11..11 {
		for j in -11..11 {
			let choose_mat: f64 = rng.gen();

			let center = Vec3(i as f64 + 0.9 * rng.gen::<f64>(), 0.2,
			                  j as f64 + 0.9 * rng.gen::<f64>());

			if (center - Vec3(4.0, 0.2, 0.0)).len() > 0.9 {
				scene.add(if choose_mat < 0.8 {
					Object::Sphere {
						center: Path::Linear(
							center,
							center + Vec3(0.0, rng.gen_range(0.0, 0.5), 0.0),
							0.0,
							1.0),

						radius: 0.2,

						material: Material {
							method: ScatteringMethod::Lambertian,
							albedo: Vec3::random_in_unit_cube() * Vec3::random_in_unit_cube()
						}
					}
				} else if choose_mat < 0.95 {
					Object::Sphere {
						center: Path::Constant(center),
						radius: 0.2,

						material: Material {
							method: ScatteringMethod::Reflective(rng.gen_range(0.0, 0.5)),
							albedo: Vec3::random_in_cube(0.5, 1.0)
						}
					}
				} else {
					Object::Sphere {
						center: Path::Constant(center),
						radius: 0.2,

						material: Material {
							method: ScatteringMethod::Refractive(1.5),
							albedo: Vec3(1.0, 1.0, 1.0)
						}
					}
				});
			}
		}
	}

	scene.add(Object::Sphere {
		center: Path::Constant(Vec3(0.0, 1.0, 0.0)),
		radius: 1.0,

		material: Material {
			method: ScatteringMethod::Refractive(1.5),
			albedo: Vec3(1.0, 1.0, 1.0)
		}
	});

	scene.add(Object::Sphere {
		center: Path::Constant(Vec3(-4.0, 1.0, 0.0)),
		radius: 1.0,

		material: Material {
			method: ScatteringMethod::Lambertian,
			albedo: Vec3(0.4, 0.2, 0.1)
		}
	});

	scene.add(Object::Sphere {
		center: Path::Constant(Vec3(4.0, 1.0, 0.0)),
		radius: 1.0,

		material: Material {
			method: ScatteringMethod::Reflective(0.0),
			albedo: Vec3(0.7, 0.6, 0.5)
		}
	});

	const ORIGIN: Vec3 = Vec3(13.0, 2.0, 3.0);
	const TARGET: Vec3 = Vec3(0.0, 0.0, 0.0);
	let aspect = width as f64 / height as f64;
	let fov = (20.0f64.to_radians().tan() * 2.0).atan().to_degrees();
	const ROTATION: f64 = 0.0;
	const APERATURE: f64 = 0.1;
	const FOCAL_LENGTH: f64 = 10.0;
	const TIME0: f64 = 0.0;
	const TIME1: f64 = 1.0;

	let camera = Camera::new(
		ORIGIN, TARGET, aspect, fov, ROTATION,
		APERATURE, FOCAL_LENGTH,
		TIME0, TIME1);

	(scene, camera)
}

fn main() {
	const WIDTH: u32 = 200;
	const HEIGHT: u32 = 100;

	let mut data = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);

	let (scene, camera) = generate_cover(WIDTH, HEIGHT);

	for color in scene.render(WIDTH, HEIGHT, camera, 0.001, std::f64::INFINITY, 75, 10) {
		// Gamma correction (1 / 2)
		data.push((color.0.sqrt() * 256.0).min(255.0) as u8);
		data.push((color.1.sqrt() * 256.0).min(255.0) as u8);
		data.push((color.2.sqrt() * 256.0).min(255.0) as u8);
	}

	let mut file = BufWriter::new(File::create("output.png").unwrap());
	let mut encoder = png::Encoder::new(&mut file, WIDTH, HEIGHT);

	encoder.set_color(png::ColorType::RGB);
	encoder
		.write_header()
		.unwrap()
		.write_image_data(data.as_slice())
		.unwrap();
}
