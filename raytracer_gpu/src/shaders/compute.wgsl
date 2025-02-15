const PCG_MULTIPLIER: u32 = 747796405u;
const PCG_INCREMENT: u32 = 2891336453u;
const NOISE1: u32 = 2246822519u;
const NOISE2: u32 = 3266489917u;
const NOISE3: u32 = 668265263u;

fn pcg_hash(seed: u32) -> u32 {
    var state = seed;
    state = state * PCG_MULTIPLIER + PCG_INCREMENT;
    state ^= state >> 17u;
    state *= NOISE1;
    state ^= state >> 15u;
    state *= NOISE2;
    state ^= state >> 16u;
    return state;
}

// returns 0.0 .. 1.0
fn random_f32(seed: ptr<function, u32>) -> f32 {
    let r = pcg_hash(*seed);
    *seed = r;
    return f32(r) / f32(0xFFFFFFFFu);
}

fn random_f32_in_range(seed: ptr<function, u32>, min: f32, max: f32) -> f32 {
	return min + (max - min) * random_f32(seed);
}

// already normalized
fn random_unit_vec3(seed: ptr<function, u32>) -> vec3<f32> {
	let x = random_f32_in_range(seed, -1.0, 1.0);
	let y = random_f32_in_range(seed, -1.0, 1.0);
	let z = random_f32_in_range(seed, -1.0, 1.0);
	return normalize(vec3<f32>(x, y, z));
}


struct Camera {
	inverse_projection: mat4x4<f32>,
	inverse_view: mat4x4<f32>,
	position: vec3<f32>,
}

struct Ray {
	origin: vec3<f32>,
	dir: vec3<f32>,
}

struct RayHit {
	hit_point: vec3<f32>,
	normal: vec3<f32>,
	front_face: bool,
}

struct Scattered {
	dir: vec3<f32>,
	attenuation: vec3<f32>,
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
	var r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
	r0 *= r0;
	return r0 + (1.0 - r0) * pow(1.0 - cosine, 5.0);
}

fn scatter(albedo: vec3<f32>, material_type: u32, material_param1: f32, ray_in: Ray, ray_hit: RayHit, seed: ptr<function, u32>) -> Scattered {
	switch (material_type) {
		// LAMBERTAIN_MATERIAL_TYPE
		case 0u, default: {
			let dir = normalize(ray_hit.normal + random_unit_vec3(seed));
			return Scattered(dir, albedo);
		}
		// METALIC_MATERIAL_TYPE
		case 1u: {
			let fuzz = material_param1;
			let dir = reflect(ray_in.dir, ray_hit.normal) + fuzz * random_unit_vec3(seed);
			return Scattered(dir, albedo);
		}
		// DIELECTRIC_MATERIAL_TYPE
		case 2u: {
			var ir = select(material_param1, 1.0 / material_param1, ray_hit.front_face);
			let cos_theta = min(dot(-ray_in.dir, ray_hit.normal), 1.0);
			let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
			let cannot_refract = ir * sin_theta > 1.0;
			let reflect = cannot_refract || (reflectance(cos_theta, ir) > random_f32_in_range(seed, 0.0, 1.0));
			let dir = select(refract(ray_in.dir, ray_hit.normal, ir), reflect(ray_in.dir, ray_hit.normal), reflect);
			return Scattered(dir, vec3<f32>(1.0));
		}
	}
}

fn emission(albedo: vec3<f32>, material_type: u32, material_param1: f32) -> vec3<f32> {
	let emission = material_param1;
	// 												LAMBERTAIN_MATERIAL_TYPE
	return select(vec3<f32>(0.0), albedo * emission, material_type == 0u);
}

struct Sphere {
    position: vec3<f32>,
    radius: f32,
    albedo: vec3<f32>,
    material_type: u32,
    material_param1: f32,
}

fn sphere_get_normal(sphere: Sphere, hit_point: vec3<f32>) -> vec3<f32> {
	return (hit_point - sphere.position) / sphere.radius;
}

