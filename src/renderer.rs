use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::{Light, LightType};
use crate::ray_intersect::{Intersect, Object, RayIntersect};
use crate::skybox::Skybox;
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;

pub const BACKGROUND_COLOR: u32 = 0x040C24;
pub const FOV: f32 = PI / 3.0;
pub const SHADOW_BIAS: f32 = 1e-3;
pub const REFLECTION_BIAS: f32 = 1e-3;
pub const REFRACTION_BIAS: f32 = 1e-3;
pub const MAX_DEPTH: u32 = 3;

/// Calcula la dirección de reflexión especular de un rayo incidente sobre una normal.
#[inline]
pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

/// Calcula la dirección del rayo refractado según la Ley de Snell.
/// Retorna `None` en caso de Reflexión Interna Total (TIR).
#[inline]
pub fn refract(incident: &Vec3, normal: &Vec3, ior: f32) -> Option<Vec3> {
    let mut cosi = -dot(incident, normal).clamp(-1.0, 1.0);
    let mut eta_i = 1.0;
    let mut eta_t = ior;
    let mut n = *normal;

    if cosi < 0.0 {
        // Rayo saliendo del interior del objeto hacia el aire
        cosi = -cosi;
        std::mem::swap(&mut eta_i, &mut eta_t);
        n = -normal;
    }

    let eta = eta_i / eta_t;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        None // Reflexión Interna Total
    } else {
        Some(incident * eta + n * (eta * cosi - k.sqrt()))
    }
}

/// Coeficiente de Fresnel usando la aproximación de Schlick (fracción de luz reflejada vs refractada).
#[inline]
pub fn fresnel(incident: &Vec3, normal: &Vec3, ior: f32) -> f32 {
    let mut cosi = -dot(incident, normal).clamp(-1.0, 1.0);
    let mut eta_i = 1.0;
    let mut eta_t = ior;

    if cosi < 0.0 {
        cosi = -cosi;
        std::mem::swap(&mut eta_i, &mut eta_t);
    }

    let r0 = ((eta_i - eta_t) / (eta_i + eta_t)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosi).powi(5)
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

/// Calcula el sombreado Phong directo (difuso + brillo especular) acumulando la contribución de múltiples luces y materiales emisivos.
pub fn shade(
    intersect: &Intersect,
    ray_origin: &Vec3,
    lights: &[Light],
    objects: &[Object],
) -> Color {
    let view_direction = (ray_origin - intersect.point).normalize();

    // 1. Componente emisiva: luz propia que emana del material (independiente de sombras o fuentes de luz externas)
    let mut total_color = if intersect.emissive > 0.0 {
        intersect.color * intersect.emissive
    } else {
        Color::new(0, 0, 0)
    };

    for light in lights {
        let light_vec = light.position - intersect.point;
        let light_distance = light_vec.magnitude();
        let light_direction = light_vec / light_distance.max(1e-4);

        // Verificación de sombra para esta luz específica
        let in_shadow = cast_shadow(intersect, &light_direction, light, objects);
        let shadow_factor = if in_shadow { 0.1 } else { 1.0 };

        // Factor de atenuación: Las luces puntuales decaen con la distancia; las direccionales (Sol/Luna) permanecen constantes
        let attenuation = match light.light_type {
            LightType::Directional => 1.0,
            LightType::Point => 1.0 / (1.0 + 0.15 * light_distance + 0.05 * light_distance * light_distance),
        };
        let effective_intensity = light.intensity * attenuation * shadow_factor;

        // Componente difusa (Lambertiana)
        let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
        let diffuse = intersect.color
            * (diffuse_intensity * intersect.diffuse * effective_intensity);

        // Componente especular (Phong)
        let specular = if in_shadow {
            Color::new(0, 0, 0)
        } else {
            let reflect_direction = reflect(&-light_direction, &intersect.normal);
            let specular_intensity = dot(&view_direction, &reflect_direction)
                .max(0.0)
                .powf(intersect.shininess);
            light.color * (specular_intensity * intersect.specular * effective_intensity)
        };

        total_color = total_color + diffuse + specular;
    }

    total_color
}

/// Dispara un rayo en la escena y retorna el color del objeto impactado o el fondo / Skybox.
/// Integra iluminación directa de múltiples luces, sombras, reflexión especular, refracción y efecto Fresnel.
pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Object],
    lights: &[Light],
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

    // 1. Iluminación directa acumulada de todas las luces (solo si tiene componentes directas difusa o especular)
    let direct_color = if intersect.diffuse > 0.0 || intersect.specular > 0.0 {
        shade(&intersect, ray_origin, lights, objects)
    } else {
        Color::new(0, 0, 0)
    };

    // Retorno rápido si el material no es reflectivo ni transparente
    if intersect.reflective <= 0.0 && intersect.refractive <= 0.0 {
        return direct_color;
    }

    // 2. Coeficiente de Fresnel para materiales transparentes/reflectivos
    let mut kr = if intersect.refractive > 0.0 {
        fresnel(ray_direction, &intersect.normal, intersect.refractive_index)
    } else {
        1.0
    };

    // 3. Rayo de Refracción (evaluado primero para comprobar si ocurre Reflexión Interna Total)
    let mut refraction_color = Color::new(0, 0, 0);
    let mut has_refraction = false;
    if intersect.refractive > 0.0 {
        if let Some(refract_dir) = refract(ray_direction, &intersect.normal, intersect.refractive_index) {
            let refract_orig = if dot(ray_direction, &intersect.normal) < 0.0 {
                intersect.point - intersect.normal * REFRACTION_BIAS
            } else {
                intersect.point + intersect.normal * REFRACTION_BIAS
            };
            refraction_color = cast_ray(
                &refract_orig,
                &refract_dir,
                objects,
                lights,
                skybox,
                depth + 1,
            );
            has_refraction = true;
        } else {
            // En Reflexión Interna Total (TIR), el 100% de la energía refractada se refleja
            kr = 1.0;
        }
    }

    // 4. Rayo de Reflexión (ponderado con kr actualizado por TIR)
    let mut reflection_color = Color::new(0, 0, 0);
    let reflect_weight = intersect.reflective + if intersect.refractive > 0.0 { intersect.refractive * kr } else { 0.0 };
    if reflect_weight > 0.0 {
        let reflect_dir = reflect(ray_direction, &intersect.normal);
        let reflect_orig = if dot(ray_direction, &intersect.normal) < 0.0 {
            intersect.point + intersect.normal * REFLECTION_BIAS
        } else {
            intersect.point - intersect.normal * REFLECTION_BIAS
        };
        reflection_color = cast_ray(
            &reflect_orig,
            &reflect_dir,
            objects,
            lights,
            skybox,
            depth + 1,
        );
    }

    let refract_weight = if has_refraction { intersect.refractive * (1.0 - kr) } else { 0.0 };
    let reflect_contrib = reflection_color * reflect_weight;
    let refract_contrib = refraction_color * refract_weight;

    direct_color + reflect_contrib + refract_contrib
}

