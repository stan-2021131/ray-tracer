
use crate::box3d::Box3D;
use crate::color::Color;
use crate::light::Light;
use crate::materials;
use crate::plane::Plane;
use crate::ray_intersect::{Material, Object};
use crate::scene::Scene;
use crate::sphere::Sphere;
use crate::texture::TextureWrap;
use crate::texture_manager::TextureManager;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

/// Tamaño universal de bloque para la cuadrícula modular del diorama.
pub const BLOCK_SIZE: f32 = 1.0;

/// Colección de materiales reutilizables para los elementos del diorama.
pub struct DioramaMaterials {
    pub grass: Material,
    pub dirt: Material,
    pub path: Material,
    pub water: Material,
    pub rock: Material,
    pub wood_dark: Material,
    pub wood_light: Material,
    pub foliage_main: Material,
    pub lamp_metal: Material,
    pub lamp_bulb: Material,
    pub fire: Material,
    pub smoke: Material,
    pub plants: Vec<Material>,
    pub telescope_body: Material,
    pub telescope_joint: Material,
    pub cabin_foundation: Material,
    pub cabin_placeholder: Material,
    pub cabin_roof: Material,
    pub cabin_window: Material,
    pub cabin_door: Material,
}

impl DioramaMaterials {
    pub fn new() -> Self {
        let tm = TextureManager::new();
        Self::from_texture_manager(&tm)
    }

    pub fn from_texture_manager(tm: &TextureManager) -> Self {
        let grass = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.grass.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let dirt = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.dirt.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let path = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.path.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let water = materials::water(Color::new(85, 185, 230));

        let rock = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.stone.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let wood_dark = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.wood.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let wood_light = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.wood_2.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let foliage_main = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.leaves.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);

        let lamp_metal = materials::cobalt(Color::new(45, 45, 50));
        let lamp_bulb = materials::glass(Color::new(255, 235, 140)).with_emissive(1.6);

        let fire = materials::emissive(Color::new(255, 255, 255), 1.8)
            .with_animated_textures(tm.fire_frames.clone(), 10.0);

        let smoke = materials::diffuse(Color::new(220, 220, 220))
            .with_animated_textures(tm.smoke_frames.clone(), 8.0);

        let plants: Vec<Material> = tm
            .plants
            .iter()
            .map(|tex| {
                materials::diffuse(Color::new(255, 255, 255)).with_texture(tex.clone())
            })
            .collect();

        let telescope_body = materials::cobalt(Color::new(65, 75, 90));
        let telescope_joint = materials::cobalt(Color::new(35, 35, 40));
        let cabin_foundation = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.wood.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);
        let cabin_placeholder = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.wall.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);
        let cabin_roof = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.wood_2.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Repeat);
        let cabin_window = materials::reflective_window(Color::new(255, 255, 255))
            .with_texture(tm.window.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Clamp);
        let cabin_door = materials::diffuse(Color::new(255, 255, 255))
            .with_texture(tm.door.clone())
            .with_uv_scale(1.0, 1.0)
            .with_wrap(TextureWrap::Clamp);

        Self {
            grass,
            dirt,
            path,
            water,
            rock,
            wood_dark,
            wood_light,
            foliage_main,
            lamp_metal,
            lamp_bulb,
            fire,
            smoke,
            plants,
            telescope_body,
            telescope_joint,
            cabin_foundation,
            cabin_placeholder,
            cabin_roof,
            cabin_window,
            cabin_door,
        }
    }
}

/// Estilos de copa para diversificar el follaje de los árboles cúbicos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeCanopyStyle {
    /// Roble estándar: copa en 2 niveles (base 3x3 + cúspide 1x1).
    Standard,
    /// Roble frondoso / escalonado: copa en 3 niveles (base 4x4 + medio 3x3 + cúspide 1x1).
    Full,
    /// Copa cónica / estilizada: copa en 3 niveles (base 3x3 + medio 2x2 + cúspide 1x1).
    Conical,
}

