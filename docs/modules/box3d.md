# Box3D

## ¿Para qué sirve?

`Box3D` representa un **prisma rectangular** (caja) con soporte de rotación arbitraria en los tres ejes. Es la primitiva más compleja del motor: a diferencia de la esfera, el cálculo de intersección no tiene una solución cerrada tan directa, y la rotación introduce la necesidad de transformar entre espacios de coordenadas.

---

## Estructura

```rust
pub struct Box3D {
    pub center: Vec3,
    pub size: Vec3,          // Dimensiones: ancho X, alto Y, profundidad Z
    pub half_size: Vec3,     // size / 2, precalculado para evitar divisiones repetidas
    pub rotation: Vec3,      // Ángulos de Euler en radianes: (rot_x, rot_y, rot_z)
    pub rot_mat: Mat3,       // Matriz rotación local → mundo
    pub inv_rot_mat: Mat3,   // Matriz rotación mundo → local (= transpuesta)
    pub is_rotated: bool,    // Flag: evita multiplicaciones matriciales si no hay rotación
    pub min: Vec3,           // Extremo mínimo del AABB en espacio de mundo
    pub max: Vec3,           // Extremo máximo del AABB en espacio de mundo
    pub material: Material,
}
```

---

## Matrices de rotación — `create_rotation_matrix`

Dado un vector de ángulos de Euler `(rx, ry, rz)` en radianes, se construye la matriz de rotación 3×3 con orden **Y × X × Z** (yaw primero, luego pitch, luego roll):

```rust
let (sx, cx) = rot.x.sin_cos();
let (sy, cy) = rot.y.sin_cos();
let (sz, cz) = rot.z.sin_cos();

// Fila 0: eje X del espacio local expresado en coordenadas de mundo
let r00 = cy * cz + sy * sx * sz;
let r01 = -cy * sz + sy * sx * cz;
let r02 = sy * cx;

// Fila 1: eje Y del espacio local
let r10 = cx * sz;
let r11 = cx * cz;
let r12 = -sx;

// Fila 2: eje Z del espacio local
let r20 = -sy * cz + cy * sx * sz;
let r21 = sy * sz + cy * sx * cz;
let r22 = cy * cx;
```

Cada columna de esta matriz es uno de los ejes locales de la caja expresado en coordenadas de mundo. Multiplicar un punto local por esta matriz lo transforma al espacio de mundo.

**La inversa es la transpuesta.** Para matrices de rotación pura (sin escala ni traslación), `R⁻¹ = Rᵀ`. Esto se cumple porque las columnas de R son vectores unitarios ortonormales, y la transpuesta de una matriz ortonormal es su inversa. Es mucho más barato transponer que invertir una matriz 3×3:

```rust
let inv_rot_mat = rot_mat.transpose();
```

Si los tres ángulos son cero, se devuelven matrices identidad y `is_rotated = false` para cortocircuitar los cálculos en el path crítico.

---

## AABB de la caja rotada — `compute_world_aabb`

Cuando la caja está rotada, su caja envolvente alineada a los ejes (AABB) en espacio de mundo ya no es `center ± half_size`. Hay que calcular cuánto se extiende la caja rotada en cada dirección del mundo.

El método estándar es proyectar cada uno de los semiejes locales sobre cada eje de mundo usando el **valor absoluto** de los coeficientes de la matriz de rotación:

```
extent_x = |R₀₀| · h.x + |R₀₁| · h.y + |R₀₂| · h.z
extent_y = |R₁₀| · h.x + |R₁₁| · h.y + |R₁₂| · h.z
extent_z = |R₂₀| · h.x + |R₂₁| · h.y + |R₂₂| · h.z
```

```rust
let rx = rot_mat.m11.abs() * half.x + rot_mat.m12.abs() * half.y + rot_mat.m13.abs() * half.z;
```

La intuición es la siguiente: los semiejes locales (`h.x`, `h.y`, `h.z`) se "proyectan" sobre cada eje de mundo. El valor absoluto es necesario porque una proyección negativa también contribuye a la extensión. La suma de estas proyecciones es la extensión máxima posible en ese eje de mundo, independientemente de cuánto esté rotada la caja.

---

## Intersección — El Slab Method

La intersección rayo-caja se resuelve con el **Slab Method** (método de losas). Una "losa" es el espacio entre dos planos paralelos. Una caja rectangular puede verse como la intersección de tres pares de losas (una por eje). El rayo intersecta la caja si y solo si intersecta las tres losas simultáneamente.

### Paso 1: Transformar el rayo al espacio local

