use nalgebra_glm::Vec3;
use std::f32::consts::PI;

const PITCH_LIMIT: f32 = PI / 2.0 - 0.1;

pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    forward: Vec3,
    right: Vec3,
    cam_up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let mut camera = Camera {
            eye,
            center,
            up,
            forward: Vec3::zeros(),
            right: Vec3::zeros(),
            cam_up: Vec3::zeros(),
        };
        camera.update_basis();
        camera
    }

    /// Actualiza la base ortonormal cuando la posición o el objetivo de la cámara cambian.
    pub fn update_basis(&mut self) {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();
        let cam_up = right.cross(&forward).normalize();

        self.forward = forward;
        self.right = right;
        self.cam_up = cam_up;
    }

    /// Transforma una dirección desde el espacio de cámara al espacio de mundo.
    #[inline]
    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        vector.x * self.right + vector.y * self.cam_up - vector.z * self.forward
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        let current_yaw = radius_vector.z.atan2(radius_vector.x);
        let radius_xz =
            (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let new_pitch = (current_pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        self.eye = self.center
            + Vec3::new(
                radius * new_yaw.cos() * new_pitch.cos(),
                -radius * new_pitch.sin(),
                radius * new_yaw.sin() * new_pitch.cos(),
            );
        self.update_basis();
    }
}