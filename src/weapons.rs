use crate::types::Class;

/// Efecto pasivo que aplica un arma al empezar la partida. Los porcentajes se
/// guardan como fracciones (0.12 = +12%).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeaponEffect {
    Damage(f32),
    MaxHp(f32),
    Speed(f32),
    AttackSpeed(f32),
    Crit(f32),
    CritMult(f32),
    Armor(f32),
    Gold(f32),
    Luck(f32),
    Aoe(f32),
    Regen(f32),
    Block(f32),
    Execute(f32),
    SkeletonCap(u32),
    SkeletonDmg(f32),
    SkeletonInherit(f32),
    ExtraProjectiles(u32),
    Pierce(u32),
    Burn(f32),
    Chain(u32),
    Shockwave(u32),
}

#[derive(Clone, Copy)]
pub struct WeaponDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub cost: u32,
    pub req_floor: u32,
    pub effects: &'static [WeaponEffect],
}

/// Tres armas por personaje. El índice 0 del equipamiento siempre significa
/// "sin arma": las armas forjadas ocupan los índices 1, 2 y 3.
pub const CLASS_WEAPONS: [[WeaponDef; 3]; 7] = [
    // Caballero
    [
        WeaponDef {
            name: "Espada del juramento",
            desc: "Una hoja pesada que castiga a quien cruce la puerta.",
            cost: 180,
            req_floor: 3,
            effects: &[WeaponEffect::Damage(0.12), WeaponEffect::Armor(1.0)],
        },
        WeaponDef {
            name: "Escudo de la puerta",
            desc: "El muro que nunca cedió. Aguanta por ti.",
            cost: 320,
            req_floor: 6,
            effects: &[WeaponEffect::MaxHp(0.20), WeaponEffect::Block(0.08)],
        },
        WeaponDef {
            name: "Maza sísmica",
            desc: "Cada golpe resuena como un terremoto bajo la cripta.",
            cost: 520,
            req_floor: 10,
            effects: &[WeaponEffect::Shockwave(1), WeaponEffect::Damage(0.10)],
        },
    ],
    // Arquero
    [
        WeaponDef {
            name: "Arco de niebla",
            desc: "La bruma guía la flecha hacia el punto débil.",
            cost: 180,
            req_floor: 3,
            effects: &[WeaponEffect::Crit(0.08), WeaponEffect::Damage(0.10)],
        },
        WeaponDef {
            name: "Ballesta perforante",
            desc: "Un virote que atraviesa la primera línea sin frenar.",
            cost: 320,
            req_floor: 6,
            effects: &[WeaponEffect::Pierce(1), WeaponEffect::Damage(0.15)],
        },
        WeaponDef {
            name: "Carcaj infinito",
            desc: "Siempre queda una flecha más. Siempre.",
            cost: 520,
            req_floor: 10,
            effects: &[
                WeaponEffect::ExtraProjectiles(1),
                WeaponEffect::AttackSpeed(0.05),
            ],
        },
    ],
    // Mago
    [
        WeaponDef {
            name: "Cetro de runas",
            desc: "Las runas del cetro amplían cada explosión.",
            cost: 180,
            req_floor: 3,
            effects: &[WeaponEffect::Aoe(0.25), WeaponEffect::Damage(0.10)],
        },
        WeaponDef {
            name: "Orbe ígneo",
            desc: "Fuego viejo que no se apaga con sangre.",
            cost: 320,
            req_floor: 6,
            effects: &[WeaponEffect::Burn(3.0), WeaponEffect::Damage(0.15)],
        },
        WeaponDef {
            name: "Grimorio de cadenas",
            desc: "Cada conjuro arrastra a otro enemigo al mismo destino.",
            cost: 520,
            req_floor: 10,
            effects: &[WeaponEffect::Chain(1), WeaponEffect::ExtraProjectiles(1)],
        },
    ],
    // Asesino
    [
        WeaponDef {
            name: "Dagas gemelas",
            desc: "Dos filos, un solo movimiento. La sombra ni lo nota.",
            cost: 180,
            req_floor: 3,
            effects: &[WeaponEffect::AttackSpeed(0.18), WeaponEffect::Crit(0.08)],
        },
        WeaponDef {
            name: "Filo del verdugo",
            desc: "Remata a los heridos antes de que pidan clemencia.",
            cost: 320,
            req_floor: 6,
            effects: &[WeaponEffect::Execute(0.25), WeaponEffect::Damage(0.10)],
        },
        WeaponDef {
            name: "Manto de sombra",
            desc: "Te mueves entre las sombras y golpeas más fuerte.",
            cost: 520,
            req_floor: 10,
            effects: &[WeaponEffect::Speed(0.10), WeaponEffect::CritMult(0.25)],
        },
    ],
    // Nigromante
    [
        WeaponDef {
            name: "Cetro de huesos",
            desc: "Los muertos obedecen al que sostiene el cetro.",
            cost: 180,
            req_floor: 3,
            effects: &[
                WeaponEffect::SkeletonCap(1),
                WeaponEffect::SkeletonDmg(0.15),
            ],
        },
        WeaponDef {
            name: "Corona funeraria",
            desc: "Una corona de vértebras que enfurece a la legión.",
            cost: 320,
            req_floor: 6,
            effects: &[
                WeaponEffect::SkeletonDmg(0.25),
                WeaponEffect::SkeletonCap(1),
            ],
        },
        WeaponDef {
            name: "Libro de almas",
            desc: "Cada alma atrapada refuerza a tus esqueletos.",
            cost: 520,
            req_floor: 10,
            effects: &[
                WeaponEffect::SkeletonInherit(0.25),
                WeaponEffect::Damage(0.10),
            ],
        },
    ],
    // Clérigo
    [
        WeaponDef {
            name: "Báculo del alba",
            desc: "La luz del amanecer cierra las heridas más rápido.",
            cost: 180,
            req_floor: 3,
            effects: &[WeaponEffect::Regen(0.8), WeaponEffect::MaxHp(0.10)],
        },
        WeaponDef {
            name: "Cáliz de fe",
            desc: "Un cáliz que rebosa vida y protección.",
            cost: 320,
            req_floor: 6,
            effects: &[WeaponEffect::MaxHp(0.15), WeaponEffect::Block(0.05)],
        },
        WeaponDef {
            name: "Campana del santuario",
            desc: "Su tañido ahuyenta el daño y llama a los fieles.",
            cost: 520,
            req_floor: 10,
            effects: &[
                WeaponEffect::Armor(2.0),
                WeaponEffect::Block(0.10),
                WeaponEffect::Regen(0.4),
            ],
        },
    ],
    // Mercader
    [
        WeaponDef {
            name: "Balanza de oro",
            desc: "Pesa cada moneda y siempre inclina la balanza a tu favor.",
            cost: 180,
            req_floor: 3,
            effects: &[WeaponEffect::Gold(0.35)],
        },
        WeaponDef {
            name: "Dado cargado",
            desc: "Nunca falla. Al menos, no cuando importa.",
            cost: 320,
            req_floor: 6,
            effects: &[WeaponEffect::Luck(0.20), WeaponEffect::Crit(0.10)],
        },
        WeaponDef {
            name: "Bolsa sin fondo",
            desc: "Cabe más oro del que debería. Mucho más.",
            cost: 520,
            req_floor: 10,
            effects: &[
                WeaponEffect::Gold(0.50),
                WeaponEffect::Luck(0.10),
                WeaponEffect::Speed(0.05),
            ],
        },
    ],
];

