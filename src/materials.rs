#![allow(dead_code)]

use crate::color::Color;
use crate::ray_intersect::Material;
use crate::texture::Texture;
use std::sync::Arc;

/// Retorna un material difuso/mate genérico para un color dado.
pub fn diffuse(color: Color) -> Material {
    Material::new(color, 0.9, 0.1, 0.0, 0.0, 10.0, 1.0)
}

/// Marfil: plástico brillante con tono crema y ligera reflectividad.
pub fn ivory() -> Material {
    Material::new(Color::new(100, 100, 80), 0.6, 0.3, 0.1, 0.0, 50.0, 1.0)
}

/// Goma roja: material mate con baja especularidad y sin reflexión.
pub fn rubber() -> Material {
    Material::new(Color::new(80, 0, 0), 0.9, 0.1, 0.0, 0.0, 10.0, 1.0)
}

/// Cobalto: material metálico azul con brillo especular alto.
pub fn cobalt() -> Material {
    Material::new(Color::new(40, 80, 140), 0.7, 0.4, 0.15, 0.0, 80.0, 1.0)
}

/// Jade: material verde difuso con brillo medio.
pub fn jade() -> Material {
    Material::new(Color::new(60, 130, 100), 0.8, 0.25, 0.05, 0.0, 30.0, 1.0)
}

/// Espejo: material altamente reflectivo (85% reflexión especular indirecta).
pub fn mirror() -> Material {
    Material::new(Color::new(240, 240, 255), 0.1, 0.1, 0.85, 0.0, 125.0, 1.0)
}

/// Vidrio común: dieléctrico transparente con índice de refracción IOR = 1.52.
pub fn glass() -> Material {
    Material::new(Color::new(255, 255, 255), 0.0, 0.1, 0.05, 0.95, 125.0, 1.52)
}

/// Agua: material transparente con índice de refracción IOR = 1.333.
pub fn water() -> Material {
    Material::new(Color::new(240, 250, 255), 0.0, 0.1, 0.05, 0.95, 125.0, 1.333)
}

/// Oro: metal reflectivo con tonalidad dorada y brillo especular pulido.
pub fn gold() -> Material {
    Material::new(Color::new(255, 215, 0), 0.2, 0.4, 0.7, 0.0, 100.0, 1.0)
}

/// Material compuesto para bloques: base marfil con textura aplicada.
pub fn textured_wall(texture_path: &str) -> Material {
    let texture = Arc::new(Texture::new(texture_path));
    ivory().with_texture(texture)
}
