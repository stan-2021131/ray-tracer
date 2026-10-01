use crate::box3d::Box3D;
use crate::color::Color;
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

    /// Actualiza todas las animaciones de materiales en los objetos de la escena para un tiempo dado (en segundos).
    pub fn update_time(&mut self, time: f32) {
        for obj in &mut self.objects {
            obj.update_time(time);
        }
    }

    /// Retorna si la escena contiene al menos un objeto con animación de textura.
    pub fn has_animations(&self) -> bool {
        self.objects.iter().any(|obj| obj.is_animated())
    }
}

/// Escena diorama exterior completa (estanque, muelle, fogata, telescopio, silla, caminos, lámparas, árboles y cerca).
pub fn diorama_scene() -> Scene {
    crate::diorama_builder::build_diorama_scene()
}

/// Escena espacial con planetas, luna, estrellas y naves observadas por el telescopio.
pub fn space_scene() -> Scene {
    crate::space_builder::build_space_scene()
}

/// Escena técnica demostrativa: muestra las capacidades del motor (materiales, reflexión, refracción, texturas)
/// usando esferas, cubos y planos en materiales variados. No depende de los assets del diorama.
///
/// Cámara sugerida: eye = (0.0, 4.0, 10.0), center = (0.0, 0.0, 0.0).
#[allow(dead_code)]
pub fn demo_scene() -> Scene {
    let mut objects: Vec<Object> = Vec::new();

    // --- Plano: suelo (jade difuso) ---
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        30.0,
        30.0,
        materials::jade(Color::new(60, 130, 100)),
    )));

    // --- Esferas: demostración de materiales especulares e ópticos ---
    // Espejo reflectivo (izquierda)
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(-3.0, 0.0, 0.0),
        1.0,
        materials::mirror(Color::new(240, 240, 255)),
    )));
    // Vidrio refractivo con Fresnel (centro)
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        1.0,
        materials::glass(Color::new(200, 220, 255)),
    )));
    // Oro pulido (derecha)
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(3.0, 0.0, 0.0),
        1.0,
        materials::gold(Color::new(255, 215, 0)),
    )));

    // --- Box3D: demostración de cubos con materiales sólidos ---
    // Cobalto metálico (fondo izquierda)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(-3.5, -0.25, -3.5),
        Vec3::new(1.5, 1.5, 1.5),
        materials::cobalt(Color::new(40, 80, 140)),
    )));
    // Marfil brillante (fondo derecha)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(3.5, -0.25, -3.5),
        Vec3::new(1.5, 1.5, 1.5),
        materials::ivory(Color::new(220, 200, 170)),
    )));

    // --- Box3D: muro de fondo texturizado (demuestra mapping UV con textura) ---
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 0.75, -6.5),
        Vec3::new(10.0, 3.0, 0.3),
        materials::apply_texture(
            materials::diffuse(Color::new(255, 255, 255)),
            "./textures/wall.png",
        ),
    )));

    let lights = vec![
        Light::new(Vec3::new(5.0, 8.0, 8.0), Color::new(255, 255, 255), 1.2),
        Light::new(Vec3::new(-6.0, 5.0, 3.0), Color::new(200, 220, 255), 0.7),
        Light::new(Vec3::new(0.0, 4.0, 5.0), Color::new(255, 180, 100), 0.5),
    ];

    Scene::new(objects, lights)
}