/// Construye un árbol cúbico modular con altura de tronco y estilo de copa configurables.
pub fn add_tree(
    objects: &mut Vec<Object>,
    position: Vec3,
    height_blocks: f32,
    style: TreeCanopyStyle,
    mats: &DioramaMaterials,
) {
    let b = BLOCK_SIZE;
    let trunk_w = 1.0 * b;
    let trunk_h = height_blocks * b;
    let trunk_center_y = position.y + (trunk_h / 2.0);

    // Tronco principal de madera
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(position.x, trunk_center_y, position.z),
        Vec3::new(trunk_w, trunk_h, trunk_w),
        mats.wood_dark.clone(),
    )));

    // Follaje según la variante elegida
    match style {
        TreeCanopyStyle::Standard => {
            // Nivel inferior (3x1x3 bloques)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 0.5 * b, position.z),
                Vec3::new(3.0 * b, 1.0 * b, 3.0 * b),
                mats.foliage_main.clone(),
            )));
            // Nivel superior (1x1x1 bloque)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 1.5 * b, position.z),
                Vec3::new(1.0 * b, 1.0 * b, 1.0 * b),
                mats.foliage_main.clone(),
            )));
        }
        TreeCanopyStyle::Full => {
            // Nivel inferior amplio (4x1x4 bloques)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 0.5 * b, position.z),
                Vec3::new(4.0 * b, 1.0 * b, 4.0 * b),
                mats.foliage_main.clone(),
            )));
            // Nivel medio (3x1x3 bloques)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 1.5 * b, position.z),
                Vec3::new(3.0 * b, 1.0 * b, 3.0 * b),
                mats.foliage_main.clone(),
            )));
            // Cúspide superior (1x1x1 bloque)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 2.5 * b, position.z),
                Vec3::new(1.0 * b, 1.0 * b, 1.0 * b),
                mats.foliage_main.clone(),
            )));
        }
        TreeCanopyStyle::Conical => {
            // Nivel inferior (3x1x3 bloques)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 0.5 * b, position.z),
                Vec3::new(3.0 * b, 1.0 * b, 3.0 * b),
                mats.foliage_main.clone(),
            )));
            // Nivel medio angosto (2x1x2 bloques)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 1.5 * b, position.z),
                Vec3::new(2.0 * b, 1.0 * b, 2.0 * b),
                mats.foliage_main.clone(),
            )));
            // Cúspide superior (1x1x1 bloque)
            objects.push(Object::Box3D(Box3D::new(
                Vec3::new(position.x, position.y + trunk_h + 2.5 * b, position.z),
                Vec3::new(1.0 * b, 1.0 * b, 1.0 * b),
                mats.foliage_main.clone(),
            )));
        }
    }
}

/// Añade una farola con poste vertical metálico y bombilla esférica emisiva.
pub fn add_lamp(
    objects: &mut Vec<Object>,
    lights: &mut Vec<Light>,
    position: Vec3,
    has_real_light: bool,
    mats: &DioramaMaterials,
) {
    let b = BLOCK_SIZE;
    let post_h = 3.0 * b;
    let post_w = 0.3 * b;

    // Poste vertical
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(position.x, position.y + post_h / 2.0, position.z),
        Vec3::new(post_w, post_h, post_w),
        mats.lamp_metal.clone(),
    )));

    // Bombilla esférica superior (posicionada sobre el poste)
    let bulb_radius = 0.35 * b;
    let bulb_y = position.y + post_h + bulb_radius;
    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(position.x, bulb_y, position.z),
        bulb_radius,
        mats.lamp_bulb.clone(),
    )));

    // Fuente de luz puntual cálida (ubicada en el centro de la bombilla de vidrio transparente para irradiar 360°)
    if has_real_light {
        lights.push(Light::new(
            Vec3::new(position.x, bulb_y, position.z),
            Color::new(255, 225, 130),
            0.45,
        ));
    }
}

