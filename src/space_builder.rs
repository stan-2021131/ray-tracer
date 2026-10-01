use crate::color::Color;
use crate::light::Light;
use crate::materials;
use crate::plane::Plane;
use crate::ray_intersect::Object;
use crate::scene::Scene;
use crate::sphere::Sphere;
use nalgebra_glm::Vec3;

/// Construye la escena espacial / planetaria para la vista a través del telescopio.
/// Cubre un panorama completo de 360° con la Luna, planetas a distintas distancias,
/// constelaciones densas de estrellas emisivas en todas direcciones (arriba, abajo, lados)
/// y planos orientados hacia el centro (OVNIs, naves, meteoritos).
pub fn build_space_scene() -> Scene {
    let mut objects: Vec<Object> = Vec::new();
    let lights: Vec<Light> = Vec::new(); // Sin luces solares artificiales: pura oscuridad espacial

    let origin = Vec3::new(0.0, 0.0, 0.0);

    // =========================================================================
    // 1. LA LUNA (Sector Noroeste: yaw ~ 315°)
    // =========================================================================
    let moon_pos = Vec3::new(-95.0, 65.0, -145.0);
    let moon_radius = 9.0;
    let moon_mat = materials::apply_texture(
        materials::emissive(Color::new(230, 230, 235), 0.95),
        "./textures/space/moon.png",
    );
    objects.push(Object::Sphere(Sphere::new(moon_pos, moon_radius, moon_mat)));

    // =========================================================================
    // 2. PLANETAS DISTRIBUIDOS EN 360° (Norte, Sur, Este, Oeste y diagonales)
    // =========================================================================

    // A. Planeta Gaseoso Gigante tipo Júpiter (Sector Sureste: yaw ~ 135°)
    let jupiter_pos = Vec3::new(140.0, 50.0, 110.0);
    let jupiter_radius = 12.0;
    let jupiter_mat = materials::apply_texture(
        materials::emissive(Color::new(220, 165, 110), 0.90),
        "./textures/space/jupiter.png",
    );
    objects.push(Object::Sphere(Sphere::new(jupiter_pos, jupiter_radius, jupiter_mat)));

    // B. Planeta Rocoso Rojo tipo Marte (Sector Suroeste: yaw ~ 225°)
    let mars_pos = Vec3::new(-130.0, 45.0, 95.0);
    let mars_radius = 5.5;
    let mars_mat = materials::apply_texture(
        materials::emissive(Color::new(215, 75, 45), 0.92),
        "./textures/space/mars.png",
    );
    objects.push(Object::Sphere(Sphere::new(mars_pos, mars_radius, mars_mat)));

    // C. Planeta Helado Azulado tipo Neptuno (Sector Noreste: yaw ~ 45°)
    let neptune_pos = Vec3::new(125.0, 70.0, -135.0);
    let neptune_radius = 7.0;
    let neptune_mat = materials::apply_texture(
        materials::emissive(Color::new(75, 145, 245), 0.90),
        "./textures/space/neptune.png",
    );
    objects.push(Object::Sphere(Sphere::new(neptune_pos, neptune_radius, neptune_mat)));

    // D. Planeta Dorado Anillado / Distante (Sector Sur Profundo: yaw ~ 180°)
    let gold_planet_pos = Vec3::new(15.0, 80.0, 180.0);
    let gold_planet_radius = 5.0;
    let gold_mat = materials::emissive(Color::new(245, 215, 120), 0.88);
    objects.push(Object::Sphere(Sphere::new(
        gold_planet_pos,
        gold_planet_radius,
        gold_mat,
    )));

    // E. Planeta Jade / Exoplaneta (Sector Oeste: yaw ~ 270°)
    let emerald_pos = Vec3::new(-175.0, 55.0, -10.0);
    let emerald_radius = 4.2;
    let emerald_mat = materials::emissive(Color::new(80, 220, 160), 0.85);
    objects.push(Object::Sphere(Sphere::new(
        emerald_pos,
        emerald_radius,
        emerald_mat,
    )));

    // F. Planeta Amatista / Enano (Sector Este: yaw ~ 90°)
    let amethyst_pos = Vec3::new(170.0, 60.0, -20.0);
    let amethyst_radius = 3.8;
    let amethyst_mat = materials::emissive(Color::new(190, 110, 245), 0.85);
    objects.push(Object::Sphere(Sphere::new(
        amethyst_pos,
        amethyst_radius,
        amethyst_mat,
    )));

    // G. Planeta Helado Pálido (Sector Norte Alto: yaw ~ 0°)
    let ice_pos = Vec3::new(0.0, 95.0, -165.0);
    let ice_radius = 4.0;
    let ice_mat = materials::emissive(Color::new(210, 240, 255), 0.90);
    objects.push(Object::Sphere(Sphere::new(ice_pos, ice_radius, ice_mat)));

    // =========================================================================
    // 3. PLANOS BILLBOARD ORIENTADOS (OVNIs, Naves, Satélites, Meteoritos)
    // =========================================================================

    // A. OVNI / Platillo Volador (Sector Nor-Noroeste)
    let ufo_pos = Vec3::new(-65.0, 30.0, -110.0);
    let ufo_normal = (origin - ufo_pos).normalize();
    let ufo_mat = materials::apply_texture(
        materials::emissive(Color::new(130, 230, 255), 1.0),
        "./textures/space/ufo.png",
    );
    objects.push(Object::Plane(Plane::new_with_normal(
        ufo_pos,
        ufo_normal,
        6.0,
        3.8,
        ufo_mat,
    )));

    // B. Nave Espacial de Exploración (Sector Sureste)
    let ship_pos = Vec3::new(95.0, 40.0, 90.0);
    let ship_normal = (origin - ship_pos).normalize();
    let ship_mat = materials::apply_texture(
        materials::emissive(Color::new(240, 240, 245), 1.0),
        "./textures/space/spaceship.png",
    );
    objects.push(Object::Plane(Plane::new_with_normal(
        ship_pos,
        ship_normal,
        5.5,
        5.5,
        ship_mat,
    )));

    // C. Nave Sonda de Patrulla (Sector Suroeste)
    let sat_pos = Vec3::new(-105.0, 55.0, 75.0);
    let sat_normal = (origin - sat_pos).normalize();
    let sat_mat = materials::apply_texture(
        materials::emissive(Color::new(255, 220, 110), 1.0),
        "./textures/space/spaceship.png",
    );
    objects.push(Object::Plane(Plane::new_with_normal(
        sat_pos,
        sat_normal,
        4.5,
        4.5,
        sat_mat,
    )));

    // D. OVNI / Sonda No Identificada Lejana (Sector Noreste)
    let probe_pos = Vec3::new(80.0, 65.0, -100.0);
    let probe_normal = (origin - probe_pos).normalize();
    let probe_mat = materials::apply_texture(
        materials::emissive(Color::new(200, 230, 255), 1.0),
        "./textures/space/ufo.png",
    );
    objects.push(Object::Plane(Plane::new_with_normal(
        probe_pos,
        probe_normal,
        4.0,
        4.0,
        probe_mat,
    )));

    // E. Meteorito / Bólido Brillante 1 (Noroeste hacia Norte)
    let meteor1_pos = Vec3::new(-50.0, 75.0, -85.0);
    let meteor1_normal = (origin - meteor1_pos).normalize();
    let meteor1_mat = materials::apply_texture(
        materials::emissive(Color::new(255, 245, 190), 2.2),
        "./textures/space/meteor.png",
    );
    objects.push(Object::Plane(Plane::new_with_normal(
        meteor1_pos,
        meteor1_normal,
        9.0,
        1.8,
        meteor1_mat,
    )));

    // F. Meteorito / Bólido Brillante 2 (Sureste hacia Este)
    let meteor2_pos = Vec3::new(85.0, 85.0, 80.0);
    let meteor2_normal = (origin - meteor2_pos).normalize();
    let meteor2_mat = materials::apply_texture(
        materials::emissive(Color::new(170, 235, 255), 2.0),
        "./textures/space/meteor.png",
    );
    objects.push(Object::Plane(Plane::new_with_normal(
        meteor2_pos,
        meteor2_normal,
        8.0,
        1.6,
        meteor2_mat,
    )));

    // =========================================================================
    // 4. CAMPO DENSO DE ESTRELLAS EMISIVAS EN 360° (Arriba, Abajo, Lados)
    // =========================================================================
    generate_starfield(&mut objects, 260, 130.0);

    Scene::new(objects, lights)
}

