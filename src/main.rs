#![allow(unused_imports)]
mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod plane;
mod ray_intersect;
mod sphere;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::plane::Plane;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::sphere::Sphere;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BACKGROUND_COLOR: u32 = 0x040C24;

const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 45.0;

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

pub fn shade(intersect: &Intersect, ray_origin: &Vec3, light: &Light) -> Color {
    let light_direction = (light.position - intersect.point).normalize();
    let view_direction = (ray_origin - intersect.point).normalize();

    let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
    let diffuse = intersect.material.diffuse
        * (diffuse_intensity * intersect.material.albedo[0] * light.intensity);

    let reflect_direction = reflect(&-light_direction, &intersect.normal);
    let specular_intensity = dot(&view_direction, &reflect_direction)
        .max(0.0)
        .powf(intersect.material.specular);

    let specular =
        light.color * (specular_intensity * intersect.material.albedo[1] * light.intensity);

    diffuse + specular
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
) -> Color {
    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest.is_none_or(|current| intersect.distance < current.distance) {
                closest = Some(intersect);
            }
        }
    }

    match closest {
        Some(intersect) => shade(&intersect, ray_origin, light),
        None => Color::from_hex(BACKGROUND_COLOR),
    }
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
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

                        *pixel = cast_ray(&camera.eye, &ray_direction, objects, light).to_hex();
                    }
                }
            });
        }
    });
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new("Cube", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let ivory = Material::new(Color::new(100, 100, 80), 50.0, [0.6, 0.3]);
    let _rubber = Material::new(Color::new(80, 0, 0), 10.0, [0.9, 0.1]);
    let _cobalt = Material::new(Color::new(40, 80, 140), 80.0, [0.7, 0.4]);
    let _jade = Material::new(Color::new(60, 130, 100), 30.0, [0.8, 0.25]);

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(Cube::new(Vec3::new(0.0, 0.0, 0.0), 1.5, ivory)),
    ];

    let light = Light::new(Vec3::new(-6.0, 6.0, 8.0), Color::new(255, 255, 255), 1.5);

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_moved = true;

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
            render(&mut framebuffer, &objects, &camera, &light);
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}