// returns 1e5 if not hit
fn ray_sphere_distance(ray: Ray, sphere: Sphere) -> f32 {
	let oc = ray.origin - sphere.position;
	let a = dot(ray.dir, ray.dir);
	let b = 2.0 * dot(oc, ray.dir);
	let c = dot(oc, oc) - sphere.radius * sphere.radius;
	let discriminant = b * b - 4.0 * a * c;

	if (discriminant >= 0.0) {
	    return (-b - sqrt(discriminant)) / (2.0 * a);
	} else {
		return 1e5;
	}
}

struct Cube {
	position: vec3<f32>,
	half_extend: vec3<f32>,
	albedo: vec3<f32>,
	material_type: u32,
    material_param1: f32,
}

// returns 1e5 if not hit
fn ray_cube_distance(ray: Ray, cube: Cube) -> f32 {
    let min_bound = cube.position - cube.half_extend;
    let max_bound = cube.position + cube.half_extend;

    let t1 = (min_bound - ray.origin) / ray.dir;
    let t2 = (max_bound - ray.origin) / ray.dir;

    let t_min = min(t1, t2);
    let t_max = max(t1, t2);

    let t_enter = max(max(t_min.x, t_min.y), t_min.z);
    let t_exit = min(min(t_max.x, t_max.y), t_max.z);

    if t_enter > t_exit || t_exit < 0.0 {
        return 1e5;
    }

    return max(t_enter, 0.0);
}

fn signum(x: f32) -> f32 {
	if (x > 0.0) {
		return 1.0;
	} else {
		return -1.0;
	}
}

fn cube_get_normal(cube: Cube, hit_point: vec3<f32>) -> vec3<f32> {
	let rel_p = hit_point - cube.position;
	let maxc = max(max(abs(rel_p.x), abs(rel_p.y)), abs(rel_p.z));
	if (maxc == abs(rel_p.x)) {
		return vec3<f32>(signum(rel_p.x), 0.0, 0.0);
	}
	if (maxc == abs(rel_p.y)) {
		return vec3<f32>(0.0, signum(rel_p.y), 0.0);
	}
	return vec3<f32>(0.0, 0.0, signum(rel_p.z));
}

fn get_ray_point(ray: Ray, distance: f32) -> vec3<f32> {
	return ray.origin + ray.dir * distance;
}

/*
normal skybox color implementation
fn sky_color(ray: Ray) -> vec3<f32> {
	let a = 0.5 * (ray.dir.y + 1.0);
	return vec3<f32>(1.0) * (1.0 - a) + vec3<f32>(0.5, 0.7, 1.0) * a;
}*/


// wallpaper scene sky color implementation
fn sky_color(ray: Ray) -> vec3<f32> {
	var strength = 0.5 * (-ray.dir.y + 0.25);
	let t = 0.5 * (ray.dir.x + 1.0);
	strength *= 50.0 * pow(99.0, pow(2.0 * t - 1.0, 2.0) - 1.0);
	let a = vec3<f32>(0.94, 0.02, 0.99);
	let b = vec3<f32>(0.0, 0.85, 0.98);
	let c = vec3<f32>(0.0, 0.45, 0.98);
	let d = vec3<f32>(0.0, 0.98, 0.45);
	return (a * (1.0 - t) + b * t) * strength + 0.25 * c * (0.5 * (ray.dir.y + 1.0)) + 0.15 * d * (0.5 * (-ray.dir.x + 1.0));
}