/// Renderiza la escena completa en el framebuffer usando render_parallel para distribución multihilo.
pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Object],
    camera: &Camera,
    lights: &[Light],
    skybox: Option<&Skybox>,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let perspective_scale = (FOV / 2.0).tan();

    framebuffer.set_background_color(BACKGROUND_COLOR);
    framebuffer.clear();

    framebuffer.render_parallel(|x, y| {
        let screen_y = (-(2.0 * y as f32) / height + 1.0) * perspective_scale;
        let screen_x = ((2.0 * x as f32) / width - 1.0) * aspect_ratio * perspective_scale;

        let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
        let ray_direction = camera.basis_change(&ray_direction);

        let color = cast_ray(&camera.eye, &ray_direction, objects, lights, skybox, 0);
        color.to_hex()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_incidence_refraction() {
        // Rayo entrando perpendicularmente a la superficie (0, -1, 0) sobre normal (0, 1, 0)
        let incident = Vec3::new(0.0, -1.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let refracted = refract(&incident, &normal, 1.5).unwrap();
        // Debe continuar en la misma dirección (0, -1, 0)
        assert!((refracted.x).abs() < 1e-4);
        assert!((refracted.y - (-1.0)).abs() < 1e-4);
        assert!((refracted.z).abs() < 1e-4);
    }

    #[test]
    fn test_total_internal_reflection() {
        // Rayo saliendo de vidrio (IOR 1.5) al aire con ángulo mayor al ángulo crítico (~41.8°)
        // Ángulo de 60 grados: (sin(60°), cos(60°), 0)
        let angle: f32 = 60.0_f32.to_radians();
        let incident = Vec3::new(angle.sin(), angle.cos(), 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0); // Saliendo (incident . normal > 0)
        let refracted = refract(&incident, &normal, 1.5);
        // Debe ser None debido a Reflexión Interna Total
        assert!(refracted.is_none());
    }

    #[test]
    fn test_fresnel_values() {
        let normal = Vec3::new(0.0, 1.0, 0.0);
        // Incidencia normal (0 grados): R0 = ((1 - 1.5)/(1 + 1.5))^2 = (0.5/2.5)^2 = 0.04 (4% de reflexión)
        let incident_normal = Vec3::new(0.0, -1.0, 0.0);
        let kr_normal = fresnel(&incident_normal, &normal, 1.5);
        assert!((kr_normal - 0.04).abs() < 1e-3);

        // Ángulo rasante (casi 90 grados): kr tiende a 1.0 (100% de reflexión)
        let incident_grazing = Vec3::new(0.999, -0.001, 0.0).normalize();
        let kr_grazing = fresnel(&incident_grazing, &normal, 1.5);
        assert!(kr_grazing > 0.95);
    }
}