/// Añade una roca posicionada sobre la superficie del agua del río.
pub fn add_rock(
    objects: &mut Vec<Object>,
    water_surface_pos: Vec3,
    radius: f32,
    scale_y: f32,
    mats: &DioramaMaterials,
) {
    let center_y = water_surface_pos.y + radius * (scale_y * 0.35);

    objects.push(Object::Sphere(Sphere::new(
        Vec3::new(water_surface_pos.x, center_y, water_surface_pos.z),
        radius,
        mats.rock.clone(),
    )));
}

/// Añade el puente de madera sobre el cauce del río con columnas cúbicas en las esquinas.
pub fn add_bridge(
    objects: &mut Vec<Object>,
    position: Vec3,
    size: Vec3,
    mats: &DioramaMaterials,
) {
    let b = BLOCK_SIZE;

    // Superficie principal del puente
    objects.push(Object::Box3D(Box3D::new(
        position,
        size,
        mats.wood_light.clone(),
    )));

    // 4 Columnas de soporte/barandal en las esquinas
    let cube_size = Vec3::new(1.0 * b, 1.0 * b, 1.0 * b);
    let half_w = (size.x / 2.0) - (0.5 * b);
    let half_d = (size.z / 2.0) - (0.5 * b);
    let cube_y = position.y + (size.y / 2.0) + (0.5 * b);

    let cube_offsets = [
        (-half_w, -half_d),
        (half_w, -half_d),
        (-half_w, half_d),
        (half_w, half_d),
    ];

    for &(ox, oz) in &cube_offsets {
        objects.push(Object::Box3D(Box3D::new(
            Vec3::new(position.x + ox, cube_y, position.z + oz),
            cube_size,
            mats.wood_light.clone(),
        )));
    }
}

