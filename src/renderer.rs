use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, Object, RayIntersect};
use crate::skybox::Skybox;
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;

pub const BACKGROUND_COLOR: u32 = 0x040C24;
pub const FOV: f32 = PI / 3.0;
pub const SHADOW_BIAS: f32 = 1e-3;
pub const REFLECTION_BIAS: f32 = 1e-3;
pub const MAX_DEPTH: u32 = 3;

/// Calcula la dirección de reflexión especular de un rayo incidente sobre una normal.
#[inline]
pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

/// Comprueba si el punto de intersección está bloqueado respecto a la fuente de luz por otro objeto.
pub fn cast_shadow(
    intersect: &Intersect,
    light_direction: &Vec3,
    light: &Light,
    objects: &[Object],
) -> bool {
    let shadow_ray_origin = intersect.point + intersect.normal * SHADOW_BIAS;
    let light_distance = (light.position - intersect.point).magnitude();

    objects.iter().any(|object| {
        object.ray_intersect_distance(&shadow_ray_origin, light_direction, light_distance)
    })
}

/// Calcula el sombreado Phong (difuso + especular) teniendo en cuenta las sombras arrojadas.
pub fn shade(
    intersect: &Intersect,
    ray_origin: &Vec3,
    light: &Light,
    objects: &[Object],
) -> Color {
    let light_direction = (light.position - intersect.point).normalize();
    let view_direction = (ray_origin - intersect.point).normalize();

    // Verificación de sombra
    let in_shadow = cast_shadow(intersect, &light_direction, light, objects);
    let shadow_factor = if in_shadow { 0.1 } else { 1.0 };

    // Componente difusa (Lambertiana)
    let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
    let diffuse = intersect.diffuse_color
        * (diffuse_intensity * intersect.albedo[0] * light.intensity * shadow_factor);

    // Componente especular (Phong)
    let specular = if in_shadow {
        Color::new(0, 0, 0)
    } else {
        let reflect_direction = reflect(&-light_direction, &intersect.normal);
        let specular_intensity = dot(&view_direction, &reflect_direction)
            .max(0.0)
            .powf(intersect.specular);
        light.color * (specular_intensity * intersect.albedo[1] * light.intensity)
    };

    diffuse + specular
}

/// Dispara un rayo en la escena y retorna el color del objeto impactado o el fondo / Skybox.
///
/// # Integración del Skybox:
/// - Si el rayo no intersecta ningún objeto de la escena (o excede MAX_DEPTH), se muestrea el color
///   del Skybox usando únicamente la dirección tridimensional del rayo.
/// - La posición de origen del rayo no afecta el color del skybox (simulando una bóveda celeste en el infinito).
/// - El skybox no recibe sombras, iluminación difusa ni atenuación de materiales.
/// - Funciona de forma idéntica y consistente para rayos primarios (cámara) y rayos secundarios (reflexión/refracción).
/// - Si `skybox` es `None`, se utiliza el color de fondo estático `BACKGROUND_COLOR` como respaldo.
pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Object],
    light: &Light,
    skybox: Option<&Skybox>,
    depth: u32,
) -> Color {
    if depth > MAX_DEPTH {
        return skybox
            .map(|sb| sb.sample(ray_direction))
            .unwrap_or_else(|| Color::from_hex(BACKGROUND_COLOR));
    }
    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest
                .as_ref()
                .is_none_or(|current| intersect.distance < current.distance)
            {
                closest = Some(intersect);
            }
        }
    }

    let Some(intersect) = closest else {
        return skybox
            .map(|sb| sb.sample(ray_direction))
            .unwrap_or_else(|| Color::from_hex(BACKGROUND_COLOR));
    };

    let color = shade(&intersect, ray_origin, light, objects);

    let reflectivity = intersect.albedo[2];

    if reflectivity <= 0.0 {
        return color;
    }

    let reflect_direction = reflect(&-ray_direction, &intersect.normal);
    let reflection_origin = intersect.point + intersect.normal * REFLECTION_BIAS;
    let reflection_color = cast_ray(
        &reflection_origin,
        &reflect_direction,
        objects,
        light,
        skybox,
        depth + 1,
    );

    color * (1.0 - reflectivity) + reflection_color * reflectivity
}

/// Renderiza la escena completa de forma multihilo en el framebuffer.
pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Object],
    camera: &Camera,
    light: &Light,
    skybox: Option<&Skybox>,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;

    let fb_width = framebuffer.width;
    let fb_height = framebuffer.height;
    let perspective_scale = (FOV / 2.0).tan();

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let rows_per_chunk = (fb_height + num_threads - 1) / num_threads;

    std::thread::scope(|s| {
        for (chunk_idx, chunk) in framebuffer
            .buffer
            .chunks_mut(rows_per_chunk * fb_width)
            .enumerate()
        {
            let start_y = chunk_idx * rows_per_chunk;

            s.spawn(move || {
                for (local_y, row) in chunk.chunks_exact_mut(fb_width).enumerate() {
                    let y = start_y + local_y;
                    let screen_y = -(2.0 * y as f32) / height + 1.0;
                    let screen_y = screen_y * perspective_scale;

                    for (x, pixel) in row.iter_mut().enumerate() {
                        let screen_x = (2.0 * x as f32) / width - 1.0;
                        let screen_x = screen_x * aspect_ratio * perspective_scale;

                        let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
                        let ray_direction = camera.basis_change(&ray_direction);

                        *pixel = cast_ray(
                            &camera.eye,
                            &ray_direction,
                            objects,
                            light,
                            skybox,
                            0,
                        )
                        .to_hex();
                    }
                }
            });
        }
    });
}
