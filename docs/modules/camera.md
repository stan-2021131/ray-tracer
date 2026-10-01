# Camera

## ¿Para qué sirve?

La cámara define el punto de vista del observador: posición, orientación y modo de movimiento. Matemáticamente es un **sistema de referencia local** (una base ortonormal del espacio) que permite transformar direcciones del espacio de pantalla al espacio de mundo. También implementa la detección de colisiones del jugador en los modos de primera persona.

---

## Base ortonormal de la cámara

La cámara define tres vectores unitarios mutuamente perpendiculares que describen su orientación en el mundo:

```rust
pub forward: Vec3,  // Dirección de vista (hacia dónde mira)
pub right: Vec3,    // Dirección lateral derecha
pub cam_up: Vec3,   // "Arriba" relativo a la cámara
```

Estos tres vectores forman una **base ortonormal** del espacio de cámara. Se recalculan en `update_basis()` cada vez que cambia la orientación.

---

## `update_basis()` — Construcción de la base

### Modo Orbit

En modo orbital, la dirección de vista apunta de la cámara al centro. La base se construye con dos productos vectoriales encadenados (proceso de Gram-Schmidt):

```rust
let forward = (self.center - self.eye).normalize();
let right   = forward.cross(&self.up).normalize();    // perpendicular a forward y al "arriba" global
let cam_up  = right.cross(&forward).normalize();       // perpendicular a right y a forward
```

El primer producto vectorial `forward × up` da un vector perpendicular al plano definido por la dirección de vista y el eje Y global. El segundo `right × forward` completa la base siendo perpendicular a los otros dos.

### Modos FPS y FreeCam

Aquí la dirección de vista se construye desde los ángulos `yaw` (horizontal) y `pitch` (vertical) usando **coordenadas esféricas a cartesianas**:

```
forward.x = sin(yaw) · cos(pitch)
forward.y = sin(pitch)
forward.z = -cos(yaw) · cos(pitch)
```

```rust
let forward = Vec3::new(
    self.yaw.sin() * self.pitch.cos(),
    self.pitch.sin(),
    -self.yaw.cos() * self.pitch.cos(),
).normalize();
```

- `yaw = 0` → mira en la dirección `-Z` (frente). `yaw` incrementa hacia la derecha.
- `pitch = 0` → mira horizontalmente. `pitch > 0` mira hacia arriba.
- El factor `cos(pitch)` escala las componentes horizontales: cuando se mira directamente arriba (`pitch = π/2`), `cos(pitch) = 0` y solo queda la componente Y.

El límite `PITCH_LIMIT = π/2 - 0.1` evita el problema de gimbal lock (singularidad matemática cuando la dirección de vista es paralela al eje Y global, que haría que `forward × up = 0`).

---

## `basis_change(vector) -> Vec3`

Transforma un vector del **espacio de cámara** (donde `+X = derecha`, `+Y = arriba`, `-Z = adelante`) al **espacio de mundo**. Es la operación clave que conecta la proyección perspectiva con la orientación real de la cámara:

```rust
vector.x * self.right + vector.y * self.cam_up - vector.z * self.forward
```

Matemáticamente es una multiplicación por la matriz formada por las columnas `[right | cam_up | -forward]`. La negación de `forward` es necesaria porque en el sistema de coordenadas del proyecto la cámara "mira" hacia `-Z`, y el renderer genera rayos con componente `z = -1`.

---

## `orbit(delta_yaw, delta_pitch)`

Rota la cámara alrededor del punto `center` manteniendo la distancia constante. La nueva posición se calcula directamente en **coordenadas esféricas**:

```
eye.x = center.x - radius · sin(yaw) · cos(pitch)
eye.y = center.y - radius · sin(pitch)
eye.z = center.z + radius · cos(yaw) · cos(pitch)
```

