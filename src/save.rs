use std::fs;
use std::path::PathBuf;

use crate::types::Class;

/// Catálogo de colores de personaje. El índice 0 es el color propio de la clase.
/// Los precios son deliberadamente altos: la tienda de almas es un objetivo a
/// largo plazo, no una compra impulsiva.
pub const SKINS: [(&str, u32); 8] = [
    ("Clásico", 0),
    ("Carmesí", 120),
    ("Esmeralda", 150),
    ("Zafiro", 180),
    ("Amatista", 240),
    ("Oro viejo", 360),
    ("Escarcha", 300),
    ("Sombra", 520),
];

/// Gorras y cascos. El índice 0 significa "sin gorra".
pub const HATS: [(&str, u32); 8] = [
    ("Sin gorra", 0),
    ("Gorra", 110),
    ("Capucha", 140),
    ("Casco", 190),
    ("Corona", 420),
    ("Cuernos", 280),
    ("Halo", 340),
    ("Sombrero de mago", 240),
];

/// Máscaras y complementos faciales. El índice 0 significa "sin máscara".
pub const MASKS: [(&str, u32); 8] = [
    ("Sin máscara", 0),
    ("Antifaz", 100),
    ("Bandana", 130),
    ("Calavera", 240),
    ("Visor", 180),
    ("Cicatriz", 90),
    ("Pintura de guerra", 210),
    ("Velo", 280),
];

/// Contratos de aliado que se equipan en la taquilla. El índice 0 es "ninguno".
pub const TICKETS: [(&str, &str); 5] = [
    ("Sin contrato", "Ningún aliado te acompaña."),
    (
        "Escudero espectral",
        "Un escudero te sigue y golpea al enemigo más cercano.",
    ),
    (
        "Farolero",
        "Un farol flotante te cura una pequeña cantidad cada pocos segundos.",
    ),
    (
        "Cuervo de la suerte",
        "Un cuervo atrae oro y aumenta tu suerte.",
    ),
    (
        "Verdugo",
        "Un verdugo dispara saetas espectrales al enemigo más cercano.",
    ),
];

pub const KEY_ACTION_COUNT: usize = 8;
pub const KEY_ACTION_NAMES: [&str; KEY_ACTION_COUNT] = [
    "Arriba",
    "Abajo",
    "Izquierda",
    "Derecha",
    "Esquiva",
    "Mapa",
    "Pausa",
    "Interactuar",
];

/// Códigos de tecla portables. Se guardan como el valor numérico de
/// `macroquad::prelude::KeyCode`, que es estable en todas las plataformas.
#[derive(Clone, Copy)]
pub struct KeyBinds {
    pub codes: [u16; KEY_ACTION_COUNT],
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            codes: [0x57, 0x53, 0x41, 0x44, 0x20, 0xff09, 0xff1b, 0x45],
        }
    }
}

impl KeyBinds {
    pub fn set(&mut self, action: usize, code: u16) {
        if action < KEY_ACTION_COUNT {
            self.codes[action] = code;
        }
    }

    /// Devuelve el índice de la acción que ya usa esa tecla, si la hay.
    pub fn duplicate(&self, action: usize, code: u16) -> Option<usize> {
        self.codes
            .iter()
            .enumerate()
            .position(|(i, &c)| i != action && c == code)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone)]
