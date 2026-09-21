use crate::color::Color;
use crate::cube::Cube;
use crate::plane::Plane;
use crate::sphere::Sphere;
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

/// Propiedades ópticas y físicas de una superficie en el trazado de rayos.
#[derive(Debug, Clone)]
pub struct Material {
    pub color: Color,
    pub diffuse: f32,           // Peso de reflexión difusa (Lambertiana)
    pub specular: f32,          // Peso de brillo especular directo (Phong)
    pub reflective: f32,        // Peso de reflexión especular indirecta (Espejo)
    pub refractive: f32,        // Peso de refracción / transmisión (Transparencia)
    pub shininess: f32,         // Exponente de brillo Phong
    pub refractive_index: f32,  // Índice de refracción (IOR)
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(
        color: Color,
        diffuse: f32,
        specular: f32,
        reflective: f32,
        refractive: f32,
        shininess: f32,
        refractive_index: f32,
    ) -> Self {
        Material {
            color,
            diffuse,
            specular,
            reflective,
            refractive,
            shininess,
            refractive_index,
            texture: None,
        }
    }

    /// Asigna una textura a este material retornando el material modificado (patrón Builder).
    pub fn with_texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    /// Retorna el color modulando el color base del material con la textura si existe.
    #[inline]
    pub fn get_color(&self, u: f32, v: f32) -> Color {
        if let Some(tex) = &self.texture {
            self.color * tex.get_color(u, v)
        } else {
            self.color
        }
    }

    /// Convierte los datos del material en una estructura `Intersect` para el impacto de un rayo.
    #[inline]
    pub fn to_intersect(&self, point: Vec3, normal: Vec3, distance: f32, u: f32, v: f32) -> Intersect {
        Intersect {
            point,
            normal,
            distance,
            color: self.get_color(u, v),
            diffuse: self.diffuse,
            specular: self.specular,
            reflective: self.reflective,
            refractive: self.refractive,
            shininess: self.shininess,
            refractive_index: self.refractive_index,
        }
    }
}

/// Información del punto de impacto de un rayo sobre una superficie.
#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub color: Color,
    pub diffuse: f32,
    pub specular: f32,
    pub reflective: f32,
    pub refractive: f32,
    pub shininess: f32,
    pub refractive_index: f32,
}

pub trait RayIntersect: Sync + Send {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;

    /// Comprueba rápidamente si un rayo intersecta el objeto dentro de una distancia máxima.
    /// Útil para cálculo rápido de sombras sin necesidad de calcular normales ni texturas.
    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        self.ray_intersect(ray_origin, ray_direction)
            .is_some_and(|hit| hit.distance < max_distance)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Object {
    Cube(Cube),
    Sphere(Sphere),
    Plane(Plane),
}

impl RayIntersect for Object {
    #[inline]
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        match self {
            Object::Cube(c) => c.ray_intersect(ray_origin, ray_direction),
            Object::Sphere(s) => s.ray_intersect(ray_origin, ray_direction),
            Object::Plane(p) => p.ray_intersect(ray_origin, ray_direction),
        }
    }

    #[inline]
    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        match self {
            Object::Cube(c) => c.ray_intersect_distance(ray_origin, ray_direction, max_distance),
            Object::Sphere(s) => s.ray_intersect_distance(ray_origin, ray_direction, max_distance),
            Object::Plane(p) => p.ray_intersect_distance(ray_origin, ray_direction, max_distance),
        }
    }
}