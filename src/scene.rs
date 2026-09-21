#![allow(dead_code)]

use crate::cube::Cube;
use crate::materials;
use crate::plane::Plane;
use crate::ray_intersect::Object;
use crate::sphere::Sphere;
use nalgebra_glm::Vec3;

/// Escena completa: Pirámide escalonada de cubos texturizados, suelo de jade y dos esferas (espejo y vidrio).
pub fn pyramid_with_spheres_scene() -> Vec<Object> {
    let mut objects = pyramid_only_scene();

    // Esfera espejo reflectiva (derecha)
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(1.8, 0.0, 1.2),
        0.65,
        materials::mirror(),
    )));

    // Esfera de vidrio transparente y refractiva (izquierda)
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-1.8, 0.0, 1.2),
        0.65,
        materials::glass(),
    )));

    objects
}

/// Escena de pirámide: Solo la pirámide de cubos exterior sobre el plano de suelo.
pub fn pyramid_only_scene() -> Vec<Object> {
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

    objects
}

/// Escena de esferas: Muestra de materiales (espejo, vidrio, oro, cobalto, marfil) sobre suelo.
pub fn spheres_scene() -> Vec<Object> {
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

    objects
}
