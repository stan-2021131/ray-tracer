#![allow(unused_imports)]
mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod materials;
mod plane;
mod ray_intersect;
mod renderer;
mod scene;
mod skybox;
mod sphere;
mod texture;

use minifb::{Key, Scale, Window, WindowOptions};
use nalgebra_glm::Vec3;
use std::f32::consts::PI;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::renderer::render;
use crate::skybox::Skybox;
use crate::texture::TextureFilter;

const WIDTH: usize = 650;
const HEIGHT: usize = 450;
const ROTATION_SPEED: f32 = PI / 45.0;

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Ray Tracer",
        WIDTH,
        HEIGHT,
        WindowOptions { ..WindowOptions::default() },
    )
    .unwrap();

    // ==========================================
    // 1. SKYBOX NOCTURNO (CUBEMAP 6 CARAS)
    // ==========================================
    let skybox = Skybox::new(
        "./textures/skybox/up.png",
        "./textures/skybox/down.png",
        "./textures/skybox/left.png",
        "./textures/skybox/right.png",
        "./textures/skybox/front.png",
        "./textures/skybox/back.png",
    )
    .with_filter(TextureFilter::Nearest);

    // ==========================================
    // 2. SELECCIÓN DE ESCENA (OBJETOS Y LUCES)
    // ==========================================
    // Opciones disponibles en scene.rs
    let scene = scene::spheres_scene();

    let mut camera = Camera::new(
        Vec3::new(0.0, 2.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_moved = true;

    // ==========================================
    // 3. BUCLE PRINCIPAL DE RENDER Y EVENTOS
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
            render(
                &mut framebuffer,
                &scene.objects,
                &camera,
                &scene.lights,
                Some(&skybox),
            );
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}