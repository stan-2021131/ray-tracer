use crate::box3d::Box3D;
use crate::color::Color;
use crate::plane::Plane;
use crate::sphere::Sphere;
use crate::texture::{Texture, TextureWrap};
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
    pub emissive: f32,          // Emisión de luz propia (0.0 = normal, >0.0 = fuente de brillo)
    pub shininess: f32,         // Exponente de brillo Phong
    pub refractive_index: f32,  // Índice de refracción (IOR)
    pub texture: Option<Arc<Texture>>,
    pub animated_frames: Option<Arc<Vec<Arc<Texture>>>>,
    pub animation_fps: f32,
    pub uv_scale: (f32, f32),   // Multiplicador de repetición / escala UV (u_scale, v_scale)
    pub wrap_mode: TextureWrap, // Modo de envoltura: Repeat (mosaico) o Clamp (estirado)
}

#[allow(dead_code)]
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
            emissive: 0.0,
            shininess,
            refractive_index,
            texture: None,
            animated_frames: None,
            animation_fps: 1.0,
            uv_scale: (1.0, 1.0),
            wrap_mode: TextureWrap::Repeat,
        }
    }

    /// Asigna una textura a este material retornando el material modificado (patrón Builder).
    pub fn with_texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    /// Configura una secuencia de texturas animadas (ciclo de cuadros) con una velocidad dada en FPS.
    pub fn with_animated_textures(mut self, frames: Vec<Arc<Texture>>, fps: f32) -> Self {
        if let Some(first) = frames.first() {
            self.texture = Some(Arc::clone(first));
        }
        self.animated_frames = Some(Arc::new(frames));
        self.animation_fps = fps.max(0.001);
        self
    }

    /// Actualiza la textura activa según el tiempo transcurrido (en segundos).
    #[inline]
    pub fn update_time(&mut self, time: f32) {
        if let Some(frames) = &self.animated_frames {
            if !frames.is_empty() {
                let idx = ((time * self.animation_fps).floor() as usize) % frames.len();
                self.texture = Some(Arc::clone(&frames[idx]));
            }
        }
    }

    /// Configura la emisión de luz propia del material.
    pub fn with_emissive(mut self, emissive: f32) -> Self {
        self.emissive = emissive;
        self
    }

    /// Configura la escala de repetición UV (veces que se repite la textura a lo ancho y alto).
    pub fn with_uv_scale(mut self, u_scale: f32, v_scale: f32) -> Self {
        self.uv_scale = (u_scale, v_scale);
        self
    }

    /// Configura el modo de envoltura UV (`TextureWrap::Repeat` para mosaico o `TextureWrap::Clamp` para estirado).
    pub fn with_wrap(mut self, wrap: TextureWrap) -> Self {
        self.wrap_mode = wrap;
        self
    }

    /// Retorna el color modulando el color base del material con la textura escalada y envuelta si existe.
    #[inline]
    pub fn get_color(&self, u: f32, v: f32) -> Color {
        if let Some(tex) = &self.texture {
            let u_scaled = u * self.uv_scale.0;
            let v_scaled = v * self.uv_scale.1;
            self.color * tex.get_color_nearest_wrap(u_scaled, v_scaled, self.wrap_mode)
        } else {
            self.color
        }
    }

    /// Indica si el material posee una textura con canal alfa/transparencia activo.
    #[inline]
    pub fn has_alpha(&self) -> bool {
        self.texture.as_ref().map(|t| t.has_alpha).unwrap_or(false)
    }

    /// Convierte los datos del material en una estructura `Intersect` para el impacto de un rayo.
    #[inline]
    pub fn to_intersect(&self, point: Vec3, normal: Vec3, distance: f32, u: f32, v: f32) -> Intersect {
        self.to_intersect_with_color(point, normal, distance, self.get_color(u, v))
    }

    /// Convierte los datos del material en una estructura `Intersect` usando un color ya calculado (evita recalcular texturas).
    #[inline]
    pub fn to_intersect_with_color(&self, point: Vec3, normal: Vec3, distance: f32, color: Color) -> Intersect {
        Intersect {
            point,
            normal,
            distance,
            color,
            diffuse: self.diffuse,
            specular: self.specular,
            reflective: self.reflective,
            refractive: self.refractive,
            emissive: self.emissive,
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
    pub emissive: f32,
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
    Box3D(Box3D),
    Sphere(Sphere),
    Plane(Plane),
}

impl Object {
    /// Retorna la caja envolvente alineada a los ejes (AABB: min, max) del objeto en espacio de mundo.
    pub fn aabb(&self) -> (Vec3, Vec3) {
        match self {
            Object::Box3D(b) => (b.min, b.max),
            Object::Sphere(s) => {
                let r = Vec3::new(s.radius, s.radius, s.radius);
                (s.center - r, s.center + r)
            }
            Object::Plane(p) => {
                let ext_x = p.u.x.abs() * p.half_w + p.v.x.abs() * p.half_h + 0.05;
                let ext_y = p.u.y.abs() * p.half_w + p.v.y.abs() * p.half_h + 0.05;
                let ext_z = p.u.z.abs() * p.half_w + p.v.z.abs() * p.half_h + 0.05;
                let extent = Vec3::new(ext_x, ext_y, ext_z);
                (p.center - extent, p.center + extent)
            }
        }
    }

    /// Actualiza el estado de animación del material en base al tiempo actual transcurrido.
    pub fn update_time(&mut self, time: f32) {
        match self {
            Object::Box3D(b) => b.material.update_time(time),
            Object::Sphere(s) => s.material.update_time(time),
            Object::Plane(p) => p.material.update_time(time),
        }
    }

    /// Retorna verdadero si el objeto posee un ciclo de texturas animadas.
    pub fn is_animated(&self) -> bool {
        match self {
            Object::Box3D(b) => b.material.animated_frames.is_some(),
            Object::Sphere(s) => s.material.animated_frames.is_some(),
            Object::Plane(p) => p.material.animated_frames.is_some(),
        }
    }
}

impl RayIntersect for Object {
    #[inline]
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        match self {
            Object::Box3D(b) => b.ray_intersect(ray_origin, ray_direction),
            Object::Sphere(s) => s.ray_intersect(ray_origin, ray_direction),
            Object::Plane(p) => p.ray_intersect(ray_origin, ray_direction),
        }
    }

    #[inline]
    fn ray_intersect_distance(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        match self {
            Object::Box3D(b) => b.ray_intersect_distance(ray_origin, ray_direction, max_distance),
            Object::Sphere(s) => s.ray_intersect_distance(ray_origin, ray_direction, max_distance),
            Object::Plane(p) => p.ray_intersect_distance(ray_origin, ray_direction, max_distance),
        }
    }
}