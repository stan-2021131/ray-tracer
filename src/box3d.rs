use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

/// Representa una caja o prisma rectangular 3D (AABB orientada en los ejes) de dimensiones arbitrarias.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Box3D {
    pub center: Vec3,
    pub size: Vec3,
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

#[allow(dead_code)]
impl Box3D {
    /// Crea una nueva caja 3D a partir de su centro y vector de dimensiones (ancho X, alto Y, profundidad Z).
    pub fn new(center: Vec3, size: Vec3, material: Material) -> Self {
        let half = size / 2.0;
        let min = center - half;
        let max = center + half;

        Box3D {
            center,
            size,
            min,
            max,
            material,
        }
    }

    /// Crea una caja 3D a partir de sus esquinas mínima y máxima.
    pub fn from_min_max(min: Vec3, max: Vec3, material: Material) -> Self {
        let size = max - min;
        let center = min + size / 2.0;

        Box3D {
            center,
            size,
            min,
            max,
            material,
        }
    }

    /// Constructor de conveniencia para cubos uniformes.
    pub fn cube(center: Vec3, size: f32, material: Material) -> Self {
        Self::new(center, Vec3::new(size, size, size), material)
    }

    /// Calcula las coordenadas de textura UV (u, v) en el rango [0.0, 1.0] para cualquier cara impactada de la caja rectangular.
    ///
    /// Proyecta el punto de impacto normalizándolo con respecto a las dimensiones reales de cada cara.
    pub fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let min = self.min;
        let max = self.max;
        let size = self.size;

        if normal.x.abs() > 0.5 {
            // Caras laterales (+X o -X): mapeamos el plano Z (ancho) e Y (alto)
            let u = if normal.x > 0.0 {
                (max.z - point.z) / size.z.max(1e-6)
            } else {
                (point.z - min.z) / size.z.max(1e-6)
            };
            let v = (max.y - point.y) / size.y.max(1e-6);
            (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        } else if normal.y.abs() > 0.5 {
            // Caras superior e inferior (+Y o -Y): mapeamos el plano X (ancho) y Z (profundidad)
            let u = (point.x - min.x) / size.x.max(1e-6);
            let v = if normal.y > 0.0 {
                (point.z - min.z) / size.z.max(1e-6)
            } else {
                (max.z - point.z) / size.z.max(1e-6)
            };
            (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        } else {
            // Caras frontal y trasera (+Z o -Z): mapeamos el plano X (ancho) e Y (alto)
            let u = if normal.z > 0.0 {
                (point.x - min.x) / size.x.max(1e-6)
            } else {
                (max.x - point.x) / size.x.max(1e-6)
            };
            let v = (max.y - point.y) / size.y.max(1e-6);
            (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        }
    }
}

impl RayIntersect for Box3D {
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
        Some(self.material.to_intersect(point, normal, t, u, v))
    }

    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        let inv_dx = 1.0 / ray_direction.x;
        let (t0x, t1x) = if ray_direction.x >= 0.0 {
            ((self.min.x - ray_origin.x) * inv_dx, (self.max.x - ray_origin.x) * inv_dx)
        } else {
            ((self.max.x - ray_origin.x) * inv_dx, (self.min.x - ray_origin.x) * inv_dx)
        };

        let inv_dy = 1.0 / ray_direction.y;
        let (t0y, t1y) = if ray_direction.y >= 0.0 {
            ((self.min.y - ray_origin.y) * inv_dy, (self.max.y - ray_origin.y) * inv_dy)
        } else {
            ((self.max.y - ray_origin.y) * inv_dy, (self.min.y - ray_origin.y) * inv_dy)
        };

        let t_min = t0x.max(t0y);
        let t_max = t1x.min(t1y);

        if t_min > t_max {
            return false;
        }

        let inv_dz = 1.0 / ray_direction.z;
        let (t0z, t1z) = if ray_direction.z >= 0.0 {
            ((self.min.z - ray_origin.z) * inv_dz, (self.max.z - ray_origin.z) * inv_dz)
        } else {
            ((self.max.z - ray_origin.z) * inv_dz, (self.min.z - ray_origin.z) * inv_dz)
        };

        let t_min = t_min.max(t0z);
        let t_max = t_max.min(t1z);

        if t_min > t_max {
            return false;
        }

        let t = if t_min > 0.0 { t_min } else { t_max };
        t > 0.0 && t < max_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_box_slab_front_intersection() {
        let mat = crate::materials::diffuse(Color::new(255, 0, 0));
        // Rectángulo con dimensiones asimétricas: ancho 4.0, alto 2.0, profundidad 1.0
        let b = Box3D::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(4.0, 2.0, 1.0), mat);

        let ray_origin = Vec3::new(0.0, 0.0, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);

        let hit = b.ray_intersect(&ray_origin, &ray_dir).unwrap();
        assert!((hit.distance - 4.5).abs() < 1e-4);
        assert!((hit.point.z - 0.5).abs() < 1e-4);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_box_miss() {
        let mat = crate::materials::diffuse(Color::new(255, 0, 0));
        let b = Box3D::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 4.0, 2.0), mat);

        let ray_origin = Vec3::new(5.0, 5.0, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);

        let hit = b.ray_intersect(&ray_origin, &ray_dir);
        assert!(hit.is_none());
    }

    #[test]
    fn test_box_uv_mapping() {
        let mat = crate::materials::diffuse(Color::new(255, 0, 0));
        // Caja: x: [-1, 1], y: [-2, 2], z: [-0.5, 0.5]
        let b = Box3D::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 4.0, 1.0), mat);

        // Impacto en el centro de la cara frontal (+Z en z=0.5, x=0, y=0)
        let hit_pt = Vec3::new(0.0, 0.0, 0.5);
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let (u, v) = b.get_uv(&hit_pt, &normal);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);
    }
}
