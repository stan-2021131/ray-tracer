#![allow(dead_code)]

use crate::color::Color;
use crate::ray_intersect::Material;
use crate::texture::Texture;
use std::sync::Arc;

/// Retorna un material difuso/mate genérico para un color dado.
pub fn diffuse(color: Color) -> Material {
    Material::new(color, 0.9, 0.1, 0.0, 0.0, 10.0, 1.0)
}

/// Plástico brillante con ligera reflectividad y brillo especular medio (estilo marfil).
pub fn ivory(color: Color) -> Material {
    Material::new(color, 0.6, 0.3, 0.1, 0.0, 50.0, 1.0)
}

/// Goma mate con baja especularidad y sin reflexión.
pub fn rubber(color: Color) -> Material {
    Material::new(color, 0.9, 0.1, 0.0, 0.0, 10.0, 1.0)
}

/// Material metálico con brillo especular alto y ligera reflexión.
pub fn cobalt(color: Color) -> Material {
    Material::new(color, 0.7, 0.4, 0.15, 0.0, 80.0, 1.0)
}

/// Mineral / gema pulida difusa con brillo medio (estilo jade).
pub fn jade(color: Color) -> Material {
    Material::new(color, 0.8, 0.25, 0.05, 0.0, 30.0, 1.0)
}

/// Espejo: material altamente reflectivo (85% reflexión especular indirecta) con tinte configurable.
pub fn mirror(tint: Color) -> Material {
    Material::new(tint, 0.1, 0.1, 0.85, 0.0, 125.0, 1.0)
}

/// Vidrio dieléctrico transparente con índice de refracción IOR = 1.52 y tinte configurable.
pub fn glass(tint: Color) -> Material {
    Material::new(tint, 0.0, 0.1, 0.05, 0.95, 125.0, 1.52)
}

/// Agua: material transparente con índice de refracción IOR = 1.333 y tinte configurable.
pub fn water(tint: Color) -> Material {
    Material::new(tint, 0.0, 0.1, 0.05, 0.95, 125.0, 1.333)
}

/// Metal pulido reflectivo (70% reflexión especular indirecta) con color metálico configurable (oro, bronce, cobre, etc.).
pub fn gold(color: Color) -> Material {
    Material::new(color, 0.2, 0.4, 0.7, 0.0, 100.0, 1.0)
}

/// Aplica una textura desde un archivo a cualquier material base existente.
pub fn apply_texture(base_material: Material, texture_path: &str) -> Material {
    let texture = Arc::new(Texture::new(texture_path));
    base_material.with_texture(texture)
}
