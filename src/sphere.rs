use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::{dot, Vec3};

#[allow(dead_code)]
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

        let a = dot(ray_direction, ray_direction);
        let b = 2.0 * dot(&oc, ray_direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant <= 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);

        if t <= 0.0 {
            return None;
        }

        let point = ray_origin + ray_direction * t;
        let normal = (point - self.center).normalize();

        // Mapeo UV esférico
        let d = (point - self.center) / self.radius;
        let u_coord = 0.5 + d.z.atan2(d.x) / (2.0 * std::f32::consts::PI);
        let v_coord = 0.5 - d.y.clamp(-1.0, 1.0).asin() / std::f32::consts::PI;
        let material = self.material.with_uv(u_coord, v_coord);

        Some(Intersect {
            point,
            normal,
            distance: t,
            material,
        })
    }
}