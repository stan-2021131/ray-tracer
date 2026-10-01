# Skybox

## ¿Para qué sirve?

El skybox proporciona el color que recibe un rayo cuando no impacta ningún objeto de la escena. En lugar de un color plano, muestrea una imagen de las 6 caras de un **cubemap** para producir un entorno visual envolvente (cielo, horizonte, estrellas, etc.).

---

## Fundamento teórico — Cubemap

Un **cubemap** proyecta el entorno esférico completo sobre las 6 caras internas de un cubo unitario centrado en la escena. Para cualquier dirección de rayo `D = (x, y, z)` (no necesariamente normalizada), el proceso de muestreo es:

1. Identificar el eje de mayor magnitud absoluta: `max(|x|, |y|, |z|)`. La cara de ese eje es la que el rayo "mira".
2. Dividir las otras dos componentes entre esa magnitud máxima para proyectar el punto sobre el plano de la cara, obteniendo coordenadas en `[-1, 1]`.
3. Mapear linealmente `[-1, 1] → [0, 1]` para obtener las coordenadas UV de textura.

Esto es equivalente a proyectar el punto de la esfera unitaria sobre el cubo unitario a través del origen.

---

## Estructura

```rust
pub struct Skybox {
    pub up: Texture,    // +Y
    pub down: Texture,  // -Y
    pub left: Texture,  // -X
    pub right: Texture, // +X
    pub front: Texture, // -Z (dirección inicial de vista)
    pub back: Texture,  // +Z
    pub filter: TextureFilter,
    pub rotation_y: f32,
    pub has_rotation: bool,
}
```

---

## `get_face_and_uv(dir) -> (CubemapFace, f32, f32)` ⭐

Función estática que implementa el mapeo de dirección a cara + coordenadas UV.

### Selección de cara

La cara se elige comparando las magnitudes absolutas de los tres componentes:

```rust
let abs_x = dir.x.abs();
let abs_y = dir.y.abs();
let abs_z = dir.z.abs();

if abs_x >= abs_y && abs_x >= abs_z {
    // Cara derecha (+X) o izquierda (-X)
} else if abs_y >= abs_x && abs_y >= abs_z {
    // Cara superior (+Y) o inferior (-Y)
} else {
    // Cara frontal (-Z) o trasera (+Z)
}
```

### Cálculo de UV por cara

Para cada cara, las dos dimensiones restantes se proyectan dividiendo entre el componente dominante (que actúa como denominador de la proyección perspectiva plana):

**Cara Frontal (-Z dominante negativo):**
```
inv_z = 1 / dir.z
u = (1 - dir.x · inv_z) / 2    → x positivo va a la derecha (u→1)
v = (1 + dir.y · inv_z) / 2    → y positivo va arriba (v→0)
```

```rust
let inv_z = 1.0 / dir.z;
let u = (1.0 - dir.x * inv_z) * 0.5;
let v = (1.0 + dir.y * inv_z) * 0.5;
```

**Cara Derecha (+X dominante positivo):**
```
inv_x = 1 / dir.x
u = (dir.z · inv_x + 1) / 2    → z positivo (detrás) va a la derecha
v = (1 - dir.y · inv_x) / 2
```

El patrón es consistente: dividir por el componente dominante normaliza los otros dos al rango `[-1, 1]`, y la fórmula `(componente + 1) / 2` los lleva a `[0, 1]`. El signo de cada componente en la fórmula se elige para que las caras adyacentes sean continuas en sus bordes (**sin costuras visibles**).

Usar `inv_z = 1/z` y luego multiplicar evita divisiones explícitas en el bucle caliente (las multiplicaciones son más rápidas que las divisiones en la mayoría de arquitecturas).

El `.clamp(0.0, 1.0)` al final previene artefactos en direcciones exactamente en la frontera entre dos caras.

---

## `sample(ray_direction) -> Color` ⭐

Función principal de muestreo. Dada la dirección del rayo que escapó sin impactar nada, devuelve el color del cielo en esa dirección.

### Rotación horizontal

Permite girar el entorno sobre el eje Y sin mover los objetos de la escena. Se implementa con una rotación 2D en el plano XZ (la altura Y no cambia):

```
x' =  x · cos(θ) + z · sin(θ)
y' =  y
z' = -x · sin(θ) + z · cos(θ)
```

```rust
Vec3::new(
    ray_direction.x * cos_theta + ray_direction.z * sin_theta,
    ray_direction.y,
    -ray_direction.x * sin_theta + ray_direction.z * cos_theta,
)
```

Esta es la matriz de rotación estándar alrededor del eje Y aplicada solo a las componentes X y Z. El flag `has_rotation` permite omitir este cálculo cuando el ángulo es cero.

### Muestreo

Tras obtener la cara y los UVs, se delega al método de filtrado correspondiente de `Texture`:

```rust
match face {
    CubemapFace::Right => self.right.get_color_filtered(u, v, self.filter),
    CubemapFace::Left  => self.left.get_color_filtered(u, v, self.filter),
    // ...
}
```

---

## Uso en el renderer

El skybox se consulta en dos situaciones dentro de `cast_ray`:

1. **Rayo que no impacta nada:** el rayo viaja infinitamente sin encontrar geometría.
2. **Profundidad máxima alcanzada:** un rayo de reflexión o refracción que superó `MAX_DEPTH` rebotes.

```rust
return skybox
    .map(|sb| sb.sample(ray_direction))
    .unwrap_or_else(|| Color::from_hex(BACKGROUND_COLOR));
```

Los rayos de reflexión que "miran al cielo" también consultan el skybox, lo que produce reflexiones de entorno realistas en espejos y superficies de agua.

---

## Constructores y Métodos Builder

- **`Skybox::new(up, down, left, right, front, back)`**: Carga las 6 caras ortogonales independientes.
- **`Skybox::from_single_texture(path)`**: Asigna la misma textura a las 6 caras (ideal para patrones cósmicos omnidireccionales como `textures/skybox/space.png`).
- **`with_filter(filter)`**: Configura el filtrado `Nearest` o `Bilinear`.
- **`with_brightness(brightness)`**: Escala linealmente el brillo del fondo (por ejemplo, `0.35` para atenuar la nebulosa de fondo y resaltar estrellas emisivas).
- **`with_rotation_y(radians)`**: Rota el domo en el eje horizontal.

---

## Soporte de Canal Alfa / Transparencia

Cuando las texturas del skybox contienen píxeles transparentes o semitransparentes (`color.a < 255`), `Skybox::sample` realiza automáticamente una mezcla alfa sobre el vacío negro del espacio (`0x000000`):

$$\text{color\_final} = \text{color} \cdot \left(\frac{\text{alpha}}{255}\right) \cdot \text{brightness}$$

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `renderer` | `cast_ray` consulta `skybox.sample()` cuando no hay impacto |
| `texture` | Cada cara es una `Texture` con su propio buffer de `Color` |
| `color` | `sample` devuelve un `Color` directamente al renderer |
| `main` | Gestiona `diorama_skybox` y `space_skybox` según la escena activa |