pub struct Meta {
    pub souls: u32,
    pub best_floor: u32,
    pub runs: u32,
    pub hp_lv: u32,
    pub dmg_lv: u32,
    pub luck_lv: u32,
    pub total_kills: u32,
    pub name: String,
    pub skin: u8,
    pub hat: u8,
    pub mask: u8,
    pub owned_skins: u32,
    pub owned_hats: u32,
    pub owned_masks: u32,
    pub music: bool,
    pub music_vol: u8,
    pub sfx_vol: u8,
    pub enemy_vol: u8,
    /// Transición animada al bajar de piso. Se puede desactivar.
    pub floor_transitions: bool,
    /// Deuda pendiente con el mercader. Se conserva entre partidas.
    pub debt: u32,
    /// Ecos de partidas muertas y de qué clase eran.
    pub echoes: u32,
    pub echo_class: u8,
    pub echo_floor: u32,
    /// Teclas reasignables de la partida.
    pub binds: KeyBinds,
    /// Contrato/aliado equipado en la taquilla (0 = ninguno).
    pub ticket: u8,
    /// Contratos desbloqueados, bit a bit.
    pub owned_tickets: u32,
    /// Armas forjadas, bit a bit: 4 bits por clase (0 = sin arma).
    pub owned_weapons: u32,
    /// Arma equipada por clase: 4 bits por clase.
    pub equipped_weapons: u32,
    /// Dificultad elegida: 0 = Fácil, 1 = Medio, 2 = Difícil.
    pub difficulty: u8,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            souls: 0,
            best_floor: 0,
            runs: 0,
            hp_lv: 0,
            dmg_lv: 0,
            luck_lv: 0,
            total_kills: 0,
            name: "Aventurero".to_owned(),
            skin: 0,
            hat: 0,
            mask: 0,
            owned_skins: 1,
            owned_hats: 1,
            owned_masks: 1,
            music: true,
            music_vol: 30,
            sfx_vol: 80,
            enemy_vol: 70,
            floor_transitions: true,
            debt: 0,
            echoes: 0,
            echo_class: 0,
            echo_floor: 0,
            binds: KeyBinds::default(),
            ticket: 1,
            owned_tickets: 0b11111,
            owned_weapons: 0,
            equipped_weapons: 0,
            difficulty: 0,
        }
    }
}

impl Meta {
    pub fn owns_skin(&self, i: usize) -> bool {
        i < 32 && (self.owned_skins & (1 << i)) != 0
    }

    pub fn owns_hat(&self, i: usize) -> bool {
        i < 32 && (self.owned_hats & (1 << i)) != 0
    }

    pub fn owns_mask(&self, i: usize) -> bool {
        i < 32 && (self.owned_masks & (1 << i)) != 0
    }

    /// Índice del arma equipada para una clase (0 = sin arma).
    pub fn weapon_for(&self, class: Class) -> usize {
        ((self.equipped_weapons >> (class.index() * 4)) & 0xF) as usize
    }

    pub fn set_weapon(&mut self, class: Class, index: usize) {
        let shift = class.index() * 4;
        self.equipped_weapons =
            (self.equipped_weapons & !(0xF << shift)) | ((index as u32 & 0xF) << shift);
    }

    pub fn owns_weapon(&self, class: Class, index: usize) -> bool {
        index == 0 || (index <= 3 && (self.owned_weapons & (1 << (class.index() * 4 + index))) != 0)
    }

    pub fn unlock_weapon(&mut self, class: Class, index: usize) {
        if (1..=3).contains(&index) {
            self.owned_weapons |= 1 << (class.index() * 4 + index);
        }
    }
}

pub fn save_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("cripta_save.txt");
        }
    }
    PathBuf::from("cripta_save.txt")
}

pub fn load() -> Meta {
    let path = save_path();
    match fs::read_to_string(&path) {
        Ok(text) => decode(&text),
        Err(_) => Meta::default(),
    }
}

