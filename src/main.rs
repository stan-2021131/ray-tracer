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

use crate::camera::{Camera, CameraMode};
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::renderer::render;
use crate::skybox::Skybox;
use crate::texture::TextureFilter;

const WIDTH: usize = 975;
const HEIGHT: usize = 675;
const ROTATION_SPEED: f32 = PI / 45.0;
const MOVE_SPEED: f32 = 0.2;

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Ray Tracer - [Modo: Orbit] (Tab/C cambia modo, R reinicia)",
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
    // Opciones disponibles en scene.rs:
    // - scene::pyramid_with_spheres_scene()
    // - scene::pyramid_only_scene()
    // - scene::spheres_scene()
    let scene = scene::spheres_scene();

    let mut camera = Camera::new(
        Vec3::new(0.0, 2.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_moved = true;
    let mut tab_was_down = false;
    let mut c_was_down = false;

    // ==========================================
    // 3. BUCLE PRINCIPAL DE RENDER Y EVENTOS
    // ==========================================
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- Reinicio de Cámara (R) ---
        if window.is_key_down(Key::R) {
            camera.reset(
                Vec3::new(0.0, 2.0, 6.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            );
            window.set_title("Ray Tracer - [Modo: Orbit (Flechas para orbitar)] (Tab/C cambia, R reinicia)");
            camera_moved = true;
        }

        // --- Conmutación de Modo de Cámara (Tab o C) ---
        let tab_down = window.is_key_down(Key::Tab);
        let c_down = window.is_key_down(Key::C);

        if (tab_down && !tab_was_down) || (c_down && !c_was_down) {
            let mode = camera.toggle_mode();
            let mode_name = match mode {
                CameraMode::Orbit => "Orbit (Flechas para orbitar)",
                CameraMode::FpsCollision => "FPS Colisiones (WASD caminar, Flechas mirar)",
                CameraMode::FreeCam => "FreeCam (WASD + Q/E volar, Flechas mirar)",
            };
            window.set_title(&format!("Ray Tracer - [Modo: {}] (Tab/C cambia, R reinicia)", mode_name));
            camera_moved = true;
        }
        tab_was_down = tab_down;
        c_was_down = c_down;

        // --- Rotación / Dirección de Vista (Flechas) ---
        let rot_inputs = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        for (key, delta_yaw, delta_pitch) in rot_inputs {
            if window.is_key_down(key) {
                match camera.mode {
                    CameraMode::Orbit => camera.orbit(delta_yaw, delta_pitch),
                    CameraMode::FpsCollision | CameraMode::FreeCam => {
                        // En primera persona: flecha izquierda gira izquierda, flecha arriba mira arriba
                        camera.rotate_look(-delta_yaw, -delta_pitch);
                    }
                }
                camera_moved = true;
            }
        }

        // --- Desplazamiento del Jugador / Cámara (WASD + Espacio / Shift) ---
        let mut forward_input = 0.0;
        let mut strafe_input = 0.0;
        let mut up_input = 0.0;

        if window.is_key_down(Key::W) { forward_input += 1.0; }
        if window.is_key_down(Key::S) { forward_input -= 1.0; }
        if window.is_key_down(Key::D) { strafe_input += 1.0; }
        if window.is_key_down(Key::A) { strafe_input -= 1.0; }
        if window.is_key_down(Key::Q) { up_input += 1.0; }
        if window.is_key_down(Key::E) { up_input -= 1.0; }

        if forward_input != 0.0 || strafe_input != 0.0 || up_input != 0.0 {
            camera.move_player(
                forward_input,
                strafe_input,
                up_input,
                MOVE_SPEED,
                &scene.objects,
            );
            camera_moved = true;
        }

        // --- Renderizado reactivo solo al mover la cámara ---
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