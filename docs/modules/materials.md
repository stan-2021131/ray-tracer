# Materials

## ¿Para qué sirve?

`materials` es un módulo de **fábrica de materiales**: un conjunto de funciones que crean instancias de `Material` preconfiguradas para distintos acabados físicos. Abstrae la construcción de materiales ocultando los 7 parámetros de `Material::new` detrás de nombres descriptivos.

---

## Los 7 parámetros ópticos

```
Material::new(color, diffuse, specular, reflective, refractive, shininess, ior)
```

| Parámetro | Símbolo | Qué controla |
|---|---|---|
| `diffuse` | kd | Peso de la reflexión lambertiana (cuánta luz difunde la superficie) |
| `specular` | ks | Peso del brillo especular directo en el modelo Phong |
| `reflective` | kr | Peso de la reflexión especular indirecta (traza un rayo adicional) |
| `refractive` | kt | Peso de la transmisión (traza un rayo refractado) |
| `shininess` | α | Exponente en `(V·R)^α`: mayor = brillo más concentrado y pequeño |
| `ior` | η | Índice de refracción en la Ley de Snell: `η₁ sin θ₁ = η₂ sin θ₂` |

Los pesos `kr` y `kt` activan el trazado de rayos secundarios en el renderer. Un `reflective = 0.85` implica que el 85% de la energía de ese píxel proviene del rayo reflejado. Un `refractive = 0.95` implica que el 95% proviene del rayo refractado (distribuido entre reflexión y refracción según el coeficiente de Fresnel).

---

## Materiales disponibles y sus valores

### `diffuse(color)` — Superficie mate

```rust
Material::new(color, 0.9, 0.1, 0.0, 0.0, 10.0, 1.0)
```

Alta difusión (kd=0.9), sin reflexión ni refracción. El shininess bajo (10) produce un brillo especular muy difuso si lo hay. Para madera, piedra, tela.

---

### `ivory(color)` — Plástico brillante

```rust
Material::new(color, 0.6, 0.3, 0.1, 0.0, 50.0, 1.0)
```

Shininess=50 produce un punto brillante de tamaño medio. La reflexión indirecta (kr=0.1) da un ligero efecto espejo sin llegar a ser metálico.

---

### `cobalt(color)` — Metal

```rust
Material::new(color, 0.7, 0.4, 0.15, 0.0, 80.0, 1.0)
```

Mayor especularidad y reflexión que el marfil. El shininess=80 da brillos concentrados característicos de superficies metálicas.

---

### `mirror(tint)` — Espejo

```rust
Material::new(tint, 0.1, 0.1, 0.85, 0.0, 125.0, 1.0)
```

kr=0.85 hace que el 85% del color del píxel venga del rayo reflejado. Shininess alto (125) para el pequeño punto brillante especular directo. Con tinte blanco es un espejo neutro; con tinte de color, un espejo coloreado (como un espejo de oro o cobre).

---

### `glass(tint)` — Vidrio dieléctrico

```rust
Material::new(tint, 0.0, 0.1, 0.05, 0.95, 125.0, 1.52)
```

kt=0.95 activa la refracción. `ior=1.52` es el índice de refracción del vidrio borosilicato estándar. Combinado con el coeficiente de Fresnel del renderer, produce reflexiones en los bordes que aumentan con el ángulo de incidencia (efecto físicamente correcto). `kd=0.0` porque el vidrio no dispersa difusamente: toda la luz la transmite o refleja.

---

### `water(tint)` — Agua

```rust
Material::new(tint, 0.0, 0.1, 0.05, 0.95, 125.0, 1.333)
```

Igual que vidrio pero `ior=1.333` (agua a 20°C). El IOR más bajo que el vidrio significa que la luz se dobla menos al atravesarla. El ángulo crítico de Reflexión Interna Total es mayor que para el vidrio (≈48.6° vs ≈41.8°), por lo que el efecto espejo desde dentro del agua aparece a ángulos más oblicuos.

---

### `gold(color)` — Metal pulido reflectivo

```rust
Material::new(color, 0.2, 0.4, 0.7, 0.0, 100.0, 1.0)
```

kr=0.7 es muy alto pero sin refracción. A diferencia del espejo, el oro conserva algo de difusión (kd=0.2) que le da cuerpo visual. Con `Color::new(255, 215, 0)` produce oro; con `(205, 127, 50)` cobre; con `(192, 192, 192)` plata.

---

### `emissive(color, intensity)` — Material emisivo

```rust
Material::new(color, 0.1, 0.0, 0.0, 0.0, 10.0, 1.0).with_emissive(intensity)
```

El campo `emissive > 0` hace que el material emita su color propio en `shade()`, independientemente de las luces externas y las sombras:

```rust
let total_color = if intersect.emissive > 0.0 {
    intersect.color * intersect.emissive
} else {
    Color::new(0, 0, 0)
};
```

Útil para bombillas, fuego, pantallas LED o cualquier fuente de luz visible como objeto.

---

## Funciones de texturizado

### `apply_texture(base_material, texture_path) -> Material`

Carga la textura desde disco y la envuelve en `Arc` para compartirla sin copias:

```rust
let texture = Arc::new(Texture::new(texture_path));
base_material.with_texture(texture)
```

La textura modula el color base del material mediante `Color * Color` en `Material::get_color(u, v)`.

### `apply_animated_texture(base_material, paths, fps) -> Material`

Carga N texturas como frames de una animación ciclada. El frame activo se selecciona con `floor(time * fps) mod N` en `update_time()`.

---

## Relación con el resto del motor

```
scene.rs
    └── materials::glass(Color::new(200, 220, 255))
    └── materials::apply_texture(materials::diffuse(...), "./textures/wall.png")
                    ↓
            Material { diffuse, specular, reflective, refractive, ior, texture, ... }
                    ↓
    renderer::shade()     → usa kd, ks, shininess
    renderer::cast_ray()  → usa kr, kt, ior para rayos secundarios
    Material::get_color() → usa texture para modular el color
```
