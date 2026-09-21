use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::{dot, Vec3};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

#[allow(dead_code)]
impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material,
        }
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let oc = ray_origin - self.center;

        // ray_direction ya está normalizado (a = 1.0)
        let b = dot(&oc, ray_direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;

        let discriminant = b * b - c;

        if discriminant <= 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut t = -b - sqrt_d;

        if t <= 0.0 {
            t = -b + sqrt_d;
            if t <= 0.0 {
                return None;
            }
        }

        let point = ray_origin + ray_direction * t;
        // Evitar sqrt en normalización dividiendo directamente por el radio conocido
        let normal = (point - self.center) / self.radius;

        // Mapeo UV esférico (normal es equivalente a d)
        let u_coord = 0.5 + normal.z.atan2(normal.x) / (2.0 * std::f32::consts::PI);
        let v_coord = 0.5 - normal.y.clamp(-1.0, 1.0).asin() / std::f32::consts::PI;

        Some(self.material.to_intersect(point, normal, t, u_coord, v_coord))
    }

    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        let oc = ray_origin - self.center;
        let b = dot(&oc, ray_direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;

        let discriminant = b * b - c;

        if discriminant <= 0.0 {
            return false;
        }

        let sqrt_d = discriminant.sqrt();
        let mut t = -b - sqrt_d;

        if t <= 0.0 {
            t = -b + sqrt_d;
        }

        t > 0.0 && t < max_distance
    }
}