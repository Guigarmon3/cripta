use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::get_time;

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum SfxKind {
    Hit = 0,
    Crit,
    Shoot,
    Hurt,
    Die,
    Level,
    Pickup,
    Dash,
    Break,
    Boss,
    Click,
    Portal,
    Lock,
}

const COUNT: usize = 13;

/// Voz propia de cada familia de enemigos. Suenan con el volumen de enemigos,
/// separado del volumen general de efectos.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum EnemyVoice {
    Squelch = 0,
    Screech,
    Growl,
    Chant,
    Wail,
    Fuse,
    Zap,
    Clang,
    Roar,
}

pub const ENEMY_VOICE_COUNT: usize = 9;

/// Capas instrumentales de la música. Cada piso añade una; el guardián final
/// las toca todas juntas.
const LAYERS: usize = 4;

pub struct Sfx {
    sounds: [Option<Sound>; COUNT],
    last: [f32; COUNT],
    enemy_sounds: [Option<Sound>; ENEMY_VOICE_COUNT],
    last_enemy: [f32; ENEMY_VOICE_COUNT],
    music_layers: [Option<Sound>; LAYERS],
    layer_playing: [bool; LAYERS],
    music_floor: u32,
    music_on: bool,
    music_playing: bool,
    /// Motivo vocal del arquetipo enemigo que domina la sala.
    motif_sounds: [Option<Sound>; ENEMY_VOICE_COUNT],
    motif_playing: Option<usize>,
    pub music_volume: u8,
    pub sfx_volume: u8,
    pub enemy_volume: u8,
}

fn wav_bytes(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&sample_rate.to_le_bytes());
    v.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        v.extend_from_slice(&s.to_le_bytes());
    }
    v
}

#[derive(Clone, Copy)]
enum Wave {
    Square,
    Saw,
    Noise,
    Sine,
}

fn tone(f0: f32, f1: f32, dur: f32, vol: f32, wave: Wave) -> Vec<u8> {
    let sr = 22050u32;
    let n = (sr as f32 * dur).max(1.0) as usize;
    let mut samples = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    let mut noise = 0x1234_5678u32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * t;
        phase += f / sr as f32;
        let s = match wave {
            Wave::Square => {
                if phase.fract() < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Saw => phase.fract() * 2.0 - 1.0,
            Wave::Sine => (phase * std::f32::consts::TAU).sin(),
            Wave::Noise => {
                noise = noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (noise >> 16) as f32 / 32768.0 - 1.0
            }
        };
        let attack = (t * 160.0).min(1.0);
        let env = attack * (1.0 - t).powf(2.0);
        samples.push((s * env * vol * i16::MAX as f32) as i16);
    }
    wav_bytes(&samples, sr)
}

fn sequence(notes: &[(f32, f32)], vol: f32, wave: Wave) -> Vec<u8> {
    let sr = 22050u32;
    let mut samples: Vec<i16> = Vec::new();
    for &(f, d) in notes {
        let n = (sr as f32 * d).max(1.0) as usize;
        let mut phase = 0.0f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            phase += f / sr as f32;
            let s = match wave {
                Wave::Square => {
                    if phase.fract() < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Saw => phase.fract() * 2.0 - 1.0,
                Wave::Sine => (phase * std::f32::consts::TAU).sin(),
                Wave::Noise => 0.0,
            };
            let env = (t * 80.0).min(1.0) * (1.0 - t).powf(1.6);
            samples.push((s * env * vol * i16::MAX as f32) as i16);
        }
    }
    wav_bytes(&samples, sr)
}

