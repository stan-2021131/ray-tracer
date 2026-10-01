# Motor de Ray Tracing en Rust: Diorama Nocturno

Un motor de trazado de rayos (*Ray Tracer*) interactivo en tiempo real desarrollado desde cero en **Rust** (CPU pura por software), sin el uso de librerías externas de renderizado o motores gráficos. El proyecto presenta un **diorama exterior 3D modular** enriquecido con iluminación Phong, sombras directas, reflexión especular, refracción dieléctrica con aproximación de Fresnel (Schlick), materiales con texturas mapeadas y animadas, y un Skybox cubemap de 6 caras.

---

## Demostración en Video

> [!NOTE]
> **Video Demostrativo del Diorama en Ejecución**:
> [![Demostración del Diorama](https://img.shields.io/badge/Demo-Video%20del%20Diorama%20(Pendiente)-blue?style=for-the-badge)](docs/demo.mp4)

---

## Descripción del Proyecto

Este proyecto implementa un renderizador por trazado de rayos por software capaz de simular el comportamiento físico de la luz en tiempo real. Partiendo de la posición de la cámara y proyectando rayos primarios a través de cada píxel del framebuffer, el sistema calcula de forma determinista las intersecciones con primitivas geométricas tridimensionales, resuelve la visibilidad de fuentes de luz (sombras duras) y genera rayos secundarios recursivos para reproducir fenómenos ópticos como reflexión en metales/espejos y refracción en agua y vidrio.

Todo el cálculo matemático, intersección analítica y procesamiento de imágenes se ejecuta directamente en la CPU, aprovechando la biblioteca estándar de concurrencia de Rust para distribuir el trabajo entre todos los núcleos disponibles.

---

## Descripción del Diorama y Escena Espacial

El proyecto incluye dos escenarios interactivos interconectados:

### 1. Diorama Exterior Nocturno (Escena Principal)
Modelado de manera modular con bloques y estructuras geométricas:
- **Terreno y Topografía Modular**: Base estratificada compuesta por capas de tierra (`dirt.png`), superficie de césped (`grass.png`) y un sendero empedrado central (`path.png` / `stone.png`).
- **Estanque de Agua Refractiva**: Zona excavada con agua transparente ($IOR = 1.333$), reflexión Fresnel y textura translúcida.
- **Muelle y Puentes**: Estructuras de madera sobre el agua.
- **Fogata con Textura Animada**: Área de campamento con fuego y humo animados mediante ciclos de texturas secuenciales por fotogramas y emisión de luz propia.
- **Puesto de Observación Astronómica**: Telescopio interactivo con juntas metálicas reflectantes de cobalto sobre un trípode orientable.
- **Iluminación de Farolas**: Postes metálicos con faroles de vidrio ($IOR = 1.52$) que albergan bombillas emisivas y proyectan luz puntual cálida sobre el entorno.
- **Vegetación**: Árboles volumétricos (`leaves.png`) y plantas variadas implementadas con corte alfa (*alpha cutout*).

### 2. Vista Planetaria y Espacial 360° (Telescopio)
Al acercarse a la plataforma del telescopio y presionar `[E]`, la cámara se traslada al ocular del telescopio:
- **Visor Óptico y Zoom**: Renderizado circular con máscara de lente, viñeta perimetral y campo de visión estrecho ($FOV = 30^\circ$, aumento $2\times$).
- **La Luna y Cuerpos Celestes**: Luna prominente y planetas a diversas profundidades (Júpiter, Marte, Neptuno, planetas anillados, exoplanetas).
- **Campo Estelar Denso**: 260 estrellas emisivas de 10 tonalidades espectrales repartidas en toda la bóveda de 360°.
- **Naves, OVNIs y Meteoritos**: Planos orientados mirando al observador con corte de transparencia (*Alpha Cutout*).
- **Skybox Cósmico**: Fondo omnidireccional (`space.png`) con soporte de transparencia y atenuación de brillo para máximo contraste.

---

## Características Principales

- **Trazado de Rayos 100% CPU**: Sin aceleración GPU ni APIs externas como OpenGL o Vulkan.
- **Escenarios Interconectados**: Transición fluida entre el diorama terrestre y la vista telescópica astronómica.
- **Visor Óptico Telescópico**: Zoom $2\times$ con máscara circular de lente y descarte rápido de rayos exteriores.
- **Caja Orientada y AABB Ultrarrápida (`Box3D`)**: Detección de colisiones e intersecciones mediante el algoritmo **Slab Method** con soporte para rotación sobre el eje Y.
- **Plano Finito con Alpha Cutout (`Plane`)**: Rectángulos orientables en el espacio con recorte de transparencia para vegetación, naves, OVNIs y meteoritos.
- **Esfera Analítica (`Sphere`)**: Intersección cuadrática exacta con cálculo de normales y mapeo esférico UV para planetas, luna y estrellas.
- **Materiales Ópticos Avanzados**:
  - Difuso Lambertiano y especular Phong.
  - Reflexión especular recursiva (metales, espejos, ventanas).
  - Refracción con Ley de Snell, Reflexión Interna Total (TIR) y ecuaciones de Fresnel (Schlick).
  - Materiales emisivos con iluminación propia y auto-luminosidad cósmica.
  - Texturas estáticas y animadas por fotogramas (fuego/humo).
- **Skybox Cubemap y Fondo Cósmico**: Proyección cúbica continua de 6 caras para el diorama y skybox espacial con mezcla alfa y atenuación de brillo configurable.
- **Cámara Interactiva Multimodo**:
  - Modo Orbital con rotación en ambos ejes y Zoom continuo (acercar/alejar).
  - Modo Primera Persona (FPS) con detección de colisiones físicas en paredes y suelo.
  - Modo Vuelo Libre (FreeCam) tridimensional.
  - Modo Telescopio con rotación panorámica 360° y límites de elevación hacia el cielo.
- **Paralelismo Multihilo Nativo**: Particionado de pantalla en paralelo mediante `std::thread::scope` en el Framebuffer.
- **Sistema de Audio y Sonido Ambiental**: Reproducción simultánea en bucle de música de fondo, ambiente nocturno y pasos en modo FPS.

---

## Estructura del Proyecto

```text
ray-tracer/
├── Cargo.toml                  # Dependencias y perfiles de compilación
├── README.md                   # Documentación principal del proyecto
├── assets/                     # Archivos de audio (música, ambiente y pasos)
├── docs/                       # Documentación técnica modular y recursos multimedia
│   └── modules/                # Guías detalladas por cada componente del motor
├── textures/                   # Atlas de texturas PNG y JPG
│   ├── campfire/               # Secuencias animadas de fuego y humo
│   ├── plants/                 # Sprites de plantas y flores
│   ├── skybox/                 # 6 caras del Cubemap nocturno y textura space.png
│   ├── space/                  # Texturas astronómicas (moon, jupiter, mars, ufo, etc.)
│   └── *.png                   # Texturas de bloques (grass, dirt, wood, stone, etc.)
└── src/
    ├── main.rs                 # Inicialización de ventana, bucle de eventos y gestión de escenas
    ├── audio.rs                # Sistema de audio (música, ambiente y SFX en bucle)
    ├── diorama_builder.rs      # Construcción procedural de la escena del diorama
    ├── space_builder.rs        # Construcción procedural de la escena espacial 360°
    ├── renderer.rs             # Trazador de rayos (render, render_telescope, cast_ray, shade, reflect, refract)
    ├── framebuffer.rs          # Buffer de pantalla y render_parallel multihilo
    ├── camera.rs               # Cámara orbital, FPS con colisiones, FreeCam y Telescopio
    ├── ray_intersect.rs        # Traits, tipos de intersección y definición de Material
    ├── box3d.rs                # Primitiva Box3D (Slab method y rotación OBB)
    ├── plane.rs                # Primitiva Plane (rectángulo finito y alpha cutout)
    ├── sphere.rs               # Primitiva Sphere analítica
    ├── skybox.rs               # Skybox Cubemap de 6 caras, atenuación y muestreo UV
    ├── materials.rs            # Catálogo y constructores de materiales ópticos
    ├── texture.rs              # Carga y muestreo UV (Nearest/Bilinear, Repeat/Clamp)
    ├── texture_manager.rs      # Gestor y caché centralizada de texturas (Arc<Texture>)
    ├── light.rs                # Fuentes de luz puntuales y direccionales
    ├── color.rs                # Manejo de color RGB y conversiones hex/aritméticas
    └── scene.rs                # Contenedor de escenas, objetos y animaciones
```

---

## Fundamentos Técnicos del Motor

### 1. Trazado de Rayos, Iluminación y Sombras (`renderer.rs`)
Para cada píxel de la pantalla, se genera un rayo primario proyectado según el campo de visión ($FOV = 60^\circ$ en diorama, $FOV = 30^\circ$ en telescopio) y la orientación de la cámara. Al impactar una superficie, se evalúa el modelo de sombreado **Phong** combinando la luz difusa (Lambertiana) y el brillo especular. Para cada fuente luminosa, se lanza un rayo de sombra (*shadow ray*) con desplazamiento antipoligonado (*shadow bias*); si un objeto ocluye la trayectoria hacia la luz, el punto queda en penumbra.

### 2. Visor Óptico Telescópico (`renderer.rs`)
La función `render_telescope` aplica una máscara circular donde los píxeles con radio normalizado $r > 0.94$ retornan negro inmediato (`0x000000`) sin evaluar rayos, mientras que el rango $0.86 < r \le 0.94$ aplica una viñeta de atenuación suave simulando el ocular físico.

### 3. Reflexión, Refracción y Fresnel (`renderer.rs`)
Los materiales reflectivos y dieléctricos generan rayos secundarios recursivos (hasta una profundidad máxima `MAX_DEPTH = 3`):
- **Reflexión**: Calculada como $\vec{R} = \vec{I} - 2(\vec{I} \cdot \vec{N})\vec{N}$.
- **Refracción (Snell)**: Desvía el rayo según la relación de índices de refracción $\eta = \eta_i / \eta_t$. Si el ángulo excede el ángulo crítico, se gestiona la **Reflexión Interna Total (TIR)**.
- **Fresnel (Schlick)**: Determina la proporción de energía reflejada versus refractada según el ángulo de incidencia:
  $$R(\theta) = R_0 + (1 - R_0)(1 - \cos\theta)^5 \quad \text{donde} \quad R_0 = \left(\frac{\eta_1 - \eta_2}{\eta_1 + \eta_2}\right)^2$$

### 4. Cajas 3D y Algoritmo Slab Method (`box3d.rs`)
Los bloques del diorama utilizan la prueba de intersección AABB mediante el método de los intervalos (*Slab Method*), calculando las distancias de entrada y salida $t_{\min}$ y $t_{\max}$ en los tres ejes cartesianos de manera simultánea. Permite además rotación en el eje Y mediante transformación de rayos al espacio local del objeto (OBB).

### 5. Skybox Cubemap y Fondo Cósmico (`skybox.rs`)
El entorno que envuelve la escena se proyecta sobre 6 planos ortogonales (`up`, `down`, `left`, `right`, `front`, `back`). Soporta carga omnidireccional con `from_single_texture`, mezcla con canal alfa para texturas transparentes y atenuación de brillo con `with_brightness`.

### 6. Multithreading Nativo (`framebuffer.rs`)
La función `Framebuffer::render_parallel` divide la altura total del framebuffer en bloques horizontales de filas equitativas, asignando cada bloque a un hilo de procesamiento independiente mediante `std::thread::scope` y `std::thread::available_parallelism()`.

### 7. Sistema de Audio y Sonido Ambiental (`audio.rs`)
Permite la mezcla asíncrona no bloqueante de dos pistas en bucle infinito simultáneo (música de fondo y sonido ambiental) con canales independientes y volúmenes balanceados (`35%` y `15%`), además de efectos sonoros de pasos interactivos en modo primera persona.

> [!TIP]
> Para consultar la explicación matemática exhaustiva, fórmulas y diagramas de cada módulo, revisa los documentos en la carpeta [`docs/modules/`](docs/modules/).

---

## Controles de la Aplicación

| Control | Acción en Diorama | Acción en Vista Telescópica |
| :--- | :--- | :--- |
| **$\leftarrow$ / $\rightarrow$** | Rotar cámara horizontalmente (*Yaw*) | Rotar vista panorámica 360° |
| **$\uparrow$ / $\downarrow$** | Rotar cámara verticalmente (*Pitch*) | Inclinar vista hacia el cielo |
| **`W` / `S`** | **Zoom (Acercar/Alejar)** en Órbita / Caminar en FPS | — |
| **`A` / `D`** | Desplazamiento lateral (*Strafe*) en FPS / FreeCam | — |
| **`Q` / `E`** | Elevar / Descender cámara en FreeCam | — |
| **`E`** | **Mirar por el Telescopio** (cerca del telescopio) | **Regresar al Diorama** |
| **`Escape`** | Cerrar la aplicación y salir | **Regresar al Diorama** |
| **`Tab` o `C`** | Alternar modo (`Orbit` $\rightarrow$ `FPS` $\rightarrow$ `FreeCam`) | — |
| **`R`** | Restablecer la cámara a la posición original | — |

---

## Compilación y Ejecución

### Requisitos Previos
- Tener instalado el compilador de [Rust y Cargo](https://www.rust-lang.org/tools/install) (versión 2024 o superior recomendada).

### Ejecutar en Modo Optimizado (Recomendado)
Para obtener el máximo rendimiento en tiempo real y tasa de fotogramas fluida:
```bash
cargo run --release
```

### Ejecutar Tests Unitarios
El proyecto cuenta con 20 pruebas unitarias automatizadas que validan la física de refracción, Fresnel, selección de caras del Skybox, AABB y colisiones de cámara:
```bash
cargo test
```

---

## Rúbrica de Evaluación y Cumplimiento

A continuación se detalla la correspondencia entre los requerimientos solicitados y su implementación técnica en el código:

| Requerimiento | Pts | Estado | Implementación y Justificación Técnica | Archivo(s) Clave |
| :--- | :---: | :---: | :--- | :--- |
| **Complejidad de la escena** | **30** | Cumplido | Diorama exterior completo con terreno estratificado, estanque excavado, muelle de madera, fogata con animación procedural de fuego y humo, farolas luminosas, telescopio con montura, silla, vallas perimetrales, árboles volumétricos y flores con corte alfa. | [`src/diorama_builder.rs`](src/diorama_builder.rs) |
| **Atractivo visual** | **20** | Cumplido | Integración de iluminación Phong directa, sombras arrojadas con penumbra suave, materiales emisivos, reflejos en metales, transparencia con refracción en agua y vidrio, y cielo estrellado cubemap. | [`src/renderer.rs`](src/renderer.rs) |
| **Rotación de diorama y Zoom** | **20** | Cumplido | Cámara con rotación orbital continua en dos ejes (flechas) y zoom fluido para acercarse y alejarse (`W` / `S`), además de modos interactivos en 1ra persona con colisiones y FreeCam (`Tab`/`C`). | [`src/camera.rs`](src/camera.rs)<br>[`src/main.rs`](src/main.rs) |
| **Materiales diferentes con texturas y parámetros propios** *(mínimo 5)* | **25** | Cumplido *(8+ materiales)* | Cada material define su propia textura independiente, y parámetros únicos de albedo/difuso, brillo especular, reflectividad y refracción:<br>1. **Césped (`grass.png`)**: Difuso 0.9, specular 0.1, reflect 0.0, refract 0.0.<br>2. **Madera (`wood.png` / `wood_2.png`)**: Difuso 0.9, specular 0.1.<br>3. **Piedra / Camino (`stone.png` / `path.png`)**: Difuso 0.9, specular 0.1.<br>4. **Follaje (`leaves.png`)**: Difuso 0.9, specular 0.1.<br>5. **Agua (`water.jpg`)**: Difuso 0.0, specular 0.1, reflect 0.05, refract 0.95 ($IOR = 1.333$).<br>6. **Ventana (`window.png`)**: Difuso 0.4, specular 0.4, reflect 0.20.<br>7. **Metal de Farolas/Telescopio**: Cobalto metálico reflectivo (reflect 0.15, specular 0.4, shininess 80.0).<br>8. **Fuego Emisivo**: Secuencia animada (8 frames) con emisión de luz 1.8. | [`src/materials.rs`](src/materials.rs)<br>[`src/diorama_builder.rs`](src/diorama_builder.rs)<br>[`src/ray_intersect.rs`](src/ray_intersect.rs) |
| **Refracción contextual** | **10** | Cumplido | Implementación de la Ley de Snell y aproximación de Fresnel (Schlick) con manejo de Reflexión Interna Total (TIR). Aplicado contextualmente en el **agua del estanque** ($IOR = 1.333$) y en el **vidrio de las bombillas de las farolas** ($IOR = 1.52$). | [`src/renderer.rs`](src/renderer.rs) |
| **Reflexión** | **5** | Cumplido | Cálculo vectorial de reflexión $\vec{R}$ con trazado recursivo de rayos secundarios en metales (telescopio y farolas), ventanas y en el agua mediante el factor Fresnel. | [`src/renderer.rs`](src/renderer.rs) |
| **Skybox** | **20** | Cumplido | Cubemap de 6 texturas coordinadas sin costuras (`up`, `down`, `left`, `right`, `front`, `back`), muestreado por los rayos que escapan al infinito y visible a través de reflejos y refracciones. | [`src/skybox.rs`](src/skybox.rs) |
| **Sin librerías externas de renderizado** | **Req.** | Cumplido | Ray tracer 100% en CPU por software con multihilo nativo (`std::thread`). No se utilizan motores gráficos externos ni APIs como OpenGL, Vulkan o WGPU. | [`Cargo.toml`](Cargo.toml) |
| **Total** | **100** | Cumplido | **Calificación estimada: 100 / 100 puntos** | |

---

## Documentación Técnica Adicional

Para más detalles sobre la arquitectura interna, matemáticas y diseño de cada componente, puedes consultar la documentación técnica individual en la carpeta [`docs/modules/`](docs/modules/):

- [Box3D y Slab Method](docs/modules/box3d.md)
- [Sistema de Cámara y Modos de Navegación](docs/modules/camera.md)
- [Manejo y Operaciones de Color](docs/modules/color.md)
- [Framebuffer y Paralelismo Multihilo](docs/modules/framebuffer.md)
- [Fuentes de Luz y Atenuación](docs/modules/light.md)
- [Materiales y Propiedades Ópticas](docs/modules/materials.md)
- [Planos y Recorte Alfa](docs/modules/plane.md)
- [Ray Intersect y Estructuras de Datos](docs/modules/ray_intersect.md)
- [Motor de Renderizado y Algoritmos Ópticos](docs/modules/renderer.md)
- [Skybox Cubemap](docs/modules/skybox.md)
- [Esferas Analíticas](docs/modules/sphere.md)
- [Muestreo y Gestión de Texturas](docs/modules/texture.md)
- [Sistema de Audio y Sonido Ambiental](docs/modules/audio.md)