/// Construye la cabaña completa: cimientos, paredes, techo escalonado piramidal, puerta y ventanas en planos.
pub fn add_cabin(
    objects: &mut Vec<Object>,
    center_xz: Vec3,
    ground_y: f32,
    mats: &DioramaMaterials,
) {
    let b = BLOCK_SIZE;
    let cx = center_xz.x;
    let cz = center_xz.z;

    // 1. Cimientos / Base de la cabaña (12 bloques de ancho x 1 de alto x 8 de profundidad)
    let foundation_w = 12.0 * b;
    let foundation_h = 1.0 * b;
    let foundation_d = 8.0 * b;
    let foundation_y = ground_y + (foundation_h / 2.0);

    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(cx, foundation_y, cz),
        Vec3::new(foundation_w, foundation_h, foundation_d),
        mats.cabin_foundation.clone(),
    )));

    let floor_y = ground_y + foundation_h;

    // 2. Estructura principal de paredes (8 bloques de ancho x 3 de alto x 6 de profundidad)
    let body_w = 8.0 * b;
    let body_h = 3.0 * b;
    let body_d = 6.0 * b;
    let body_center_y = floor_y + (body_h / 2.0);

    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(cx, body_center_y, cz),
        Vec3::new(body_w, body_h, body_d),
        mats.cabin_placeholder.clone(),
    )));

    // 3. Techo piramidal escalonado en 3 niveles de bloques
    let roof_base_y = floor_y + body_h;

    // Nivel 1 del techo (alero ancho que sobresale 1 bloque a cada lado: 10 x 1 x 8)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(cx, roof_base_y + 0.5 * b, cz),
        Vec3::new(10.0 * b, 1.0 * b, 8.0 * b),
        mats.cabin_roof.clone(),
    )));

    // Nivel 2 del techo (nivel medio: 8 x 1 x 6)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(cx, roof_base_y + 1.5 * b, cz),
        Vec3::new(8.0 * b, 1.0 * b, 6.0 * b),
        mats.cabin_roof.clone(),
    )));

    // Nivel 3 del techo (cúspide superior: 6 x 1 x 4)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(cx, roof_base_y + 2.5 * b, cz),
        Vec3::new(6.0 * b, 1.0 * b, 4.0 * b),
        mats.cabin_roof.clone(),
    )));

    // 4. Puerta frontal (Plano negro en la parte derecha de la fachada sur)
    let front_z = cz + (body_d / 2.0) + 0.005 * b;
    let door_w = 1.0 * b;
    let door_h = 2.0 * b;
    let door_x = cx + 1.5 * b;
    let door_y = floor_y + (door_h / 2.0);

    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(door_x, door_y, front_z),
        Vec3::new(0.0, 0.0, 1.0),
        door_w,
        door_h,
        mats.cabin_door.clone(),
    )));

    // 5. Ventana frontal (Plano azul cian en la parte izquierda de la fachada sur)
    let front_win_w = 3.0 * b;
    let front_win_h = 1.5 * b;
    let front_win_x = cx - 1.5 * b;
    let front_win_y = floor_y + 1.65 * b;

    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(front_win_x, front_win_y, front_z),
        Vec3::new(0.0, 0.0, 1.0),
        front_win_w,
        front_win_h,
        mats.cabin_window.clone(),
    )));

    // 6. Ventanas laterales (2 planos en cada pared lateral este y oeste)
    let side_win_w = 1.5 * b;
    let side_win_h = 1.5 * b;
    let side_win_y = floor_y + 1.65 * b;
    let win_offset_z = 1.5 * b;

    // Pared Oeste
    let west_x = cx - (body_w / 2.0) - 0.005 * b;
    for &z_sign in &[-1.0, 1.0] {
        objects.push(Object::Plane(Plane::new_with_normal(
            Vec3::new(west_x, side_win_y, cz + z_sign * win_offset_z),
            Vec3::new(-1.0, 0.0, 0.0),
            side_win_w,
            side_win_h,
            mats.cabin_window.clone(),
        )));
    }

    // Pared Este
    let east_x = cx + (body_w / 2.0) + 0.005 * b;
    for &z_sign in &[-1.0, 1.0] {
        objects.push(Object::Plane(Plane::new_with_normal(
            Vec3::new(east_x, side_win_y, cz + z_sign * win_offset_z),
            Vec3::new(1.0, 0.0, 0.0),
            side_win_w,
            side_win_h,
            mats.cabin_window.clone(),
        )));
    }
}

/// Añade la fogata con base de leños, planos cruzados de fuego animado y columna de humo.
pub fn add_campfire(
    objects: &mut Vec<Object>,
    lights: &mut Vec<Light>,
    position: Vec3,
    mats: &DioramaMaterials,
) {
    let b = BLOCK_SIZE;

    // Leños cruzados en la base
    let log_size = Vec3::new(1.5 * b, 0.22 * b, 0.22 * b);
    let log_rotations = [0.0, PI / 3.0, -PI / 3.0];
    for &rot_y in &log_rotations {
        objects.push(Object::Box3D(
            Box3D::new(
                Vec3::new(position.x, position.y + 0.15 * b, position.z),
                log_size,
                mats.wood_dark.clone(),
            )
            .with_rotation_y(rot_y),
        ));
    }

    // Planos cruzados de llamas animadas
    let flame_w = 1.3 * b;
    let flame_h = 1.4 * b;
    let flame_pos = Vec3::new(position.x, position.y + 0.75 * b, position.z);

    objects.push(Object::Plane(Plane::new_with_normal(
        flame_pos,
        Vec3::new(0.0, 0.0, 1.0),
        flame_w,
        flame_h,
        mats.fire.clone(),
    )));

    objects.push(Object::Plane(Plane::new_with_normal(
        flame_pos,
        Vec3::new(1.0, 0.0, 0.0),
        flame_w,
        flame_h,
        mats.fire.clone(),
    )));

    // Plano de humo animado superior
    let smoke_w = 1.4 * b;
    let smoke_h = 2.4 * b;
    let smoke_pos = Vec3::new(position.x, position.y + 2.0 * b, position.z);
    objects.push(Object::Plane(Plane::new_with_normal(
        smoke_pos,
        Vec3::new(0.0, 0.0, 1.0),
        smoke_w,
        smoke_h,
        mats.smoke.clone(),
    )));

    // Luz puntual cálida
    lights.push(Light::new(
        Vec3::new(position.x, position.y + 1.0 * b, position.z),
        Color::new(255, 140, 50),
        1.0,
    ));
}