/// Música ritual generada en memoria. La obra está dividida en capas que
/// comparten duración y modo (La menor), de modo que pueden entrar y salir
/// sin chocar: drone, arpegio de campanas, coro y percusión.
fn music_layer(layer: usize) -> Vec<u8> {
    let sr = 22050u32;
    let total = 12.8f32;
    let n = (sr as f32 * total) as usize;
    let mut buf = vec![0.0f32; n];

    for i in 0..n {
        let t = i as f32 / sr as f32;
        match layer {
            0 => {
                // Drone de la cripta: fundamental y quinta, con pulso lento.
                let pulse = 0.75 + 0.25 * (t * 0.35 * std::f32::consts::TAU).sin();
                buf[i] += (t * 55.0 * std::f32::consts::TAU).sin() * 0.12 * pulse;
                buf[i] += (t * 82.41 * std::f32::consts::TAU).sin() * 0.075 * pulse;
                buf[i] += (t * 110.0 * std::f32::consts::TAU).sin() * 0.035;
            }
            1 => {
                // Arpegio de campanas sobre la escala pentatónica de La menor.
                let step = (t / 0.4) as usize;
                let lt = t % 0.4;
                let notes = [
                    220.0f32, 261.63, 329.63, 392.0, 329.63, 261.63, 196.0, 246.94,
                ];
                let f = notes[step % notes.len()];
                let env = (-lt * 5.5).exp() * (1.0 - (-lt * 90.0).exp());
                buf[i] += (t * f * std::f32::consts::TAU).sin() * 0.055 * env;
                buf[i] += (t * f * 2.0 * std::f32::consts::TAU).sin() * 0.018 * env;
            }
            2 => {
                // Coro lejano: tres voces con vibrato y respiración lenta.
                let swell = 0.45 + 0.55 * (t * 0.22 * std::f32::consts::TAU).sin().abs();
                for (k, f) in [220.0f32, 261.63, 329.63].iter().enumerate() {
                    let vib = 1.0 + (t * 5.2 + k as f32).sin() * 0.006;
                    let w = t * f * vib * std::f32::consts::TAU;
                    buf[i] += ((w).sin() * 0.5 + (w * 2.0).sin() * 0.16) * 0.032 * swell;
                }
            }
            _ => {
                // Percusión ritual: tambor grave y sonaja de hueso.
                let beat = t % 1.6;
                if beat < 0.22 {
                    let p = beat / 0.22;
                    let f = 78.0 - 40.0 * p;
                    buf[i] += (beat * f * std::f32::consts::TAU).sin() * 0.20 * (1.0 - p).powf(2.2);
                }
                let shake = t % 0.4;
                if shake < 0.045 {
                    let mut noise = (i as u32)
                        .wrapping_mul(1_664_525)
                        .wrapping_add(1_013_904_223);
                    noise ^= noise >> 13;
                    let s = (noise >> 8) as f32 / 8_388_608.0 - 1.0;
                    buf[i] += s * 0.055 * (1.0 - shake / 0.045);
                }
            }
        }
    }

    // Eco corto para dar profundidad a las capas melódicas.
    if layer != 3 {
        let delay = (sr as f32 * 0.31) as usize;
        let mut delay_buf = vec![0.0f32; delay.max(1)];
        for i in 0..n {
            let d = delay_buf[i % delay.max(1)];
            let dry = buf[i];
            delay_buf[i % delay.max(1)] = dry + d * 0.28;
            buf[i] = dry + d * 0.22;
        }
    }

    // Entrada y salida breves para que el bucle no chasquee.
    let fade = (sr as f32 * 0.06) as usize;
    for i in 0..n {
        let mut g = 1.0;
        if i < fade {
            g *= i as f32 / fade as f32;
        }
        if i + fade > n {
            g *= (n - i) as f32 / fade as f32;
        }
        buf[i] *= g;
    }

    let samples: Vec<i16> = buf
        .iter()
        .map(|v| (v.clamp(-1.0, 1.0) * 0.82 * i16::MAX as f32) as i16)
        .collect();
    wav_bytes(&samples, sr)
}

/// Motivo vocal corto de cada arquetipo. Suena en bucle mientras la sala
/// está en combate, por debajo de la música, como si los enemigos cantaran.
fn voice_motif(voice: EnemyVoice) -> Vec<u8> {
    let sr = 22050u32;
    let notes: &[(f32, f32)] = match voice {
        EnemyVoice::Squelch => &[(130.0, 0.18), (98.0, 0.14), (78.0, 0.24)],
        EnemyVoice::Screech => &[(980.0, 0.10), (1240.0, 0.08), (860.0, 0.16)],
        EnemyVoice::Growl => &[(150.0, 0.16), (132.0, 0.16), (112.0, 0.26)],
        EnemyVoice::Chant => &[(196.0, 0.22), (164.81, 0.22), (130.81, 0.36)],
        EnemyVoice::Wail => &[(523.25, 0.24), (466.16, 0.22), (392.0, 0.36)],
        EnemyVoice::Fuse => &[(740.0, 0.07), (880.0, 0.07), (740.0, 0.07), (988.0, 0.14)],
        EnemyVoice::Zap => &[(1318.0, 0.06), (1046.0, 0.06), (1568.0, 0.12)],
        EnemyVoice::Clang => &[(330.0, 0.12), (392.0, 0.10), (262.0, 0.20)],
        EnemyVoice::Roar => &[(98.0, 0.24), (82.0, 0.22), (65.0, 0.42)],
    };
    let gap = 0.035f32;
    let mut samples: Vec<i16> = Vec::new();
    for &(f, d) in notes {
        let n = (sr as f32 * d).max(1.0) as usize;
        let mut phase = 0.0f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            let vib = 1.0 + (t * d * 42.0).sin() * 0.016;
            phase += f * vib / sr as f32;
            let w = phase * std::f32::consts::TAU;
            let s = w.sin() * 0.55 + (w * 2.0).sin() * 0.24 + (w * 3.0).sin() * 0.10;
            let env = (t * 45.0).min(1.0) * (1.0 - t).powf(1.4);
            samples.push((s * env * 0.30 * i16::MAX as f32) as i16);
        }
        for _ in 0..(sr as f32 * gap) as usize {
            samples.push(0);
        }
    }
    // Pequeño fundido en los extremos del bucle.
    let fade = (sr as f32 * 0.02) as usize;
    for i in 0..fade.min(samples.len()) {
        let g = i as f32 / fade as f32;
        samples[i] = (samples[i] as f32 * g) as i16;
        let j = samples.len() - 1 - i;
        samples[j] = (samples[j] as f32 * g) as i16;
    }
    wav_bytes(&samples, sr)
}

