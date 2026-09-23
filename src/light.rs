use crate::color::Color;
use nalgebra_glm::Vec3;

/// Tipo de comportamiento de atenuación para una fuente de luz.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LightType {
    /// Luz direccional/global sin atenuación por distancia (ideal para Sol, Luna y ambiente general).
    #[default]
    Directional,
    /// Luz puntual local con atenuación cuadrática por distancia (ideal para lámparas, antorchas, fogatas).
    Point,
}

#[derive(Debug, Clone)]
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub light_type: LightType,
}

#[allow(dead_code)]
impl Light {
    /// Crea una luz con comportamiento global/direccional por defecto (mantiene retrocompatibilidad).
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            light_type: LightType::Directional,
        }
    }

    /// Crea una luz puntual local con atenuación por distancia (para lámparas, bombillas o fuego).
    pub fn point(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            light_type: LightType::Point,
        }
    }

    /// Crea una luz global sin atenuación por distancia (para Sol, Luna o ambiente).
    pub fn directional(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            light_type: LightType::Directional,
        }
    }
}