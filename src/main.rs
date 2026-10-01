mod audio;
mod box3d;
mod camera;
mod color;
mod diorama_builder;
mod framebuffer;
mod light;
mod materials;
mod plane;
mod ray_intersect;
mod renderer;
mod scene;
mod skybox;
mod space_builder;
mod sphere;
mod texture;
mod texture_manager;

use minifb::{Key, Scale, Window, WindowOptions};
use nalgebra_glm::Vec3;
use std::f32::consts::PI;
use std::time::Duration;

use crate::audio::AudioPlayer;
use crate::camera::{Camera, CameraMode};
use crate::framebuffer::Framebuffer;
use crate::renderer::{render, render_telescope};
use crate::skybox::Skybox;
use crate::texture::TextureFilter;

const WIDTH: usize = 560;
const HEIGHT: usize = 380;
const FAST_MAX_DEPTH: u32 = 1; // En movimiento: iluminación directa y texturas sin reflexiones recursivas pesadas
const FULL_MAX_DEPTH: u32 = 3; // En reposo: reflexiones completas y refracciones de vidrio cristalinas
const ROTATION_SPEED: f32 = PI / 45.0;
const MOVE_SPEED: f32 = 0.2;

const TELESCOPE_POS: Vec3 = Vec3::new(9.0, 1.0, 7.5);
const INTERACTION_RADIUS: f32 = 2.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameSceneState {
    Diorama,
    Telescope,
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Ray Tracer - [Modo: Orbit] (Tab/C cambia modo, R reinicia)",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    // ==========================================
    // 1. SKYBOXES (DIORAMA Y ESPACIAL)
    // ==========================================
    let diorama_skybox = Skybox::new(
        "./textures/skybox/up.png",
        "./textures/skybox/down.png",
        "./textures/skybox/left.png",
        "./textures/skybox/right.png",
        "./textures/skybox/front.png",
        "./textures/skybox/back.png",
    )
    .with_filter(TextureFilter::Nearest);

    let space_skybox = Skybox::from_single_texture("./textures/skybox/space.png")
        .with_filter(TextureFilter::Nearest)
        .with_brightness(0.35);

    // ==========================================
    // 2. SELECCIÓN DE ESCENAS (DIORAMA Y ESPACIAL)
    // ==========================================
    let mut diorama_scene = scene::diorama_scene();
    let mut space_scene = scene::space_scene();

    let mut camera = Camera::new(
        Vec3::new(0.0, 20.0, 24.0),
        Vec3::new(0.0, 2.0, -3.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut saved_diorama_camera = camera.clone();
    let mut current_state = GameSceneState::Diorama;

    let start_time = std::time::Instant::now();
    let mut camera_moved = true;
    let mut needs_refine = false;
    let mut tab_was_down = false;
    let mut c_was_down = false;
    let mut e_was_down = false;
    let mut esc_was_down = false;

    // ==========================================
    // 3. SISTEMA DE AUDIO (MÚSICA + AMBIENTE)
    // ==========================================
    let mut audio = AudioPlayer::new();
    audio.start_all();

    // ==========================================
    // 4. BUCLE PRINCIPAL DE RENDER Y EVENTOS
    // ==========================================
    while window.is_open() {
        let esc_down = window.is_key_down(Key::Escape);
        let e_down = window.is_key_down(Key::E);

        // --- Gestión de Transiciones de Escena (E / ESC) ---
        if current_state == GameSceneState::Telescope {
            if (e_down && !e_was_down) || (esc_down && !esc_was_down) {
                camera = saved_diorama_camera.clone();
                current_state = GameSceneState::Diorama;
                window.set_title("Ray Tracer - [Modo: Orbit (Flechas para orbitar)] (Tab/C cambia, R reinicia)");
                camera_moved = true;
                e_was_down = e_down;
                esc_was_down = esc_down;
                continue;
            }
        } else {
            if esc_down {
                break; // Salir de la aplicación desde el Diorama
            }

            let dist_to_telescope = (camera.eye - TELESCOPE_POS).magnitude();
            let is_near_telescope = dist_to_telescope <= INTERACTION_RADIUS;

            if is_near_telescope && e_down && !e_was_down {
                saved_diorama_camera = camera.clone();
                current_state = GameSceneState::Telescope;
                camera.mode = CameraMode::Telescope;
                camera.eye = Vec3::new(0.0, 0.0, 0.0);
                camera.yaw = 0.0;
                camera.pitch = 0.35;
                camera.update_basis();

                window.set_title("Ray Tracer - [Vista Espacial 360°] (Flechas: rotar 360°, [E]/[ESC]: regresar)");
                camera_moved = true;
                e_was_down = e_down;
                esc_was_down = esc_down;
                continue;
            }

            // Atajos de cámara en Diorama
            if window.is_key_down(Key::R) {
                camera.reset(Vec3::new(0.0, 20.0, 24.0), Vec3::new(0.0, 2.0, -3.0), Vec3::new(0.0, 1.0, 0.0));
                window.set_title("Ray Tracer - [Modo: Orbit (Flechas para orbitar)] (Tab/C cambia, R reinicia)");
                camera_moved = true;
            }

            let tab_down = window.is_key_down(Key::Tab);
            let c_down = window.is_key_down(Key::C);
            if (tab_down && !tab_was_down) || (c_down && !c_was_down) {
                let mode = camera.toggle_mode();
                let mode_name = match mode {
                    CameraMode::Orbit => "Orbit (Flechas para orbitar)",
                    CameraMode::FpsCollision => "FPS Colisiones (WASD caminar, Flechas mirar)",
                    CameraMode::FreeCam => "FreeCam (WASD + Q/E volar, Flechas mirar)",
                    CameraMode::Telescope => "Telescopio",
                };
                window.set_title(&format!("Ray Tracer - [Modo: {}] (Tab/C cambia, R reinicia)", mode_name));
                camera_moved = true;
            }
            tab_was_down = tab_down;
            c_was_down = c_down;

            if is_near_telescope && matches!(camera.mode, CameraMode::FpsCollision | CameraMode::FreeCam) {
                window.set_title("Ray Tracer - [E] Mirar por el Telescopio (Tab/C cambia modo)");
            }

            // Movimiento WASD del jugador en Diorama
            let mut f_in = 0.0; let mut s_in = 0.0; let mut u_in = 0.0;
            if window.is_key_down(Key::W) { f_in += 1.0; }
            if window.is_key_down(Key::S) { f_in -= 1.0; }
            if window.is_key_down(Key::D) { s_in += 1.0; }
            if window.is_key_down(Key::A) { s_in -= 1.0; }
            if window.is_key_down(Key::Q) { u_in += 1.0; }
            if window.is_key_down(Key::E) && !is_near_telescope { u_in -= 1.0; }

            if f_in != 0.0 || s_in != 0.0 || u_in != 0.0 {
                camera.move_player(f_in, s_in, u_in, MOVE_SPEED, &diorama_scene.objects);
                if matches!(camera.mode, CameraMode::FpsCollision) && (f_in != 0.0 || s_in != 0.0) {
                    audio.try_play_footstep();
                }
                camera_moved = true;
            }
        }

        // --- Rotación Unificada (Flechas) ---
        let rot_inputs = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];
        for (key, dy, dp) in rot_inputs {
            if window.is_key_down(key) {
                if camera.mode == CameraMode::Orbit {
                    camera.orbit(dy, dp);
                } else {
                    camera.rotate_look(-dy, -dp);
                }
                camera_moved = true;
            }
        }

        // --- Renderizado Reactivo con Optimización de Profundidad de Rayos ---
        let elapsed = start_time.elapsed().as_secs_f32();
        let (active_scene, is_telescope) = match current_state {
            GameSceneState::Diorama => (&mut diorama_scene, false),
            GameSceneState::Telescope => (&mut space_scene, true),
        };

        let has_animations = active_scene.has_animations();
        if has_animations {
            active_scene.update_time(elapsed);
        }

        let is_moving = camera_moved;
        let should_render = is_moving || needs_refine || has_animations;

        if should_render {
            // En movimiento usamos FAST_MAX_DEPTH = 1 (resolución completa 1x1, sin rebotes pesados);
            // al detenerse la cámara, se renderiza con FULL_MAX_DEPTH = 3 (reflexiones y refracciones completas).
            let max_depth = if is_moving { FAST_MAX_DEPTH } else { FULL_MAX_DEPTH };
            if is_telescope {
                // En el espacio: skybox cósmico omnidireccional con visor telescópico
                render_telescope(&mut framebuffer, &active_scene.objects, &camera, &active_scene.lights, Some(&space_skybox), max_depth);
            } else {
                render(&mut framebuffer, &active_scene.objects, &camera, &active_scene.lights, Some(&diorama_skybox), max_depth);
            }
            needs_refine = is_moving;
            camera_moved = false;
        }

        e_was_down = e_down;
        esc_was_down = esc_down;

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}