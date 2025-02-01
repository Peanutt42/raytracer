struct Sphere {
    position: vec3<f32>,
    radius: f32,
}

@group(0) @binding(0) var<storage> spheres: array<Sphere>;
@group(0) @binding(1) var output_image: texture_storage_2d<rgba8unorm, write>;

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

    // Camera setup
    let ray_origin = vec3<f32>(0.0, 0.0, 0.0);
    let ray_dir = normalize(vec3<f32>(uv, -1.0));

    // Ray-sphere intersection
    var closest_hit = f32(1e5);
    var hit_color = vec3<f32>(0.0);

    let light_dir = vec3<f32>(1.0, 1.0, 1.0);

    for (var i = 0u; i < arrayLength(&spheres); i++) {
        let sphere = spheres[i];
        let oc = ray_origin - sphere.position;
        let a = dot(ray_dir, ray_dir);
        let b = 2.0 * dot(oc, ray_dir);
        let c = dot(oc, oc) - sphere.radius * sphere.radius;
        let discriminant = b * b - 4.0 * a * c;

        if (discriminant >= 0.0) {
            let t = (-b - sqrt(discriminant)) / (2.0 * a);
            if (t > 0.0 && t < closest_hit) {
                closest_hit = t;
                let hit_point = ray_origin + ray_dir * t;
                let normal = (hit_point - sphere.position) / sphere.radius;
                let light_intensity = dot(normal, light_dir);
                hit_color = vec3<f32>(0.8, 0.2, 0.2) * light_intensity;
            }
        }
    }

    let color = vec4<f32>(hit_color, 1.0);
    textureStore(output_image, vec2<i32>(global_id.xy), color);
}