use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::{dot, Vec3};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub center: Vec3,
    pub normal: Vec3,
    pub width: f32,
    pub height: f32,
    pub u: Vec3,
    pub v: Vec3,
    pub half_w: f32,
    pub half_h: f32,
    pub material: Material,
}

#[allow(dead_code)]
impl Plane {
    pub fn new(center: Vec3, width: f32, height: f32, material: Material) -> Self {
        Self::new_with_normal(center, Vec3::new(0.0, 1.0, 0.0), width, height, material)
    }

    pub fn new_with_normal(
        center: Vec3,
        normal: Vec3,
        width: f32,
        height: f32,
        material: Material,
    ) -> Self {
        let n = normal.normalize();

        // Generar base ortonormal (u, v) una sola vez al construir el plano
        let (u, v) = if n.x.abs() > 0.9 {
            let u = Vec3::new(0.0, 1.0, 0.0).cross(&n).normalize();
            let v = n.cross(&u).normalize();
            (u, v)
        } else if n.y.abs() > 0.9 {
            let u = Vec3::new(0.0, 0.0, 1.0).cross(&n).normalize();
            let v = n.cross(&u).normalize();
            (u, v)
        } else {
            let u = Vec3::new(0.0, 1.0, 0.0).cross(&n).normalize();
            let v = n.cross(&u).normalize();
            (u, v)
        };

        Plane {
            center,
            normal: n,
            width,
            height,
            u,
            v,
            half_w: width / 2.0,
            half_h: height / 2.0,
            material,
        }
    }
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let denom = dot(&self.normal, ray_direction);

        // Si el rayo es casi paralelo al plano, no hay intersección
        if denom.abs() < 1e-6 {
            return None;
        }

        let p0_l0 = self.center - ray_origin;
        let t = dot(&p0_l0, &self.normal) / denom;

        if t <= 0.0 {
            return None;
        }

        let point = ray_origin + ray_direction * t;
        let d = point - self.center;

        // Comprobar límites proyectando sobre los ejes u y v precalculados
        let u_proj = dot(&d, &self.u);
        let v_proj = dot(&d, &self.v);

        if u_proj.abs() > self.half_w || v_proj.abs() > self.half_h {
            return None;
        }

        // Normal orientada hacia el origen del rayo
        let normal = if denom < 0.0 {
            self.normal
        } else {
            -self.normal
        };

        Some(Intersect {
            point,
            normal,
            distance: t,
            material: self.material,
        })
    }
}
