# Ray Intersect

## ¿Para qué sirve?

Este módulo define las **abstracciones centrales** del motor: el trait `RayIntersect`, las estructuras `Material` e `Intersect`, y el enum `Object`. Todo objeto que participe en el trazado de rayos debe implementar `RayIntersect`. Es el contrato que desacopla la geometría de la iluminación.

---

## `Material` — Modelo óptico de una superficie

```rust
pub struct Material {
    pub color: Color,
    pub diffuse: f32,           // Peso de reflexión lambertiana
    pub specular: f32,          // Peso de brillo especular directo (Phong)
    pub reflective: f32,        // Peso de reflexión especular indirecta (espejo)
    pub refractive: f32,        // Peso de transmisión / transparencia
    pub emissive: f32,          // Luz propia emitida (0 = ninguna)
    pub shininess: f32,         // Exponente α en (V·R)^α del modelo Phong
    pub refractive_index: f32,  // IOR: η en la Ley de Snell
    pub texture: Option<Arc<Texture>>,
    pub animated_frames: Option<Arc<Vec<Arc<Texture>>>>,
    pub animation_fps: f32,
    pub uv_scale: (f32, f32),   // Multiplicador de repetición UV (u_scale, v_scale)
    pub wrap_mode: TextureWrap,
}
```

Cada propiedad flotante es un **peso de mezcla** que el renderer usa para ponderar distintos componentes de iluminación. La suma de los pesos no necesita ser 1.0; el renderer los aplica como contribuciones independientes.

### Patrón Builder

`Material::new(...)` acepta los 7 parámetros principales. Los campos opcionales se añaden encadenando métodos:

```rust
Material::new(color, 0.0, 0.1, 0.05, 0.95, 125.0, 1.52)  // vidrio base
    .with_texture(Arc::new(Texture::new("glass_tint.png")))
    .with_uv_scale(2.0, 2.0)
    .with_wrap(TextureWrap::Clamp)
```

Cada método de Builder toma `mut self` y devuelve `Self`, lo que permite encadenar sin variables intermedias.

### `get_color(u, v) -> Color`

Retorna el color definitivo de la superficie en las coordenadas de textura `(u, v)`. Primero escala las coordenadas por `uv_scale`:

```rust
let u_scaled = u * self.uv_scale.0;
let v_scaled = v * self.uv_scale.1;
```

Luego muestrea la textura y **modula** el color base con el texel usando multiplicación componente a componente `Color * Color`:

```rust
self.color * tex.get_color_nearest_wrap(u_scaled, v_scaled, self.wrap_mode)
```

Si no hay textura, devuelve `self.color` directamente.

### `update_time(time: f32)` — Animación de texturas

Para materiales con frames animados, selecciona el frame activo usando aritmética modular:

```
idx = floor(time * fps) mod num_frames
```

```rust
let idx = ((time * self.animation_fps).floor() as usize) % frames.len();
self.texture = Some(Arc::clone(&frames[idx]));
```

`floor(time * fps)` convierte el tiempo continuo en un índice discreto que avanza exactamente `fps` veces por segundo. El módulo garantiza el ciclo continuo entre frames.

### `to_intersect(...)` / `to_intersect_with_color(...)`

Empaqueta la información geométrica del impacto (punto, normal, distancia, UV) junto con las propiedades ópticas del material en un `Intersect`. `to_intersect_with_color` acepta un color precalculado para evitar muestrear la textura dos veces cuando ya se hizo antes (optimización usada en `Box3D` y `Plane`).

---

## `Intersect` — Snapshot del punto de impacto

```rust
pub struct Intersect {
    pub point: Vec3,       // Coordenadas 3D del impacto en espacio de mundo
    pub normal: Vec3,      // Normal de la superficie en ese punto (unitaria)
    pub distance: f32,     // Distancia t desde el origen del rayo
    pub color: Color,      // Color ya muestreado (incluye textura y color base)
    pub diffuse: f32,
    pub specular: f32,
    pub reflective: f32,
    pub refractive: f32,
    pub emissive: f32,
    pub shininess: f32,
    pub refractive_index: f32,
}
```

`Intersect` es un snapshot inmutable del estado de la superficie en el punto de impacto. El renderer trabaja exclusivamente con `Intersect` una vez calculado: no necesita saber si es esfera, caja o plano. Esto desacopla completamente la geometría de la iluminación.

`Copy` (derivado implícitamente junto con `Clone`) permite que el renderer copie intersecciones baratas por valor sin referencias, simplificando el código de comparación de distancias.

---

## `RayIntersect` — El trait que define un objeto trazable

```rust
pub trait RayIntersect: Sync + Send {
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Option<Intersect>;
    fn ray_intersect_distance(&self, origin: &Vec3, direction: &Vec3, max_distance: f32) -> bool;
}
```

**`ray_intersect`:** Calcula el impacto más cercano del rayo con el objeto. Devuelve `None` si no hay intersección o si el impacto está detrás del rayo (`t ≤ 0`).

**`ray_intersect_distance`:** Versión optimizada para pruebas de sombra. La implementación por defecto reutiliza `ray_intersect`, pero cada primitiva puede sobreescribirla con una versión más eficiente que no calcula normales ni muestrea texturas. La diferencia de rendimiento es significativa porque esta función se llama una vez por objeto por rayo de sombra por luz.

El trait hereda `Sync + Send` porque los objetos se comparten entre los hilos de `render_parallel`.

---

## `Object` — Despacho polimórfico sin vtable

```rust
pub enum Object {
    Box3D(Box3D),
    Sphere(Sphere),
    Plane(Plane),
}
```

Un enum en Rust ocupa el espacio del variante más grande más un discriminante. Almacenar objetos en un `Vec<Object>` los mantiene **contiguos en memoria** (mejor localidad de caché) a diferencia de `Vec<Box<dyn RayIntersect>>` que almacena punteros a objetos dispersos en el heap.

El despacho mediante `match` genera código con saltos directos (branch tables), evitando el overhead de indirección de vtables:

```rust
impl RayIntersect for Object {
    #[inline]
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Option<Intersect> {
        match self {
            Object::Box3D(b)  => b.ray_intersect(origin, direction),
            Object::Sphere(s) => s.ray_intersect(origin, direction),
            Object::Plane(p)  => p.ray_intersect(origin, direction),
        }
    }
}
```

---

## Relación con el resto del motor

```
renderer::cast_ray
    └─ for object in objects
           └─ object.ray_intersect()       → Option<Intersect>
               └─ renderer::shade(&intersect)
                       └─ usa intersect.{diffuse, specular, shininess, ...}
               └─ renderer::cast_shadow()
                       └─ object.ray_intersect_distance()  → bool
```
