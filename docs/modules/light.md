# Light

## ¿Para qué sirve?

`Light` representa una fuente de luz en la escena. Define su posición, tinte de color e intensidad. El tipo de luz (`Directional` o `Point`) determina si la intensidad se atenúa con la distancia o permanece constante.

---

## Estructura

```rust
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub light_type: LightType,
}
```

---

## Tipos de luz y su modelo de atenuación

### `LightType::Directional`

Modela una fuente de luz que se asume infinitamente lejana (el Sol, la luna). Su intensidad es constante independientemente de la distancia al punto de impacto:

```
atenuación = 1.0
```

El vector de dirección a la luz se calcula igualmente desde `light.position`, pero dado que la posición está muy lejana en práctica, el vector resultante varía poco entre puntos cercanos de la escena.

### `LightType::Point`

Modela una bombilla, antorcha u otra fuente local. Sigue la **ley del cuadrado inverso** aproximada con la fórmula de Blinn (atenuación cuadrática suavizada):

```
atenuación = 1 / (kc + kl·d + kq·d²)
```

Con constantes `kc = 1.0` (constante), `kl = 0.15` (lineal) y `kq = 0.05` (cuadrática):

```rust
LightType::Point => 1.0 / (1.0 + 0.15 * d + 0.05 * d * d)
```

La componente constante (`kc = 1.0`) evita la singularidad a distancia cero (sin ella, la intensidad sería infinita en el punto de la fuente). La componente cuadrática domina a grandes distancias, reproduciendo la caída de intensidad física real de la luz. A distancia 1: ≈ 0.87. A distancia 5: ≈ 0.27. A distancia 10: ≈ 0.10.

Esta fórmula es más barata que la ley del cuadrado inverso pura (`1/d²`) y produce resultados visualmente similares sin la singularidad.

---

## Constructores

### `Light::new(position, color, intensity)` — Luz direccional

```rust
Light::new(Vec3::new(5.0, 8.0, 8.0), Color::new(255, 255, 255), 1.2)
```

La `position` actúa como dirección (vector desde el origen hacia la fuente), ya que las luces direccionales no tienen una distancia real.

### `Light::point(position, color, intensity)` — Luz puntual

```rust
Light::point(Vec3::new(0.0, 3.5, 2.0), Color::new(255, 180, 80), 1.5)
```

La `position` es la posición real en el mundo. El renderer calcula `d = |light.position - intersect.point|` para la atenuación.

---

## Rol del color de la luz

El color de la luz `light.color` modula el componente especular calculado en `shade()`:

```rust
light.color * (specular_intensity * intersect.specular * effective_intensity)
```

Una luz de color naranja `(255, 140, 0)` hará que los brillos especulares tengan ese tinte cálido. El componente difuso usa el color del propio objeto (`intersect.color`), no el de la luz. Esto aproxima el comportamiento real donde la luz coloreada afecta principalmente a los brillos directos.

---

## Relación con el resto del motor

| Módulo | Relación |
|---|---|
| `renderer::shade` | Itera sobre `&[Light]`, calcula atenuación y acumula difuso + especular |
| `renderer::cast_shadow` | Usa `light.position` como destino del rayo de sombra |
| `scene` | Las escenas definen un `Vec<Light>` junto con los objetos |