/// Ganancia relativa de cada capa para que la mezcla no sature.
/// Exporta las cuatro capas de música a WAV para la postproducción de la guía.
/// Es una función pura: no toca el estado de audio del juego.
pub fn export_music_layers(dir: &std::path::Path) {
    let _ = std::fs::create_dir_all(dir);
    for layer in 0..LAYERS {
        let path = dir.join(format!("music_{layer}.wav"));
        let _ = std::fs::write(path, music_layer(layer));
    }
}

fn layer_gain(layer: usize) -> f32 {
    match layer {
        0 => 0.62,
        1 => 0.42,
        2 => 0.34,
        _ => 0.40,
    }
}

impl Sfx {
    pub async fn load() -> Self {
        let mut s = Self {
            sounds: std::array::from_fn(|_| None),
            last: [0.0; COUNT],
            enemy_sounds: std::array::from_fn(|_| None),
            last_enemy: [0.0; ENEMY_VOICE_COUNT],
            music_layers: std::array::from_fn(|_| None),
            layer_playing: [false; LAYERS],
            music_floor: 1,
            music_on: true,
            music_playing: false,
            motif_sounds: std::array::from_fn(|_| None),
            motif_playing: None,
            music_volume: 30,
            sfx_volume: 80,
            enemy_volume: 70,
        };
        for i in 0..LAYERS {
            s.music_layers[i] = load_sound_from_bytes(&music_layer(i)).await.ok();
        }
        s.sounds[SfxKind::Hit as usize] =
            load_sound_from_bytes(&tone(240.0, 120.0, 0.07, 0.32, Wave::Square))
                .await
                .ok();
        s.sounds[SfxKind::Crit as usize] =
            load_sound_from_bytes(&tone(520.0, 260.0, 0.10, 0.38, Wave::Saw))
                .await
                .ok();
        s.sounds[SfxKind::Shoot as usize] =
            load_sound_from_bytes(&tone(760.0, 480.0, 0.05, 0.22, Wave::Square))
                .await
                .ok();
        s.sounds[SfxKind::Hurt as usize] =
            load_sound_from_bytes(&tone(180.0, 80.0, 0.16, 0.40, Wave::Saw))
                .await
                .ok();
        s.sounds[SfxKind::Die as usize] =
            load_sound_from_bytes(&tone(160.0, 50.0, 0.18, 0.30, Wave::Noise))
                .await
                .ok();
        s.sounds[SfxKind::Level as usize] = load_sound_from_bytes(&sequence(
            &[(523.0, 0.09), (659.0, 0.09), (784.0, 0.16)],
            0.32,
            Wave::Square,
        ))
        .await
        .ok();
        s.sounds[SfxKind::Pickup as usize] =
            load_sound_from_bytes(&tone(880.0, 1320.0, 0.06, 0.22, Wave::Sine))
                .await
                .ok();
        s.sounds[SfxKind::Dash as usize] =
            load_sound_from_bytes(&tone(420.0, 180.0, 0.09, 0.18, Wave::Noise))
                .await
                .ok();
        s.sounds[SfxKind::Break as usize] =
            load_sound_from_bytes(&tone(300.0, 90.0, 0.11, 0.30, Wave::Noise))
                .await
                .ok();
        s.sounds[SfxKind::Boss as usize] =
            load_sound_from_bytes(&tone(90.0, 45.0, 0.45, 0.42, Wave::Saw))
                .await
                .ok();
        s.sounds[SfxKind::Click as usize] =
            load_sound_from_bytes(&tone(620.0, 620.0, 0.03, 0.18, Wave::Square))
                .await
                .ok();
        // Candado cerrado: tres golpes secos de cerradura. Suena cuando una
        // puerta, una bóveda o un acceso bloqueado se niegan a abrirse.
        s.sounds[SfxKind::Lock as usize] = load_sound_from_bytes(&sequence(
            &[(210.0, 0.045), (120.0, 0.055), (78.0, 0.14)],
            0.36,
            Wave::Square,
        ))
        .await
        .ok();
        s.sounds[SfxKind::Portal as usize] = load_sound_from_bytes(&sequence(
            &[(392.0, 0.08), (523.0, 0.08), (659.0, 0.18)],
            0.26,
            Wave::Sine,
        ))
        .await
        .ok();

        // Voces de enemigos: cada familia tiene una textura distinta.
        s.enemy_sounds[EnemyVoice::Squelch as usize] =
            load_sound_from_bytes(&tone(210.0, 60.0, 0.13, 0.30, Wave::Noise))
                .await
                .ok();
        s.enemy_sounds[EnemyVoice::Screech as usize] =
            load_sound_from_bytes(&tone(1350.0, 820.0, 0.09, 0.20, Wave::Saw))
                .await
                .ok();
        s.enemy_sounds[EnemyVoice::Growl as usize] =
            load_sound_from_bytes(&tone(120.0, 52.0, 0.24, 0.30, Wave::Saw))
                .await
                .ok();
        s.enemy_sounds[EnemyVoice::Chant as usize] = load_sound_from_bytes(&sequence(
            &[(196.0, 0.12), (147.0, 0.12), (110.0, 0.20)],
            0.22,
            Wave::Sine,
        ))
        .await
        .ok();
        s.enemy_sounds[EnemyVoice::Wail as usize] =
            load_sound_from_bytes(&tone(720.0, 260.0, 0.34, 0.22, Wave::Sine))
                .await
                .ok();
        s.enemy_sounds[EnemyVoice::Fuse as usize] = load_sound_from_bytes(&sequence(
            &[(880.0, 0.035), (1040.0, 0.035), (920.0, 0.035)],
            0.16,
            Wave::Square,
        ))
        .await
        .ok();
        s.enemy_sounds[EnemyVoice::Zap as usize] =
            load_sound_from_bytes(&tone(1500.0, 420.0, 0.08, 0.20, Wave::Square))
                .await
                .ok();
        s.enemy_sounds[EnemyVoice::Clang as usize] =
            load_sound_from_bytes(&tone(340.0, 160.0, 0.12, 0.24, Wave::Noise))
                .await
                .ok();
        s.enemy_sounds[EnemyVoice::Roar as usize] =
            load_sound_from_bytes(&tone(78.0, 32.0, 0.55, 0.44, Wave::Saw))
                .await
                .ok();
        for i in 0..ENEMY_VOICE_COUNT {
            let voice = match i {
                0 => EnemyVoice::Squelch,
                1 => EnemyVoice::Screech,
                2 => EnemyVoice::Growl,
                3 => EnemyVoice::Chant,
                4 => EnemyVoice::Wail,
                5 => EnemyVoice::Fuse,
                6 => EnemyVoice::Zap,
                7 => EnemyVoice::Clang,
                _ => EnemyVoice::Roar,
            };
            s.motif_sounds[i] = load_sound_from_bytes(&voice_motif(voice)).await.ok();
        }
        s
    }