/// Interpreta el contenido de `cripta_save.txt`. Separado de `load` para poder
/// probar la ida y vuelta completa sin tocar el disco.
pub fn decode(text: &str) -> Meta {
    let mut m = Meta::default();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let raw = v.trim();
            let val = raw.parse::<u32>().unwrap_or(0);
            match key {
                "souls" => m.souls = val,
                "best_floor" => m.best_floor = val,
                "runs" => m.runs = val,
                "hp_lv" => m.hp_lv = val,
                "dmg_lv" => m.dmg_lv = val,
                "luck_lv" => m.luck_lv = val,
                "total_kills" => m.total_kills = val,
                "name" if !raw.is_empty() => {
                    m.name = raw.chars().take(14).collect::<String>();
                }
                "skin" => m.skin = (val as u8).min(SKINS.len() as u8 - 1),
                "hat" => m.hat = (val as u8).min(HATS.len() as u8 - 1),
                "mask" => m.mask = (val as u8).min(MASKS.len() as u8 - 1),
                "owned_skins" => m.owned_skins = val | 1,
                "owned_hats" => m.owned_hats = val | 1,
                "owned_masks" => m.owned_masks = val | 1,
                "music" => m.music = val != 0,
                "music_vol" => m.music_vol = val.min(100) as u8,
                "sfx_vol" => m.sfx_vol = val.min(100) as u8,
                "enemy_vol" => m.enemy_vol = val.min(100) as u8,
                "floor_transitions" => m.floor_transitions = val != 0,
                "debt" => m.debt = val,
                "echoes" => m.echoes = val.min(999),
                "echo_class" => m.echo_class = (val as u8).min(Class::ALL.len() as u8 - 1),
                "echo_floor" => m.echo_floor = val,
                "key0" => m.binds.codes[0] = val as u16,
                "key1" => m.binds.codes[1] = val as u16,
                "key2" => m.binds.codes[2] = val as u16,
                "key3" => m.binds.codes[3] = val as u16,
                "key4" => m.binds.codes[4] = val as u16,
                "key5" => m.binds.codes[5] = val as u16,
                "key6" => m.binds.codes[6] = val as u16,
                "key7" => m.binds.codes[7] = val as u16,
                "ticket" => m.ticket = (val as u8).min(TICKETS.len() as u8 - 1),
                "owned_tickets" => m.owned_tickets = val | 1,
                "owned_weapons" => m.owned_weapons = val,
                "equipped_weapons" => m.equipped_weapons = val,
                "difficulty" => m.difficulty = (val as u8).min(2),
                _ => {}
            }
        }
    }
    m
}

/// Serializa el progreso con el mismo formato que lee `decode`.
pub fn encode(m: &Meta) -> String {
    let text = format!(
        "souls={}\nbest_floor={}\nruns={}\nhp_lv={}\ndmg_lv={}\nluck_lv={}\ntotal_kills={}\nname={}\nskin={}\nhat={}\nmask={}\nowned_skins={}\nowned_hats={}\nowned_masks={}\nmusic={}\nmusic_vol={}\nsfx_vol={}\nenemy_vol={}\nfloor_transitions={}\ndebt={}\nechoes={}\necho_class={}\necho_floor={}\n",
        m.souls,
        m.best_floor,
        m.runs,
        m.hp_lv,
        m.dmg_lv,
        m.luck_lv,
        m.total_kills,
        m.name.replace('\n', " "),
        m.skin,
        m.hat,
        m.mask,
        m.owned_skins,
        m.owned_hats,
        m.owned_masks,
        if m.music { 1 } else { 0 },
        m.music_vol,
        m.sfx_vol,
        m.enemy_vol,
        if m.floor_transitions { 1 } else { 0 },
        m.debt,
        m.echoes,
        m.echo_class,
        m.echo_floor
    );
    let mut text = text;
    text.push_str(&format!(
        "ticket={}\nowned_tickets={}\nowned_weapons={}\nequipped_weapons={}\ndifficulty={}\n",
        m.ticket, m.owned_tickets, m.owned_weapons, m.equipped_weapons, m.difficulty
    ));
    for (i, code) in m.binds.codes.iter().enumerate() {
        text.push_str(&format!("key{}={}\n", i, code));
    }
    text
}

pub fn save(m: &Meta) {
    // El modo de grabación de la guía nunca debe tocar la partida real.
    if std::env::var_os("CRIPTA_DEMO").is_some() {
        return;
    }
    let text = encode(m);
    let path = save_path();
    if fs::write(&path, text).is_err() {
        let _ = fs::write("cripta_save.txt", "souls=0\n");
    }
}

pub fn upgrade_cost(kind: usize, level: u32) -> u32 {
    match kind {
        0 => 120 + level * 90,
        1 => 180 + level * 130,
        _ => 150 + level * 110,
    }
}

