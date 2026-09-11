use crate::color::Color;
use std::fmt;

#[derive(Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl Texture {
    /// Carga una textura desde un archivo de imagen (PNG, JPG, etc.).
    /// Si el archivo no existe o falla la carga, genera una textura placeholder visible de respaldo.
    pub fn new(path: &str) -> Self {
        match image::open(path) {
            Ok(img) => {
                let rgb = img.to_rgb8();
                let (width, height) = rgb.dimensions();
                let buffer = rgb
                    .pixels()
                    .map(|p| {
                        let [r, g, b] = p.0;
                        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                    })
                    .collect();
                Texture {
                    width: width as usize,
                    height: height as usize,
                    buffer,
                }
            }
            Err(err) => {
                eprintln!(
                    "Aviso: No se pudo cargar la textura desde '{}' ({}). Usando textura placeholder temporal.",
                    path, err
                );
                Self::placeholder(64, 64)
            }
        }
    }

    /// Genera una textura placeholder de tablero (útil cuando la ruta aún no tiene un archivo físico)
    pub fn placeholder(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        let block_size = 8;
        for y in 0..height {
            for x in 0..width {
                let is_even = ((x / block_size) + (y / block_size)) % 2 == 0;
                let color = if is_even {
                    0xD4AF37 // Dorado / Ocre
                } else {
                    0x2A2A2A // Gris oscuro / Carbón
                };
                buffer.push(color);
            }
        }
        Texture {
            width,
            height,
            buffer,
        }
    }

    /// Retorna el valor hexadecimal de un píxel en coordenadas de imagen enteras
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if self.width == 0 || self.height == 0 || self.buffer.is_empty() {
            return 0xFFFFFF;
        }
        let px = x.min(self.width - 1);
        let py = y.min(self.height - 1);
        self.buffer[py * self.width + px]
    }

    /// Muestrea la textura usando coordenadas normalizadas (u, v) en el rango [0.0, 1.0].
    /// Aplica wrapping seguro para coordenadas de cualquier forma 3D.
    pub fn get_color(&self, u: f32, v: f32) -> Color {
        if self.width == 0 || self.height == 0 || self.buffer.is_empty() {
            return Color::new(255, 255, 255);
        }

        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);

        let x = ((u * (self.width - 1) as f32).round() as usize).min(self.width - 1);
        let y = ((v * (self.height - 1) as f32).round() as usize).min(self.height - 1);

        Color::from_hex(self.get_pixel(x, y))
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Texture({}x{}, {} pixels)", self.width, self.height, self.buffer.len())
    }
}
