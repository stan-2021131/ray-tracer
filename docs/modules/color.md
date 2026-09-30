# Color

## ¿Para qué sirve?

`Color` es la unidad básica de información visual del motor. Representa un color mediante sus canales **rojo, verde, azul y alfa** (RGBA), cada uno como un entero de 8 bits (`u8`, rango 0–255). Prácticamente todos los módulos del proyecto devuelven o consumen colores.

---

## Estructura

```rust
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,  // 255 = opaco, 0 = transparente
}
```

Los colores se almacenan en espacio `[0, 255]` por eficiencia de memoria, pero las operaciones aritméticas los convierten temporalmente a `f32` para evitar pérdida de precisión, y los clampean de vuelta al rango válido al finalizar.

---

## Conversión hexadecimal

### `Color::from_hex(hex: u32) -> Color`

Desempaqueta un entero de 32 bits en formato `0x00RRGGBB` extrayendo cada byte mediante desplazamiento y máscara:

```rust
r: ((hex >> 16) & 0xFF) as u8,
g: ((hex >>  8) & 0xFF) as u8,
b: ( hex        & 0xFF) as u8,
```

`>> N` desplaza los bits `N` posiciones a la derecha, moviendo el byte deseado a la posición más baja. `& 0xFF` aplica la máscara `00000000_11111111` para aislar solo ese byte y descartar el resto.

### `Color::to_hex() -> u32`

El proceso inverso: empaqueta los tres canales en un `u32`:

```rust
((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
```

Es la operación que el framebuffer usa para almacenar píxeles. El canal alfa no se incluye porque el formato de la ventana es RGB de 24 bits.

---

## Interpolación lineal — `lerp(self, other, t) -> Color`

Mezcla dos colores usando la fórmula de interpolación lineal estándar para cada canal:

```
resultado = A * (1 - t) + B * t,   t ∈ [0, 1]
```

Equivalentemente: parte de `A` cuando `t = 0`, llega a `B` cuando `t = 1`. En código:

```rust
let r = (self.r as f32 * (1.0 - t) + other.r as f32 * t).round() as u8;
```

El `.round()` antes de truncar a `u8` minimiza el error de redondeo acumulado, que sería visible en gradientes suaves.

---

## Operadores aritméticos

### `Color + Color` — Suma saturada

Suma canal a canal usando **aritmética saturante**: si el resultado supera 255, se queda en 255 sin desbordarse ni wrappear:

```rust
r: self.r.saturating_add(other.r),
```

Esto es crítico en el renderer: al acumular contribuciones de múltiples luces (`difuso + especular`), los canales no se corrompen cuando la suma supera el máximo. El alfa toma el mayor de los dos valores.

### `Color * f32` — Escalar por intensidad

Multiplica cada canal por un escalar, clampendo al rango `[0, 255]`. El alfa permanece inalterado:

```rust
r: (self.r as f32 * scalar).clamp(0.0, 255.0) as u8,
```

Esta operación implementa la ponderación de intensidad luminosa: `color * (diffuse_factor * light_intensity)`.

### `Color * Color` — Modulación (Hadamard product normalizado)

Multiplica los colores canal a canal en espacio `[0, 1]`, equivalente a `(a/255) * (b/255) * 255 = a*b/255`:

```rust
r: ((self.r as u32 * other.r as u32) / 255) as u8,
```

**Por qué `/ 255` y no `/ 256`:** la división por 255 conserva correctamente los valores extremos. `255 * 255 / 255 = 255` (blanco × blanco = blanco). Con 256 el resultado sería 254, introduciendo un error sistemático.

Esta operación se usa para combinar el color base del material con el texel muestreado: si el material es rojo puro `(255, 0, 0)` y la textura es gris `(128, 128, 128)`, el resultado es rojo oscuro `(128, 0, 0)`.

---

## Relación con el resto del motor

| Módulo | Cómo usa `Color` |
|---|---|
| `framebuffer` | Almacena `color.to_hex()` en cada celda del buffer |
| `texture` | Cada texel del buffer interno es un `Color` |
| `material` | `color` base modulado con `Color * Color` contra la textura |
| `renderer` | Acumula contribuciones de luz con `+` y escala con `* f32` |
| `light` | Define el tinte de color de cada fuente de luz |
