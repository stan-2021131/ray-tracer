use crate::color::Color;
use crate::cube::Cube;
use crate::plane::Plane;
use crate::sphere::Sphere;
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 3],
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 3]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            texture: None,
        }
    }

    /// Asigna una textura a este material retornando el material modificado (patrón Builder).
    pub fn with_texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    /// Retorna el color difuso modulando el color base del material con la textura si existe.
    #[inline]
    pub fn get_diffuse_color(&self, u: f32, v: f32) -> Color {
        if let Some(tex) = &self.texture {
            self.diffuse * tex.get_color(u, v)
        } else {
            self.diffuse
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub diffuse_color: Color,
    pub specular: f32,
    pub albedo: [f32; 3],
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