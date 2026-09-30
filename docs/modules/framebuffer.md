# Framebuffer

## ¿Para qué sirve?

El framebuffer es la "pantalla virtual" del motor: un arreglo lineal en memoria donde cada celda guarda el color de un píxel. Al final de cada fotograma, este buffer se vuelca a la ventana real. Conceptualmente es el equivalente en software de un framebuffer de GPU.

---

## Estructura y layout de memoria

```rust
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,   // Arreglo lineal de píxeles en formato 0x00RRGGBB
    background_color: u32,
    current_color: u32,
}
```

Los píxeles se almacenan en **orden row-major** (fila por fila, de arriba a abajo). La conversión de coordenadas 2D a índice 1D sigue la fórmula:

```
índice = y × width + x
```

Esto significa que píxeles adyacentes en la misma fila están contiguos en memoria (acceso secuencial eficiente para el caché de CPU). Píxeles de filas distintas pero misma columna están separados por `width` enteros.

---

## `Framebuffer::new(width, height)`

Reserva `width × height` enteros de 32 bits, inicializados en negro:

```rust
buffer: vec![0; width * height],
background_color: 0x000000,
current_color: 0xFFFFFF,
```

Para una resolución 800×600, el buffer ocupa `800 × 600 × 4 bytes = 1.92 MB`.

---

## `clear()`

Rellena todo el buffer con el color de fondo en una sola operación de memoria:

```rust
self.buffer.fill(self.background_color);
```

`fill` es equivalente a `memset` en C y está optimizado por el compilador para usar instrucciones SIMD cuando es posible, haciendo la limpieza del buffer prácticamente instantánea.

---

## `point(x, y)`

Escribe un píxel individual. Incluye verificación de límites implícita: si `x ≥ width` o `y ≥ height`, la operación se descarta sin pánico (los tipos son `usize`, así que no pueden ser negativos):

```rust
if x < self.width && y < self.height {
    self.buffer[y * self.width + x] = self.current_color;
}
```

---

## `render_parallel<F>(render_pixel: F)` ⭐

Esta es la función más importante del módulo. Distribuye el cálculo de rayos entre todos los núcleos de CPU disponibles procesando franjas horizontales del buffer en paralelo.

### Signatura del closure

```rust
F: Fn(usize, usize) -> u32 + Sync + Send
```

El closure recibe `(x, y)` y devuelve el color del píxel como `u32`. Debe ser `Sync + Send` porque múltiples hilos lo invocan concurrentemente.

### Algoritmo de partición

Se divide el buffer en **chunks de filas** de tamaño proporcional al número de núcleos detectados:

```rust
let num_threads = std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(4);

let rows_per_chunk = (height + num_threads - 1) / num_threads;  // ceil(height / N)
let chunk_size = rows_per_chunk * width;
```

La fórmula `(height + N - 1) / N` es la división entera con redondeo hacia arriba (equivalente a `ceil(height/N)` sin usar flotantes). Esto garantiza que todas las filas queden cubiertas aunque `height` no sea divisible exactamente entre el número de hilos.

### Ejecución con scoped threads

```rust
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
```

`chunks_mut` divide el buffer en segmentos disjuntos y mutuamente exclusivos. Cada hilo trabaja sobre su propio segmento, lo que hace que el acceso sea **libre de condiciones de carrera** sin necesidad de `Mutex` ni `Arc`. `thread::scope` garantiza que todos los hilos terminan antes de que la función retorne, por lo que no hay posibilidad de referencias colgantes.

### Ganancia de rendimiento

Sin paralelismo, 480,000 píxeles (800×600) se calculan secuencialmente. Con 8 núcleos, cada hilo procesa ~60,000 píxeles. El speedup teórico es proporcional al número de núcleos (Ley de Amdahl, asumiendo que el cuello de botella es el trazado de rayos).

---

## Relación con el resto del motor

```
renderer::render()
    └── framebuffer.render_parallel(|x, y| {
            let ray_dir = camera.basis_change(&Vec3::new(screen_x, screen_y, -1.0));
            let color = cast_ray(&camera.eye, &ray_dir, ...);
            color.to_hex()         // → u32 almacenado en buffer[y * width + x]
        });
```
