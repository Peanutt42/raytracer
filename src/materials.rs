use crate::{random, Ray, RayHit, Scalar, Vec3};

#[derive(Clone, Copy, Debug)]
pub enum Material {
    Lambertain { albedo: Vec3, emission: Scalar },
    Metal { albedo: Vec3, fuzz: Scalar },
    Dielectric { ir: Scalar },
}

pub struct Scattered {
    pub attenuation: Vec3,
    pub scattered: Ray,
}

impl Material {
    #[inline(always)]
    pub fn scatter(
        &self,
        ray_in: &Ray,
        hit: &RayHit,
        rand: &mut rand::prelude::ThreadRng,
    ) -> Option<Scattered> {
        match self {
            Self::Lambertain { albedo, .. } => {
                let mut scatter_direction = hit.normal + Vec3::random_unit_vector(rand);
                if scatter_direction.near_zero() {
                    scatter_direction = hit.normal;
                }
                Some(Scattered {
                    attenuation: *albedo,
                    scattered: Ray::new(hit.point, scatter_direction),
                })
            }
            Self::Metal { albedo, fuzz } => {
                let reflected = ray_in.dir.normalize().reflect(hit.normal);
                let scattered_dir = reflected + *fuzz * Vec3::random_unit_vector(rand);
                if scattered_dir.dot(hit.normal) <= 0.0 {
                    return None;
                }
                let scattered = Ray::new(hit.point, scattered_dir);
                Some(Scattered {
                    attenuation: *albedo,
                    scattered,
                })
            }
            Self::Dielectric { ir } => {
                let refration_ratio = if hit.front_face { 1.0 / *ir } else { *ir };

                let unit_dir = ray_in.dir.normalize();
                let cos_theta = Scalar::min((-unit_dir).dot(hit.normal), 1.0);
                let sin_theta = Scalar::sqrt(1.0 - (cos_theta * cos_theta));

                let cannot_refract = refration_ratio * sin_theta > 1.0;
                let direction = if cannot_refract
                    || (Self::reflectance(cos_theta, refration_ratio) > random(0.0, 1.0, rand))
                {
                    unit_dir.reflect(hit.normal)
                } else {
                    Vec3::refract(unit_dir, hit.normal, refration_ratio)
                };

                Some(Scattered {
                    attenuation: Vec3::one(),
                    scattered: Ray::new(hit.point, direction),
                })
            }
        }
    }

    #[inline(always)]
    pub fn emission_color(&self) -> Vec3 {
        match self {
            Material::Dielectric { .. } => Vec3::zero(),
            Material::Lambertain { albedo, emission } => (*albedo) * (*emission),
            Material::Metal { .. } => Vec3::zero(),
        }
    }

    #[inline(always)]
    fn reflectance(cosine: Scalar, ref_idx: Scalar) -> Scalar {
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1. - ref_idx) / (1. + ref_idx);
        r0 = r0 * r0;
        r0 + (1. - r0) * (1. - cosine).powf(5.)
    }
}
