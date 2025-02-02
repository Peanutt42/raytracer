const PCG_MULTIPLIER: u32 = 747796405u;
const PCG_INCREMENT: u32 = 2891336453u;
const NOISE1: u32 = 2246822519u;
const NOISE2: u32 = 3266489917u;
const NOISE3: u32 = 668265263u;

fn pcg_hash(seed: u32) -> u32 {
    var state = seed;
    // LCG step
    state = state * PCG_MULTIPLIER + PCG_INCREMENT;
    // XSH-RR permutation
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


struct Camera {
	position: vec3<f32>,
}

struct Sphere {
    position: vec3<f32>,
    emission: f32,
    color: vec3<f32>,
    radius: f32,
}

fn sphere_get_normal(sphere: Sphere, point: vec3<f32>) -> vec3<f32> {
	return (point - sphere.position) / sphere.radius;
}

fn sphere_emission(sphere: Sphere) -> vec3<f32> {
	return sphere.color * sphere.emission;
}

struct Ray {
	origin: vec3<f32>,
	dir: vec3<f32>,
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

fn get_ray_point(ray: Ray, distance: f32) -> vec3<f32> {
	return ray.origin + ray.dir * distance;
}

/*
TODO:

let unit_dir = ray_dir.normalize();
let a = 0.5 * (unit_dir.y + 1.0);
Vec3::one() * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
*/
fn sky_color(ray: Ray) -> vec3<f32> {
	return vec3<f32>(0.5, 0.7, 1.0);
}

// returns wheter to continue tracing the ray (returns false when sky was hit)
fn ray_color(light: ptr<function, vec3<f32>>, contribution: ptr<function, vec3<f32>>, ray: ptr<function, Ray>) -> bool {
	var closest_distance = f32(1e5);
	var closest_sphere = Sphere(vec3<f32>(0.0), 0.0, vec3<f32>(0.0), 0.0);

	for (var i = 0u; i < arrayLength(&spheres); i++) {
	    let sphere = spheres[i];
	    let distance = ray_sphere_distance(*ray, sphere);

	    if (distance > 0.0 && distance < closest_distance) {
	        closest_distance = distance;
	        closest_sphere = sphere;
	    }
	}

	if (closest_distance == 1e5) {
		*light += *contribution * sky_color(*ray);
		return false;
	}

	let hit_point = get_ray_point(*ray, closest_distance);
	let normal = sphere_get_normal(closest_sphere, hit_point);
	*contribution *= closest_sphere.color;
	*light += sphere_emission(closest_sphere);
	let reflection_dir = reflect((*ray).dir, normal);
	*ray = Ray(hit_point, reflection_dir);
	return true;
}

@group(0) @binding(0) var<storage> spheres: array<Sphere>;
@group(0) @binding(1) var output_image: texture_storage_2d<rgba32float, read_write>;

@group(1) @binding(0) var<uniform> camera: Camera;

@group(2) @binding(0) var<uniform> frame_counter: u32;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_size = textureDimensions(output_image);

    let x = i32(global_id.x);
    let y = i32(global_id.y);
    if (x >= texture_size.x || y >= texture_size.y) {
        return;
    }

    let aspect_ratio = f32(texture_size.x) / f32(texture_size.y);
    var uv = vec2<f32>(
        (f32(global_id.x) + 0.5) / f32(texture_size.x) * 2.0 - 1.0,
        (f32(global_id.y) + 0.5) / f32(texture_size.y) * 2.0 - 1.0
    );

    uv.x *= aspect_ratio;

    var pcg_state = global_id.x * 256u + global_id.y * 16u + frame_counter;
    let random_uv_offset_x = (random_f32_in_range(&pcg_state, -0.5, 0.5) / f32(texture_size.x)) * aspect_ratio;
    let random_uv_offset_y = random_f32_in_range(&pcg_state, -0.5, 0.5) / f32(texture_size.y);

    uv += vec2<f32>(random_uv_offset_x, random_uv_offset_y);
    var ray = Ray(camera.position, normalize(vec3<f32>(uv, -1.0)));
    var light = vec3<f32>(0.0);
    var contribution = vec3<f32>(1.0);

    //				  max_depth: 3
    for (var i = 0u; i < 3u; i++) {
    	if (!ray_color(&light, &contribution, &ray)) {
     		break;
       	}
    }

    let texture_coord = vec2<i32>(global_id.xy);
    let old_color: vec4<f32> = textureLoad(output_image, texture_coord);
    let new_color = old_color + vec4<f32>(light, 1.0);
    textureStore(output_image, texture_coord, new_color);
}