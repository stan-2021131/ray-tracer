# Texture

## ¿Para qué sirve?

`Texture` carga imágenes desde disco y las expone como tablas de `Color` que los materiales muestrean durante el renderizado. Implementa dos métodos de filtrado (**Nearest Neighbor** y **Bilinear**) y dos modos de envoltura UV (**Repeat** y **Clamp**).

---

## Estructura y layout en memoria

```rust
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Color>,   // Buffer lineal row-major de texels
    pub has_alpha: bool,      // Algún píxel tiene alfa < 255
}
```

Los texels se almacenan en **orden row-major**: el texel `(x, y)` se accede como `buffer[y * width + x]`. Esto hace que texels adyacentes horizontalmente sean contiguos en memoria (favorable para el caché durante el muestreo secuencial de filas).

---

## `Texture::new(path)` — Carga desde disco

Usa la crate `image` para abrir cualquier formato común (PNG, JPG, BMP, etc.) y convierte todos a RGBA de 8 bits por canal. Durante la conversión detecta si algún píxel tiene `a < 255`:

```rust
let [r, g, b, a] = p.0;
if a < 255 { has_alpha = true; }
Color::new_rgba(r, g, b, a)
```

Si la ruta no existe o el archivo está corrupto, en lugar de fallar genera una **textura placeholder** de tablero dorado/oscuro de 64×64. Esto permite que el motor funcione durante el desarrollo aunque falten assets.

---

## Coordenadas UV

Las coordenadas de textura son flotantes normalizados `(u, v) ∈ [0, 1]`:
- `(0, 0)` → esquina superior izquierda
- `(1, 1)` → esquina inferior derecha

Para convertir de UV normalizado a píxel entero, se escala al rango `[0, max]` y se redondea:

```rust
let x = ((u_wrapped * max_x as f32).round() as usize).min(max_x);
let y = ((v_wrapped * max_y as f32).round() as usize).min(max_y);
```

El `.min(max_x)` previene el acceso fuera del buffer cuando `u_wrapped = 1.0` (que daría `x = width` en vez de `width - 1`).

---

## Modos de envoltura — `wrap_coord(val, mode)`

Transforma una coordenada UV que puede estar fuera de `[0, 1]` al rango válido.

### `TextureWrap::Repeat`

Usa el módulo euclidiano (siempre positivo, a diferencia del `%` de C):

```rust
val.rem_euclid(1.0)
```

`rem_euclid(1.0)` devuelve la parte fraccionaria de `val` siempre en `[0, 1)`, independientemente del signo. Así `u = 1.5` → `0.5`, `u = 2.75` → `0.75`, `u = -0.25` → `0.75`. Esto produce un mosaico continuo.

### `TextureWrap::Clamp`

Satura al rango `[0, 1]`:

```rust
val.clamp(0.0, 1.0)
```

Los valores fuera del rango se "aplastan" al borde más cercano, repitiendo el píxel del borde. Útil para el skybox o texturas que no deben repetirse.

---

## Filtrado Nearest Neighbor — `get_color_nearest_wrap`

El método más simple y rápido. Selecciona el texel más cercano al punto de muestreo redondeando las coordenadas a entero:

```rust
let x = ((u_wrapped * max_x as f32).round() as usize).min(max_x);
let y = ((v_wrapped * max_y as f32).round() as usize).min(max_y);
self.buffer[y * self.width + x]
```

**Ventaja:** O(1), sin multiplicaciones adicionales.  
**Inconveniente:** Produce el efecto "pixelado" cuando la textura se agranda más que su resolución nativa.

---

## Filtrado Bilineal — `get_color_bilinear_wrap`

Interpola entre los 4 texels vecinos más cercanos al punto de muestreo. Elimina el pixelado a costa de más cálculos.

### Algoritmo

Dado un punto de muestreo continuo `(gx, gy)` en espacio de píxeles:

```rust
let gx = u_wrapped * max_x as f32;  // posición continua en X
let gy = v_wrapped * max_y as f32;  // posición continua en Y
```

Se identifican los 4 texels que lo rodean y las fracciones de interpolación `(fx, fy)`:

```rust
let x0 = gx.floor() as usize;   // texel izquierdo
let x1 = (x0 + 1).min(max_x);  // texel derecho
let y0 = gy.floor() as usize;   // texel superior
let y1 = (y0 + 1).min(max_y);  // texel inferior

let fx = gx - gx.floor();  // fracción horizontal [0,1): qué tan lejos está de x0 hacia x1
let fy = gy - gy.floor();  // fracción vertical   [0,1)
```

La interpolación bilineal aplica interpolación lineal primero en X (por cada fila) y luego en Y:

```
c_top = lerp(c00, c10, fx)   ← mezcla fila superior
c_bot = lerp(c01, c11, fx)   ← mezcla fila inferior
c     = lerp(c_top, c_bot, fy) ← mezcla las dos filas
```

En código por canal (canal R como ejemplo):

```rust
let r_top = c00.r as f32 * (1.0 - fx) + c10.r as f32 * fx;
let r_bot = c01.r as f32 * (1.0 - fx) + c11.r as f32 * fx;
let r = (r_top * (1.0 - fy) + r_bot * fy).round().clamp(0.0, 255.0) as u8;
```

El resultado es un color suavizado que corresponde al promedio ponderado de los 4 texels vecinos, con pesos proporcionales a la distancia al punto de muestreo.

---

## Uso de `Arc<Texture>`

Las texturas se envuelven en `Arc<Texture>` (puntero de referencia atómica) cuando se asignan a materiales:

```rust
pub texture: Option<Arc<Texture>>,
```

`Arc` permite que múltiples materiales o múltiples frames de animación compartan el **mismo objeto de textura en memoria** sin copiar el buffer. Las operaciones de clonar un `Arc` son O(1): solo incrementan un contador de referencias atómica. Además, `Arc<T>` es `Send + Sync`, lo que permite compartirlo entre los hilos de `render_parallel` sin restricciones.

---

## Relación con el resto del motor

```
Material::get_color(u, v)
    └── tex.get_color_nearest_wrap(u * scale, v * scale, wrap_mode)
            └── wrap_coord(u, mode) → coordenada en [0,1]
            └── buffer[y * width + x] → Color

Skybox::sample(ray_direction)
    └── face_texture.get_color_filtered(u, v, filter)
```