/// Genera un campo denso y orgánico de estrellas emisivas en todas direcciones (360° yaw y elevación completa).
fn generate_starfield(objects: &mut Vec<Object>, count: usize, base_distance: f32) {
    let colors = [
        0xFFFFFF, // Blanco estelar brillante
        0xAEE8FF, // Azul celeste
        0xDFF9FB, // Diamante cian
        0xFFEAA7, // Dorado suave
        0xFED330, // Ámbar cálido
        0xFFCCCC, // Rubí tenue
        0xE056FD, // Violeta nébula
        0x70A1FF, // Zafiro brillante
        0x67E6DC, // Turquesa cósmica
        0xFFA801, // Naranja estelar
    ];

    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let mut rand_f32 = || -> f32 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 32) as u32 as f32) / (u32::MAX as f32)
    };

    for i in 0..count {
        // Dirección yaw completa [0, 2*PI]
        let theta = rand_f32() * 2.0 * std::f32::consts::PI;
        // Elevación phi: desde ligeramente debajo del horizonte (-0.15 rad) hasta el cenit (+1.5 rad)
        let phi = -0.15 + rand_f32() * 1.65;
        // Distancia escalonada para dar profundidad
        let dist = base_distance + rand_f32() * 85.0;

        let cos_phi = phi.cos();
        let x = dist * theta.sin() * cos_phi;
        let y = dist * phi.sin();
        let z = -dist * theta.cos() * cos_phi;

        let radius = 0.40 + rand_f32() * 0.70;
        let color_hex = colors[i % colors.len()];
        let emissive_power = 1.6 + rand_f32() * 1.5;

        let star_mat = materials::emissive(Color::from_hex(color_hex), emissive_power);
        objects.push(Object::Sphere(Sphere::new(Vec3::new(x, y, z), radius, star_mat)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_scene_composition() {
        let scene = build_space_scene();

        // Luna (1) + Planetas (7) + Billboards/Naves (6) + Estrellas (260) = 274 objetos
        assert_eq!(scene.objects.len(), 274, "La escena espacial debe contener 274 objetos celestes distribuidos en 360°");

        // Sin luces artificiales (oscuridad total del espacio)
        assert_eq!(scene.lights.len(), 0, "No debe haber luces solares artificiales en el espacio");
    }
}
