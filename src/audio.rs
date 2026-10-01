use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::{Cursor, Read};
use std::time::{Duration, Instant};

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

#[allow(dead_code)]
impl AudioPlayer {
    pub fn new() -> Self {
        // Inicialización segura del dispositivo de audio del sistema
        let (stream, stream_handle, music_sink, ambient_sink) = match OutputStream::try_default() {
            Ok((stream, handle)) => {
                let m_sink = Sink::try_new(&handle).ok();
                let a_sink = Sink::try_new(&handle).ok();
                (Some(stream), Some(handle), m_sink, a_sink)
            }
            Err(err) => {
                eprintln!("[Audio] Advertencia: No se pudo inicializar el dispositivo de audio: {err}");
                (None, None, None, None)
            }
        };

        // Carga de archivos a memoria (rutas placeholder configurables)
        let music_data = Self::load_file_to_memory("./assets/music.ogg");
        let ambient_data = Self::load_file_to_memory("./assets/ambient.ogg");
        let footstep_data = Self::load_file_to_memory("./assets/footstep.ogg");

        let player = AudioPlayer {
            _stream: stream,
            stream_handle,
            music_sink,
            ambient_sink,
            music_data,
            ambient_data,
            footstep_data,
            last_footstep: Instant::now() - Duration::from_secs(10),
        };

        // Volúmenes predeterminados independientes
        player.set_music_volume(0.20);   // Música al 20%
        player.set_ambient_volume(0.05); // Ambiente al 5%

        player
    }

    fn load_file_to_memory(path: &str) -> Option<Vec<u8>> {
        match File::open(path) {
            Ok(mut file) => {
                let mut data = Vec::new();
                if file.read_to_end(&mut data).is_ok() {
                    Some(data)
                } else {
                    eprintln!("[Audio] Advertencia: No se pudo leer el contenido de {path}");
                    None
                }
            }
            Err(_) => {
                eprintln!("[Audio] Nota: Archivo de audio placeholder no encontrado: {path}");
                None
            }
        }
    }

    /// Inicia la música de fondo en bucle infinito
    pub fn start_music(&self) {
        if let (Some(sink), Some(data)) = (&self.music_sink, &self.music_data) {
            if sink.empty() {
                let cursor = Cursor::new(data.clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    sink.append(decoder.repeat_infinite());
                    sink.play();
                } else {
                    eprintln!("[Audio] Falló la decodificación de la música.");
                }
            } else if sink.is_paused() {
                sink.play();
            }
        }
    }

    /// Inicia el sonido ambiente en bucle infinito
    pub fn start_ambient(&self) {
        if let (Some(sink), Some(data)) = (&self.ambient_sink, &self.ambient_data) {
            if sink.empty() {
                let cursor = Cursor::new(data.clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    sink.append(decoder.repeat_infinite());
                    sink.play();
                } else {
                    eprintln!("[Audio] Falló la decodificación del audio ambiental.");
                }
            } else if sink.is_paused() {
                sink.play();
            }
        }
    }

    /// Inicia simultáneamente la música y el sonido ambiente
    pub fn start_all(&self) {
        self.start_music();
        self.start_ambient();
    }

    /// Ajusta el volumen de la música (0.0 = silencio, 1.0 = 100%)
    pub fn set_music_volume(&self, volume: f32) {
        if let Some(sink) = &self.music_sink {
            sink.set_volume(volume);
        }
    }

    /// Ajusta el volumen del sonido ambiental (0.0 = silencio, 1.0 = 100%)
    pub fn set_ambient_volume(&self, volume: f32) {
        if let Some(sink) = &self.ambient_sink {
            sink.set_volume(volume);
        }
    }

    pub fn pause_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
        }
    }

    pub fn pause_ambient(&self) {
        if let Some(sink) = &self.ambient_sink {
            sink.pause();
        }
    }

    pub fn resume_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.play();
        }
    }

    pub fn resume_ambient(&self) {
        if let Some(sink) = &self.ambient_sink {
            sink.play();
        }
    }

    pub fn stop_all(&self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
            sink.clear();
        }
        if let Some(sink) = &self.ambient_sink {
            sink.pause();
            sink.clear();
        }
    }

    /// Reproduce el efecto de pasos con cooldown para movimiento
    pub fn try_play_footstep(&mut self) {
        if self.last_footstep.elapsed() > Duration::from_millis(400) {
            if let (Some(handle), Some(data)) = (&self.stream_handle, &self.footstep_data) {
                let cursor = Cursor::new(data.clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.set_volume(0.55);
                        sink.append(decoder);
                        sink.detach();
                        self.last_footstep = Instant::now();
                    }
                }
            }
        }
    }
}
