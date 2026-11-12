use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub const ALL: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

    pub fn from_index(i: u8) -> Self {
        Self::ALL[(i as usize).min(Self::ALL.len() - 1)]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&d| d == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "Fácil",
            Difficulty::Medium => "Medio",
            Difficulty::Hard => "Difícil",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Difficulty::Easy => "La experiencia original. Para explorar y aprender.",
            Difficulty::Medium => "Más vida, pega y enemigos. Requiere pensar la build.",
            Difficulty::Hard => "Guardianes feroces, oleadas extra y élites frecuentes.",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Difficulty::Easy => Color::from_rgba(105, 210, 140, 255),
            Difficulty::Medium => Color::from_rgba(245, 190, 85, 255),
            Difficulty::Hard => Color::from_rgba(240, 90, 95, 255),
        }
    }

    /// Multiplicadores de enemigos normales. Fácil conserva el balance actual.
    pub fn enemy_hp(self) -> f32 {
        match self {
            Difficulty::Easy => 1.00,
            Difficulty::Medium => 1.22,
            Difficulty::Hard => 1.50,
        }
    }

    pub fn enemy_damage(self) -> f32 {
        match self {
            Difficulty::Easy => 1.00,
            Difficulty::Medium => 1.15,
            Difficulty::Hard => 1.35,
        }
    }

    pub fn enemy_speed(self) -> f32 {
        match self {
            Difficulty::Easy => 1.00,
            Difficulty::Medium => 1.04,
            Difficulty::Hard => 1.10,
        }
    }

    pub fn enemy_cooldown(self) -> f32 {
        match self {
            Difficulty::Easy => 1.00,
            Difficulty::Medium => 0.94,
            Difficulty::Hard => 0.86,
        }
    }

    pub fn boss_hp(self) -> f32 {
        match self {
            Difficulty::Easy => 1.00,
            Difficulty::Medium => 1.16,
            Difficulty::Hard => 1.38,
        }
    }

    pub fn boss_damage(self) -> f32 {
        match self {
            Difficulty::Easy => 1.00,
            Difficulty::Medium => 1.12,
            Difficulty::Hard => 1.28,
        }
    }

    pub fn extra_spawns(self) -> usize {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Medium => 1,
            Difficulty::Hard => 2,
        }
    }

    pub fn extra_elite_chance(self) -> f32 {
        match self {
            Difficulty::Easy => 0.0,
            Difficulty::Medium => 0.06,
            Difficulty::Hard => 0.13,
        }
    }

    pub fn extra_wave(self) -> u32 {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Medium => 0,
            Difficulty::Hard => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Knight,
    Archer,
    Mage,
    Assassin,
    Necro,
    Cleric,
    Merchant,
}

impl Class {
    pub const ALL: [Class; 7] = [
        Class::Knight,
        Class::Archer,
        Class::Mage,
        Class::Assassin,
        Class::Necro,
        Class::Cleric,
        Class::Merchant,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Class::Knight => "Caballero",
            Class::Archer => "Arquero",
            Class::Mage => "Mago",
            Class::Assassin => "Asesino",
            Class::Necro => "Nigromante",
            Class::Cleric => "Clérigo",
            Class::Merchant => "Mercader",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Class::Knight => "Bloquea y devuelve daño",
            Class::Archer => "Críticos y perforación",
            Class::Mage => "Daño en área y quema",
            Class::Assassin => "Rápido, letal, ejecuta",
            Class::Necro => "Invoca esqueletos",
            Class::Cleric => "Cura y protege",
            Class::Merchant => "Oro y suerte",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Class::Knight => Color::from_rgba(170, 185, 205, 255),
            Class::Archer => Color::from_rgba(105, 200, 120, 255),
            Class::Mage => Color::from_rgba(105, 145, 235, 255),
            Class::Assassin => Color::from_rgba(185, 110, 230, 255),
            Class::Necro => Color::from_rgba(120, 220, 160, 255),
            Class::Cleric => Color::from_rgba(240, 214, 140, 255),
            Class::Merchant => Color::from_rgba(245, 180, 90, 255),
        }
    }

    /// Índice estable dentro de `ALL`, usado por el guardado y el catálogo.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&c| c == self).unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "Común",
            Rarity::Uncommon => "Poco común",
            Rarity::Rare => "Raro",
            Rarity::Epic => "Épico",
            Rarity::Legendary => "Legendario",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Rarity::Common => Color::from_rgba(175, 175, 185, 255),
            Rarity::Uncommon => Color::from_rgba(95, 205, 120, 255),
            Rarity::Rare => Color::from_rgba(90, 155, 245, 255),
            Rarity::Epic => Color::from_rgba(195, 115, 245, 255),
            Rarity::Legendary => Color::from_rgba(245, 195, 85, 255),
        }
    }

    pub fn weight(self) -> u32 {
        match self {
            Rarity::Common => 50,
            Rarity::Uncommon => 30,
            Rarity::Rare => 14,
            Rarity::Epic => 5,
            Rarity::Legendary => 1,
        }
    }
}

/// Xorshift64*: rapido, deterministico y sin dependencias.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 32) as u32
    }

    pub fn f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.f32() * (max - min)
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    pub fn usize(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u32() as usize) % n
        }
    }

    pub fn i32(&mut self, n: i32) -> i32 {
        if n <= 0 {
            0
        } else {
            (self.next_u32() % n as u32) as i32
        }
    }
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng::new(1234);
        let mut b = Rng::new(1234);
        for _ in 0..1000 {
            let x = a.f32();
            assert_eq!(x, b.f32());
            assert!((0.0..=1.0).contains(&x));
        }
        let mut c = Rng::new(9);
        for _ in 0..1000 {
            let v = c.range(-5.0, 5.0);
            assert!((-5.0..=5.0).contains(&v));
            assert!(c.usize(7) < 7);
        }
    }
}
