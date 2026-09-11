use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

#[allow(dead_code)]
pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

#[allow(dead_code)]
impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        let half = size / 2.0;
        let min = center - Vec3::new(half, half, half);
        let max = center + Vec3::new(half, half, half);

        Cube {
            center,
            size,
            min,
            max,
            material,
        }
    }

    /// Calcula las coordenadas de textura UV (u, v) en el rango [0.0, 1.0] para cualquier cara impactada del cubo.
    pub fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let half = self.size / 2.0;
        let d = (point - self.center) / half;

        if normal.x.abs() > 0.5 {
            // Caras laterales (+X o -X): usamos los ejes Z e Y
            let u = if normal.x > 0.0 {
                (1.0 - d.z) / 2.0
            } else {
                (d.z + 1.0) / 2.0
            };
            let v = (1.0 - d.y) / 2.0;
            (u, v)
        } else if normal.y.abs() > 0.5 {
            // Caras superior e inferior (+Y o -Y): usamos los ejes X y Z
            let u = (d.x + 1.0) / 2.0;
            let v = if normal.y > 0.0 {
                (d.z + 1.0) / 2.0
            } else {
                (1.0 - d.z) / 2.0
            };
            (u, v)
        } else {
            // Caras frontal y trasera (+Z o -Z): usamos los ejes X e Y
            let u = if normal.z > 0.0 {
                (d.x + 1.0) / 2.0
            } else {
                (1.0 - d.x) / 2.0
            };
            let v = (1.0 - d.y) / 2.0;
            (u, v)
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let inv_d = Vec3::new(
            1.0 / ray_direction.x,
            1.0 / ray_direction.y,
            1.0 / ray_direction.z,
        );

        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        let mut normal_near = Vec3::new(0.0, 0.0, 0.0);
        let mut normal_far = Vec3::new(0.0, 0.0, 0.0);

        // Eje X
        let (t0x, t1x, nx) = if ray_direction.x >= 0.0 {
            (
                (self.min.x - ray_origin.x) * inv_d.x,
                (self.max.x - ray_origin.x) * inv_d.x,
                Vec3::new(-1.0, 0.0, 0.0),
            )
        } else {
            (
                (self.max.x - ray_origin.x) * inv_d.x,
                (self.min.x - ray_origin.x) * inv_d.x,
                Vec3::new(1.0, 0.0, 0.0),
            )
        };
        if t0x > t_min {
            t_min = t0x;
            normal_near = nx;
        }
        if t1x < t_max {
            t_max = t1x;
            normal_far = -nx;
        }
        if t_min > t_max {
            return None;
        }

        // Eje Y
        let (t0y, t1y, ny) = if ray_direction.y >= 0.0 {
            (
                (self.min.y - ray_origin.y) * inv_d.y,
                (self.max.y - ray_origin.y) * inv_d.y,
                Vec3::new(0.0, -1.0, 0.0),
            )
        } else {
            (
                (self.max.y - ray_origin.y) * inv_d.y,
                (self.min.y - ray_origin.y) * inv_d.y,
                Vec3::new(0.0, 1.0, 0.0),
            )
        };
        if t0y > t_min {
            t_min = t0y;
            normal_near = ny;
        }
        if t1y < t_max {
            t_max = t1y;
            normal_far = -ny;
        }
        if t_min > t_max {
            return None;
        }

        // Eje Z
        let (t0z, t1z, nz) = if ray_direction.z >= 0.0 {
            (
                (self.min.z - ray_origin.z) * inv_d.z,
                (self.max.z - ray_origin.z) * inv_d.z,
                Vec3::new(0.0, 0.0, -1.0),
            )
        } else {
            (
                (self.max.z - ray_origin.z) * inv_d.z,
                (self.min.z - ray_origin.z) * inv_d.z,
                Vec3::new(0.0, 0.0, 1.0),
            )
        };
        if t0z > t_min {
            t_min = t0z;
            normal_near = nz;
        }
        if t1z < t_max {
            t_max = t1z;
            normal_far = -nz;
        }
        if t_min > t_max {
            return None;
        }

        let (t, normal) = if t_min > 0.0 {
            (t_min, normal_near)
        } else if t_max > 0.0 {
            (t_max, normal_far)
        } else {
            return None;
        };

        let point = ray_origin + ray_direction * t;
        let (u, v) = self.get_uv(&point, &normal);
        let material = self.material.with_uv(u, v);

        Some(Intersect {
            point,
            normal,
            distance: t,
            material,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_slab_front_intersection() {
        let mat = Material::new(Color::new(255, 0, 0), 10.0, [0.8, 0.2]);
        let cube = Cube::new(Vec3::new(0.0, 0.0, 0.0), 2.0, mat);

        let ray_origin = Vec3::new(0.0, 0.0, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);

        let hit = cube.ray_intersect(&ray_origin, &ray_dir).unwrap();
        assert!((hit.distance - 4.0).abs() < 1e-4);
        assert!((hit.point.z - 1.0).abs() < 1e-4);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_slab_miss() {
        let mat = Material::new(Color::new(255, 0, 0), 10.0, [0.8, 0.2]);
        let cube = Cube::new(Vec3::new(0.0, 0.0, 0.0), 2.0, mat);

        let ray_origin = Vec3::new(5.0, 5.0, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);

        let hit = cube.ray_intersect(&ray_origin, &ray_dir);
        assert!(hit.is_none());
    }
}
