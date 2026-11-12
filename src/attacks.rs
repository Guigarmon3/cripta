use crate::game::Element;
use crate::types::{Class, Rarity};
use macroquad::prelude::Color;

/// Número máximo de ataques que un personaje puede llevar equipados.
pub const MAX_EQUIPPED: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttackKind {
    Bolt,
    Orbit,
    Aura,
    Nova,
    Chain,
    Mine,
    Trail,
    Vortex,
    Bolas,
    Mend,
    Beam,
    Meteor,
    Swarm,
    Shockwave,
    Spikes,
    Drain,
}

impl AttackKind {
    /// Nombre corto del arquetipo, útil para la interfaz y las fichas.
    pub fn label(self) -> &'static str {
        match self {
            AttackKind::Bolt => "Proyectil",
            AttackKind::Orbit => "Órbita",
            AttackKind::Aura => "Aura",
            AttackKind::Nova => "Nova",
            AttackKind::Chain => "Cadena",
            AttackKind::Mine => "Mina",
            AttackKind::Trail => "Rastro",
            AttackKind::Vortex => "Vórtice",
            AttackKind::Bolas => "Bolas",
            AttackKind::Mend => "Curación",
            AttackKind::Beam => "Rayo",
            AttackKind::Meteor => "Meteoro",
            AttackKind::Swarm => "Enjambre",
            AttackKind::Shockwave => "Onda",
            AttackKind::Spikes => "Pinchos",
            AttackKind::Drain => "Drenaje",
        }
    }
}

#[derive(Clone)]
pub struct AttackDef {
    pub name: String,
    pub desc: String,
    pub class: Class,
    pub element: Element,
    pub kind: AttackKind,
    pub rarity: Rarity,
    pub max_level: u32,
    pub base_damage: f32,
    pub base_cd: f32,
    pub base_radius: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct EquippedAttack {
    /// Índice dentro del catálogo de ataques.
    pub def: usize,
    pub level: u32,
    pub cd: f32,
}

impl EquippedAttack {
    pub fn new(def: usize) -> Self {
        Self {
            def,
            level: 1,
            cd: 0.0,
        }
    }
}

/// Ocho arquetipos por personaje. Diez elementos por arquetipo dan ochenta
/// ataques por personaje y quinientos sesenta en total.
fn archetypes(class: Class) -> &'static [(AttackKind, &'static str, Rarity)] {
    match class {
        Class::Knight => &[
            (AttackKind::Orbit, "Filo orbital", Rarity::Common),
            (AttackKind::Shockwave, "Martillo sísmico", Rarity::Uncommon),
            (AttackKind::Aura, "Presencia de acero", Rarity::Rare),
            (AttackKind::Nova, "Estallido de guardia", Rarity::Rare),
            (AttackKind::Chain, "Cadena de juramento", Rarity::Uncommon),
            (AttackKind::Spikes, "Lanza de tierra", Rarity::Uncommon),
            (AttackKind::Trail, "Estela de brasas", Rarity::Common),
            (AttackKind::Bolas, "Bolas de hierro", Rarity::Rare),
        ],
        Class::Archer => &[
            (AttackKind::Bolt, "Disparo certero", Rarity::Common),
            (AttackKind::Swarm, "Enjambre de flechas", Rarity::Epic),
            (AttackKind::Chain, "Flecha rebotante", Rarity::Uncommon),
            (AttackKind::Mine, "Trampa de cazador", Rarity::Uncommon),
            (AttackKind::Trail, "Rastro de pólvora", Rarity::Common),
            (AttackKind::Beam, "Flecha perforante", Rarity::Epic),
            (AttackKind::Meteor, "Lluvia de flechas", Rarity::Rare),
            (AttackKind::Vortex, "Trampa de viento", Rarity::Rare),
        ],
        Class::Mage => &[
            (AttackKind::Nova, "Nova arcana", Rarity::Common),
            (AttackKind::Meteor, "Meteoro", Rarity::Epic),
            (AttackKind::Beam, "Rayo", Rarity::Rare),
            (AttackKind::Vortex, "Vórtice", Rarity::Rare),
            (AttackKind::Chain, "Cadena arcana", Rarity::Uncommon),
            (AttackKind::Aura, "Aura elemental", Rarity::Uncommon),
            (AttackKind::Swarm, "Misiles arcanos", Rarity::Epic),
            (AttackKind::Mend, "Sello de curación", Rarity::Uncommon),
        ],
        Class::Assassin => &[
            (AttackKind::Bolt, "Daga arrojadiza", Rarity::Common),
            (AttackKind::Swarm, "Abanico de dagas", Rarity::Epic),
            (AttackKind::Spikes, "Trampa de pinchos", Rarity::Uncommon),
            (AttackKind::Mine, "Bomba de humo", Rarity::Uncommon),
            (AttackKind::Trail, "Rastro venenoso", Rarity::Common),
            (AttackKind::Chain, "Danza de sombras", Rarity::Rare),
            (AttackKind::Drain, "Corte vital", Rarity::Rare),
            (AttackKind::Shockwave, "Explosión de sombra", Rarity::Rare),
        ],
        Class::Necro => &[
            (AttackKind::Swarm, "Enjambre de almas", Rarity::Common),
            (AttackKind::Drain, "Toque de la muerte", Rarity::Uncommon),
            (AttackKind::Aura, "Aura de huesos", Rarity::Rare),
            (AttackKind::Chain, "Cadena de almas", Rarity::Uncommon),
            (AttackKind::Trail, "Rastro de podredumbre", Rarity::Common),
            (AttackKind::Mend, "Pulso de costura", Rarity::Uncommon),
            (AttackKind::Mine, "Mina ritual", Rarity::Rare),
            (AttackKind::Vortex, "Remolino de almas", Rarity::Epic),
        ],
        Class::Cleric => &[
            (AttackKind::Beam, "Lanza de luz", Rarity::Rare),
            (AttackKind::Aura, "Halo sagrado", Rarity::Uncommon),
            (AttackKind::Nova, "Círculo de fe", Rarity::Rare),
            (AttackKind::Mend, "Plegaria", Rarity::Common),
            (AttackKind::Chain, "Cadena de salmos", Rarity::Uncommon),
            (AttackKind::Orbit, "Cáliz orbital", Rarity::Common),
            (AttackKind::Vortex, "Remolino bendito", Rarity::Rare),
            (AttackKind::Shockwave, "Golpe de campana", Rarity::Epic),
        ],
        Class::Merchant => &[
            (AttackKind::Bolt, "Moneda arrojadiza", Rarity::Common),
            (AttackKind::Mine, "Cofre trampa", Rarity::Uncommon),
            (AttackKind::Trail, "Rastro de oro", Rarity::Common),
            (AttackKind::Swarm, "Lluvia de monedas", Rarity::Epic),
            (AttackKind::Bolas, "Saco de oro", Rarity::Uncommon),
            (AttackKind::Meteor, "Lingote celeste", Rarity::Rare),
            (AttackKind::Vortex, "Remolino de deudas", Rarity::Rare),
            (AttackKind::Chain, "Cadena de pagarés", Rarity::Uncommon),
        ],
    }
}

