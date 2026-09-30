# Plane

## ¿Para qué sirve?

`Plane` representa un **rectángulo finito** en el espacio 3D. Geométricamente es una sección acotada de un plano infinito definido por su normal. Se usa para suelos, paredes, paneles y cualquier superficie plana que necesite límites.

---

## Estructura

```rust
pub struct Plane {
    pub center: Vec3,
    pub normal: Vec3,   // Normal unitaria perpendicular al plano
    pub width: f32,
    pub height: f32,
    pub u: Vec3,        // Eje horizontal "dentro" del plano
    pub v: Vec3,        // Eje vertical "dentro" del plano
    pub half_w: f32,    // width / 2
    pub half_h: f32,    // height / 2
    pub material: Material,
}
```

Los vectores `u` y `v` forman, junto con `normal`, una **base ortonormal** del espacio del plano. Son el equivalente a "derecha" y "arriba" vistos desde la superficie.

---

## Construcción de la base ortonormal

El algoritmo elige un vector de referencia ortogonal a la normal y genera la base con dos productos vectoriales. El vector de referencia varía para evitar el caso degenerado donde normal y referencia son paralelos (el producto vectorial sería cero):

```rust
if n.x.abs() > 0.9 {
    // Normal apunta principalmente en X: usar Y como referencia
    let u = Vec3::new(0.0, 1.0, 0.0).cross(&n).normalize();
    let v = n.cross(&u).normalize();
} else if n.y.abs() > 0.9 {
    // Normal apunta principalmente en Y (plano horizontal): usar Z como referencia
    let u = Vec3::new(0.0, 0.0, 1.0).cross(&n).normalize();
    let v = n.cross(&u).normalize();
} else {
    // Normal apunta principalmente en Z: usar Y como referencia
    let u = Vec3::new(0.0, 1.0, 0.0).cross(&n).normalize();
    let v = n.cross(&u).normalize();
}
```

El producto vectorial `A × B` produce un vector perpendicular a ambos. Encadenando dos productos vectoriales desde la normal se obtienen dos ejes que yacen sobre el plano y son perpendiculares entre sí.

---

## Fundamento matemático — Intersección rayo-plano

### Ecuación del plano infinito

Un plano queda definido por un punto `C` (el centro) y una normal unitaria `N`. Todo punto `P` del plano satisface:

```
(P - C) · N = 0
```

Es decir, el vector `(P - C)` es perpendicular a `N`.

### Cálculo de t

Sustituyendo el rayo `P(t) = O + tD` en la ecuación del plano:

```
(O + tD - C) · N = 0
t(D · N) = (C - O) · N
t = (C - O) · N / (D · N)
```

En código:

```rust
let denom = dot(&self.normal, ray_direction);  // D · N
if denom.abs() < 1e-6 { return None; }         // rayo casi paralelo al plano

let t = dot(&(self.center - ray_origin), &self.normal) / denom;
if t <= 0.0 { return None; }                   // plano detrás del rayo
```

El umbral `1e-6` es la tolerancia numérica: bajo ese valor el rayo es prácticamente paralelo al plano y el resultado de la división sería numéricamente inestable.

### Verificación de límites — proyección sobre la base del plano

El punto de impacto en el plano infinito puede caer fuera del rectángulo. Para comprobarlo se proyecta el vector `(P - C)` sobre los ejes `u` y `v` del plano usando el producto punto:

```
u_proj = (P - C) · u
v_proj = (P - C) · v
```

Si `|u_proj| > half_w` o `|v_proj| > half_h`, el punto está fuera del rectángulo y no hay intersección:

```rust
let d = point - self.center;
let u_proj = dot(&d, &self.u);
let v_proj = dot(&d, &self.v);
if u_proj.abs() > self.half_w || v_proj.abs() > self.half_h {
    return None;
}
```

La proyección sobre la base ortonormal es posible porque `u` y `v` son vectores unitarios: el producto punto con un vector unitario es exactamente la longitud de la componente paralela a ese eje.

### Normal orientada hacia la cámara

La normal del plano puede apuntar en cualquier dirección. Si el rayo viene desde el "lado trasero" (`denom > 0` significa que `D · N > 0`, es decir, rayo y normal apuntan en el mismo sentido), se invierte la normal para que siempre apunte hacia el origen del rayo. Esto garantiza iluminación y reflexión correctas desde ambas caras:

```rust
let normal = if denom < 0.0 { self.normal } else { -self.normal };
```

---

## Mapeo UV

Las proyecciones `u_proj` y `v_proj` (que ya están en unidades del espacio 3D) se normalizan al rango `[0, 1]`:

```rust
let u_coord = (u_proj / self.width) + 0.5;   // [-half_w, half_w] → [0, 1]
let v_coord = 0.5 - (v_proj / self.height);  // [half_h, -half_h] → [0, 1]
```

La resta en `v_coord` invierte el eje vertical para que el origen UV `(0, 0)` corresponda a la esquina superior izquierda (convención estándar de imágenes).

---

## Alpha Cutout

Si la textura contiene píxeles transparentes (`a < 128`), el rayo ignora ese impacto y continúa hacia el siguiente objeto, sin calcular normales ni materiales:

```rust
let color = self.material.get_color(u_coord, v_coord);
if self.material.has_alpha() && color.a < 128 {
    return None;
}
```

Esto permite simular vallas, hojas de árboles o paneles calados con una simple textura PNG con transparencia.

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `ray_intersect` | Implementa `RayIntersect`; encapsulado en `Object::Plane` |
| `renderer` | `cast_ray` prueba el plano igual que cualquier otro objeto |
| `camera` | `check_collision` usa la distancia al plano horizontal para evitar caer |
