# Sphere

## ¿Para qué sirve?

`Sphere` es la primitiva geométrica más simple del motor y el ejemplo canónico para estudiar trazado de rayos. Implementa el trait `RayIntersect` resolviendo la intersección rayo-esfera mediante una **ecuación cuadrática** derivada directamente de la definición de esfera.

---

## Estructura

```rust
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}
```

---

## Fundamento matemático — Intersección rayo-esfera

### Definición de los objetos

Un **rayo** es una función paramétrica de la distancia `t`:

```
P(t) = O + t·D,   t > 0
```

Donde `O` es el origen (posición de la cámara), `D` es la dirección unitaria (`|D| = 1`) y `t` es la distancia a lo largo del rayo.

Una **esfera** de centro `C` y radio `r` es el conjunto de puntos que satisfacen:

```
|P - C|² = r²
```

### Derivación de la ecuación cuadrática

Para encontrar dónde se intersecan, sustituimos `P = O + tD` en la ecuación de la esfera:

```
|O + tD - C|² = r²
```

Definimos `oc = O - C` (vector del centro de la esfera al origen del rayo) y expandimos:

```
|tD + oc|² = r²
t²|D|² + 2t(D·oc) + |oc|² - r² = 0
```

Como `D` está normalizado, `|D|² = 1`. La ecuación queda:

```
t² + 2bt + c = 0
```

Con:
- `b = D·oc`  (producto punto)
- `c = |oc|² - r²`  (distancia² al centro menos radio²)

El **discriminante** determina si hay solución:

```
Δ = b² - c
```

- `Δ < 0` → el rayo no toca la esfera
- `Δ = 0` → el rayo es tangente (toca en un punto)
- `Δ > 0` → el rayo atraviesa la esfera (dos puntos de intersección)

Las dos soluciones son `t = -b ± √Δ`. Se toma la **más pequeña positiva** (intersección más cercana al origen del rayo):

```rust
let oc = ray_origin - self.center;
let b  = dot(&oc, ray_direction);
let c  = dot(&oc, &oc) - self.radius * self.radius;
let discriminant = b * b - c;

let sqrt_d = discriminant.sqrt();
let mut t = -b - sqrt_d;   // solución con el signo negativo (más cercana)
if t <= 0.0 {
    t = -b + sqrt_d;       // si estamos dentro de la esfera, usamos la lejana
    if t <= 0.0 { return None; }
}
```

Nota: la ecuación cuadrática habitual es `t = (-b ± √(b²-4ac)) / 2a`. Aquí `a = 1` y `b` ya incluye el factor 2, por lo que se simplifica a `t = -b ± √(b²-c)`.

---

## Cálculo de la normal

La normal en un punto `P` de la superficie de la esfera apunta radialmente desde el centro hacia fuera:

```
N = (P - C) / r
```

En lugar de normalizar con `sqrt`, se divide por el radio conocido. Dado que `|P - C| = r` por definición de la esfera, el resultado es un vector unitario sin el coste de la raíz cuadrada:

```rust
let normal = (point - self.center) / self.radius;
```

---

## Mapeo UV esférico

Para proyectar una textura 2D sobre la esfera se convierten las coordenadas de la normal a **coordenadas esféricas** `(θ, φ)` y luego a UV normalizados:

**Longitud** (eje horizontal, φ):
```
u = 0.5 + atan2(Nz, Nx) / (2π)
```

`atan2(z, x)` devuelve el ángulo en el plano XZ en el rango `(-π, π)`. Dividir por `2π` y sumar `0.5` lo lleva al rango `[0, 1]`.

**Latitud** (eje vertical, θ):
```
v = 0.5 - asin(Ny) / π
```

`asin(Ny)` devuelve el ángulo de elevación en `(-π/2, π/2)`. Dividir por `π` y restar de `0.5` lo mapea a `[0, 1]` con `v=0` en el polo norte y `v=1` en el polo sur.

```rust
let u_coord = 0.5 + normal.z.atan2(normal.x) / (2.0 * PI);
let v_coord = 0.5 - normal.y.clamp(-1.0, 1.0).asin() / PI;
```

El `.clamp(-1.0, 1.0)` antes de `asin` previene errores numéricos cuando `|Ny|` excede ligeramente 1.0 por imprecisión flotante.

---

## `ray_intersect_distance` — versión para sombras

Variante más rápida usada al verificar sombras. No calcula la normal, no muestrea la textura, solo comprueba si `t ∈ (0, max_distance)`.

Adicionalmente, si el material es refractivo (`refractive > 0.5`), devuelve `false` directamente: los objetos transparentes no bloquean la luz de los objetos detrás de ellos:

```rust
if self.material.refractive > 0.5 {
    return false;
}
```

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `ray_intersect` | Implementa `RayIntersect`; devuelve `Intersect` con punto, normal, UV y material |
| `renderer` | `cast_ray` itera sobre `Vec<Object>` y llama a `ray_intersect` |
| `camera` | `check_collision` usa distancia euclidiana al centro para colisiones FPS |