fn describe(kind: AttackKind, element: Element) -> String {
    let adj = element.adjective();
    match kind {
        AttackKind::Bolt => format!("Lanza un proyectil {adj} que busca al enemigo más cercano."),
        AttackKind::Orbit => {
            format!("Un filo {adj} orbita a tu alrededor y corta a todo lo que toca.")
        }
        AttackKind::Aura => {
            format!("Un aura {adj} castiga a todo enemigo que se acerque demasiado.")
        }
        AttackKind::Nova => {
            format!("Cada pocos segundos estalla una nova {adj} a tu alrededor.")
        }
        AttackKind::Chain => {
            format!("El impacto {adj} salta a otro enemigo cercano con daño reducido.")
        }
        AttackKind::Mine => {
            format!("Deja una mina {adj} que detona cuando un enemigo se acerca.")
        }
        AttackKind::Trail => {
            format!("Deja un rastro {adj} que quema a quien lo pisa.")
        }
        AttackKind::Vortex => {
            format!("Invoca un remolino {adj} que arrastra a los enemigos hacia su centro.")
        }
        AttackKind::Bolas => {
            format!("Lanza bolas {adj} lentas que perforan y ralentizan la línea enemiga.")
        }
        AttackKind::Mend => {
            format!("Pulsos {adj} restauran tu vida con el tiempo.")
        }
        AttackKind::Beam => {
            format!("Dispara un rayo {adj} que atraviesa a todos los enemigos en línea.")
        }
        AttackKind::Meteor => {
            format!("Marca una zona y hace caer un meteoro {adj} tras un breve aviso.")
        }
        AttackKind::Swarm => {
            format!("Libera varios proyectiles {adj} que persiguen a los enemigos.")
        }
        AttackKind::Shockwave => {
            format!("Golpea el suelo y expande una onda {adj} que empuja y daña.")
        }
        AttackKind::Spikes => {
            format!("Pinchos {adj} emergen bajo los enemigos y los hieren.")
        }
        AttackKind::Drain => {
            format!("Drena la vida del enemigo más cercano con energía {adj}.")
        }
    }
}