```rust
let radius = (self.eye - self.center).magnitude();
self.eye = self.center + Vec3::new(
    -radius * self.yaw.sin() * self.pitch.cos(),
    -radius * self.pitch.sin(),
     radius * self.yaw.cos() * self.pitch.cos(),
);
```

Los signos son consistentes con la convención del sistema de coordenadas: `yaw = 0` coloca el ojo en `+Z`, `pitch > 0` lo eleva hacia `+Y`.

---

## `check_collision(pos, objects) -> bool`

Verifica si la posición `pos` penetra algún objeto de la escena. La prueba varía por tipo:

### Esfera

Distancia euclidiana entre `pos` y el centro de la esfera, comparada contra la suma de radios:

```
|pos - center| < radius + collision_radius
```

```rust
let dist = (pos - s.center).magnitude();
if dist < (s.radius + r) { return true; }
```

### Box3D

Prueba OBB (Oriented Bounding Box) en el espacio local de la caja. Si la caja está rotada, se transforma primero `pos` al espacio local con la inversa de la rotación:

```rust
let local_pos = if b.is_rotated {
    b.inv_rot_mat * (pos - b.center)
} else {
    pos - b.center
};
```

Luego se comprueba si el punto local cae dentro de la caja expandida por el radio de colisión:

```
|local_pos.x| ≤ half.x + r  AND  |local_pos.y| ≤ half.y + r  AND  |local_pos.z| ≤ half.z + r
```

Esto es una prueba AABB estándar, que funciona porque en el espacio local la caja está siempre alineada a los ejes.

---

## `move_player` — Separación de ejes para wall sliding

El **patrón de separación de ejes** (SAT simplificado) prueba el movimiento en cada eje de forma independiente. Si el movimiento combinado está bloqueado pero el de un solo eje no, ese eje se aplica igualmente. El efecto es que el jugador "se desliza" a lo largo de las paredes en lugar de detenerse bruscamente:

```rust
// Movimiento proyectado sobre el plano XZ (sin componente vertical en FPS)
let flat_forward = Vec3::new(self.forward.x, 0.0, self.forward.z).normalize();
let flat_right   = Vec3::new(self.right.x, 0.0, self.right.z).normalize();
let move_vec = (flat_forward * forward_input + flat_right * strafe_input) * speed;

// Eje X: probar independientemente
let test_x = Vec3::new(self.eye.x + move_vec.x, self.eye.y, self.eye.z);
if !self.check_collision(&test_x, objects) { self.eye.x = test_x.x; }

// Eje Z: probar independientemente
let test_z = Vec3::new(self.eye.x, self.eye.y, self.eye.z + move_vec.z);
if !self.check_collision(&test_z, objects) { self.eye.z = test_z.z; }
```

La proyección de `forward` y `right` sobre el plano XZ (`y = 0`) elimina la componente vertical para que el jugador se mueva horizontalmente aunque esté mirando hacia arriba o abajo.

---

## `CameraMode::Telescope` — Modo Telescopio Astronómico

En este modo:
- **Posición fija (`eye = (0, 0, 0)`)**: El jugador no se traslada con WASD (`move_player` ignora traslaciones).
- **Rotación Panorámica 360° (`rotate_look`)**:
  - `yaw`: Rotación horizontal ilimitada $360^\circ$ sobre el eje Y.
  - `pitch`: Restringido estrictamente entre la línea del horizonte ($-0.05\,\text{rad} \approx -3^\circ$) y el cenit ($+1.45\,\text{rad} \approx +83^\circ$), impidiendo que el usuario mire hacia el suelo y manteniendo la inmersión cósmica.

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `renderer` | `camera.eye` como origen de rayos; `basis_change` para orientarlos |
| `ray_intersect` | `check_collision` itera sobre `&[Object]` |
| `box3d`, `sphere`, `plane` | Pruebas de colisión específicas por tipo |
| `space_builder` | Cámara en origen para el panorama esférico 360° del espacio |

