# Renderer

## ¿Para qué sirve?

El renderer implementa el algoritmo de **trazado de rayos recursivo** completo: proyección de rayos desde la cámara, iluminación directa con el modelo Phong, sombras duras, reflexión especular, refracción dieléctrica y el coeficiente de Fresnel. Es el módulo central del motor.

---

## Constantes

```rust
pub const FOV: f32 = PI / 3.0;         // 60°: ángulo de apertura de la "lente"
pub const SHADOW_BIAS: f32 = 1e-3;     // Offset para evitar auto-intersección en sombras
pub const REFLECTION_BIAS: f32 = 1e-3; // Offset para rayos de reflexión
pub const REFRACTION_BIAS: f32 = 1e-3; // Offset para rayos de refracción
pub const MAX_DEPTH: u32 = 3;          // Límite de rebotes recursivos
```

Los **biases** son offsets pequeños que desplazan el origen de los rayos secundarios ligeramente fuera de la superficie. Sin ellos, el rayo recién generado intersecaría inmediatamente la misma superficie que lo originó por error numérico de punto flotante (auto-sombra o auto-reflexión).

---

## `reflect(incident, normal) -> Vec3`

Calcula la dirección de reflexión especular. La fórmula viene directamente de la geometría: el vector reflejado `R` es el incidente `I` menos el doble de su componente a lo largo de la normal `N`:

```
R = I - 2(I·N)N
```

El término `(I·N)` es la proyección escalar de `I` sobre `N`. Multiplicado por `N` da la componente del incidente en dirección normal. Restarla dos veces al incidente es equivalente a "voltear" esa componente, conservando la componente tangencial:

```rust
incident - normal * (2.0 * dot(incident, normal))
```

La dirección resultante forma el mismo ángulo con la normal que el incidente, pero al otro lado (ley de reflexión).

---

## `refract(incident, normal, ior) -> Option<Vec3>`

Calcula la dirección del rayo refractado según la **Ley de Snell**:

```
η₁ · sin(θ₁) = η₂ · sin(θ₂)
```

Donde `θ₁` es el ángulo de incidencia, `θ₂` el ángulo de refracción, `η₁` el IOR del medio de entrada y `η₂` el del medio de salida.

La implementación maneja explícitamente el caso en que el rayo sale del interior del objeto (cuando `cos(θ₁) < 0`, es decir, `I · N > 0`). En ese caso se invierten los índices y la normal:

```rust
if cosi < 0.0 {
    cosi = -cosi;
    std::mem::swap(&mut eta_i, &mut eta_t);
    n = -normal;
}
```

La fórmula vectorial de Snell que evita calcular ángulos explícitamente:

```
η = η₁ / η₂
k = 1 - η²(1 - cos²θ₁) = 1 - η²·sin²θ₁
T = η·I + (η·cosθ₁ - √k)·N
```

```rust
let eta = eta_i / eta_t;
let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
if k < 0.0 {
    None  // Reflexión Interna Total (TIR)
} else {
    Some(incident * eta + n * (eta * cosi - k.sqrt()))
}
```

**Reflexión Interna Total (TIR):** cuando `k < 0`, el término `sin²θ₂ > 1` matemáticamente, lo cual es imposible para un ángulo real. Físicamente significa que la luz no puede "salir" del medio más denso con ese ángulo: toda la energía se refleja. Esto es lo que da el efecto espejo a las superficies de agua vistas desde ángulos oblicuos.

---

## `fresnel(incident, normal, ior) -> f32`

El efecto Fresnel describe cómo la fracción de luz reflejada versus refractada varía con el ángulo de incidencia. A incidencia normal (perpendicular), solo una pequeña fracción se refleja; a incidencia rasante, casi toda la luz se refleja.

La **aproximación de Schlick** simplifica las ecuaciones exactas de Fresnel en una fórmula de bajo coste:

```
r₀ = ((η₁ - η₂) / (η₁ + η₂))²
kr = r₀ + (1 - r₀)(1 - cos θ₁)⁵
```

`r₀` es el coeficiente de reflexión en incidencia normal (el mínimo de reflexión). Para vidrio IOR=1.5: `r₀ = ((1-1.5)/(1+1.5))² = 0.04` (4% de reflexión en incidencia normal). El término `(1 - cosθ)⁵` escala la contribución extra a medida que el ángulo se vuelve más rasante:

```rust
let r0 = ((eta_i - eta_t) / (eta_i + eta_t)).powi(2);
r0 + (1.0 - r0) * (1.0 - cosi).powi(5)
```

El resultado `kr ∈ [0, 1]` es la fracción de energía reflejada. La fracción refractada es `1 - kr`.

---

## `cast_shadow(intersect, light_direction, light, objects) -> bool`

Lanza un **rayo de sombra** desde el punto de impacto hacia la fuente de luz. Si algún objeto opaco bloquea el camino, el punto está en sombra.

El origen del rayo se desplaza por el bias para salir de la superficie:

```rust
let shadow_ray_origin = intersect.point + intersect.normal * SHADOW_BIAS;
```

