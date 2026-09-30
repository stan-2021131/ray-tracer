# Scene

## ¿Para qué sirve?

`Scene` agrupa todos los objetos geométricos y fuentes de luz de una escena en una estructura que el renderer puede consumir directamente. También gestiona la propagación del tiempo a materiales animados.

---

## Estructura

```rust
pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}
```

Diseño deliberadamente simple: una lista plana de objetos y una lista plana de luces. No hay jerarquías ni estructuras de aceleración espacial (BVH, KD-tree). El renderer itera linealmente sobre todos los objetos para cada rayo, lo que es suficiente para escenas de tamaño moderado (decenas a cientos de objetos).

---

## `update_time(time: f32)`

Propaga el tiempo transcurrido (en segundos) a todos los objetos de la escena para actualizar las animaciones de textura:

```rust
for obj in &mut self.objects {
    obj.update_time(time);  // delega a obj.material.update_time(time)
}
```

Internamente, `Material::update_time` selecciona el frame activo del ciclo de animación:

```
frame_idx = floor(time × fps) mod num_frames
```

Esta función solo tiene efecto si el material tiene `animated_frames` configurados.

---

## `has_animations() -> bool`

Cortocircuito de optimización: evita llamar a `update_time` en cada fotograma si la escena no tiene ningún objeto animado.

```rust
self.objects.iter().any(|obj| obj.is_animated())
```

`any` es lazy: evalúa objetos uno a uno y para en cuanto encuentra uno `true`, sin evaluar el resto.

---

## Escenas predefinidas

### `demo_scene() -> Scene`

Escena técnica que demuestra las capacidades del motor sin depender de assets externos. Configura tres tipos de objetos para cubrir todos los casos del renderer:

- **Plano de suelo:** prueba intersección plano-rayo y mapeo UV proyectivo
- **Esfera espejo:** prueba reflexión especular indirecta (kr=0.85)
- **Esfera vidrio:** prueba refracción (kt=0.95, IOR=1.52) y coeficiente Fresnel
- **Esfera oro:** prueba reflexión metálica (kr=0.7) con difusión baja
- **Cubos con materiales sólidos:** prueba el Slab Method y mapeo UV por cara
- **Caja con textura:** prueba el pipeline completo `texture → UV → Color * Color → Intersect`

### `diorama_scene() -> Scene`

Delega a `diorama_builder::build_diorama_scene()`. Su propósito es separar la lógica de construcción de escena del módulo `scene`.

---

## Uso en el loop principal

```rust
let mut scene = scene::demo_scene();
let camera = Camera::new(...);
let skybox  = Skybox::new(...);

loop {
    if scene.has_animations() {
        scene.update_time(elapsed_seconds);
    }
    renderer::render(&mut framebuffer, &scene.objects, &camera, &scene.lights, Some(&skybox));
    // volcar framebuffer a ventana...
}
```

`&scene.objects` y `&scene.lights` se pasan por referencia al renderer. El renderer no modifica la escena: solo la lee para calcular rayos y colores.

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `renderer` | Recibe `&[Object]` y `&[Light]` para `cast_ray` y `shade` |
| `ray_intersect` | `Object` es el enum unificador que implementa `RayIntersect` |
| `light` | `Vec<Light>` define las fuentes de iluminación de la escena |
| `materials` | Cada objeto se construye con un material de `materials::*` |