/// Cada mejora permanente exige haber llegado a un piso concreto. Así la
/// tienda de almas se convierte en una colección de desafíos y no en una
/// simple lista de precios.
pub fn upgrade_requirement(kind: usize, level: u32) -> u32 {
    match kind {
        0 => 2 + level * 2,
        1 => 3 + level * 2,
        _ => 2 + level * 2,
    }
}

/// Requisito de un cosmético: (mejor piso, bajas totales).
pub fn cosmetic_requirement(tab: usize, idx: usize) -> Option<(u32, u32)> {
    let table: [(u32, u32); 8] = match tab {
        0 => [
            (0, 0),
            (2, 0),
            (3, 40),
            (4, 60),
            (5, 100),
            (8, 240),
            (6, 160),
            (10, 350),
        ],
        1 => [
            (0, 0),
            (2, 0),
            (3, 25),
            (4, 50),
            (9, 280),
            (6, 120),
            (7, 180),
            (5, 90),
        ],
        _ => [
            (0, 0),
            (1, 0),
            (2, 20),
            (5, 140),
            (4, 70),
            (1, 0),
            (3, 50),
            (6, 200),
        ],
    };
    let (floor, kills) = table[idx.min(table.len() - 1)];
    if floor == 0 && kills == 0 {
        None
    } else {
        Some((floor, kills))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_costs_grow() {
        for kind in 0..3 {
            let mut prev = 0;
            for level in 0..10 {
                let cost = upgrade_cost(kind, level);
                assert!(cost > prev);
                prev = cost;
            }
        }
    }

    #[test]
    fn default_meta_owns_the_free_cosmetics() {
        let m = Meta::default();
        assert!(m.owns_skin(0));
        assert!(m.owns_hat(0));
        assert!(m.owns_mask(0));
        assert!(!m.owns_hat(4));
    }

    #[test]
    fn default_binds_are_unique_and_duplicates_are_rejected() {
        let binds = KeyBinds::default();
        for i in 0..KEY_ACTION_COUNT {
            assert!(
                binds.duplicate(i, binds.codes[i]).is_none(),
                "las teclas por defecto no deben repetirse"
            );
        }
        assert_eq!(binds.duplicate(0, binds.codes[1]), Some(1));
    }

    #[test]
    fn shop_requirements_grow_with_the_catalog() {
        assert!(cosmetic_requirement(0, 0).is_none());
        let (floor_a, kills_a) = cosmetic_requirement(0, 3).unwrap();
        let (floor_b, kills_b) = cosmetic_requirement(0, 7).unwrap();
        assert!(floor_b > floor_a && kills_b > kills_a);
        assert!(upgrade_requirement(1, 4) > upgrade_requirement(1, 1));
    }

    #[test]
    fn save_round_trip_keeps_contracts_weapons_and_transitions() {
        let mut m = Meta::default();
        m.name = "Lyra".to_owned();
        m.souls = 777;
        m.ticket = 3;
        m.owned_tickets = 0b10101;
        m.unlock_weapon(Class::Cleric, 2);
        m.unlock_weapon(Class::Merchant, 3);
        m.set_weapon(Class::Cleric, 2);
        m.set_weapon(Class::Merchant, 3);
        m.floor_transitions = false;
        m.music_vol = 12;
        m.binds.codes[4] = 0x1234;
        m.difficulty = 2;

        let back = decode(&encode(&m));
        assert_eq!(back.name, "Lyra");
        assert_eq!(back.souls, 777);
        assert_eq!(back.ticket, 3);
        assert_eq!(back.owned_tickets, 0b10101);
        assert_eq!(back.weapon_for(Class::Cleric), 2);
        assert_eq!(back.weapon_for(Class::Merchant), 3);
        assert!(back.owns_weapon(Class::Merchant, 3));
        assert!(!back.owns_weapon(Class::Merchant, 1));
        assert!(!back.floor_transitions);
        assert_eq!(back.music_vol, 12);
        assert_eq!(back.binds.codes[4], 0x1234);
        assert_eq!(back.difficulty, 2);
    }
}
