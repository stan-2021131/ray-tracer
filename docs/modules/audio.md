# Sistema de Audio y Sonido Ambiental

## ¿Para qué sirve?

El módulo de audio (`src/audio.rs`) proporciona una experiencia sonora inmersiva para el diorama y los modos de exploración interactiva. Permite reproducir **dos canales simultáneos e independientes en bucle continuo** (música de fondo y sonido ambiental) con control de volumen individual, así como la emisión de **efectos de sonido (SFX)** puntuales (como pasos del jugador al desplazarse en modo primera persona).

El sistema funciona de manera **asíncrona y no bloqueante** en un hilo secundario gestionado por la biblioteca [`rodio`](https://crates.io/crates/rodio), por lo que la decodificación y mezcla de audio no afecta en absoluto la tasa de refresco ni el rendimiento del motor de ray tracing.

---

## Características Principales

1. **Doble Canal en Bucle (*Dual Looping Sinks*):**
   - **Canal de Música:** Pista musical en loop infinito (`music.ogg`).
   - **Canal de Ambiente:** Sonidos atmosféricos/ambientales continuos (`ambient.ogg` — viento, noche, grillos, agua, etc.).
2. **Control de Volumen Independiente:** Posibilidad de atenuar o balancear el volumen de cada pista por separado sin que una silencie a la otra.
3. **Efectos de Sonido con Enfriamiento (*Cooldown/Debounce*):** Soporte para reproducir sonidos de pasos (`footstep.ogg`) cuando el usuario se desplaza con `WASD` en modo FPS, evitando solapamientos excesivos mediante un temporizador `Instant`.
4. **Carga en Memoria Previa:** Los archivos de audio se precargan en buffers `Vec<u8>` en memoria RAM al inicializar el módulo, evitando lecturas de disco recurrentes durante la ejecución.
5. **Tolerancia a Fallos (*Graceful Fallback*):** Si el sistema no dispone de un dispositivo de salida de audio o falta alguno de los archivos en la carpeta `assets/`, el motor no sufre pánicos ni detiene el renderizado; simplemente omite la pista ausente y continúa funcionando normalmente.

---

## Estructura del Módulo

```rust
pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    music_sink: Option<Sink>,
    ambient_sink: Option<Sink>,
    music_data: Option<Vec<u8>>,
    ambient_data: Option<Vec<u8>>,
    footstep_data: Option<Vec<u8>>,
    last_footstep: Instant,
}
```

---

## Métodos y Funcionamiento

### 1. Inicialización (`AudioPlayer::new()`)
- Inicializa la salida de audio predeterminada del sistema operativo a través de `OutputStream::try_default()`.
- Crea los dos `Sink` principales (`music_sink` y `ambient_sink`) asociados al mismo `OutputStreamHandle`.
- Carga los archivos de audio en memoria desde las rutas configuradas en `./assets/`.
- Establece los volúmenes predeterminados:
  - Música: `0.35` (35% de ganancia).
  - Ambiente: `0.15` (15% de ganancia).

### 2. Reproducción en Bucle Infinito
Tanto para la música como para el ambiente, se utiliza el adaptador `.repeat_infinite()` sobre el decodificador de la pista:

```rust
pub fn start_music(&self) {
    if let (Some(sink), Some(data)) = (&self.music_sink, &self.music_data) {
        if sink.empty() {
            let cursor = Cursor::new(data.clone());
            if let Ok(decoder) = Decoder::new(cursor) {
                sink.append(decoder.repeat_infinite());
                sink.play();
            }
        } else if sink.is_paused() {
            sink.play();
        }
    }
}
```

### 3. Mezcla y Control de Ganancia
```rust
pub fn set_music_volume(&self, volume: f32);
pub fn set_ambient_volume(&self, volume: f32);
```
Permite ajustar dinámicamente la ganancia entre `0.0` (silencio) y `1.0` (100%).

### 4. Efectos de Sonido Puntuales (`try_play_footstep()`)
Crea un `Sink` temporal independiente que se desconecta con `.detach()`, permitiendo que el efecto de sonido se reproduzca hasta finalizar y se libere automáticamente de la memoria sin pausar las pistas de fondo.

```rust
pub fn try_play_footstep(&mut self) {
    if self.last_footstep.elapsed() > Duration::from_millis(400) {
        if let (Some(handle), Some(data)) = (&self.stream_handle, &self.footstep_data) {
            let cursor = Cursor::new(data.clone());
            if let Ok(decoder) = Decoder::new(cursor) {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(0.35);
                    sink.append(decoder);
                    sink.detach();
                    self.last_footstep = Instant::now();
                }
            }
        }
    }
}
```

---

## Formatos Soportados y Ubicación de Archivos

El decodificador soporta nativamente múltiples formatos de compresión de audio:
- **OGG Vorbis** (`.ogg`) — *Recomendado por su ligereza y calidad*
- **MP3** (`.mp3`)
- **WAV** (`.wav`)
- **FLAC** (`.flac`)

Los archivos deben ubicarse en la carpeta `assets/` del proyecto:
- `assets/music.ogg`: Música temática principal.
- `assets/ambient.ogg`: Sonidos atmosféricos de fondo.
- `assets/footstep.ogg`: Efecto de sonido de pasos.