/// Añade una planta vertical con textura de follaje y transparencia alpha recortada.
pub fn add_plant(
    objects: &mut Vec<Object>,
    position: Vec3,
    normal: Vec3,
    width: f32,
    height: f32,
    plant_idx: usize,
    mats: &DioramaMaterials,
) {
    if mats.plants.is_empty() {
        return;
    }
    let mat = mats.plants[plant_idx % mats.plants.len()].clone();
    let center = Vec3::new(position.x, position.y + height / 2.0, position.z);
    objects.push(Object::Plane(Plane::new_with_normal(
        center,
        normal,
        width,
        height,
        mat,
    )));
}

/// Añade el telescopio astronómico con trípode, rótula esférica, tubo óptico y ocular.
pub fn add_telescope(
    objects: &mut Vec<Object>,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    mats: &DioramaMaterials,
) {
    let b = BLOCK_SIZE;
    let joint_y = position.y + 1.3 * b;
    let joint_pos = Vec3::new(position.x, joint_y, position.z);

    // Patas del trípode (divergen desde la rótula central hacia el suelo)
    let leg_radius = 0.55 * b;
    let leg_dy = joint_y - (position.y + 0.03 * b);
    let tilt = (leg_radius / leg_dy).atan();

    for i in 0..3 {
        let angle = yaw + (i as f32) * (2.0 * PI / 3.0);
        let foot_x = position.x + leg_radius * angle.cos();
        let foot_z = position.z + leg_radius * angle.sin();
        let foot = Vec3::new(foot_x, position.y + 0.03 * b, foot_z);

        let leg_center = (foot + joint_pos) / 2.0;
        let leg_len = (joint_pos - foot).magnitude();

        let leg = Box3D::new(
            leg_center,
            Vec3::new(0.08 * b, leg_len, 0.08 * b),
            mats.telescope_body.clone(),
        )
        .with_rotation(Vec3::new(-tilt, (PI / 2.0) - angle, 0.0));
        objects.push(Object::Box3D(leg));
    }

    // Tubo óptico principal
    let tube_len = 1.8 * b;
    let tube_size = Vec3::new(0.2 * b, 0.2 * b, tube_len);
    let tube_rotation = Vec3::new(pitch, yaw, 0.0);
    let tube = Box3D::new(joint_pos, tube_size, mats.telescope_body.clone())
        .with_rotation(tube_rotation);
    objects.push(Object::Box3D(tube));

    // Dirección del eje local Z del tubo
    let (sx, cx) = pitch.sin_cos();
    let (sy, cy) = yaw.sin_cos();
    let dir_local_z = Vec3::new(sy * cx, -sx, cy * cx);

    // Mirilla / lente esférica en la punta superior que apunta al cielo
    let front_tip_pos = joint_pos - dir_local_z * (tube_len / 2.0 + 0.05 * b);
    objects.push(Object::Sphere(Sphere::new(
        front_tip_pos,
        0.13 * b,
        mats.telescope_joint.clone(),
    )));

    // Ocular posterior en la punta inferior
    let ocular_pos = joint_pos + dir_local_z * (tube_len / 2.0 + 0.12 * b);
    let ocular = Box3D::new(
        ocular_pos,
        Vec3::new(0.12 * b, 0.12 * b, 0.26 * b),
        mats.telescope_joint.clone(),
    )
    .with_rotation(tube_rotation);
    objects.push(Object::Box3D(ocular));
}

