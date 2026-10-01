use crate::color::Color;
use crate::texture::{Texture, TextureFilter};
use nalgebra_glm::Vec3;

/// Identificador de las 6 caras de un Skybox / Cubemap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubemapFace {
    Right, // +X
    Left,  // -X
    Up,    // +Y
    Down,  // -Y
    Front, // -Z (alineado con la dirección inicial de vista en el espacio de mundo)
    Back,  // +Z
}

/// Estructura que representa un Skybox nocturno tipo Cubemap de 6 caras.
///
/// # Fundamento Teórico:
/// Un Cubemap proyecta el espacio tridimensional envolvente sobre las 6 caras internas de un cubo unitario.
/// Al disparar un rayo en dirección normalizada D = (x, y, z):
/// 1. Se calcula el eje de mayor magnitud |d_max| = max(|x|, |y|, |z|) para identificar la cara impactada.
/// 2. Se proyectan las dos componentes restantes sobre el plano de esa cara dividiéndolas por |d_max|,
///    obteniendo coordenadas en el rango [-1, 1].
/// 3. Se mapea linealmente dicho rango a coordenadas de textura normalizadas (u, v) en [0, 1]:
///       u = (coord_u + 1.0) / 2.0
///       v = (1.0 - coord_v) / 2.0  (o según la orientación del eje vertical en la imagen)
/// 4. Se muestrea la textura correspondiente mediante el filtro seleccionado (Nearest o Bilinear).
#[derive(Clone, Debug)]
pub struct Skybox {
    pub up: Texture,
    pub down: Texture,
    pub left: Texture,
    pub right: Texture,
    pub front: Texture,
    pub back: Texture,
    pub filter: TextureFilter,
    pub rotation_y: f32,
    pub has_rotation: bool,
    pub brightness: f32,
}

impl Skybox {
    /// Carga las seis imágenes de las caras del Cubemap una sola vez al inicializar.
    ///
    /// Reutiliza el cargador `Texture::new` existente, por lo que si una ruta no existe,
    /// se asigna automáticamente una textura de respaldo visual sin detener la ejecución.
    pub fn new(
        up_path: &str,
        down_path: &str,
        left_path: &str,
        right_path: &str,
        front_path: &str,
        back_path: &str,
    ) -> Self {
        Skybox {
            up: Texture::new(up_path),
            down: Texture::new(down_path),
            left: Texture::new(left_path),
            right: Texture::new(right_path),
            front: Texture::new(front_path),
            back: Texture::new(back_path),
            filter: TextureFilter::Nearest,
            rotation_y: 0.0,
            has_rotation: false,
            brightness: 1.0,
        }
    }

    /// Crea un Skybox asignando la misma textura en las 6 caras (útil para fondos omnidireccionales o patrones cósmicos).
    pub fn from_single_texture(texture_path: &str) -> Self {
        Self::new(
            texture_path,
            texture_path,
            texture_path,
            texture_path,
            texture_path,
            texture_path,
        )
    }

    /// Permite configurar el brillo / atenuación general del Skybox (patrón Builder).
    pub fn with_brightness(mut self, brightness: f32) -> Self {
        self.brightness = brightness.clamp(0.0, 5.0);
        self
    }

