
use crate::texture::Texture;
use std::sync::Arc;

/// Administrador centralizado de recursos de texturas e imágenes de la aplicación.
/// Carga, almacena en caché y distribuye instancias `Arc<Texture>` para evitar duplicaciones en memoria.
#[allow(dead_code)]
#[derive(Clone)]
pub struct TextureManager {
    pub grass: Arc<Texture>,
    pub dirt: Arc<Texture>,
    pub path: Arc<Texture>,
    pub water: Arc<Texture>,
    pub stone: Arc<Texture>,
    pub wall: Arc<Texture>,
    pub wood: Arc<Texture>,
    pub wood_2: Arc<Texture>,
    pub leaves: Arc<Texture>,
    pub window: Arc<Texture>,
    pub door: Arc<Texture>,
    pub plants: Vec<Arc<Texture>>,
    pub fire_frames: Vec<Arc<Texture>>,
    pub smoke_frames: Vec<Arc<Texture>>,
}

impl TextureManager {
    /// Carga todo el repertorio de texturas del proyecto desde disco.
    pub fn new() -> Self {
        let grass = Arc::new(Texture::new("./textures/grass.png"));
        let dirt = Arc::new(Texture::new("./textures/dirt.png"));
        let path = Arc::new(Texture::new("./textures/path.png"));
        let water = Arc::new(Texture::new("./textures/water.jpg"));
        let stone = Arc::new(Texture::new("./textures/stone.png"));
        let wall = Arc::new(Texture::new("./textures/wall.png"));
        let wood = Arc::new(Texture::new("./textures/wood.png"));
        let wood_2 = Arc::new(Texture::new("./textures/wood_2.png"));
        let leaves = Arc::new(Texture::new("./textures/leaves.png"));
        let window = Arc::new(Texture::new("./textures/window.png"));
        let door = Arc::new(Texture::new("./textures/door.png"));

        // Carga de la colección de plantas (1.png a 7.png)
        let plant_paths = [
            "./textures/plants/1.png",
            "./textures/plants/2.png",
            "./textures/plants/3.png",
            "./textures/plants/4.png",
            "./textures/plants/5.png",
            "./textures/plants/6.png",
            "./textures/plants/7.png",
        ];
        let plants: Vec<Arc<Texture>> = plant_paths
            .iter()
            .map(|p| Arc::new(Texture::new(p)))
            .collect();

        // Carga de la secuencia de cuadros de fuego para la fogata (1.png a 8.png)
        let fire_paths = [
            "./textures/campfire/fire/1.png",
            "./textures/campfire/fire/2.png",
            "./textures/campfire/fire/3.png",
            "./textures/campfire/fire/4.png",
            "./textures/campfire/fire/5.png",
            "./textures/campfire/fire/6.png",
            "./textures/campfire/fire/7.png",
            "./textures/campfire/fire/8.png",
        ];
        let fire_frames: Vec<Arc<Texture>> = fire_paths
            .iter()
            .map(|p| Arc::new(Texture::new(p)))
            .collect();

        // Carga de la secuencia de cuadros de humo para la fogata (1.png a 8.png)
        let smoke_paths = [
            "./textures/campfire/smoke/1.png",
            "./textures/campfire/smoke/2.png",
            "./textures/campfire/smoke/3.png",
            "./textures/campfire/smoke/4.png",
            "./textures/campfire/smoke/5.png",
            "./textures/campfire/smoke/6.png",
            "./textures/campfire/smoke/7.png",
            "./textures/campfire/smoke/8.png",
        ];
        let smoke_frames: Vec<Arc<Texture>> = smoke_paths
            .iter()
            .map(|p| Arc::new(Texture::new(p)))
            .collect();

        TextureManager {
            grass,
            dirt,
            path,
            water,
            stone,
            wall,
            wood,
            wood_2,
            leaves,
            window,
            door,
            plants,
            fire_frames,
            smoke_frames,
        }
    }
}