    /// Ajusta los tres volúmenes a la vez (0..100).
    pub fn set_volumes(&mut self, music: u8, sfx: u8, enemy: u8) {
        self.music_volume = music.min(100);
        self.sfx_volume = sfx.min(100);
        self.enemy_volume = enemy.min(100);
        if self.music_playing {
            self.restart_layers();
        }
        if let Some(i) = self.motif_playing {
            if let Some(snd) = &self.motif_sounds[i] {
                stop_sound(snd);
            }
            self.motif_playing = None;
            self.start_motif(i);
        }
    }

    fn active_layers(&self) -> usize {
        if self.music_floor % 10 == 0 {
            LAYERS
        } else {
            (self.music_floor as usize).clamp(1, LAYERS)
        }
    }

    fn start_layer(&mut self, i: usize) {
        if self.layer_playing[i] || i >= LAYERS {
            return;
        }
        let volume = (self.music_volume as f32 / 100.0 * layer_gain(i)).clamp(0.0, 1.0);
        if let Some(snd) = &self.music_layers[i] {
            play_sound(
                snd,
                PlaySoundParams {
                    looped: true,
                    volume,
                },
            );
            self.layer_playing[i] = true;
        }
    }

    fn stop_layers(&mut self) {
        for i in 0..LAYERS {
            if self.layer_playing[i] {
                if let Some(snd) = &self.music_layers[i] {
                    stop_sound(snd);
                }
                self.layer_playing[i] = false;
            }
        }
        self.music_playing = false;
    }

