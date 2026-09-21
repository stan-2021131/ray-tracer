#![allow(unused_imports)]
mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod plane;
mod ray_intersect;
mod renderer;
mod skybox;
mod sphere;
mod texture;

use minifb::{Key, Scale, Window, WindowOptions};
use nalgebra_glm::Vec3;
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::plane::Plane;
use crate::ray_intersect::{Material, Object};
use crate::renderer::render;
use crate::skybox::Skybox;
use crate::sphere::Sphere;
use crate::texture::{Texture, TextureFilter};

const WIDTH: usize = 650;
const HEIGHT: usize = 450;
const ROTATION_SPEED: f32 = PI / 45.0;

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Pyramid Ray Tracer",
        WIDTH,
        HEIGHT,
        WindowOptions { ..WindowOptions::default() },
    )
    .unwrap();

    // ==========================================
    // 1. CARGA DE SKYBOX / CUBEMAP
    // ==========================================
    // Se cargan las 6 imágenes del skybox una sola vez al inicio del programa.
    // Rutas esperadas en la carpeta ./textures/skybox/ (up, down, left, right, front, back).
    // Si los archivos aún no existen, se utilizará una textura de respaldo sin detener la ejecución.
    let skybox = Skybox::new(
        "./textures/skybox/up.png",
        "./textures/skybox/down.png",
        "./textures/skybox/left.png",
        "./textures/skybox/right.png",
        "./textures/skybox/front.png",
        "./textures/skybox/back.png",
    )
    .with_filter(TextureFilter::Nearest); // Nearest por defecto para pixel art y máximo rendimiento

    // ==========================================
    // 2. CATÁLOGO DE MATERIALES Y TEXTURAS
    // ==========================================
    let ivory = Material::new(Color::new(100, 100, 80), 50.0, [0.6, 0.3, 0.1]);
    let _rubber = Material::new(Color::new(80, 0, 0), 10.0, [0.9, 0.1, 0.0]);
    let _cobalt = Material::new(Color::new(40, 80, 140), 80.0, [0.7, 0.4, 0.15]);
    let jade = Material::new(Color::new(60, 130, 100), 30.0, [0.8, 0.25, 0.05]);

    // Material reflectivo para la esfera (albedo[2] alto para reflejar nítidamente el skybox)
    let mirror_material = Material::new(Color::new(240, 240, 255), 100.0, [0.1, 0.2, 0.7]);

    // Catálogo de texturas
    let wall_texture = Arc::new(Texture::new("./textures/wall.png"));

    // Composición: Material base + Textura
    let cube_mat = ivory.clone().with_texture(wall_texture);

    let cube_size = 0.8;
    let mut objects: Vec<Object> = Vec::new();

    // ==========================================
    // 3. CONSTRUCCIÓN DE LA PIRÁMIDE DE CUBOS
    // ==========================================
    // Niveles de la pirámide: (nivel_index, radio)
    // Nivel 0 (más bajo): radio 2 -> 5x5 cubos
    // Nivel 1 (segundo nivel): radio 1 -> 3x3 cubos
    // Nivel 2 (cúspide): radio 0 -> 1x1 cubo
    let levels: [(i32, i32); 3] = [
        (0, 2),
        (1, 1),
        (2, 0),
    ];

    for &(level_idx, radius) in &levels {
        let y = (level_idx as f32 - 1.0) * cube_size;
        for x in -radius..=radius {
            for z in -radius..=radius {
                // Modo cascarón: solo incluir cubos exteriores en el perímetro de cada nivel
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
        jade,
    )));

    // ==========================================
    // 4. ESFERA REFLECTIVA (REFLEJA EL SKYBOX)
    // ==========================================
    // Esfera posicionada en la escena que refleja el entorno nocturno del Skybox
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(1.8, 0.0, 1.2),
        0.65,
        mirror_material,
    )));

    let light = Light::new(Vec3::new(5.0, 6.0, 10.0), Color::new(255, 255, 255), 1.5);

    let mut camera = Camera::new(
        Vec3::new(0.0, 2.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_moved = true;

    // ==========================================
    // 5. BUCLE PRINCIPAL DE RENDER Y EVENTOS
    // ==========================================
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        for (key, delta_yaw, delta_pitch) in orbit {
            if window.is_key_down(key) {
                camera.orbit(delta_yaw, delta_pitch);
                camera_moved = true;
            }
        }

        if camera_moved {
            render(&mut framebuffer, &objects, &camera, &light, Some(&skybox));
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}