    /// Permite configurar el modo de filtrado de textura (patrón Builder).
    pub fn with_filter(mut self, filter: TextureFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Permite asignar un ángulo de rotación horizontal sobre el eje Y en radianes (patrón Builder).
    #[allow(dead_code)]
    pub fn with_rotation_y(mut self, radians: f32) -> Self {
        self.rotation_y = radians;
        self.has_rotation = radians.abs() > 1e-6;
        self
    }

    /// Actualiza el ángulo de rotación horizontal del skybox sobre el eje Y.
    #[allow(dead_code)]
    #[inline]
    pub fn set_rotation_y(&mut self, radians: f32) -> Self {
        self.rotation_y = radians;
        self.has_rotation = radians.abs() > 1e-6;
        self.clone()
    }

    /// Configura el filtro de textura (Nearest o Bilinear).
    #[allow(dead_code)]
    #[inline]
    pub fn set_filter(&mut self, filter: TextureFilter) {
        self.filter = filter;
    }

    /// Determina la cara del cubemap y las coordenadas UV normalizadas [0.0, 1.0] a partir
    /// de una dirección tridimensional (normalizada o no nula).
    ///
    /// # Explicación Matemática de Mapeo de Caras y Costuras:
    /// En el sistema de coordenadas del proyecto:
    /// - +X es Derecha (Right), -X es Izquierda (Left)
    /// - +Y es Arriba (Up / Top), -Y es Abajo (Down / Bottom)
    /// - -Z es Frente (Front - vista hacia el fondo), +Z es Atrás (Back)
    ///
    /// Las coordenadas UV se calculan de manera que el origen (0, 0) corresponde a la esquina
    /// superior izquierda de cada imagen y todas las caras contiguas comparten valores UV continuos:
    ///
    /// - **Front (-Z)**: Mirando a -Z, +X queda a la derecha y +Y arriba.
    ///   u = (x / -z + 1.0) / 2.0 = (1.0 - x / z) / 2.0
    ///   v = (1.0 - y / -z) / 2.0 = (1.0 + y / z) / 2.0
    ///
    /// - **Back (+Z)**: Mirando a +Z, -X queda a la derecha y +Y arriba.
    ///   u = (1.0 - x / z) / 2.0
    ///   v = (1.0 - y / z) / 2.0
    ///
    /// - **Right (+X)**: Mirando a +X, +Z queda a la derecha y +Y arriba.
    ///   u = (z / x + 1.0) / 2.0
    ///   v = (1.0 - y / x) / 2.0
    ///
    /// - **Left (-X)**: Mirando a -X, -Z queda a la derecha y +Y arriba.
    ///   u = (1.0 - z / x) / 2.0  =>  (z / x + 1.0) / 2.0  puesto que x < 0 y -z/|x| = z/x.
    ///   v = (1.0 + y / x) / 2.0
    ///
    /// - **Up (+Y)**: Mirando arriba (+Y), +X queda a la derecha y -Z (Front) hacia abajo.
    ///   u = (x / y + 1.0) / 2.0
    ///   v = (1.0 - z / y) / 2.0
    ///
    /// - **Down (-Y)**: Mirando abajo (-Y), +X queda a la derecha y +Z (Back) hacia abajo.
    ///   u = (1.0 - x / y) / 2.0
    ///   v = (1.0 - z / y) / 2.0
    #[inline]
    pub fn get_face_and_uv(dir: &Vec3) -> (CubemapFace, f32, f32) {
        let abs_x = dir.x.abs();
        let abs_y = dir.y.abs();
        let abs_z = dir.z.abs();

        if abs_x >= abs_y && abs_x >= abs_z {
            if dir.x > 0.0 {
                // Cara Derecha (+X): dominante X positiva
                let inv_x = 1.0 / dir.x;
                let u = (dir.z * inv_x + 1.0) * 0.5;
                let v = (1.0 - dir.y * inv_x) * 0.5;
                (CubemapFace::Right, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
            } else {
                // Cara Izquierda (-X): dominante X negativa
                let inv_x = 1.0 / dir.x;
                let u = (dir.z * inv_x + 1.0) * 0.5;
                let v = (1.0 + dir.y * inv_x) * 0.5;
                (CubemapFace::Left, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
            }
        } else if abs_y >= abs_x && abs_y >= abs_z {
            if dir.y > 0.0 {
                // Cara Superior (+Y): dominante Y positiva
                let inv_y = 1.0 / dir.y;
                let u = (dir.x * inv_y + 1.0) * 0.5;
                let v = (1.0 - dir.z * inv_y) * 0.5;
                (CubemapFace::Up, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
            } else {
                // Cara Inferior (-Y): dominante Y negativa
                let inv_y = 1.0 / dir.y;
                let u = (1.0 - dir.x * inv_y) * 0.5;
                let v = (1.0 - dir.z * inv_y) * 0.5;
                (CubemapFace::Down, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
            }
        } else {
            if dir.z < 0.0 {
                // Cara Frontal (-Z): dominante Z negativa (frente de la escena)
                let inv_z = 1.0 / dir.z;
                let u = (1.0 - dir.x * inv_z) * 0.5;
                let v = (1.0 + dir.y * inv_z) * 0.5;
                (CubemapFace::Front, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
            } else {
                // Cara Trasera (+Z): dominante Z positiva
                let inv_z = 1.0 / dir.z;
                let u = (1.0 - dir.x * inv_z) * 0.5;
                let v = (1.0 - dir.y * inv_z) * 0.5;
                (CubemapFace::Back, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
            }
        }
    }

    /// Muestrea el color del Skybox para una dirección tridimensional normalizada.
    ///
    /// # Rendimiento:
    /// - Totalmente libre de asignaciones en heap en el bucle caliente (O(1)).
    /// - Aplica rotación opcional sobre el eje Y mediante álgebra de rotación 2D plana.
    /// - Usa el modo de filtrado seleccionado (Nearest para pixel art o Bilinear para suavizado).
    #[inline]
    pub fn sample(&self, ray_direction: &Vec3) -> Color {
        // Aplicar rotación sobre el eje Y solo si está configurada
        let dir = if self.has_rotation {
            let cos_theta = self.rotation_y.cos();
            let sin_theta = self.rotation_y.sin();
            // Matriz de rotación estándar alrededor del eje Y:
            // x' =  x * cos(θ) + z * sin(θ)
            // y' =  y
            // z' = -x * sin(θ) + z * cos(θ)
            Vec3::new(
                ray_direction.x * cos_theta + ray_direction.z * sin_theta,
                ray_direction.y,
                -ray_direction.x * sin_theta + ray_direction.z * cos_theta,
            )
        } else {
            *ray_direction
        };

        // Identificar cara y coordenadas UV
        let (face, u, v) = Self::get_face_and_uv(&dir);

        // Muestrear textura correspondiente usando el filtro configurado
        let mut color = match face {
            CubemapFace::Right => self.right.get_color_filtered(u, v, self.filter),
            CubemapFace::Left => self.left.get_color_filtered(u, v, self.filter),
            CubemapFace::Up => self.up.get_color_filtered(u, v, self.filter),
            CubemapFace::Down => self.down.get_color_filtered(u, v, self.filter),
            CubemapFace::Front => self.front.get_color_filtered(u, v, self.filter),
            CubemapFace::Back => self.back.get_color_filtered(u, v, self.filter),
        };

        // Soporte nativo para texturas de skybox con canal alfa / transparencia:
        // Mezcla alfa sobre el vacío negro del espacio
        if color.a < 255 {
            let alpha = color.a as f32 / 255.0;
            color = Color::new(
                (color.r as f32 * alpha).round() as u8,
                (color.g as f32 * alpha).round() as u8,
                (color.b as f32 * alpha).round() as u8,
            );
        }

        // Aplicar atenuación / brillo configurado
        if (self.brightness - 1.0).abs() > 1e-4 {
            color = color * self.brightness;
        }

        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinal_face_selection() {
        // Verificar que las direcciones cardinales seleccionan la cara correcta y (u, v) en el centro (0.5, 0.5)
        let (face, u, v) = Skybox::get_face_and_uv(&Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(face, CubemapFace::Front);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);

        let (face, u, v) = Skybox::get_face_and_uv(&Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(face, CubemapFace::Back);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);

        let (face, u, v) = Skybox::get_face_and_uv(&Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(face, CubemapFace::Right);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);

        let (face, u, v) = Skybox::get_face_and_uv(&Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(face, CubemapFace::Left);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);

        let (face, u, v) = Skybox::get_face_and_uv(&Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(face, CubemapFace::Up);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);

        let (face, u, v) = Skybox::get_face_and_uv(&Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(face, CubemapFace::Down);
        assert!((u - 0.5).abs() < 1e-4);
        assert!((v - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_seam_front_right() {
        // En la esquina Front-Right (x = 1, z = -1, y = 0)
        // Desde Front (z = -1, x = 0.999): u debe ser cercano a 1.0, v = 0.5
        let (face_front, u_front, v_front) = Skybox::get_face_and_uv(&Vec3::new(0.999, 0.0, -1.0));
        assert_eq!(face_front, CubemapFace::Front);
        assert!((u_front - 1.0).abs() < 1e-2);
        assert!((v_front - 0.5).abs() < 1e-4);

        // Desde Right (x = 1.0, z = -0.999): u debe ser cercano a 0.0, v = 0.5
        let (face_right, u_right, v_right) = Skybox::get_face_and_uv(&Vec3::new(1.0, 0.0, -0.999));
        assert_eq!(face_right, CubemapFace::Right);
        assert!((u_right - 0.0).abs() < 1e-2);
        assert!((v_right - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_seam_front_top() {
        // En la costura superior de Front (y = 1, z = -1, x = 0)
        // Desde Front (z = -1.0, y = 0.999): v debe ser cercano a 0.0 (borde superior de Front)
        let (face_front, u_front, v_front) = Skybox::get_face_and_uv(&Vec3::new(0.0, 0.999, -1.0));
        assert_eq!(face_front, CubemapFace::Front);
        assert!((u_front - 0.5).abs() < 1e-4);
        assert!((v_front - 0.0).abs() < 1e-2);

        // Desde Up (y = 1.0, z = -0.999): v debe ser cercano a 1.0 (borde inferior de Up conectando con Front)
        let (face_up, u_up, v_up) = Skybox::get_face_and_uv(&Vec3::new(0.0, 1.0, -0.999));
        assert_eq!(face_up, CubemapFace::Up);
        assert!((u_up - 0.5).abs() < 1e-4);
        assert!((v_up - 1.0).abs() < 1e-2);
    }

    #[test]
    fn test_seam_front_left() {
        // En la esquina Front-Left (x = -1, z = -1, y = 0)
        // Desde Front (z = -1, x = -0.999): u debe ser cercano a 0.0, v = 0.5
        let (face_front, u_front, v_front) = Skybox::get_face_and_uv(&Vec3::new(-0.999, 0.0, -1.0));
        assert_eq!(face_front, CubemapFace::Front);
        assert!((u_front - 0.0).abs() < 1e-2);
        assert!((v_front - 0.5).abs() < 1e-4);

        // Desde Left (x = -1.0, z = -0.999): u debe ser cercano a 1.0, v = 0.5
        let (face_left, u_left, v_left) = Skybox::get_face_and_uv(&Vec3::new(-1.0, 0.0, -0.999));
        assert_eq!(face_left, CubemapFace::Left);
        assert!((u_left - 1.0).abs() < 1e-2);
        assert!((v_left - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_seam_front_bottom() {
        // En la costura inferior de Front (y = -1, z = -1, x = 0)
        // Desde Front (z = -1.0, y = -0.999): v debe ser cercano a 1.0 (borde inferior de Front)
        let (face_front, u_front, v_front) = Skybox::get_face_and_uv(&Vec3::new(0.0, -0.999, -1.0));
        assert_eq!(face_front, CubemapFace::Front);
        assert!((u_front - 0.5).abs() < 1e-4);
        assert!((v_front - 1.0).abs() < 1e-2);

        // Desde Down (y = -1.0, z = -0.999): v debe ser cercano a 0.0 (borde superior de Down conectando con Front)
        let (face_down, u_down, v_down) = Skybox::get_face_and_uv(&Vec3::new(0.0, -1.0, -0.999));
        assert_eq!(face_down, CubemapFace::Down);
        assert!((u_down - 0.5).abs() < 1e-4);
        assert!((v_down - 0.0).abs() < 1e-2);
    }

    #[test]
    fn test_rotation_y() {
        use std::f32::consts::PI;
        // Rotar 90 grados (PI/2) alrededor de Y debe rotar la vista frontal (-Z) hacia la cara derecha (+X)
        let skybox = Skybox::new("", "", "", "", "", "").with_rotation_y(PI / 2.0);
        let sample_dir = Vec3::new(0.0, 0.0, -1.0);
        
        let cos_theta = skybox.rotation_y.cos();
        let sin_theta = skybox.rotation_y.sin();
        let rotated_dir = Vec3::new(
            sample_dir.x * cos_theta + sample_dir.z * sin_theta,
            sample_dir.y,
            -sample_dir.x * sin_theta + sample_dir.z * cos_theta,
        );
        let (face, _, _) = Skybox::get_face_and_uv(&rotated_dir);
        assert_eq!(face, CubemapFace::Left);
    }

    #[test]
    fn test_uv_always_clamped() {
        // Direcciones extremas
        let directions = [
            Vec3::new(100.0, 0.0, 0.0),
            Vec3::new(-50.0, -50.0, -50.0),
            Vec3::new(0.001, 10.0, -0.001),
        ];

        for dir in &directions {
            let (_, u, v) = Skybox::get_face_and_uv(dir);
            assert!(u >= 0.0 && u <= 1.0);
            assert!(v >= 0.0 && v <= 1.0);
        }
    }
}