    fn restart_layers(&mut self) {
        self.stop_layers();
        if !self.music_on || self.music_volume == 0 {
            return;
        }
        let active = self.active_layers();
        for i in 0..active {
            self.start_layer(i);
        }
        self.music_playing = active > 0;
    }

    /// Cambia el piso musical: cada piso suma una capa instrumental. En el
    /// piso del guardián final (cada décimo) suenan todas a la vez.
    pub fn set_floor(&mut self, floor: u32) {
        let floor = floor.max(1);
        if floor == self.music_floor {
            return;
        }
        self.music_floor = floor;
        if self.music_playing {
            self.restart_layers();
        }
    }

    fn start_motif(&mut self, i: usize) {
        if i >= ENEMY_VOICE_COUNT {
            return;
        }
        if let Some(snd) = &self.motif_sounds[i] {
            play_sound(
                snd,
                PlaySoundParams {
                    looped: true,
                    volume: (self.enemy_volume as f32 / 100.0 * 0.42).clamp(0.0, 1.0),
                },
            );
            self.motif_playing = Some(i);
        }
    }

    /// Hace que la música adopte el motivo vocal del arquetipo que domina la
    /// sala. Con `None` vuelve al silencio vocal.
    pub fn set_motif(&mut self, voice: Option<EnemyVoice>) {
        let next = voice.map(|v| v as usize);
        if self.motif_playing == next {
            return;
        }
        if let Some(i) = self.motif_playing {
            if let Some(snd) = &self.motif_sounds[i] {
                stop_sound(snd);
            }
        }
        self.motif_playing = None;
        if let Some(i) = next {
            self.start_motif(i);
        }
    }

    pub fn play(&mut self, kind: SfxKind, volume: f32) {
        let i = kind as usize;
        // Sin sonido cargado no se toca el reloj de macroquad: así la lógica
        // puede probarse sin ventana ni dispositivo de audio.
        if self.sounds[i].is_none() {
            return;
        }
        let now = get_time() as f32;
        if now - self.last[i] < 0.025 {
            return;
        }
        self.last[i] = now;
        let volume = volume * self.sfx_volume as f32 / 100.0;
        if let Some(snd) = &self.sounds[i] {
            play_sound(
                snd,
                PlaySoundParams {
                    looped: false,
                    volume: volume.clamp(0.0, 1.0),
                },
            );
        }
    }

    /// Reproduce la voz de un enemigo con su propio volumen.
    pub fn play_enemy(&mut self, voice: EnemyVoice, volume: f32) {
        let i = voice as usize;
        if self.enemy_sounds[i].is_none() {
            return;
        }
        let now = get_time() as f32;
        if now - self.last_enemy[i] < 0.045 {
            return;
        }
        self.last_enemy[i] = now;
        let volume = volume * self.enemy_volume as f32 / 100.0;
        if let Some(snd) = &self.enemy_sounds[i] {
            play_sound(
                snd,
                PlaySoundParams {
                    looped: false,
                    volume: volume.clamp(0.0, 1.0),
                },
            );
        }
    }

    pub fn start_music(&mut self) {
        if !self.music_on || self.music_playing || self.music_volume == 0 {
            return;
        }
        let active = self.active_layers();
        for i in 0..active {
            self.start_layer(i);
        }
        self.music_playing = active > 0;
    }

    pub fn set_music_enabled(&mut self, enabled: bool) {
        self.music_on = enabled;
        if enabled {
            self.start_music();
        } else {
            self.stop_layers();
        }
    }

    pub fn toggle_music(&mut self) -> bool {
        let next = !self.music_on;
        self.set_music_enabled(next);
        next
    }
}

#[cfg(test)]
impl Sfx {
    /// Audio inerte para pruebas: permite ejercitar la lógica sin abrir
    /// dispositivo de sonido ni depender de una ventana.
    pub fn silent() -> Self {
        Self {
            sounds: std::array::from_fn(|_| None),
            last: [0.0; COUNT],
            enemy_sounds: std::array::from_fn(|_| None),
            last_enemy: [0.0; ENEMY_VOICE_COUNT],
            music_layers: std::array::from_fn(|_| None),
            layer_playing: [false; LAYERS],
            music_floor: 1,
            music_on: false,
            music_playing: false,
            motif_sounds: std::array::from_fn(|_| None),
            motif_playing: None,
            music_volume: 30,
            sfx_volume: 80,
            enemy_volume: 70,
        }
    }
}
