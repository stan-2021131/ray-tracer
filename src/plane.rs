use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::{dot, Vec3};

#[allow(dead_code)]
#[derive(Debug, Clone)]
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

        let u_coord = (u_proj / self.width) + 0.5;
        let v_coord = 0.5 - (v_proj / self.height);

        // Soporte de transparencia Alpha Cutout: si el píxel de la textura es transparente, el rayo continúa
        let color = self.material.get_color(u_coord, v_coord);
        if self.material.has_alpha() && color.a < 128 {
            return None;
        }

        Some(self.material.to_intersect_with_color(point, normal, t, color))
    }

    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        // Materiales transparentes / refractivos (vidrio, ventanas) no bloquean sombras opacas
        if self.material.refractive > 0.5 {
            return false;
        }

        let denom = dot(&self.normal, ray_direction);

        if denom.abs() < 1e-6 {
            return false;
        }

        let p0_l0 = self.center - ray_origin;
        let t = dot(&p0_l0, &self.normal) / denom;

        if t <= 0.0 || t >= max_distance {
            return false;
        }

        let point = ray_origin + ray_direction * t;
        let d = point - self.center;

        let u_proj = dot(&d, &self.u);
        let v_proj = dot(&d, &self.v);

        if u_proj.abs() > self.half_w || v_proj.abs() > self.half_h {
            return false;
        }

        // Si la textura tiene zonas transparentes, la sombra tampoco se proyecta
        if self.material.has_alpha() {
            let u_coord = (u_proj / self.width) + 0.5;
            let v_coord = 0.5 - (v_proj / self.height);
            if self.material.get_color(u_coord, v_coord).a < 128 {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::materials;
    use crate::texture::Texture;
    use std::sync::Arc;

    #[test]
    fn test_plane_intersection_and_alpha_cutout() {
        // Textura 2x2: píxel (0,0) [superior izquierdo] transparente (alfa = 0), demás opacos rojos (alfa = 255)
        let buffer = vec![
            Color::new_rgba(0, 0, 0, 0),       // u < 0.5, v < 0.5 (arriba-izquierda) -> transparente
            Color::new_rgba(255, 0, 0, 255),
            Color::new_rgba(255, 0, 0, 255),
            Color::new_rgba(255, 0, 0, 255),
        ];
        let tex = Arc::new(Texture { width: 2, height: 2, buffer, has_alpha: true });
        let mat = materials::diffuse(Color::new(255, 255, 255)).with_texture(tex);

        let plane = Plane::new_with_normal(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            2.0,
            mat,
        );

        // Rayo hacia zona transparente (esquina superior izquierda: x = -0.5, y = 0.5)
        let ray_origin_transparent = Vec3::new(-0.5, 0.5, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);
        assert!(plane.ray_intersect(&ray_origin_transparent, &ray_dir).is_none());

        // Rayo hacia zona opaca (esquina inferior derecha: x = 0.5, y = -0.5)
        let ray_origin_opaque = Vec3::new(0.5, -0.5, 5.0);
        let hit = plane.ray_intersect(&ray_origin_opaque, &ray_dir);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().color.r, 255);
    }
}
