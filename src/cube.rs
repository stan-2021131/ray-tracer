use crate::plane::Plane;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

#[allow(dead_code)]
pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub planes: Vec<Plane>,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        let half = size / 2.0;

        let planes = vec![
            // Cara frontal (+Z)
            Plane::new_with_normal(
                center + Vec3::new(0.0, 0.0, half),
                Vec3::new(0.0, 0.0, 1.0),
                size,
                size,
                material,
            ),
            // Cara trasera (-Z)
            Plane::new_with_normal(
                center + Vec3::new(0.0, 0.0, -half),
                Vec3::new(0.0, 0.0, -1.0),
                size,
                size,
                material,
            ),
            // Cara derecha (+X)
            Plane::new_with_normal(
                center + Vec3::new(half, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                size,
                size,
                material,
            ),
            // Cara izquierda (-X)
            Plane::new_with_normal(
                center + Vec3::new(-half, 0.0, 0.0),
                Vec3::new(-1.0, 0.0, 0.0),
                size,
                size,
                material,
            ),
            // Cara superior (+Y)
            Plane::new_with_normal(
                center + Vec3::new(0.0, half, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                size,
                size,
                material,
            ),
            // Cara inferior (-Y)
            Plane::new_with_normal(
                center + Vec3::new(0.0, -half, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                size,
                size,
                material,
            ),
        ];

        Cube {
            center,
            size,
            planes,
            material,
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let mut closest: Option<Intersect> = None;

        for plane in &self.planes {
            if let Some(intersect) = plane.ray_intersect(ray_origin, ray_direction) {
                if closest.is_none_or(|current| intersect.distance < current.distance) {
                    closest = Some(intersect);
                }
            }
        }

        closest
    }
}
