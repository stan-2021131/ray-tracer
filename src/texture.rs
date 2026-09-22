use crate::color::Color;
use std::fmt;

#[derive(Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Color>,
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
                        Color::new(r, g, b)
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
        let gold = Color::from_hex(0xD4AF37);
        let dark = Color::from_hex(0x2A2A2A);

        for y in 0..height {
            for x in 0..width {
                let is_even = ((x / block_size) + (y / block_size)) % 2 == 0;
                let color = if is_even { gold } else { dark };
                buffer.push(color);
            }
        }
        Texture {
            width,
            height,
            buffer,
        }
    }

    /// Retorna el color de un píxel en coordenadas de imagen enteras
    #[allow(dead_code)]
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if self.width == 0 || self.height == 0 || self.buffer.is_empty() {
            return Color::new(255, 255, 255);
        }
        let px = x.min(self.width - 1);
        let py = y.min(self.height - 1);
        self.buffer[py * self.width + px]
    }

    /// Muestrea la textura usando coordenadas normalizadas (u, v) en el rango [0.0, 1.0] o repetidas si u,v > 1.0.
    #[inline]
    pub fn get_color(&self, u: f32, v: f32) -> Color {
        self.get_color_nearest_wrap(u, v, TextureWrap::Repeat)
    }

    /// Muestreo por vecino más cercano (Nearest Neighbor Sampling) con modo de envoltura UV.
    #[inline]
    pub fn get_color_nearest(&self, u: f32, v: f32) -> Color {
        self.get_color_nearest_wrap(u, v, TextureWrap::Repeat)
    }

    /// Muestreo por vecino más cercano con envoltura UV configurable (Repeat o Clamp).
    #[inline]
    pub fn get_color_nearest_wrap(&self, u: f32, v: f32, wrap: TextureWrap) -> Color {
        if self.width == 0 || self.height == 0 || self.buffer.is_empty() {
            return Color::new(255, 255, 255);
        }

        let u_wrapped = wrap_coord(u, wrap);
        let v_wrapped = wrap_coord(v, wrap);

        let max_x = self.width - 1;
        let max_y = self.height - 1;

        let x = ((u_wrapped * max_x as f32).round() as usize).min(max_x);
        let y = ((v_wrapped * max_y as f32).round() as usize).min(max_y);

        self.buffer[y * self.width + x]
    }

    /// Muestreo por interpolación bilineal (Bilinear Texture Filtering) con modo de envoltura UV.
    #[inline]
    pub fn get_color_bilinear(&self, u: f32, v: f32) -> Color {
        self.get_color_bilinear_wrap(u, v, TextureWrap::Repeat)
    }

    /// Muestreo bilineal con envoltura UV configurable (Repeat o Clamp).
    #[inline]
    pub fn get_color_bilinear_wrap(&self, u: f32, v: f32, wrap: TextureWrap) -> Color {
        if self.width == 0 || self.height == 0 || self.buffer.is_empty() {
            return Color::new(255, 255, 255);
        }

        let max_x = self.width - 1;
        let max_y = self.height - 1;

        let u_wrapped = wrap_coord(u, wrap);
        let v_wrapped = wrap_coord(v, wrap);

        let gx = u_wrapped * max_x as f32;
        let gy = v_wrapped * max_y as f32;

        let x0 = (gx.floor() as usize).min(max_x);
        let x1 = (x0 + 1).min(max_x);
        let y0 = (gy.floor() as usize).min(max_y);
        let y1 = (y0 + 1).min(max_y);

        let fx = gx - gx.floor();
        let fy = gy - gy.floor();

        // Acceso directo a los 4 texels vecinos
        let row0_offset = y0 * self.width;
        let row1_offset = y1 * self.width;

        let c00 = self.buffer[row0_offset + x0];
        let c10 = self.buffer[row0_offset + x1];
        let c01 = self.buffer[row1_offset + x0];
        let c11 = self.buffer[row1_offset + x1];

        // Interpolación por canal en punto flotante
        let one_minus_fx = 1.0 - fx;
        let one_minus_fy = 1.0 - fy;

        let r_top = c00.r as f32 * one_minus_fx + c10.r as f32 * fx;
        let r_bot = c01.r as f32 * one_minus_fx + c11.r as f32 * fx;
        let r = (r_top * one_minus_fy + r_bot * fy).round().clamp(0.0, 255.0) as u8;

        let g_top = c00.g as f32 * one_minus_fx + c10.g as f32 * fx;
        let g_bot = c01.g as f32 * one_minus_fx + c11.g as f32 * fx;
        let g = (g_top * one_minus_fy + g_bot * fy).round().clamp(0.0, 255.0) as u8;

        let b_top = c00.b as f32 * one_minus_fx + c10.b as f32 * fx;
        let b_bot = c01.b as f32 * one_minus_fx + c11.b as f32 * fx;
        let b = (b_top * one_minus_fy + b_bot * fy).round().clamp(0.0, 255.0) as u8;

        Color::new(r, g, b)
    }

    /// Muestrea la textura utilizando el modo de filtrado y envoltura indicados.
    #[inline]
    pub fn get_color_filtered(&self, u: f32, v: f32, filter: TextureFilter) -> Color {
        match filter {
            TextureFilter::Nearest => self.get_color_nearest(u, v),
            TextureFilter::Bilinear => self.get_color_bilinear(u, v),
        }
    }

    /// Muestrea la textura con filtrado y modo de envoltura explícitos.
    #[inline]
    pub fn get_color_wrapped(&self, u: f32, v: f32, wrap: TextureWrap, filter: TextureFilter) -> Color {
        match filter {
            TextureFilter::Nearest => self.get_color_nearest_wrap(u, v, wrap),
            TextureFilter::Bilinear => self.get_color_bilinear_wrap(u, v, wrap),
        }
    }
}

/// Aplica la envoltura (Repeat o Clamp) a una coordenada UV continua.
#[inline(always)]
pub fn wrap_coord(val: f32, mode: TextureWrap) -> f32 {
    match mode {
        TextureWrap::Repeat => val.rem_euclid(1.0),
        TextureWrap::Clamp => val.clamp(0.0, 1.0),
    }
}

/// Modo de envoltura para coordenadas de textura (UV Wrapping).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureWrap {
    /// Repite la textura en mosaico continuo (por defecto para texturas de superficies).
    #[default]
    Repeat,
    /// Clampa/estira los bordes de la textura entre 0.0 y 1.0.
    Clamp,
}

/// Modo de filtrado para el muestreo de texturas y skyboxes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFilter {
    /// Vecino más cercano: óptimo para Pixel Art y alto rendimiento (por defecto).
    #[default]
    Nearest,
    /// Interpolación bilineal: óptimo para texturas de alta resolución y degradados continuos.
    Bilinear,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Texture({}x{}, {} pixels)", self.width, self.height, self.buffer.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_repeat_and_clamp() {
        assert_eq!(wrap_coord(0.25, TextureWrap::Repeat), 0.25);
        assert!((wrap_coord(1.25, TextureWrap::Repeat) - 0.25).abs() < 1e-5);
        assert!((wrap_coord(2.75, TextureWrap::Repeat) - 0.75).abs() < 1e-5);

        // Clamp
        assert_eq!(wrap_coord(1.5, TextureWrap::Clamp), 1.0);
        assert_eq!(wrap_coord(-0.5, TextureWrap::Clamp), 0.0);
    }
}
