#![allow(dead_code)]

use crate::box3d::Box3D;
use crate::color::Color;
use crate::light::Light;
use crate::materials;
use crate::texture::TextureWrap;
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

/// Escena completa: Pirámide escalonada de losas rectangulares texturizadas, suelo de jade y dos esferas (espejo y vidrio).
pub fn pyramid_with_spheres_scene() -> Scene {
    let mut scene = pyramid_only_scene();

    // Esfera espejo reflectiva (derecha)
    scene.objects.push(Object::Sphere(Sphere::new(
        Vec3::new(2.8, -0.2, 1.2),
        0.65,
        materials::mirror(Color::new(240, 240, 255)),
    )));

    // Esfera de vidrio transparente y refractiva (izquierda)
    scene.objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-2.8, -0.2, 1.2),
        0.65,
        materials::glass(Color::new(255, 255, 255)),
    )));

    scene
}

/// Escena de pirámide: Construida con losas rectangulares continuas (Box3D) escalonadas.
pub fn pyramid_only_scene() -> Scene {
    let wall_mat = materials::apply_texture(
        materials::diffuse(Color::new(255, 255, 255)),
        "./textures/wall.png",
    );
    let floor_mat = materials::apply_texture(
        materials::diffuse(Color::new(255, 255, 255)),
        "./textures/grass.png",
    ).with_uv_scale(5.0, 5.0).with_wrap(TextureWrap::Repeat);
    let mut objects: Vec<Object> = Vec::new();

    // Altura de cada escalón / losa rectangular
    let step_h = 0.4;

    // Niveles de la pirámide: (ancho_x, profundidad_z, offset_y)
    let tiers = [
        (4.8, 4.8, -0.6), // Base amplia
        (3.6, 3.6, -0.2), // Nivel medio inferior
        (2.4, 2.4,  0.2), // Nivel medio superior
        (1.2, 1.2,  0.6), // Cúspide / altar superior
    ];

    for &(w, d, y) in &tiers {
        objects.push(Object::Box3D(Box3D::new(
            Vec3::new(0.0, y, 0.0),
            Vec3::new(w, step_h, d),
            wall_mat.clone(),
        )));
    }

    // Plano / suelo debajo de la pirámide
    let floor_y = -0.8;
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, floor_y, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        20.0,
        floor_mat,
    )));

    // Configuración de iluminación para la pirámide (Luz clave blanca + Luz de relleno cálida)
    let lights = vec![
        Light::new(Vec3::new(5.0, 6.0, 10.0), Color::new(255, 255, 255), 1.3),
        Light::new(Vec3::new(-6.0, 4.0, 4.0), Color::new(220, 230, 255), 0.5),
    ];

    Scene::new(objects, lights)
}

/// Escena arquitectónica: Estructuras rectangulares (muros, puertas, columnas, mesa) demostrando Box3D.
pub fn rectangular_structures_scene() -> Scene {
    let wall_mat = materials::apply_texture(
        materials::diffuse(Color::new(255, 255, 255)),
        "./textures/wall.png",
    );
    let cobalt_mat = materials::cobalt(Color::new(40, 80, 140));
    let gold_mat = materials::gold(Color::new(255, 215, 0));
    let glass_mat = materials::glass(Color::new(255, 255, 255));
    let mut objects: Vec<Object> = Vec::new();

    // Suelo
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        25.0,
        25.0,
        materials::jade(Color::new(60, 130, 100)),
    )));

    // Muro trasero alargado: ancho 8.0, alto 2.5, grosor 0.3
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 0.25, -3.0),
        Vec3::new(8.0, 2.5, 0.3),
        wall_mat.clone(),
    )));

    // Marco de puerta: poste izquierdo, poste derecho y dintel superior
    // Poste izquierdo (alto y delgado: 0.3 x 2.2 x 0.3)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(-0.9, 0.1, 0.0),
        Vec3::new(0.3, 2.2, 0.3),
        cobalt_mat.clone(),
    )));
    // Poste derecho
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.9, 0.1, 0.0),
        Vec3::new(0.3, 2.2, 0.3),
        cobalt_mat.clone(),
    )));
    // Dintel horizontal superior
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 1.3, 0.0),
        Vec3::new(2.1, 0.2, 0.3),
        gold_mat.clone(),
    )));

    // Cristal interior de la puerta (delgado y transparente)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 0.1, 0.0),
        Vec3::new(1.5, 2.2, 0.05),
        glass_mat,
    )));

    // Mesa en primer plano: superficie rectangular plana
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, -0.6, 2.0),
        Vec3::new(2.5, 0.1, 1.2),
        materials::mirror(Color::new(240, 240, 255)),
    )));
    // Patas de la mesa
    for &px in &[-1.1, 1.1] {
        for &pz in &[1.5, 2.5] {
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(px, -0.8, pz),
                Vec3::new(0.12, 0.4, 0.12),
                gold_mat.clone(),
            )));
        }
    }

    let lights = vec![
        Light::new(Vec3::new(4.0, 6.0, 5.0), Color::new(255, 255, 255), 1.2),
        Light::new(Vec3::new(-5.0, 3.0, 2.0), Color::new(100, 200, 255), 0.6),
        Light::new(Vec3::new(0.0, 4.0, -4.0), Color::new(255, 150, 100), 0.8),
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
        materials::jade(Color::new(60, 130, 100)),
    )));

    // Esfera central de vidrio refractivo
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        1.0,
        materials::glass(Color::new(255, 255, 255)),
    )));

    // Esfera izquierda de espejo puro
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-2.4, 0.0, 0.5),
        0.8,
        materials::mirror(Color::new(240, 240, 255)),
    )));

    // Esfera derecha de oro pulido
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(2.4, 0.0, 0.5),
        0.8,
        materials::gold(Color::new(255, 215, 0)),
    )));

    // Esfera trasera de cobalto metálico
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-1.0, -0.2, -2.0),
        0.7,
        materials::cobalt(Color::new(40, 80, 140)),
    )));

    // Esfera delantera de marfil brillante
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(1.0, -0.3, 2.0),
        0.6,
        materials::ivory(Color::new(100, 100, 80)),
    )));

    let lights = vec![
        Light::new(Vec3::new(4.0, 7.0, 5.0), Color::new(255, 255, 255), 1.1),
        Light::new(Vec3::new(-6.0, 4.0, 3.0), Color::new(80, 190, 255), 0.7),
        Light::new(Vec3::new(0.0, 5.0, -5.0), Color::new(255, 120, 180), 0.8),
    ];

    Scene::new(objects, lights)
}
