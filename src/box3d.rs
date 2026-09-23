use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::{Mat3, Vec3};

/// Representa una caja o prisma rectangular 3D con dimensiones arbitrarias y rotación 3D en el espacio.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Box3D {
    pub center: Vec3,
    pub size: Vec3,
    pub half_size: Vec3,
    pub rotation: Vec3,         // Ángulos de Euler en radianes (rot_x, rot_y, rot_z)
    pub rot_mat: Mat3,          // Matriz de rotación local -> mundo
    pub inv_rot_mat: Mat3,      // Matriz de rotación inversa mundo -> local
    pub is_rotated: bool,       // Flag para evitar multiplicación de matrices si no está rotado
    pub min: Vec3,              // AABB en coordenadas de mundo
    pub max: Vec3,
    pub material: Material,
}

/// Calcula las matrices de rotación y su inversa a partir de ángulos de Euler (orden Y * X * Z).
fn create_rotation_matrix(rot: &Vec3) -> (Mat3, Mat3, bool) {
    if rot.x.abs() < 1e-6 && rot.y.abs() < 1e-6 && rot.z.abs() < 1e-6 {
        return (Mat3::identity(), Mat3::identity(), false);
    }

    let (sx, cx) = rot.x.sin_cos();
    let (sy, cy) = rot.y.sin_cos();
    let (sz, cz) = rot.z.sin_cos();

    let r00 = cy * cz + sy * sx * sz;
    let r01 = -cy * sz + sy * sx * cz;
    let r02 = sy * cx;

    let r10 = cx * sz;
    let r11 = cx * cz;
    let r12 = -sx;

    let r20 = -sy * cz + cy * sx * sz;
    let r21 = sy * sz + cy * sx * cz;
    let r22 = cy * cx;

    let rot_mat = Mat3::new(
        r00, r01, r02,
        r10, r11, r12,
        r20, r21, r22,
    );

    let inv_rot_mat = rot_mat.transpose();
    (rot_mat, inv_rot_mat, true)
}

fn compute_world_aabb(center: &Vec3, half_size: &Vec3, rot_mat: &Mat3, is_rotated: bool) -> (Vec3, Vec3) {
    if !is_rotated {
        return (center - half_size, center + half_size);
    }
    let rx = rot_mat.m11.abs() * half_size.x + rot_mat.m12.abs() * half_size.y + rot_mat.m13.abs() * half_size.z;
    let ry = rot_mat.m21.abs() * half_size.x + rot_mat.m22.abs() * half_size.y + rot_mat.m23.abs() * half_size.z;
    let rz = rot_mat.m31.abs() * half_size.x + rot_mat.m32.abs() * half_size.y + rot_mat.m33.abs() * half_size.z;
    let extent = Vec3::new(rx, ry, rz);
    (center - extent, center + extent)
}

#[allow(dead_code)]
impl Box3D {
    /// Crea una nueva caja 3D a partir de su centro y vector de dimensiones (ancho X, alto Y, profundidad Z).
    pub fn new(center: Vec3, size: Vec3, material: Material) -> Self {
        let half_size = size / 2.0;
        let rotation = Vec3::new(0.0, 0.0, 0.0);
        let (rot_mat, inv_rot_mat, is_rotated) = create_rotation_matrix(&rotation);
        let (min, max) = compute_world_aabb(&center, &half_size, &rot_mat, is_rotated);

        Box3D {
            center,
            size,
            half_size,
            rotation,
            rot_mat,
            inv_rot_mat,
            is_rotated,
            min,
            max,
            material,
        }
    }

    /// Crea una caja 3D con rotación especificada en radianes para los ejes X, Y y Z.
    pub fn new_rotated(center: Vec3, size: Vec3, rotation: Vec3, material: Material) -> Self {
        let mut b = Self::new(center, size, material);
        b.set_rotation(rotation);
        b
    }

    /// Crea una caja 3D a partir de sus esquinas mínima y máxima no rotadas.
    pub fn from_min_max(min: Vec3, max: Vec3, material: Material) -> Self {
        let size = max - min;
        let center = min + size / 2.0;
        Self::new(center, size, material)
    }

    /// Constructor de conveniencia para cubos uniformes.
    pub fn cube(center: Vec3, size: f32, material: Material) -> Self {
        Self::new(center, Vec3::new(size, size, size), material)
    }