Si la caja está rotada, primero se transforma el rayo al **espacio local de la caja** (donde la caja es un AABB sin rotación). Esto simplifica enormemente el problema:

```rust
let local_origin = self.inv_rot_mat * (ray_origin - self.center);
let local_dir    = self.inv_rot_mat * ray_direction;
```

Nota: la traslación (`- center`) se aplica solo al origen del rayo, no a la dirección (los vectores dirección son independientes de la posición).

### Paso 2: Cálculo de t para cada eje

Para cada eje `k ∈ {X, Y, Z}`, las dos losas son los planos `xₖ = -hₖ` y `xₖ = +hₖ` (donde `hₖ` es el semi-tamaño en ese eje). Los valores de `t` donde el rayo atraviesa cada plano son:

```
t₀ = (-hₖ - origin_k) / dir_k    (entrada en la losa)
t₁ = (+hₖ - origin_k) / dir_k    (salida de la losa)
```

Si `dir_k < 0`, el rayo va en dirección negativa y los valores se invierten (t₀ > t₁), por lo que se intercambian. En código, en lugar de intercambiarlos, se detecta la dirección y se calculan directamente en el orden correcto con su normal asociada:

```rust
let (t0x, t1x, nx) = if local_dir.x >= 0.0 {
    ((-half.x - local_origin.x) * inv_d.x,
     ( half.x - local_origin.x) * inv_d.x,
     Vec3::new(-1.0, 0.0, 0.0))   // normal de la losa de entrada en X
} else {
    (( half.x - local_origin.x) * inv_d.x,
     (-half.x - local_origin.x) * inv_d.x,
     Vec3::new(1.0, 0.0, 0.0))    // normal invertida porque el rayo va en -X
};
```

Usar `inv_d.x = 1.0 / local_dir.x` convierte las divisiones en multiplicaciones, que son más rápidas en la mayoría de arquitecturas.

### Paso 3: Acumulación de t_min y t_max

Se acumula el **máximo de las entradas** y el **mínimo de las salidas** a lo largo de los tres ejes:

```
t_min = max(t0x, t0y, t0z)   ← momento en que el rayo entra en las 3 losas
t_max = min(t1x, t1y, t1z)   ← momento en que el rayo sale de alguna losa
```

Si `t_min > t_max`, significa que el rayo sale de una losa antes de entrar en otra: no hay intersección simultánea con las tres, por lo que el rayo no toca la caja.

La normal se actualiza cada vez que `t_min` avanza (cada vez que una nueva losa "gana" como la última en ser entrada). Esto permite saber qué cara fue impactada sin análisis adicional al final:

```rust
if t0x > t_min {
    t_min = t0x;
    normal_near = nx;  // la cara de entrada es ahora la cara X
}
```

### Paso 4: Seleccionar la intersección válida

```rust
let (t, local_normal) = if t_min > 0.0 {
    (t_min, normal_near)   // el rayo entra desde fuera: usar la cara frontal
} else if t_max > 0.0 {
    (t_max, normal_far)    // el rayo parte desde dentro: usar la cara trasera
} else {
    return None;           // la caja está completamente detrás del rayo
}
```

### Paso 5: Alpha cutout y transformación de vuelta

Si el material tiene transparencia, se muestrea el texel antes de confirmar el impacto. Si `a < 128`, el rayo "pasa" como si no hubiera caja.

El punto y la normal se transforman de vuelta al espacio de mundo:

```rust
let world_point  = self.center + self.rot_mat * local_point;
let world_normal = self.rot_mat * local_normal;
```

Para la normal basta con multiplicar por `rot_mat` (sin la traslación) porque las normales son vectores libres (no tienen posición).

---

## Mapeo UV — `get_uv_local`

La cara se identifica por el eje dominante de la normal local. Las otras dos dimensiones del punto local se usan como coordenadas UV (en unidades de espacio 3D, proporcionales al tamaño real):

| Normal local | Eje U | Eje V | Orientación |
|---|---|---|---|
| `|nx| > 0.5` (cara X) | Z | Y | depende del signo de nx |
| `|ny| > 0.5` (cara Y) | X | Z | depende del signo de ny |
| `|nz| > 0.5` (cara Z) | X | Y | depende del signo de nz |

Las coordenadas se orientan para que sean consistentes entre caras (sin espejos ni rotaciones inesperadas al visualizar una textura continua).

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `ray_intersect` | Implementa `RayIntersect`; encapsulado en `Object::Box3D` |
| `camera` | `check_collision` usa el AABB expandido por `collision_radius` para colisiones FPS |
| `renderer` | `cast_ray` lo trata como cualquier otro `Object` |