fn stats(kind: AttackKind) -> (f32, f32, f32, u32) {
    // (daño base como multiplicador, recarga, radio, nivel máximo)
    match kind {
        AttackKind::Bolt => (0.62, 1.55, 10.0, 5),
        AttackKind::Orbit => (0.34, 0.55, 30.0, 5),
        AttackKind::Aura => (0.13, 0.70, 38.0, 5),
        AttackKind::Nova => (0.95, 4.20, 52.0, 5),
        AttackKind::Chain => (0.48, 1.75, 46.0, 5),
        AttackKind::Mine => (1.25, 3.10, 40.0, 5),
        AttackKind::Trail => (0.10, 0.30, 12.0, 5),
        AttackKind::Vortex => (0.16, 5.80, 46.0, 5),
        AttackKind::Bolas => (0.55, 2.90, 12.0, 5),
        AttackKind::Mend => (0.00, 6.00, 30.0, 5),
        AttackKind::Beam => (1.10, 3.30, 110.0, 5),
        AttackKind::Meteor => (1.65, 4.60, 42.0, 5),
        AttackKind::Swarm => (0.36, 2.60, 10.0, 5),
        AttackKind::Shockwave => (1.00, 3.80, 58.0, 5),
        AttackKind::Spikes => (0.90, 3.40, 34.0, 5),
        AttackKind::Drain => (0.82, 2.30, 80.0, 5),
    }
}

/// Construye el catálogo completo. Los nombres y descripciones se generan una
/// sola vez al arrancar el juego.
pub fn catalog() -> Vec<AttackDef> {
    let mut out = Vec::with_capacity(560);
    for class in Class::ALL {
        for &(kind, noun, rarity) in archetypes(class) {
            for element in Element::ALL {
                let (base_damage, base_cd, base_radius, max_level) = stats(kind);
                out.push(AttackDef {
                    name: format!("{} {}", noun, element.adjective()),
                    desc: describe(kind, element),
                    class,
                    element,
                    kind,
                    rarity,
                    max_level,
                    base_damage,
                    base_cd,
                    base_radius,
                    color: element.color(),
                });
            }
        }
    }
    out
}

pub fn level_damage(base: f32, level: u32) -> f32 {
    base * (1.0 + 0.28 * (level.saturating_sub(1)) as f32)
}

pub fn level_cd(base: f32, level: u32) -> f32 {
    (base * (1.0 - 0.07 * (level.saturating_sub(1)) as f32)).max(0.30)
}

pub fn level_radius(base: f32, level: u32) -> f32 {
    base * (1.0 + 0.09 * (level.saturating_sub(1)) as f32)
}

/// Ataque inicial de cada personaje. Todos empiezan con algo reconocible y
/// propio para que la primera sala no se juegue solo con el ataque básico.
pub fn starter(class: Class) -> usize {
    // El catálogo recorre clase, arquetipo y elemento en ese orden. El primer
    // arquetipo de cada clase con el elemento Hueso es el índice base.
    let class_index = Class::ALL.iter().position(|&c| c == class).unwrap_or(0);
    let element_index = Element::ALL
        .iter()
        .position(|&e| e == Element::Bone)
        .unwrap_or(0);
    class_index * 80 + element_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_hundreds_of_unique_class_attacks() {
        let cat = catalog();
        assert_eq!(cat.len(), 560);
        for class in Class::ALL {
            let mut names = std::collections::HashSet::new();
            let count = cat.iter().filter(|a| a.class == class).count();
            assert_eq!(count, 80);
            for a in cat.iter().filter(|a| a.class == class) {
                assert!(names.insert(a.name.clone()), "nombre duplicado: {}", a.name);
                assert!(a.max_level >= 5);
                assert!(!a.desc.is_empty());
            }
        }
    }

    #[test]
    fn starters_belong_to_their_class() {
        let cat = catalog();
        for class in Class::ALL {
            let i = starter(class);
            assert!(i < cat.len());
            assert_eq!(cat[i].class, class);
        }
    }
}
