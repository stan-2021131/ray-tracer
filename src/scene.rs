#![allow(dead_code)]

use crate::color::Color;
use crate::cube::Cube;
use crate::light::Light;
use crate::materials;
use crate::plane::Plane;
use crate::ray_intersect::Object;
use crate::sphere::Sphere;
use nalgebra_glm::Vec3;

/// Representa una escena 3D completa: colección de objetos geométricos y fuentes de luz.
pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new(objects: Vec<Object>, lights: Vec<Light>) -> Self {
        Scene { objects, lights }
    }
}

/// Escena completa: Pirámide escalonada de cubos texturizados, suelo de jade y dos esferas (espejo y vidrio).
pub fn pyramid_with_spheres_scene() -> Scene {
    let mut scene = pyramid_only_scene();

    // Esfera espejo reflectiva (derecha)
    scene.objects.push(Object::Sphere(Sphere::new(
        Vec3::new(1.8, 0.0, 1.2),
        0.65,
        materials::mirror(),
    )));

    // Esfera de vidrio transparente y refractiva (izquierda)
    scene.objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-1.8, 0.0, 1.2),
        0.65,
        materials::glass(),
    )));

    scene
}

/// Escena de pirámide: Solo la pirámide de cubos exterior sobre el plano de suelo con luces principal y de relleno.
pub fn pyramid_only_scene() -> Scene {
    let cube_mat = materials::textured_wall("./textures/wall.png");
    let floor_mat = materials::jade();
    let cube_size = 0.8;
    let mut objects: Vec<Object> = Vec::new();

    // Niveles de la pirámide: (nivel_index, radio)
    let levels: [(i32, i32); 3] = [
        (0, 2), // Base: 5x5
        (1, 1), // Medio: 3x3
        (2, 0), // Cúspide: 1x1
    ];

    for &(level_idx, radius) in &levels {
        let y = (level_idx as f32 - 1.0) * cube_size;
        for x in -radius..=radius {
            for z in -radius..=radius {
                let is_outer = radius == 0 || x.abs() == radius || z.abs() == radius;
                if is_outer {
                    let center = Vec3::new(x as f32 * cube_size, y, z as f32 * cube_size);
                    objects.push(Object::Cube(Cube::new(center, cube_size, cube_mat.clone())));
                }
            }
        }
    }

    // Plano / suelo debajo de la pirámide
    let floor_y = -1.5 * cube_size;
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, floor_y, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        16.0,
        16.0,
        floor_mat,
    )));

    // Configuración de iluminación para la pirámide (Luz clave blanca + Luz de relleno cálida)
    let lights = vec![
        Light::new(Vec3::new(5.0, 6.0, 10.0), Color::new(255, 255, 255), 1.3),
        Light::new(Vec3::new(-6.0, 4.0, 4.0), Color::new(220, 230, 255), 0.5),
    ];

    Scene::new(objects, lights)
}

/// Escena de esferas: Muestra de materiales (espejo, vidrio, oro, cobalto, marfil) con iluminación ambiental y de contraste.
pub fn spheres_scene() -> Scene {
    let mut objects: Vec<Object> = Vec::new();

    // Suelo reflectivo suave
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        20.0,
        materials::jade(),
    )));

    // Esfera central de vidrio refractivo
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        1.0,
        materials::glass(),
    )));

    // Esfera izquierda de espejo puro
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-2.4, 0.0, 0.5),
        0.8,
        materials::mirror(),
    )));

    // Esfera derecha de oro pulido
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(2.4, 0.0, 0.5),
        0.8,
        materials::gold(),
    )));

    // Esfera trasera de cobalto metálico
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-1.0, -0.2, -2.0),
        0.7,
        materials::cobalt(),
    )));

    // Esfera delantera de marfil brillante
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(1.0, -0.3, 2.0),
        0.6,
        materials::ivory(),
    )));

    // Configuración de iluminación para la galería de esferas (3 luces con colores y ángulos distintos):
    // 1. Luz principal frontal-derecha blanca (iluminación clave y sombras definidas)
    // 2. Luz de relleno lateral izquierda cian / azul fría (resalta reflejos y refracciones del vidrio)
    // 3. Contraluz trasera magenta / ámbar cálida (crea efecto de borde / rim light en las siluetas)
    let lights = vec![
        Light::new(Vec3::new(4.0, 7.0, 5.0), Color::new(255, 255, 255), 1.1),
        Light::new(Vec3::new(-6.0, 4.0, 3.0), Color::new(80, 190, 255), 0.7),
        Light::new(Vec3::new(0.0, 5.0, -5.0), Color::new(255, 120, 180), 0.8),
    ];

    Scene::new(objects, lights)
}