pub fn class_weapons(class: Class) -> &'static [WeaponDef; 3] {
    &CLASS_WEAPONS[class.index()]
}

/// Texto corto de un efecto para las fichas de la forja.
pub fn effect_label(effect: WeaponEffect) -> String {
    match effect {
        WeaponEffect::Damage(v) => format!("+{:.0}% daño", v * 100.0),
        WeaponEffect::MaxHp(v) => format!("+{:.0}% vida", v * 100.0),
        WeaponEffect::Speed(v) => format!("+{:.0}% velocidad", v * 100.0),
        WeaponEffect::AttackSpeed(v) => format!("+{:.0}% cadencia", v * 100.0),
        WeaponEffect::Crit(v) => format!("+{:.0}% crítico", v * 100.0),
        WeaponEffect::CritMult(v) => format!("+{:.0}% daño crítico", v * 100.0),
        WeaponEffect::Armor(v) => format!("+{:.0} armadura", v),
        WeaponEffect::Gold(v) => format!("+{:.0}% oro", v * 100.0),
        WeaponEffect::Luck(v) => format!("+{:.0}% suerte", v * 100.0),
        WeaponEffect::Aoe(v) => format!("+{:.0}% área", v * 100.0),
        WeaponEffect::Regen(v) => format!("+{:.1} regeneración", v),
        WeaponEffect::Block(v) => format!("+{:.0}% bloqueo", v * 100.0),
        WeaponEffect::Execute(v) => format!("+{:.0}% ejecución", v * 100.0),
        WeaponEffect::SkeletonCap(v) => format!("+{} esqueleto", v),
        WeaponEffect::SkeletonDmg(v) => format!("+{:.0}% daño de esqueletos", v * 100.0),
        WeaponEffect::SkeletonInherit(v) => format!("+{:.0}% herencia", v * 100.0),
        WeaponEffect::ExtraProjectiles(v) => format!("+{} proyectil", v),
        WeaponEffect::Pierce(v) => format!("+{} perforación", v),
        WeaponEffect::Burn(v) => format!("+{:.0} quema", v),
        WeaponEffect::Chain(v) => format!("+{} cadena", v),
        WeaponEffect::Shockwave(v) => format!("+{} onda sísmica", v),
    }
}

/// Devuelve el arma equipada. El índice 0 representa "sin arma".
pub fn get(class: Class, index: usize) -> Option<&'static WeaponDef> {
    if index == 0 {
        None
    } else {
        class_weapons(class).get(index - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_has_three_weapons_with_distinct_names() {
        let mut all = std::collections::HashSet::new();
        for class in Class::ALL {
            for w in class_weapons(class) {
                assert!(w.cost > 0);
                assert!(w.req_floor > 0);
                assert!(!w.effects.is_empty());
                assert!(all.insert(w.name), "arma duplicada: {}", w.name);
            }
        }
        assert_eq!(all.len(), Class::ALL.len() * 3);
    }

    #[test]
    fn weapon_zero_is_empty_and_one_is_the_first_weapon() {
        assert!(get(Class::Knight, 0).is_none());
        assert_eq!(get(Class::Knight, 1).unwrap().name, "Espada del juramento");
        assert!(get(Class::Merchant, 4).is_none());
    }
}