Solo se comprueba si hay intersección dentro de la distancia `light_distance` (no tiene sentido buscar sombras más allá de la luz):

```rust
objects.iter().any(|object| {
    object.ray_intersect_distance(&shadow_ray_origin, light_direction, light_distance)
})
```

---

## `shade(intersect, ray_origin, lights, objects) -> Color`

Implementa el **modelo de iluminación de Phong**, que descompone la luz en tres componentes: emisiva, difusa y especular.

### 1. Componente emisiva

Luz propia del material, independiente de cualquier fuente externa:

```rust
let total_color = if intersect.emissive > 0.0 {
    intersect.color * intersect.emissive
} else {
    Color::new(0, 0, 0)
};
```

### 2. Componente difusa (Lambertiana)

La reflexión difusa asume que la superficie esparce la luz en todas las direcciones de forma uniforme. La intensidad depende solo del ángulo entre la normal de la superficie y la dirección a la luz, según la **Ley de Lambert**:

```
L_d = k_d · (N · L) · I_luz · C_objeto
```

Donde `N · L` es el coseno del ángulo de incidencia (producto punto entre normal y dirección a la luz). Si `N · L < 0` (luz detrás de la superficie), la contribución es cero:

```rust
let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
let diffuse = intersect.color * (diffuse_intensity * intersect.diffuse * effective_intensity);
```

### 3. Componente especular (Phong)

Modela el "punto brillante" de reflexión especular directa. Depende del ángulo entre la dirección de vista y la dirección de reflexión de la luz:

```
L_s = k_s · (V · R)^α · I_luz · C_luz
```

Donde `R` es la reflexión de `-L` respecto a `N`, `V` es la dirección de vista y `α` es el exponente de brillo (shininess). A mayor `α`, el punto brillante es más pequeño y concentrado:

```rust
let reflect_direction = reflect(&-light_direction, &intersect.normal);
let specular_intensity = dot(&view_direction, &reflect_direction)
    .max(0.0)
    .powf(intersect.shininess);
```

### Atenuación de luces puntuales

Las luces puntuales siguen la **ley del cuadrado inverso** aproximada por la fórmula de Blinn:

```
atenuación = 1 / (1 + kl·d + kq·d²)
```

Con constantes `kl = 0.15` (lineal) y `kq = 0.05` (cuadrático). La componente constante evita singularidades a distancia cero. Las luces direccionales tienen atenuación = 1.0 siempre.

---

## `cast_ray` — El algoritmo recursivo ⭐

Lanza un rayo al mundo y combina iluminación directa, reflexión y refracción de forma recursiva.

### Proyección y búsqueda del objeto más cercano

Se itera linealmente sobre todos los objetos buscando el de menor `t` positivo:

```rust
for object in objects {
    if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
        if closest.is_none_or(|c| intersect.distance < c.distance) {
            closest = Some(intersect);
        }
    }
}
```

### Combinación de componentes

Una vez encontrado el impacto, el color final es una combinación ponderada:

```
C_total = C_directo + C_reflexion · W_r + C_refraccion · W_t
```

Donde:
- `W_r = reflective + refractive · kr`  (peso de reflexión, aumentado si hay TIR)
- `W_t = refractive · (1 - kr)`  (peso de refracción, balanceado con Fresnel)

```rust
let reflect_weight = intersect.reflective + if intersect.refractive > 0.0 {
    intersect.refractive * kr
} else { 0.0 };
let refract_weight = if has_refraction { intersect.refractive * (1.0 - kr) } else { 0.0 };

direct_color + reflection_color * reflect_weight + refraction_color * refract_weight
```

La recursividad acumula `depth + 1` en cada rebote. Cuando `depth > MAX_DEPTH`, el rayo "muere" y retorna el color del skybox.

---

## `render` — Proyección perspectiva ⭐

Para cada píxel `(x, y)` se calcula la dirección del rayo en **espacio NDC** (Normalized Device Coordinates, rango `[-1, 1]`) y se transforma al espacio de mundo:

```
screen_x = (2x / W - 1) · aspect_ratio · tan(FOV/2)
screen_y = (1 - 2y / H) · tan(FOV/2)
```

```rust
let perspective_scale = (FOV / 2.0).tan();
let screen_x = ((2.0 * x as f32) / width - 1.0) * aspect_ratio * perspective_scale;
let screen_y = (-(2.0 * y as f32) / height + 1.0) * perspective_scale;
```

- `2x/W - 1` normaliza de `[0, W]` a `[-1, 1]`. El signo negativo en `screen_y` invierte el eje vertical (en pantalla Y crece hacia abajo, en escena hacia arriba).
- `tan(FOV/2)` escala el plano de imagen según el ángulo de apertura.
- `aspect_ratio = W/H` corrige la deformación en pantallas no cuadradas.

La dirección base `(screen_x, screen_y, -1)` apunta hacia `-Z` (frente de la cámara en espacio local). `camera.basis_change()` la rota al espacio de mundo según la orientación actual de la cámara.