/// Construye la escena completa del diorama con terreno modular en bloques.
pub fn build_diorama_scene() -> Scene {
    let b = BLOCK_SIZE;
    let mats = DioramaMaterials::new();
    let mut objects: Vec<Object> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    // Dimensiones del terreno: 28 de ancho x 30 de profundidad total (zona norte ampliada a 16 bloques)
    let island_w = 28.0 * b;
    let bank_north_d = 16.0 * b; // Z de -2.0 a -18.0 (centro en -10.0)
    let river_d = 4.0 * b;       // Z de -2.0 a +2.0 (centro en 0.0)
    let bank_south_d = 10.0 * b; // Z de +2.0 a +12.0 (centro en +7.0)
    let total_island_d = bank_north_d + river_d + bank_south_d; // 30.0 bloques
    let island_center_z = -3.0 * b;

    let bank_h = 1.0 * b;
    let ground_y = 1.0 * b;

    // -------------------------------------------------------------
    // 1. TERRENO ESTRUCTURAL E ISLA FLOTANTE
    // -------------------------------------------------------------
    // Base inferior de la isla
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, -1.0 * b, island_center_z),
        Vec3::new(island_w, 2.0 * b, total_island_d),
        mats.dirt.clone(),
    )));

    // Orilla norte de tierra (ampliada)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 0.5 * b, -10.0 * b),
        Vec3::new(island_w, bank_h, bank_north_d),
        mats.dirt.clone(),
    )));

    // Superficie de pasto en la orilla norte (con repetición 1:1 por bloque)
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, ground_y + 0.005 * b, -10.0 * b),
        Vec3::new(0.0, 1.0, 0.0),
        island_w,
        bank_north_d,
        mats.grass.clone().with_uv_scale(island_w / b, bank_north_d / b),
    )));

    // Orilla sur de tierra
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 0.5 * b, 7.0 * b),
        Vec3::new(island_w, bank_h, bank_south_d),
        mats.dirt.clone(),
    )));

    // Superficie de pasto en la orilla sur (con repetición 1:1 por bloque)
    objects.push(Object::Plane(Plane::new_with_normal(
        Vec3::new(0.0, ground_y + 0.005 * b, 7.0 * b),
        Vec3::new(0.0, 1.0, 0.0),
        island_w,
        bank_south_d,
        mats.grass.clone().with_uv_scale(island_w / b, bank_south_d / b),
    )));

    // Lecho del río con volumen de agua (con micro-separación para evitar Z-fighting / puntos negros)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, 0.405 * b, 0.0),
        Vec3::new(island_w + 0.1 * b, 0.79 * b, river_d - 0.02 * b),
        mats.water.clone(),
    )));

    // -------------------------------------------------------------
    // 2. PUENTE SOBRE EL RÍO
    // -------------------------------------------------------------
    add_bridge(
        &mut objects,
        Vec3::new(0.0, ground_y + 0.1 * b, 0.0),
        Vec3::new(5.0 * b, 0.2 * b, 6.0 * b),
        &mats,
    );

    // -------------------------------------------------------------
    // 3. CAMINOS MODULARES
    // -------------------------------------------------------------
    // Camino norte (conecta el río con la entrada frontal de la cabaña)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, ground_y + 0.02 * b, -4.0 * b),
        Vec3::new(5.0 * b, 0.05 * b, 4.0 * b),
        mats.path.clone(),
    )));
    // Camino sur (abarca desde el río hasta el borde sur del escenario: Z = 2 a 12)
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(0.0, ground_y + 0.02 * b, 7.0 * b),
        Vec3::new(5.0 * b, 0.05 * b, 10.0 * b),
        mats.path.clone(),
    )));

    // -------------------------------------------------------------
    // 4. ÁREA DE LA CABAÑA (Estructura, techo piramidal, puerta y ventanas)
    // -------------------------------------------------------------
    add_cabin(
        &mut objects,
        Vec3::new(0.0, 0.0, -10.0 * b),
        ground_y,
        &mats,
    );

    // -------------------------------------------------------------
    // 5. CLARO DE FOGATA
    // -------------------------------------------------------------
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(9.0 * b, ground_y + 0.02 * b, -9.0 * b),
        Vec3::new(5.0 * b, 0.05 * b, 5.0 * b),
        mats.path.clone(),
    )));
    add_campfire(&mut objects, &mut lights, Vec3::new(9.0 * b, ground_y, -9.0 * b), &mats);

    // -------------------------------------------------------------
    // 6. PLATAFORMA DEL TELESCOPIO
    // -------------------------------------------------------------
    objects.push(Object::Box3D(Box3D::new(
        Vec3::new(9.0 * b, ground_y + 0.02 * b, 7.5 * b),
        Vec3::new(5.0 * b, 0.05 * b, 5.0 * b),
        mats.path.clone(),
    )));
    add_telescope(
        &mut objects,
        Vec3::new(9.0 * b, ground_y, 7.5 * b),
        -PI / 4.0,
        PI / 5.5,
        &mats,
    );

    // -------------------------------------------------------------
    // 7. ROCAS EN EL RÍO
    // -------------------------------------------------------------
    let water_surface_y = 0.8 * b;
    let rocks = [
        (-10.5 * b, -0.4 * b, 0.85 * b, 0.50),
        (-6.5 * b,   1.2 * b, 0.70 * b, 0.40),
        (-3.8 * b,   1.4 * b, 0.80 * b, 0.45),
        ( 3.8 * b,   1.4 * b, 0.80 * b, 0.50),
        ( 6.5 * b,  -0.5 * b, 0.75 * b, 0.45),
        (10.5 * b,   1.1 * b, 0.70 * b, 0.50),
    ];
    for &(rx, rz, radius, scale_y) in &rocks {
        add_rock(&mut objects, Vec3::new(rx, water_surface_y, rz), radius, scale_y, &mats);
    }

    // -------------------------------------------------------------
    // 8. ÁRBOLES CÚBICOS (Distribución en bosque norte, riberas y sur)
    // -------------------------------------------------------------
    let trees = [
        // Zona Norte ampliada (detrás y a los costados de la cabaña)
        (-11.0 * b, -10.0 * b, 4.0, TreeCanopyStyle::Full),     // Noroeste
        (-6.0 * b,  -15.5 * b, 4.0, TreeCanopyStyle::Conical),  // Detrás de cabaña (izq)
        ( 6.0 * b,  -15.5 * b, 5.0, TreeCanopyStyle::Full),     // Detrás de cabaña (der)
        (12.0 * b,  -14.0 * b, 5.0, TreeCanopyStyle::Full),     // Noreste
        // Riberas del río
        (-10.0 * b,  -3.5 * b, 5.0, TreeCanopyStyle::Conical),  // Ribera norte-oeste
        (  5.5 * b,  -3.5 * b, 3.0, TreeCanopyStyle::Standard), // Ribera norte-este
        (-10.0 * b,   7.0 * b, 4.0, TreeCanopyStyle::Full),     // Suroeste
        ( -5.0 * b,   3.5 * b, 3.0, TreeCanopyStyle::Standard), // Ribera sur-oeste
        (  5.0 * b,   3.5 * b, 4.0, TreeCanopyStyle::Conical),  // Ribera sur-este
        // Zona Sur
        ( -5.0 * b,   8.5 * b, 3.0, TreeCanopyStyle::Standard), // Sur profundo
        (  4.2 * b,   9.5 * b, 3.0, TreeCanopyStyle::Standard), // Entre camino sur y plataforma telescopio
    ];
    for &(tx, tz, h_blocks, style) in &trees {
        add_tree(&mut objects, Vec3::new(tx, ground_y, tz), h_blocks, style, &mats);
    }

    // -------------------------------------------------------------
    // 9. LÁMPARAS DE ILUMINACIÓN LOCAL
    // -------------------------------------------------------------
    let lamps = [
        (-3.2 * b, -4.0 * b, true),
        ( 3.2 * b, -4.0 * b, false),
        (-3.2 * b,  3.5 * b, false),
        ( 3.2 * b,  3.5 * b, true),
    ];
    for &(lx, lz, has_real_light) in &lamps {
        add_lamp(&mut objects, &mut lights, Vec3::new(lx, ground_y, lz), has_real_light, &mats);
    }

    // -------------------------------------------------------------
    // 10. VEGETACIÓN Y PLANTAS
    // -------------------------------------------------------------
    let plants_data = [
        // Ribera norte
        (-8.5 * b,  -2.3 * b, 0.9 * b, 0.9 * b, Vec3::new(0.0, 0.0, 1.0), 0),
        (-3.5 * b,  -2.2 * b, 0.8 * b, 0.8 * b, Vec3::new(0.3, 0.0, 0.9), 1),
        ( 4.0 * b,  -2.2 * b, 0.9 * b, 0.9 * b, Vec3::new(-0.2, 0.0, 0.9), 2),
        ( 9.5 * b,  -2.3 * b, 0.8 * b, 0.8 * b, Vec3::new(0.0, 0.0, 1.0), 3),
        // Ribera sur
        (-8.0 * b,   2.3 * b, 0.9 * b, 0.9 * b, Vec3::new(0.0, 0.0, 1.0), 4),
        (-3.5 * b,   2.2 * b, 0.8 * b, 0.8 * b, Vec3::new(-0.3, 0.0, 0.9), 5),
        ( 3.8 * b,   2.2 * b, 0.9 * b, 0.9 * b, Vec3::new(0.2, 0.0, 0.9), 6),
        ( 9.0 * b,   2.3 * b, 0.8 * b, 0.8 * b, Vec3::new(0.0, 0.0, 1.0), 0),
        // Entorno de la cabaña y bosque norte
        (-11.5 * b, -14.5 * b, 1.0 * b, 1.0 * b, Vec3::new(0.5, 0.0, 0.8), 1),
        ( 11.5 * b, -12.0 * b, 1.0 * b, 1.0 * b, Vec3::new(-0.4, 0.0, 0.9), 2),
        (  0.0 * b, -15.5 * b, 1.0 * b, 1.0 * b, Vec3::new(0.0, 0.0, 1.0), 3),
        // Entorno sur
        (-10.5 * b,   8.5 * b, 1.0 * b, 1.0 * b, Vec3::new(0.3, 0.0, 0.9), 3),
        (  6.5 * b,   8.5 * b, 0.9 * b, 0.9 * b, Vec3::new(0.0, 0.0, 1.0), 4),
    ];

    for &(px, pz, pw, ph, norm, idx) in &plants_data {
        add_plant(&mut objects, Vec3::new(px, ground_y, pz), norm, pw, ph, idx, &mats);
    }

    // -------------------------------------------------------------
    // 11. ILUMINACIÓN AMBIENTAL GENERAL
    // -------------------------------------------------------------
    lights.push(Light::new(
        Vec3::new(5.0 * b, 16.0 * b, 7.0 * b),
        Color::new(175, 205, 255),
        0.55,
    ));

    Scene::new(objects, lights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diorama_scene_counts() {
        let scene = build_diorama_scene();

        let primitive_count = scene.objects.len();
        println!("Total de primitivas creadas: {}", primitive_count);
        assert!(
            primitive_count >= 95 && primitive_count <= 115,
            "Cantidad de primitivas fuera de rango: {}",
            primitive_count
        );

        let light_count = scene.lights.len();
        println!("Total de luces reales creadas: {}", light_count);
        assert_eq!(light_count, 4, "Debe haber exactamente 4 luces reales");
    }
}