    /// Asigna una rotación en radianes (rot_x, rot_y, rot_z) retornando la caja modificada (patrón Builder).
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.set_rotation(rotation);
        self
    }

    /// Asigna una rotación en el eje Y (giro horizontal) en radianes.
    pub fn with_rotation_y(self, angle_rad: f32) -> Self {
        self.with_rotation(Vec3::new(0.0, angle_rad, 0.0))
    }

    /// Actualiza la rotación y recalcula las matrices de transformación y el AABB.
    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.rotation = rotation;
        let (rot_mat, inv_rot_mat, is_rotated) = create_rotation_matrix(&self.rotation);
        self.rot_mat = rot_mat;
        self.inv_rot_mat = inv_rot_mat;
        self.is_rotated = is_rotated;
        let (min, max) = compute_world_aabb(&self.center, &self.half_size, &self.rot_mat, self.is_rotated);
        self.min = min;
        self.max = max;
    }

    /// Calcula las coordenadas de textura UV (u, v) en el rango [0.0, 1.0] para cualquier cara impactada en espacio local.
    pub fn get_uv_local(&self, local_point: &Vec3, local_normal: &Vec3) -> (f32, f32) {
        let half = self.half_size;
        let size = self.size;

        if local_normal.x.abs() > 0.5 {
            // Caras laterales (+X o -X): mapeamos el plano Z (ancho) e Y (alto)
            let u = if local_normal.x > 0.0 {
                (half.z - local_point.z) / size.z.max(1e-6)
            } else {
                (local_point.z + half.z) / size.z.max(1e-6)
            };
            let v = (half.y - local_point.y) / size.y.max(1e-6);
            (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        } else if local_normal.y.abs() > 0.5 {
            // Caras superior e inferior (+Y o -Y): mapeamos el plano X (ancho) y Z (profundidad)
            let u = (local_point.x + half.x) / size.x.max(1e-6);
            let v = if local_normal.y > 0.0 {
                (local_point.z + half.z) / size.z.max(1e-6)
            } else {
                (half.z - local_point.z) / size.z.max(1e-6)
            };
            (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        } else {
            // Caras frontal y trasera (+Z o -Z): mapeamos el plano X (ancho) e Y (alto)
            let u = if local_normal.z > 0.0 {
                (local_point.x + half.x) / size.x.max(1e-6)
            } else {
                (half.x - local_point.x) / size.x.max(1e-6)
            };
            let v = (half.y - local_point.y) / size.y.max(1e-6);
            (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        }
    }
}

impl RayIntersect for Box3D {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        // Transformar rayo al espacio local del objeto
        let (local_origin, local_dir) = if self.is_rotated {
            (
                self.inv_rot_mat * (ray_origin - self.center),
                self.inv_rot_mat * ray_direction,
            )
        } else {
            (
                ray_origin - self.center,
                *ray_direction,
            )
        };

        let inv_d = Vec3::new(
            1.0 / local_dir.x,
            1.0 / local_dir.y,
            1.0 / local_dir.z,
        );

        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        let mut normal_near = Vec3::new(0.0, 0.0, 0.0);
        let mut normal_far = Vec3::new(0.0, 0.0, 0.0);

        let half = self.half_size;

        // Eje X local [-half.x, half.x]
        let (t0x, t1x, nx) = if local_dir.x >= 0.0 {
            (
                (-half.x - local_origin.x) * inv_d.x,
                (half.x - local_origin.x) * inv_d.x,
                Vec3::new(-1.0, 0.0, 0.0),
            )
        } else {
            (
                (half.x - local_origin.x) * inv_d.x,
                (-half.x - local_origin.x) * inv_d.x,
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

        // Eje Y local [-half.y, half.y]
        let (t0y, t1y, ny) = if local_dir.y >= 0.0 {
            (
                (-half.y - local_origin.y) * inv_d.y,
                (half.y - local_origin.y) * inv_d.y,
                Vec3::new(0.0, -1.0, 0.0),
            )
        } else {
            (
                (half.y - local_origin.y) * inv_d.y,
                (-half.y - local_origin.y) * inv_d.y,
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

        // Eje Z local [-half.z, half.z]
        let (t0z, t1z, nz) = if local_dir.z >= 0.0 {
            (
                (-half.z - local_origin.z) * inv_d.z,
                (half.z - local_origin.z) * inv_d.z,
                Vec3::new(0.0, 0.0, -1.0),
            )
        } else {
            (
                (half.z - local_origin.z) * inv_d.z,
                (-half.z - local_origin.z) * inv_d.z,
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

        let (t, local_normal) = if t_min > 0.0 {
            (t_min, normal_near)
        } else if t_max > 0.0 {
            (t_max, normal_far)
        } else {
            return None;
        };

        let local_point = local_origin + local_dir * t;
        let (u, v) = self.get_uv_local(&local_point, &local_normal);

        // Soporte de transparencia Alpha Cutout: si el píxel de la textura es transparente, el rayo continúa
        let color = self.material.get_color(u, v);
        if self.material.has_alpha() && color.a < 128 {
            return None;
        }

        // Transformar punto y normal de vuelta a coordenadas del mundo
        let world_point = if self.is_rotated {
            self.center + self.rot_mat * local_point
        } else {
            self.center + local_point
        };

        let world_normal = if self.is_rotated {
            self.rot_mat * local_normal
        } else {
            local_normal
        };

        Some(self.material.to_intersect_with_color(world_point, world_normal, t, color))
    }

    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        let (local_origin, local_dir) = if self.is_rotated {
            (
                self.inv_rot_mat * (ray_origin - self.center),
                self.inv_rot_mat * ray_direction,
            )
        } else {
            (
                ray_origin - self.center,
                *ray_direction,
            )
        };

        let inv_dx = 1.0 / local_dir.x;
        let half = self.half_size;
        let (t0x, t1x) = if local_dir.x >= 0.0 {
            ((-half.x - local_origin.x) * inv_dx, (half.x - local_origin.x) * inv_dx)
        } else {
            ((half.x - local_origin.x) * inv_dx, (-half.x - local_origin.x) * inv_dx)
        };

        let inv_dy = 1.0 / local_dir.y;
        let (t0y, t1y) = if local_dir.y >= 0.0 {
            ((-half.y - local_origin.y) * inv_dy, (half.y - local_origin.y) * inv_dy)
        } else {
            ((half.y - local_origin.y) * inv_dy, (-half.y - local_origin.y) * inv_dy)
        };

        let t_min = t0x.max(t0y);
        let t_max = t1x.min(t1y);

        if t_min > t_max {
            return false;
        }

        let inv_dz = 1.0 / local_dir.z;
        let (t0z, t1z) = if local_dir.z >= 0.0 {
            ((-half.z - local_origin.z) * inv_dz, (half.z - local_origin.z) * inv_dz)
        } else {
            ((half.z - local_origin.z) * inv_dz, (-half.z - local_origin.z) * inv_dz)
        };

        let t_min = t_min.max(t0z);
        let t_max = t_max.min(t1z);

        if t_min > t_max {
            return false;
        }

        let t = if t_min > 0.0 { t_min } else { t_max };
        if t <= 0.0 || t >= max_distance {
            return false;
        }

        // Solo calcular normales y muestreo de textura si el material realmente contiene transparencia
        if self.material.has_alpha() {
            let local_point = local_origin + local_dir * t;
            let local_normal = if (local_point.x - half.x).abs() < 1e-3 {
                Vec3::new(1.0, 0.0, 0.0)
            } else if (local_point.x + half.x).abs() < 1e-3 {
                Vec3::new(-1.0, 0.0, 0.0)
            } else if (local_point.y - half.y).abs() < 1e-3 {
                Vec3::new(0.0, 1.0, 0.0)
            } else if (local_point.y + half.y).abs() < 1e-3 {
                Vec3::new(0.0, -1.0, 0.0)
            } else if (local_point.z - half.z).abs() < 1e-3 {
                Vec3::new(0.0, 0.0, 1.0)
            } else {
                Vec3::new(0.0, 0.0, -1.0)
            };

            let (u, v) = self.get_uv_local(&local_point, &local_normal);
            if self.material.get_color(u, v).a < 128 {
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
    use std::f32::consts::FRAC_PI_2;

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
    fn test_box_rotation_y_intersection() {
        let mat = crate::materials::diffuse(Color::new(255, 0, 0));
        // Caja de 2x2x2 rotada 90 grados (PI/2) en eje Y
        let b = Box3D::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(4.0, 2.0, 2.0), mat)
            .with_rotation_y(FRAC_PI_2);

        let ray_origin = Vec3::new(0.0, 0.0, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);

        let hit = b.ray_intersect(&ray_origin, &ray_dir).unwrap();
        // Con rotación de 90° en Y, la cara de ancho 4.0 ahora apunta a lo largo de Z (half = 2.0)
        assert!((hit.distance - 3.0).abs() < 1e-4);
        assert!((hit.point.z - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_box_uv_mapping() {
        let mat = crate::materials::diffuse(Color::new(255, 0, 0));
        // Caja: x: [-1, 1], y: [-2, 2], z: [-0.5, 0.5]
        let b = Box3D::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 4.0, 1.0), mat);

        // Impacto en el centro de la cara frontal (+Z en z=0.5, x=0, y=0)
        let hit_pt = Vec3::new(0.0, 0.0, 0.5);
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let (u, v) = b.get_uv_local(&hit_pt, &normal);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);
    }
}
