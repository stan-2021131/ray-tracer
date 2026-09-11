use crate::color::Color;
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2],
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 2]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            texture: None,
        }
    }

    pub fn new_with_texture(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 2],
        texture: Arc<Texture>,
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            texture: Some(texture),
        }
    }

    /// Retorna el color difuso evaluado en coordenadas UV si tiene textura, o el color base.
    pub fn get_diffuse_color(&self, u: f32, v: f32) -> Color {
        if let Some(tex) = &self.texture {
            tex.get_color(u, v)
        } else {
            self.diffuse
        }
    }

    /// Genera una copia del material con el color difuso evaluado en (u, v).
    pub fn with_uv(&self, u: f32, v: f32) -> Self {
        let mut mat = self.clone();
        if self.texture.is_some() {
            mat.diffuse = self.get_diffuse_color(u, v);
        }
        mat
    }
}

#[derive(Debug, Clone)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub material: Material,
}

pub trait RayIntersect: Sync + Send {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}