use crate::types::{Class, Rarity, Rng};

pub const BUFF_COUNT: usize = 33;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum BuffId {
    Fury = 0,
    Vigor,
    Swift,
    Haste,
    Crit,
    CritDmg,
    Armor,
    Regen,
    Lifesteal,
    Thorns,
    DashCd,
    Luck,
    Wisdom,
    Greed,
    Legion,
    BoneDmg,
    BoneExplode,
    BoneInherit,
    Block,
    Reflect,
    Shockwave,
    Multishot,
    Pierce,
    Aim,
    Pyro,
    Chain,
    Execute,
    Bloodlust,
    Lethal,
    Crown,
    Faith,
    Sanctuary,
    Commerce,
}

#[derive(Clone, Copy)]
pub struct BuffDef {
    pub id: BuffId,
    pub name: &'static str,
    pub desc: &'static str,
    pub rarity: Rarity,
    pub class: Option<Class>,
    pub max_stacks: u32,
}

const fn b(
    id: BuffId,
    name: &'static str,
    desc: &'static str,
    rarity: Rarity,
    class: Option<Class>,
    max_stacks: u32,
) -> BuffDef {
    BuffDef {
        id,
        name,
        desc,
        rarity,
        class,
        max_stacks,
    }
}

pub const POOL: [BuffDef; BUFF_COUNT] = [
    b(
        BuffId::Fury,
        "Furia",
        "Tu daño aumenta un 20%.",
        Rarity::Common,
        None,
        99,
    ),
    b(
        BuffId::Vigor,
        "Vigor",
        "Tu vida máxima sube un 20% y te cura esa cantidad.",
        Rarity::Common,
        None,
        99,
    ),
    b(
        BuffId::Swift,
        "Botas ligeras",
        "Te mueves un 12% más rápido.",
        Rarity::Common,
        None,
        5,
    ),
    b(
        BuffId::Haste,
        "Manos rápidas",
        "Atacas un 12% más rápido.",
        Rarity::Uncommon,
        None,
        4,
    ),
    b(
        BuffId::Crit,
        "Ojo crítico",
        "Ganas un 8% de probabilidad de golpe crítico.",
        Rarity::Uncommon,
        None,
        5,
    ),
    b(
        BuffId::CritDmg,
        "Golpe brutal",
        "Tus golpes críticos hacen un 40% más de daño.",
        Rarity::Rare,
        None,
        3,
    ),
    b(
        BuffId::Armor,
        "Piel de hierro",
        "Ganas 2 puntos de armadura.",
        Rarity::Uncommon,
        None,
        5,
    ),
    b(
        BuffId::Regen,
        "Regeneración",
        "Recuperas 0,8 de vida por segundo.",
        Rarity::Rare,
        None,
        4,
    ),
    b(
        BuffId::Lifesteal,
        "Vampirismo",
        "Recuperas un 4% del daño que haces.",
        Rarity::Rare,
        None,
        4,
    ),
    b(
        BuffId::Thorns,
        "Espinas",
        "Devuelves un 25% del daño que recibes.",
        Rarity::Uncommon,
        None,
        4,
    ),
    b(
        BuffId::DashCd,
        "Esquiva veloz",
        "La esquiva se recarga un 20% más rápido.",
        Rarity::Uncommon,
        None,
        3,
    ),
    b(
        BuffId::Luck,
        "Suerte",
        "Aumenta un 15% la probabilidad de encontrar objetos raros.",
        Rarity::Uncommon,
        None,
        3,
    ),
    b(
        BuffId::Wisdom,
        "Sabiduría",
        "Ganas un 15% más de experiencia.",
        Rarity::Common,
        None,
        3,
    ),
    b(
        BuffId::Greed,
        "Codicia",
        "Recoges un 25% más de oro.",
        Rarity::Common,
        None,
        3,
    ),
    b(
        BuffId::Legion,
        "Legión",
        "Puedes tener 2 esqueletos más.",
        Rarity::Rare,
        Some(Class::Necro),
        2,
    ),
    b(
        BuffId::BoneDmg,
        "Huesos afilados",
        "Tus esqueletos hacen un 35% más de daño.",
        Rarity::Uncommon,
        Some(Class::Necro),
        4,
    ),
    b(
        BuffId::BoneExplode,
        "Estallido óseo",
        "Tus esqueletos explotan al desaparecer.",
        Rarity::Epic,
        Some(Class::Necro),
        1,
    ),
    b(
        BuffId::BoneInherit,
        "Herencia",
        "Tus esqueletos heredan un 25% de tu daño.",
        Rarity::Epic,
        Some(Class::Necro),
        2,
    ),
    b(
        BuffId::Block,
        "Muro de acero",
        "Ganas un 12% de probabilidad de bloquear.",
        Rarity::Rare,
        Some(Class::Knight),
        3,
    ),
    b(
        BuffId::Reflect,
        "Represalia",
        "Devuelves el 50% del daño bloqueado.",
        Rarity::Epic,
        Some(Class::Knight),
        2,
    ),
    b(
        BuffId::Shockwave,
        "Golpe sísmico",
        "Cada 5 ataques provocas una onda de choque.",
        Rarity::Rare,
        Some(Class::Knight),
        2,
    ),
    b(
        BuffId::Multishot,
        "Multidisparo",
        "Disparas un proyectil adicional.",
        Rarity::Epic,
        Some(Class::Archer),
        2,
    ),
    b(
        BuffId::Pierce,
        "Perforante",
        "Tus proyectiles atraviesan un enemigo más.",
        Rarity::Rare,
        Some(Class::Archer),
        3,
    ),
    b(
        BuffId::Aim,
        "Puntería",
        "Tu alcance sube un 15% y tu crítico un 10%.",
        Rarity::Uncommon,
        Some(Class::Archer),
        3,
    ),
    b(
        BuffId::Pyro,
        "Piromancia",
        "Tu área de explosión sube un 35% y tus ataques queman.",
        Rarity::Rare,
        Some(Class::Mage),
        3,
    ),
    b(
        BuffId::Chain,
        "Cadena",
        "Tus proyectiles rebotan una vez más.",
        Rarity::Epic,
        Some(Class::Mage),
        2,
    ),
    b(
        BuffId::Execute,
        "Ejecutor",
        "Haces un 30% más de daño a enemigos heridos.",
        Rarity::Rare,
        Some(Class::Assassin),
        3,
    ),
    b(
        BuffId::Bloodlust,
        "Sed de sangre",
        "Al matar, te mueves un 30% más rápido durante 2 segundos.",
        Rarity::Rare,
        Some(Class::Assassin),
        2,
    ),
    b(
        BuffId::Lethal,
        "Crítico letal",
        "Ganas un 12% de crítico y un 50% de daño crítico.",
        Rarity::Epic,
        Some(Class::Assassin),
        2,
    ),
    b(
        BuffId::Crown,
        "Corona del abismo",
        "Todas tus estadísticas suben un 15%.",
        Rarity::Legendary,
        None,
        1,
    ),
    b(
        BuffId::Faith,
        "Fervor",
        "Tu regeneración sube 0,7 y tu vida máxima un 8%.",
        Rarity::Rare,
        Some(Class::Cleric),
        3,
    ),
    b(
        BuffId::Sanctuary,
        "Santuario",
        "Ganas un 10% de bloqueo y 1 punto de armadura.",
        Rarity::Uncommon,
        Some(Class::Cleric),
        3,
    ),
    b(
        BuffId::Commerce,
        "Comercio",
        "Recoges un 25% más de oro y ganas un 10% de suerte.",
        Rarity::Common,
        Some(Class::Merchant),
        4,
    ),
];

