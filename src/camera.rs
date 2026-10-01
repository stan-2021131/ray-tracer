use crate::ray_intersect::Object;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

const PITCH_LIMIT: f32 = PI / 2.0 - 0.1;

/// Modos de cámara disponibles en la aplicación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    /// Modo orbital: rota alrededor de un punto central de la escena.
    #[default]
    Orbit,
    /// Modo primera persona con colisiones: desplazamiento en suelo con deslizamiento por paredes.
    FpsCollision,
    /// Modo vuelo libre (FreeCam / Noclip): desplazamiento 3D sin restricciones físicas.
    FreeCam,
    /// Modo telescopio: vista óptica fija con rotación horizontal y elevación hacia el cielo.
    Telescope,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub mode: CameraMode,
    pub collision_radius: f32,
    pub eye_height: f32,
    pub forward: Vec3,
    pub right: Vec3,
    pub cam_up: Vec3,
}

impl Camera {
    /// Inicializa una cámara en modo orbital por defecto calculando ángulos yaw y pitch iniciales.
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let dir = (center - eye).normalize();
        let current_yaw = dir.x.atan2(-dir.z);
        let radius_xz = (dir.x * dir.x + dir.z * dir.z).sqrt();
        let current_pitch = dir.y.atan2(radius_xz).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        let mut camera = Camera {
            eye,
            center,
            up,
            yaw: current_yaw,
            pitch: current_pitch,
            mode: CameraMode::Orbit,
            collision_radius: 0.35,
            eye_height: 1.5,
            forward: Vec3::zeros(),
            right: Vec3::zeros(),
            cam_up: Vec3::zeros(),
        };
        camera.update_basis();
        camera
    }

    /// Alterna cíclicamente el modo de cámara (Orbit -> FpsCollision -> FreeCam -> Orbit).
    pub fn toggle_mode(&mut self) -> CameraMode {
        self.mode = match self.mode {
            CameraMode::Orbit => {
                // Al entrar a FPS, derivamos yaw y pitch desde la dirección de vista actual hacia center
                let dir = (self.center - self.eye).normalize();
                self.yaw = dir.x.atan2(-dir.z);
                let radius_xz = (dir.x * dir.x + dir.z * dir.z).sqrt();
                self.pitch = dir.y.atan2(radius_xz).clamp(-PITCH_LIMIT, PITCH_LIMIT);
                // Colocar la cámara a la altura de los ojos de un personaje sobre el suelo (suelo a Y = 1.0)
                self.eye.y = 1.0 + self.eye_height;
                CameraMode::FpsCollision
            }
            CameraMode::FpsCollision => CameraMode::FreeCam,
            CameraMode::FreeCam | CameraMode::Telescope => {
                // Al volver a Orbit, colocamos center a una distancia fija hacia adelante
                self.center = self.eye + self.forward * 4.0;
                CameraMode::Orbit
            }
        };
        self.update_basis();
        self.mode
    }

    /// Restablece la cámara a los parámetros iniciales en modo orbital.
    pub fn reset(&mut self, eye: Vec3, center: Vec3, up: Vec3) {
        self.eye = eye;
        self.center = center;
        self.up = up;
        self.mode = CameraMode::Orbit;
        let dir = (center - eye).normalize();
        self.yaw = dir.x.atan2(-dir.z);
        let radius_xz = (dir.x * dir.x + dir.z * dir.z).sqrt();
        self.pitch = dir.y.atan2(radius_xz).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        self.update_basis();
    }

    /// Actualiza la base ortonormal (forward, right, cam_up) según el modo activo.
    pub fn update_basis(&mut self) {
        match self.mode {
            CameraMode::Orbit => {
                let forward = (self.center - self.eye).normalize();
                let right = forward.cross(&self.up).normalize();
                let cam_up = right.cross(&forward).normalize();

                self.forward = forward;
                self.right = right;
                self.cam_up = cam_up;
            }
            CameraMode::FpsCollision | CameraMode::FreeCam | CameraMode::Telescope => {
                let cos_pitch = self.pitch.cos();
                let forward = Vec3::new(
                    self.yaw.sin() * cos_pitch,
                    self.pitch.sin(),
                    -self.yaw.cos() * cos_pitch,
                )
                .normalize();

                let right = forward.cross(&self.up).normalize();
                let cam_up = right.cross(&forward).normalize();

                self.forward = forward;
                self.right = right;
                self.cam_up = cam_up;
            }
        }
    }

    /// Transforma una dirección desde el espacio de cámara al espacio de mundo.
    #[inline]
    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        vector.x * self.right + vector.y * self.cam_up - vector.z * self.forward
    }

    /// Rotación orbital alrededor del objetivo (modo Orbit).
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius = (self.eye - self.center).magnitude();

        self.yaw = (self.yaw + delta_yaw) % (2.0 * PI);
        self.pitch = (self.pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        let cos_pitch = self.pitch.cos();
        self.eye = self.center
            + Vec3::new(
                -radius * self.yaw.sin() * cos_pitch,
                -radius * self.pitch.sin(),
                radius * self.yaw.cos() * cos_pitch,
            );
        self.update_basis();
    }

    /// Rota la orientación de la vista (cabeceo y giro de cabeza).
    /// En modo telescopio restringe el pitch hacia el cielo (-0.05 a 1.45 rad).
    pub fn rotate_look(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw = (self.yaw + delta_yaw) % (2.0 * PI);
        let limit = if self.mode == CameraMode::Telescope {
            (-0.05, 1.45)
        } else {
            (-PITCH_LIMIT, PITCH_LIMIT)
        };
        self.pitch = (self.pitch + delta_pitch).clamp(limit.0, limit.1);
        self.update_basis();
    }

    /// Rota la orientación en modo telescopio: yaw libre 360°, pitch restringido hacia el cielo (-0.05 a 1.45 rad).
    #[allow(dead_code)]
    pub fn rotate_look_telescope(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.rotate_look(delta_yaw, delta_pitch);
    }

    /// Comprueba si una posición espacial 3D colisiona con algún objeto sólido de la escena.
    pub fn check_collision(&self, pos: &Vec3, objects: &[Object]) -> bool {
        let r = self.collision_radius;

        for obj in objects {
            match obj {
                Object::Sphere(s) => {
                    let dist = (pos - s.center).magnitude();
                    if dist < (s.radius + r) {
                        return true;
                    }
                }
                Object::Box3D(b) => {
                    // Verificación OBB en espacio local de la caja expandida por el radio del jugador
                    let local_pos = if b.is_rotated {
                        b.inv_rot_mat * (pos - b.center)
                    } else {
                        pos - b.center
                    };
                    if local_pos.x.abs() <= (b.half_size.x + r)
                        && local_pos.y.abs() <= (b.half_size.y + r)
                        && local_pos.z.abs() <= (b.half_size.z + r)
                    {
                        return true;
                    }
                }
                Object::Plane(p) => {
                    // Evitar caer por debajo del plano de suelo si es horizontal
                    if p.normal.y > 0.9 && pos.y < (p.center.y + self.eye_height * 0.5) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Procesa el desplazamiento del jugador según el modo activo.
    ///
    /// En `FpsCollision`, aplica el patrón de separación de ejes (deslizamiento por paredes) de `player.rs`.
    /// En `FreeCam`, aplica movimiento 3D libre sin colisiones.
    pub fn move_player(
        &mut self,
        forward_input: f32,
        strafe_input: f32,
        up_input: f32,
        speed: f32,
        objects: &[Object],
    ) {
        match self.mode {
            CameraMode::Orbit => {
                // En modo órbita, W/S puede acercar o alejar el zoom hacia el centro
                if forward_input.abs() > 0.0 {
                    let to_center = (self.center - self.eye).normalize();
                    let new_eye = self.eye + to_center * (forward_input * speed);
                    if (new_eye - self.center).magnitude() > 0.5 {
                        self.eye = new_eye;
                        self.update_basis();
                    }
                }
            }
            CameraMode::FpsCollision => {
                // Movimiento horizontal en el plano XZ derivado del vector forward y right
                let flat_forward = Vec3::new(self.forward.x, 0.0, self.forward.z).normalize();
                let flat_right = Vec3::new(self.right.x, 0.0, self.right.z).normalize();

                let move_vec = (flat_forward * forward_input + flat_right * strafe_input) * speed;

                // 1. Probar y aplicar movimiento en eje X por separado (permite deslizamiento)
                let test_x = Vec3::new(self.eye.x + move_vec.x, self.eye.y, self.eye.z);
                if !self.check_collision(&test_x, objects) {
                    self.eye.x = test_x.x;
                }

                // 2. Probar y aplicar movimiento en eje Z por separado (permite deslizamiento)
                let test_z = Vec3::new(self.eye.x, self.eye.y, self.eye.z + move_vec.z);
                if !self.check_collision(&test_z, objects) {
                    self.eye.z = test_z.z;
                }

                self.update_basis();
            }
            CameraMode::FreeCam => {
                // Movimiento libre en 3D (dirección completa de vista + vector lateral + eje vertical Y)
                let move_vec = self.forward * (forward_input * speed)
                    + self.right * (strafe_input * speed)
                    + Vec3::new(0.0, up_input * speed, 0.0);

                self.eye += move_vec;
                self.update_basis();
            }
            CameraMode::Telescope => {
                // En modo telescopio, la posición del observador es fija (sin traslación)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box3d::Box3D;
    use crate::materials;
    use crate::sphere::Sphere;

    #[test]
    fn test_camera_mode_toggle() {
        let mut camera = Camera::new(
            Vec3::new(0.0, 2.0, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        assert_eq!(camera.mode, CameraMode::Orbit);

        let m1 = camera.toggle_mode();
        assert_eq!(m1, CameraMode::FpsCollision);

        let m2 = camera.toggle_mode();
        assert_eq!(m2, CameraMode::FreeCam);

        let m3 = camera.toggle_mode();
        assert_eq!(m3, CameraMode::Orbit);
    }

    #[test]
    fn test_collision_sphere() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 2.0), 1.0, materials::diffuse(crate::color::Color::new(255, 0, 0)));
        let objects = vec![Object::Sphere(sphere)];

        // Posición dentro del radio de colisión (distancia 0.8 < 1.0 + 0.35)
        assert!(camera.check_collision(&Vec3::new(0.0, 0.0, 1.2), &objects));

        // Posición lejana
        assert!(!camera.check_collision(&Vec3::new(0.0, 0.0, 5.0), &objects));
    }

    #[test]
    fn test_collision_box() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let b = Box3D::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 3.0, 1.5), materials::diffuse(crate::color::Color::new(255, 0, 0)));
        let objects = vec![Object::Box3D(b)];

        // Punto dentro del AABB expandido
        assert!(camera.check_collision(&Vec3::new(1.1, 0.0, 0.0), &objects));

        // Punto fuera del AABB
        assert!(!camera.check_collision(&Vec3::new(3.0, 0.0, 0.0), &objects));
    }

    #[test]
    fn test_orbit_continuity() {
        let eye_init = Vec3::new(0.0, 15.0, 18.0);
        let center_init = Vec3::new(0.0, 0.0, 0.0);
        let mut camera = Camera::new(
            eye_init,
            center_init,
            Vec3::new(0.0, 1.0, 0.0),
        );

        // Sin movimiento delta, la posición calculada debe ser idéntica a eye inicial
        camera.orbit(0.0, 0.0);
        assert!((camera.eye.x - eye_init.x).abs() < 1e-4);
        assert!((camera.eye.y - eye_init.y).abs() < 1e-4);
        assert!((camera.eye.z - eye_init.z).abs() < 1e-4);

        // Con un movimiento delta pequeño, la cámara debe desplazarse suavemente sin saltar a otro cuadrante
        camera.orbit(0.01, 0.0);
        assert!((camera.eye.x - (-0.234)).abs() < 0.1);
        assert!((camera.eye.y - 15.0).abs() < 0.1);
        assert!((camera.eye.z - 18.0).abs() < 0.1);
    }
}