// returns wheter to continue tracing the ray (returns false when sky was hit)
fn ray_color(light: ptr<function, vec3<f32>>, contribution: ptr<function, vec3<f32>>, ray: ptr<function, Ray>, seed: ptr<function, u32>) -> bool {
	var closest_sphere_distance = f32(1e5);
	var closest_sphere = Sphere(vec3<f32>(0.0), 0.0, vec3<f32>(0.0), 0u, 0.0);
	var closest_cube_distance = f32(1e5);
	var closest_cube = Cube(vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), 0u, 0.0);

	for (var i = 0u; i < arrayLength(&spheres); i++) {
	    let sphere = spheres[i];
	    let distance = ray_sphere_distance(*ray, sphere);

	    if (distance > 0.0 && distance < closest_sphere_distance) {
	        closest_sphere_distance = distance;
	        closest_sphere = sphere;
	    }
	}
	for (var i = 0u; i < arrayLength(&cubes); i++) {
	    let cube = cubes[i];
	    let distance = ray_cube_distance(*ray, cube);

	    if (distance > 0.0 && distance < closest_cube_distance) {
	        closest_cube_distance = distance;
	        closest_cube = cube;
	    }
	}

	if (closest_sphere_distance == 1e5 && closest_cube_distance == 1e5) {
		*light += *contribution * sky_color(*ray);
		return false;
	}

	let closest_distance = min(closest_sphere_distance, closest_cube_distance);
	let hit_point = get_ray_point(*ray, closest_distance);
	var normal = vec3<f32>(0.0);
	var scattered = Scattered(vec3<f32>(0.0), vec3<f32>(0.0));
	var emission = vec3<f32>(0.0);

	if (closest_sphere_distance < closest_cube_distance) {
		normal = sphere_get_normal(closest_sphere, hit_point);
		let front_face = dot((*ray).dir, normal) < 0.0;
		if (!front_face) {
			normal = -normal;
		}
		let ray_hit = RayHit(hit_point, normal, front_face);
		emission = emission(closest_sphere.albedo, closest_sphere.material_type, closest_sphere.material_param1);
		scattered = scatter(closest_sphere.albedo, closest_sphere.material_type, closest_sphere.material_param1, *ray, ray_hit, seed);
	} else {
		normal = cube_get_normal(closest_cube, hit_point);
		let front_face = dot((*ray).dir, normal) < 0.0;
		if (!front_face) {
			normal = -normal;
		}
		let ray_hit = RayHit(hit_point, normal, front_face);
		emission = emission(closest_cube.albedo, closest_cube.material_type, closest_cube.material_param1);
		scattered = scatter(closest_cube.albedo, closest_cube.material_type, closest_cube.material_param1, *ray, ray_hit, seed);
	}

	let emission_strength = dot(normal, (*ray).dir);
	*light += emission * (*contribution);
	*contribution *= scattered.attenuation;
	// this fixes circular banding of glass material due to floating point precision issues with ray collision inside
	let new_ray_origin = hit_point - normal * 1e-4;
	*ray = Ray(new_ray_origin, scattered.dir);
	return true;
}

fn ray_dir(global_id: vec3<u32>, texture_size: vec2<i32>, pcg_state: ptr<function, u32>) -> vec3<f32> {
	let sample_offset = vec2<f32>(random_f32_in_range(pcg_state, -0.5, 0.5), random_f32_in_range(pcg_state, -0.5, 0.5));

	var coord = (vec2<f32>(global_id.xy) + sample_offset) / vec2<f32>(texture_size);
	coord = coord * 2.0 - vec2<f32>(1.0); // -1.0 -> 1.0

	let target_ = camera.inverse_projection * vec4<f32>(coord, 1.0, 1.0);
	return (camera.inverse_view * vec4<f32>(normalize(target_.xyz / target_.w), 0.0)).xyz;
}

@group(0) @binding(0) var<storage> spheres: array<Sphere>;
@group(0) @binding(1) var<storage> cubes: array<Cube>;
@group(0) @binding(2) var output_image: texture_storage_2d<rgba32float, read_write>;

@group(1) @binding(0) var<uniform> camera: Camera;

@group(2) @binding(0) var<uniform> frame_counter: u32;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let texture_size: vec2<i32> = textureDimensions(output_image);

    // this is more noisy at first:
    var pcg_state = global_id.x * u32(texture_size.x) + global_id.y + frame_counter * u32(texture_size.x * texture_size.y);

    // this is more smooth at first, looks more like a painting:
    // var pcg_state = frame_counter;

    let x = i32(global_id.x);
    let y = i32(global_id.y);
    if (x >= texture_size.x || y >= texture_size.y) {
        return;
    }

    var ray = Ray(camera.position, ray_dir(global_id, texture_size, &pcg_state));
    var light = vec3<f32>(0.0);
    var contribution = vec3<f32>(1.0);

    //				  max_depth: 5
    for (var i = 0u; i < 5u; i++) {
    	if (!ray_color(&light, &contribution, &ray, &pcg_state)) {
     		break;
       	}
    }

    let texture_coord = vec2<i32>(global_id.xy);
    let old_color: vec4<f32> = textureLoad(output_image, texture_coord);
    let new_color = old_color + vec4<f32>(light, 1.0);
    textureStore(output_image, texture_coord, new_color);
}