fn weighted_pick(
    rng: &mut Rng,
    class: Class,
    stacks: &[u32; BUFF_COUNT],
    chosen: &[usize],
    class_only: bool,
) -> Option<usize> {
    let mut total = 0.0f32;
    for (i, def) in POOL.iter().enumerate() {
        if stacks[i] >= def.max_stacks || chosen.contains(&i) {
            continue;
        }
        if let Some(c) = def.class {
            if c != class {
                continue;
            }
        }
        if class_only && def.class != Some(class) {
            continue;
        }
        let mut w = def.rarity.weight() as f32;
        if def.class == Some(class) {
            w *= 3.0;
        }
        total += w;
    }
    if total <= 0.0 {
        return None;
    }
    let mut roll = rng.f32() * total;
    for (i, def) in POOL.iter().enumerate() {
        if stacks[i] >= def.max_stacks || chosen.contains(&i) {
            continue;
        }
        if let Some(c) = def.class {
            if c != class {
                continue;
            }
        }
        if class_only && def.class != Some(class) {
            continue;
        }
        let mut w = def.rarity.weight() as f32;
        if def.class == Some(class) {
            w *= 3.0;
        }
        roll -= w;
        if roll <= 0.0 {
            return Some(i);
        }
    }
    None
}

pub fn pick_cards(
    rng: &mut Rng,
    class: Class,
    stacks: &[u32; BUFF_COUNT],
    level: u32,
) -> [BuffDef; 3] {
    let mut out = [POOL[0]; 3];
    let mut chosen: [usize; 3] = [usize::MAX; 3];
    let mut n = 0usize;

    if level % 4 == 0 {
        if let Some(i) = weighted_pick(rng, class, stacks, &chosen[..n], true) {
            chosen[n] = i;
            out[n] = POOL[i];
            n += 1;
        }
    }
    while n < 3 {
        match weighted_pick(rng, class, stacks, &chosen[..n], false) {
            Some(i) => {
                chosen[n] = i;
                out[n] = POOL[i];
                n += 1;
            }
            None => break,
        }
    }
    while n < 3 {
        out[n] = POOL[0];
        n += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_are_unique_and_class_valid() {
        let mut rng = Rng::new(0xDEAD_BEEF);
        for class in Class::ALL {
            for level in 1..40 {
                let stacks = [0u32; BUFF_COUNT];
                let cards = pick_cards(&mut rng, class, &stacks, level);
                let mut ids = [usize::MAX; 3];
                for (i, c) in cards.iter().enumerate() {
                    if let Some(only) = c.class {
                        assert_eq!(only, class, "bufo de otra clase ofrecido");
                    }
                    let id = c.id as usize;
                    assert!(!ids[..i].contains(&id), "bufo duplicado");
                    ids[i] = id;
                }
            }
        }
    }

    #[test]
    fn pity_offers_class_buff_on_level_multiple_of_four() {
        let mut rng = Rng::new(7);
        for class in Class::ALL {
            let stacks = [0u32; BUFF_COUNT];
            let cards = pick_cards(&mut rng, class, &stacks, 4);
            assert!(cards.iter().any(|c| c.class == Some(class)));
        }
    }

    #[test]
    fn respects_max_stacks() {
        let mut rng = Rng::new(42);
        let mut stacks = [0u32; BUFF_COUNT];
        for (i, def) in POOL.iter().enumerate() {
            stacks[i] = def.max_stacks;
        }
        let cards = pick_cards(&mut rng, Class::Knight, &stacks, 5);
        for c in cards {
            assert_eq!(c.id, BuffId::Fury);
        }
    }
}
