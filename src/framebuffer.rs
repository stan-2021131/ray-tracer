#[allow(dead_code)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

#[allow(dead_code)]
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
    }

    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    /// Renderiza píxeles en paralelo distribuyendo las filas entre los hilos disponibles.
    pub fn render_parallel<F>(&mut self, render_pixel: F)
    where
        F: Fn(usize, usize) -> u32 + Sync + Send,
    {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let width = self.width;
        let height = self.height;
        let rows_per_chunk = (height + num_threads - 1) / num_threads;
        let chunk_size = rows_per_chunk * width;
        let render_pixel_ref = &render_pixel;

        std::thread::scope(|s| {
            for (chunk_idx, chunk) in self.buffer.chunks_mut(chunk_size).enumerate() {
                let start_row = chunk_idx * rows_per_chunk;
                s.spawn(move || {
                    let chunk_rows = chunk.len() / width;
                    for local_y in 0..chunk_rows {
                        let y = start_row + local_y;
                        for x in 0..width {
                            chunk[local_y * width + x] = render_pixel_ref(x, y);
                        }
                    }
                });
            }
        });
    }
}