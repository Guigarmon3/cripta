use crate::attacks::{self, AttackDef, AttackKind, EquippedAttack, MAX_EQUIPPED};
use crate::audio::{EnemyVoice, Sfx, SfxKind};
use crate::buffs::{self, BuffDef, BuffId, BUFF_COUNT};
use crate::figures;
use crate::save::{self, Meta};
use crate::sprites::{self, Sprites, VH, VW};
use crate::types::{with_alpha, Class, Difficulty, Rarity, Rng};
use crate::ui::Ui;
use crate::weapons::{self, WeaponEffect};
use macroquad::prelude::*;
use std::fmt::Write as FmtWrite;

const GRID: i32 = 5;
const ARENA_L: f32 = 24.0;
const ARENA_R: f32 = 456.0;
const ARENA_T: f32 = 44.0;
const ARENA_B: f32 = 228.0;
const MAX_PARTICLES: usize = 512;
const MAX_FLOATERS: usize = 48;

const C_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const C_GOLD: Color = Color::new(0.96, 0.78, 0.34, 1.0);
const C_RED: Color = Color::new(0.86, 0.26, 0.30, 1.0);
const C_GREEN: Color = Color::new(0.42, 0.85, 0.48, 1.0);
const C_CYAN: Color = Color::new(0.40, 0.90, 0.90, 1.0);
const C_PURPLE: Color = Color::new(0.72, 0.44, 0.92, 1.0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Playing,
    LevelUp,
    Paused,
    Dead,
    Victory,
    Shop,
    Locker,
    Stats,
    Audio,
    Library,
    Settings,
    Inventory,
    Tickets,
    Forge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomKind {
    Start,
    Combat,
    Boss,
    Shop,
    Rest,
    Treasure,
    Vault,
}

impl RoomKind {
    pub fn name(self) -> &'static str {
        match self {
            RoomKind::Start => "Entrada",
            RoomKind::Combat => "Sala de combate",
            RoomKind::Boss => "Sala del guardián",
            RoomKind::Shop => "Tienda",
            RoomKind::Rest => "Sala de descanso",
            RoomKind::Treasure => "Cámara del tesoro",
            RoomKind::Vault => "Bóveda sellada",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            RoomKind::Start => "Entrada",
            RoomKind::Combat => "Combate",
            RoomKind::Boss => "Guardián",
            RoomKind::Shop => "Tienda",
            RoomKind::Rest => "Descanso",
            RoomKind::Treasure => "Tesoro",
            RoomKind::Vault => "Bóveda",
        }
    }

    pub fn color(self) -> Color {
        match self {
            RoomKind::Start => C_GREEN,
            RoomKind::Combat => Color::from_rgba(225, 150, 120, 255),
            RoomKind::Boss => C_RED,
            RoomKind::Shop => C_GOLD,
            RoomKind::Rest => Color::from_rgba(120, 210, 235, 255),
            RoomKind::Treasure => C_PURPLE,
            RoomKind::Vault => Color::from_rgba(90, 70, 130, 255),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomSize {
    Tiny,
    Small,
    Medium,
    Large,
    Giant,
}

impl RoomSize {
    pub fn name(self) -> &'static str {
        match self {
            RoomSize::Tiny => "Enana",
            RoomSize::Small => "Pequeña",
            RoomSize::Medium => "Mediana",
            RoomSize::Large => "Grande",
            RoomSize::Giant => "Gigante",
        }
    }

    /// Límites interiores de la sala dentro del mundo de 480x270. El borde
    /// inferior queda siempre por encima del HUD para no tapar las puertas.
    pub fn bounds(self) -> (f32, f32, f32, f32) {
        match self {
            RoomSize::Tiny => (150.0, 330.0, 92.0, 202.0),
            RoomSize::Small => (118.0, 362.0, 76.0, 212.0),
            RoomSize::Medium => (64.0, 416.0, 62.0, 222.0),
            RoomSize::Large => (28.0, 452.0, 54.0, 228.0),
            RoomSize::Giant => (16.0, 464.0, 50.0, 228.0),
        }
    }
}

#[derive(Clone)]
pub struct Room {
    pub gx: i32,
    pub gy: i32,
    pub kind: RoomKind,
    pub size: RoomSize,
    pub visited: bool,
    pub cleared: bool,
    pub used: bool,
    pub pact_used: bool,
    pub has_chest: bool,
    pub chest_opened: bool,
    pub doors: [bool; 4],
    pub locked: [bool; 4],
    pub map_anim: f32,
    /// Altura relativa de la cámara respecto a la entrada. Los puentes y las
    /// escaleras se dibujan según esta diferencia.
    pub elevation: i8,
    /// La bóveda se abre con una llave especial, no con una llave normal.
    pub vault_open: bool,
    /// Semilla visual de la sala: suelos, grietas, estandartes y runas únicos.
    pub decor_seed: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyAttack {
    Melee,
    Bolt,
    Spread,
    Homing,
    Charge,
    Bomb,
    SlowAura,
    Summon,
    Teleport,
    Orbit,
    Burst,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Element {
    Bone,
    Ember,
    Frost,
    Venom,
    Void,
    Iron,
    Blood,
    Spirit,
    Fungus,
    Sand,
}

impl Element {
    pub const ALL: [Element; 10] = [
        Element::Bone,
        Element::Ember,
        Element::Frost,
        Element::Venom,
        Element::Void,
        Element::Iron,
        Element::Blood,
        Element::Spirit,
        Element::Fungus,
        Element::Sand,
    ];

    pub fn adjective(self) -> &'static str {
        match self {
            Element::Bone => "de hueso",
            Element::Ember => "de ceniza",
            Element::Frost => "glacial",
            Element::Venom => "venenoso",
            Element::Void => "del vacío",
            Element::Iron => "de hierro",
            Element::Blood => "de sangre",
            Element::Spirit => "espectral",
            Element::Fungus => "de esporas",
            Element::Sand => "del desierto",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Element::Bone => Color::from_rgba(232, 226, 210, 255),
            Element::Ember => Color::from_rgba(245, 132, 62, 255),
            Element::Frost => Color::from_rgba(122, 210, 235, 255),
            Element::Venom => Color::from_rgba(120, 200, 90, 255),
            Element::Void => Color::from_rgba(170, 105, 230, 255),
            Element::Iron => Color::from_rgba(170, 178, 190, 255),
            Element::Blood => Color::from_rgba(215, 62, 78, 255),
            Element::Spirit => Color::from_rgba(150, 220, 220, 255),
            Element::Fungus => Color::from_rgba(190, 170, 90, 255),
            Element::Sand => Color::from_rgba(230, 198, 120, 255),
        }
    }

    pub fn voice(self) -> EnemyVoice {
        match self {
            Element::Bone | Element::Iron => EnemyVoice::Clang,
            Element::Ember | Element::Sand => EnemyVoice::Fuse,
            Element::Frost | Element::Spirit => EnemyVoice::Wail,
            Element::Venom | Element::Fungus => EnemyVoice::Squelch,
            Element::Void => EnemyVoice::Chant,
            Element::Blood => EnemyVoice::Growl,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pact {
    Iron,
    Fire,
    Greed,
    Blood,
    Time,
}

impl Pact {
    pub const ALL: [Pact; 5] = [Pact::Iron, Pact::Fire, Pact::Greed, Pact::Blood, Pact::Time];

    pub fn name(self) -> &'static str {
        match self {
            Pact::Iron => "Pacto de hierro",
            Pact::Fire => "Pacto de fuego",
            Pact::Greed => "Pacto de codicia",
            Pact::Blood => "Pacto de sangre",
            Pact::Time => "Pacto de tiempo",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Pact::Iron => "+5 de armadura, -8% de velocidad.",
            Pact::Fire => "+30% de daño, -15% de vida máxima.",
            Pact::Greed => "+45% de oro, -10% de vida máxima.",
            Pact::Blood => "+15% de robo de vida, -18% de vida máxima.",
            Pact::Time => "+18% de velocidad de ataque, -10% de velocidad.",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Pact::Iron => Color::from_rgba(180, 190, 205, 255),
            Pact::Fire => Color::from_rgba(245, 132, 62, 255),
            Pact::Greed => C_GOLD,
            Pact::Blood => C_RED,
            Pact::Time => C_CYAN,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ring {
    pub active: bool,
    pub pos: Vec2,
    pub radius: f32,
    pub max_radius: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub thickness: f32,
    pub fill: bool,
}

#[derive(Clone, Copy)]
pub enum ShopKind {
    Heal,
    Item(Item),
    Buff(BuffDef),
    Key,
    SpecialKey,
}

#[derive(Clone)]
pub struct ShopOffer {
    pub name: String,
    pub desc: String,
    pub cost: u32,
    pub kind: ShopKind,
    pub sold: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Slime,
    Bat,
    Brute,
    Cultist,
    Wraith,
    Bomber,
    Sentinel,
    Eye,
    Boss,
}

impl EnemyKind {
    pub const NORMAL: [EnemyKind; 8] = [
        EnemyKind::Slime,
        EnemyKind::Bat,
        EnemyKind::Brute,
        EnemyKind::Cultist,
        EnemyKind::Wraith,
        EnemyKind::Bomber,
        EnemyKind::Sentinel,
        EnemyKind::Eye,
    ];

    /// Tres linajes por arquetipo. Con los diez elementos salen 240 enemigos
    /// normales distintos, cada uno con su nombre y su color.
    pub fn variants(self) -> [&'static str; 3] {
        match self {
            EnemyKind::Slime => ["Limo", "Baba", "Moco"],
            EnemyKind::Bat => ["Murciélago", "Nocturno", "Chupasangre"],
            EnemyKind::Brute => ["Bruto", "Verdugo", "Carnicero"],
            EnemyKind::Cultist => ["Cultista", "Acólito", "Sacerdote"],
            EnemyKind::Wraith => ["Espectro", "Ánima", "Lamento"],
            EnemyKind::Bomber => ["Saboteador", "Bombardero", "Incendiario"],
            EnemyKind::Sentinel => ["Centinela", "Vigía", "Guardián de hierro"],
            EnemyKind::Eye => ["Ojo", "Vidente", "Mirada"],
            EnemyKind::Boss => ["Guardián", "Señor", "Custodio"],
        }
    }

    /// Diez títulos de guardián. Con los diez elementos salen 100 jefes
    /// distintos, cada uno con su nombre y su estilo de combate.
    pub fn boss_titles() -> [&'static str; 10] {
        [
            "Guardián",
            "Señor",
            "Custodio",
            "Verdugo",
            "Devorador",
            "Heresiarca",
            "Coloso",
            "Oráculo",
            "Tirano",
            "Abismo",
        ]
    }

    pub fn attack(self) -> EnemyAttack {
        match self {
            EnemyKind::Slime => EnemyAttack::SlowAura,
            EnemyKind::Bat => EnemyAttack::Charge,
            EnemyKind::Brute => EnemyAttack::Melee,
            EnemyKind::Cultist => EnemyAttack::Bolt,
            EnemyKind::Wraith => EnemyAttack::Homing,
            EnemyKind::Bomber => EnemyAttack::Bomb,
            EnemyKind::Sentinel => EnemyAttack::Orbit,
            EnemyKind::Eye => EnemyAttack::Spread,
            EnemyKind::Boss => EnemyAttack::Burst,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    Barrel,
    Crate,
    Chest,
    Urn,
    Statue,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    None,
    Fog,
    Darkness,
    Greed,
}

#[derive(Clone, Copy)]
pub enum ItemKind {
    Damage,
    MaxHp,
    Speed,
    Crit,
    Lifesteal,
    Thorns,
    Armor,
    SkeletonCap,
    Regen,
    AllStats,
    GlassCannon,
    CrystalHeart,
    Orbit,
    Homing,
    Chain,
    Nova,
    Mine,
    Retaliate,
    DashBurst,
    Vortex,
    Bolas,
    Trail,
    Aura,
    Mend,
}

impl ItemKind {
    pub fn label(self) -> &'static str {
        match self {
            ItemKind::Damage => "Daño",
            ItemKind::MaxHp => "Vida",
            ItemKind::Speed => "Velocidad",
            ItemKind::Crit => "Crítico",
            ItemKind::Lifesteal => "Robo de vida",
            ItemKind::Thorns => "Espinas",
            ItemKind::Armor => "Armadura",
            ItemKind::SkeletonCap => "Esqueletos",
            ItemKind::Regen => "Regeneración",
            ItemKind::AllStats => "Todas las stats",
            ItemKind::GlassCannon => "Cañón de cristal",
            ItemKind::CrystalHeart => "Corazón de cristal",
            ItemKind::Orbit => "Órbita",
            ItemKind::Homing => "Persecución",
            ItemKind::Chain => "Cadena",
            ItemKind::Nova => "Nova",
            ItemKind::Mine => "Mina",
            ItemKind::Retaliate => "Represalia",
            ItemKind::DashBurst => "Estallido de esquiva",
            ItemKind::Vortex => "Vórtice",
            ItemKind::Bolas => "Bolas",
            ItemKind::Trail => "Rastro",
            ItemKind::Aura => "Aura",
            ItemKind::Mend => "Curación",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Item {
    pub name: &'static str,
    pub kind: ItemKind,
    pub rarity: Rarity,
    pub value: f32,
}

#[derive(Clone, Copy)]
pub enum PickupKind {
    Gold(u32),
    Xp(f32),
    Heal(f32),
    Item(Item),
}

pub struct Pickup {
    pub pos: Vec2,
    pub vel: Vec2,
    pub kind: PickupKind,
    pub life: f32,
    pub bob: f32,
}

/// Una de las tres opciones que aparecen al subir de nivel.
#[derive(Clone, Copy)]
pub enum LevelChoice {
    NewAttack(usize),
    UpgradeAttack(usize),
    Buff(BuffDef),
}

#[derive(Clone, Copy)]
pub struct BeamFx {
    pub pos: Vec2,
    pub dir: Vec2,
    pub length: f32,
    pub width: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct Meteor {
    pub pos: Vec2,
    pub t: f32,
    pub max_t: f32,
    pub radius: f32,
    pub damage: f32,
    pub color: Color,
}

#[derive(Clone)]
pub struct Player {
    pub class: Class,
    pub pos: Vec2,
    pub facing: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub attack_speed: f32,
    pub range: f32,
    pub crit: f32,
    pub crit_mult: f32,
    pub armor: f32,
    pub lifesteal: f32,
    pub thorns: f32,
    pub block: f32,
    pub reflect: f32,
    pub regen: f32,
    pub xp_mult: f32,
    pub gold_mult: f32,
    pub luck: f32,
    pub aoe: f32,
    pub pierce: u32,
    pub extra_projectiles: u32,
    pub execute: f32,
    pub dash_cd_max: f32,
    pub dash_cd: f32,
    pub dash_t: f32,
    pub dash_dir: Vec2,
    pub invuln: f32,
    pub attack_t: f32,
    pub attack_count: u32,
    pub hit_flash: f32,
    pub level: u32,
    pub xp: f32,
    pub xp_next: f32,
    pub gold: u32,
    pub kills: u32,
    pub skeleton_cap: u32,
    pub skeleton_dmg: f32,
    pub skeleton_inherit: f32,
    pub skeleton_explode: bool,
    pub shockwave: u32,
    pub bloodlust: u32,
    pub kill_speed_t: f32,
    pub burn_on_hit: f32,
    pub chain: u32,
    pub slow_t: f32,
    pub orbit: u32,
    pub homing: f32,
    pub nova: u32,
    pub mine: u32,
    pub retaliate: u32,
    pub dash_burst: u32,
    pub orbit_cd: f32,
    pub vortex: u32,
    pub bolas: u32,
    pub trail: u32,
    pub aura: u32,
    pub mend: u32,
    pub vortex_cd: f32,
    pub trail_cd: f32,
    pub aura_tick: f32,
    pub mend_cd: f32,
    /// Reloj de caminata: avanza al moverse y anima el balanceo.
    pub walk_t: f32,
    /// Se movió este frame (para decidir el balanceo).
    pub moving: bool,
    /// Tiempo restante de la animación de golpe (impulso o retroceso).
    pub swing_t: f32,
}

impl Player {
    pub fn new(class: Class, meta: &Meta) -> Self {
        let (
            max_hp,
            speed,
            damage,
            attack_speed,
            range,
            crit,
            crit_mult,
            armor,
            block,
            aoe,
            execute,
            skel_cap,
            skel_dmg,
            dash_cd,
        ) = match class {
            Class::Knight => (
                130.0, 84.0, 11.0, 1.25, 24.0, 0.05, 2.0, 2.0, 0.10, 0.0, 0.0, 0, 0.0, 2.6,
            ),
            Class::Archer => (
                82.0, 96.0, 7.0, 1.9, 94.0, 0.15, 2.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 2.2,
            ),
            Class::Mage => (
                74.0, 88.0, 13.0, 1.0, 102.0, 0.08, 2.0, 0.0, 0.0, 16.0, 0.0, 0, 0.0, 2.4,
            ),
            Class::Assassin => (
                78.0, 104.0, 9.0, 2.1, 21.0, 0.22, 2.2, 0.0, 0.0, 0.0, 0.2, 0, 0.0, 1.8,
            ),
            Class::Necro => (
                88.0, 90.0, 8.0, 1.35, 90.0, 0.08, 2.0, 0.0, 0.0, 0.0, 0.0, 3, 0.28, 2.4,
            ),
            Class::Cleric => (
                96.0, 90.0, 9.0, 1.45, 86.0, 0.06, 2.0, 1.0, 0.05, 6.0, 0.0, 0, 0.0, 2.3,
            ),
            Class::Merchant => (
                84.0, 94.0, 8.0, 1.60, 80.0, 0.10, 2.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 2.2,
            ),
        };
        let mut p = Self {
            class,
            pos: vec2(240.0, 150.0),
            facing: vec2(0.0, -1.0),
            hp: max_hp,
            max_hp,
            speed,
            damage,
            attack_speed,
            range,
            crit,
            crit_mult,
            armor,
            lifesteal: 0.0,
            thorns: 0.0,
            block,
            reflect: 0.0,
            regen: 0.0,
            xp_mult: 1.0,
            gold_mult: 1.0,
            luck: 0.0,
            aoe,
            pierce: 0,
            extra_projectiles: 0,
            execute,
            dash_cd_max: dash_cd,
            dash_cd: 0.0,
            dash_t: 0.0,
            dash_dir: vec2(1.0, 0.0),
            invuln: 0.0,
            attack_t: 0.0,
            attack_count: 0,
            hit_flash: 0.0,
            level: 1,
            xp: 0.0,
            xp_next: 20.0,
            gold: 0,
            kills: 0,
            skeleton_cap: skel_cap,
            skeleton_dmg: skel_dmg,
            skeleton_inherit: 0.0,
            skeleton_explode: false,
            shockwave: 0,
            bloodlust: 0,
            kill_speed_t: 0.0,
            burn_on_hit: 0.0,
            chain: 0,
            slow_t: 0.0,
            orbit: 0,
            homing: 0.0,
            nova: 0,
            mine: 0,
            retaliate: 0,
            dash_burst: 0,
            orbit_cd: 0.0,
            vortex: 0,
            bolas: 0,
            trail: 0,
            aura: 0,
            mend: 0,
            vortex_cd: 0.0,
            trail_cd: 0.0,
            aura_tick: 0.0,
            mend_cd: 0.0,
            walk_t: 0.0,
            moving: false,
            swing_t: 0.0,
        };
        p.max_hp += meta.hp_lv as f32 * 10.0;
        p.hp = p.max_hp;
        p.damage *= 1.0 + meta.dmg_lv as f32 * 0.05;
        p.luck += meta.luck_lv as f32 * 0.10;
        match class {
            Class::Cleric => p.regen += 0.45,
            Class::Merchant => {
                p.gold_mult = 1.25;
                p.luck += 0.10;
            }
            _ => {}
        }
        if let Some(weapon) = weapons::get(class, meta.weapon_for(class)) {
            apply_weapon(&mut p, weapon);
        }
        p
    }
}

pub struct Skeleton {
    pub pos: Vec2,
    pub vel: Vec2,
    pub damage: f32,
    pub life: f32,
    pub attack_t: f32,
    pub bob: f32,
}

/// Aliado espectral equipado con un contrato de la taquilla. No puede morir:
/// orbita al jugador y aporta daño, curación o suerte según el contrato.
pub struct Ally {
    pub kind: u8,
    pub pos: Vec2,
    pub bob: f32,
    pub attack_t: f32,
    pub orbit_t: f32,
    /// Animación de golpe: el aliado se impulsa hacia su objetivo.
    pub swing_t: f32,
}

impl Ally {
    pub fn new(kind: u8, pos: Vec2) -> Self {
        Self {
            kind,
            pos,
            bob: 0.0,
            attack_t: 0.6,
            orbit_t: 0.0,
            swing_t: 0.0,
        }
    }

    pub fn color(&self) -> Color {
        match self.kind {
            1 => Color::from_rgba(150, 220, 220, 255),
            2 => Color::from_rgba(245, 214, 120, 255),
            3 => Color::from_rgba(230, 190, 90, 255),
            4 => Color::from_rgba(205, 90, 110, 255),
            _ => C_WHITE,
        }
    }
}

pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub radius: f32,
    pub attack_t: f32,
    pub attack_cd: f32,
    pub hit_flash: f32,
    pub elite: bool,
    pub shiny: bool,
    pub boss: bool,
    pub slow_t: f32,
    pub burn_t: f32,
    pub burn_dps: f32,
    pub special_t: f32,
    pub life: f32,
    pub bob: f32,
    pub dead: bool,
    pub name: String,
    pub attack: EnemyAttack,
    pub element: Element,
    pub voice: EnemyVoice,
    pub tint: Color,
    pub state_t: f32,
    pub charge_dir: Vec2,
    /// Destino anunciado del teletransporte. Se dibuja durante el aviso.
    pub teleport_to: Vec2,
    pub summoned: bool,
    pub boss_skin: usize,
    /// Reloj del ataque de firma de los guardianes.
    pub signature_t: f32,
    /// Sacudida breve tras desatar la firma.
    pub cast_t: f32,
    /// Animación de aparición: cae desde arriba durante este tiempo.
    pub spawn_t: f32,
    /// Animación de muerte antes de retirar el cadáver.
    pub death_t: f32,
    /// Reloj de caminata para animar las patas/alas.
    pub walk_t: f32,
    /// Animación de golpe cuerpo a cuerpo: impulso breve hacia el jugador.
    pub swing_t: f32,
    /// Salto de guardián: aviso en el suelo antes de despegar.
    pub jump_wind: f32,
    /// Tiempo restante en el aire. Mientras dura, ignora obstáculos.
    pub jump_t: f32,
    pub jump_from: Vec2,
    pub jump_to: Vec2,
    pub jump_cd: f32,
    /// Detector de atasco: si el jefe no avanza, fuerza un salto.
    pub stuck_t: f32,
    pub last_pos: Vec2,
}

pub struct Prop {
    pub pos: Vec2,
    pub kind: PropKind,
    pub hp: f32,
    pub hit_flash: f32,
    pub dead: bool,
    pub bob: f32,
    pub opened: bool,
    pub open_t: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    Pillar,
    Rock,
    Bones,
    Brazier,
    Spikes,
}

#[derive(Clone, Copy)]
pub struct Obstacle {
    pub pos: Vec2,
    pub radius: f32,
    pub kind: ObstacleKind,
    pub bob: f32,
}

pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub damage: f32,
    pub radius: f32,
    pub life: f32,
    pub friendly: bool,
    pub color: Color,
    pub pierce: u32,
    pub crit: bool,
    pub aoe: f32,
    pub slow: f32,
    pub burn: f32,
    pub chain: u32,
    pub homing: f32,
    pub hits: [u32; 8],
    pub hit_count: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HazardKind {
    Trail,
    Vortex,
}

/// Zona persistente en el suelo: brasas que deja el portador o un vórtice que
/// arrastra a los enemigos hacia su centro.
#[derive(Clone, Copy)]
pub struct Hazard {
    pub pos: Vec2,
    pub radius: f32,
    pub life: f32,
    pub max_life: f32,
    pub dps: f32,
    pub tick: f32,
    pub pull: f32,
    pub color: Color,
    pub kind: HazardKind,
}

#[derive(Clone, Copy)]
pub struct Mine {
    pub pos: Vec2,
    pub t: f32,
    pub damage: f32,
    pub radius: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub active: bool,
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
    pub gravity: f32,
    pub drag: f32,
}

impl Particle {
    pub fn empty() -> Self {
        Self {
            active: false,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            life: 0.0,
            max_life: 1.0,
            size: 1.0,
            color: C_WHITE,
            gravity: 0.0,
            drag: 0.0,
        }
    }
}

pub struct Floater {
    pub active: bool,
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
    pub text: String,
}

pub struct Toast {
    pub text: String,
    pub color: Color,
    pub t: f32,
}

#[derive(Clone, Copy)]
/// Entrada simulada para el modo de grabación de la guía. Solo lo usa
/// `demo.rs`; en una partida normal queda inactiva.
pub struct DemoInput {
    pub active: bool,
    pub dir: Vec2,
    pub dash: bool,
    pub interact: bool,
    pub buy: Option<usize>,
}

impl Default for DemoInput {
    fn default() -> Self {
        Self {
            active: false,
            dir: Vec2::ZERO,
            dash: false,
            interact: false,
            buy: None,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct View {
    pub scale: f32,
    pub ox: f32,
    pub oy: f32,
    pub mx: f32,
    pub my: f32,
}

fn safe_norm(v: Vec2) -> Vec2 {
    let l = v.length();
    if l > 0.0001 {
        v / l
    } else {
        Vec2::ZERO
    }
}

fn rotate(v: Vec2, a: f32) -> Vec2 {
    let (s, c) = a.sin_cos();
    vec2(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// Empuja un círculo fuera de los obstáculos sólidos de la sala.
fn push_out_of_obstacles(pos: &mut Vec2, radius: f32, obstacles: &[Obstacle]) {
    for o in obstacles {
        // Los pinchos son suelo peligroso, no una pared: se puede caminar
        // sobre ellos (y duele).
        if o.kind == ObstacleKind::Spikes {
            continue;
        }
        let d = *pos - o.pos;
        let min = radius + o.radius;
        let dist = d.length();
        if dist > 0.001 && dist < min {
            *pos += d / dist * (min - dist);
        }
    }
}

fn enemy_shot(
    pos: Vec2,
    vel: Vec2,
    damage: f32,
    color: Color,
    radius: f32,
    life: f32,
    homing: f32,
    slow: f32,
    aoe: f32,
) -> Projectile {
    Projectile {
        pos,
        vel,
        damage,
        radius,
        life,
        friendly: false,
        color,
        pierce: 0,
        crit: false,
        aoe,
        slow,
        burn: 0.0,
        chain: 0,
        homing,
        hits: [0; 8],
        hit_count: 0,
    }
}

fn spawn_particle(
    particles: &mut Vec<Particle>,
    pos: Vec2,
    vel: Vec2,
    life: f32,
    size: f32,
    color: Color,
    gravity: f32,
    drag: f32,
) {
    if let Some(p) = particles.iter_mut().find(|p| !p.active) {
        p.active = true;
        p.pos = pos;
        p.vel = vel;
        p.life = life;
        p.max_life = life;
        p.size = size;
        p.color = color;
        p.gravity = gravity;
        p.drag = drag;
    }
}

fn burst(particles: &mut Vec<Particle>, pos: Vec2, count: usize, color: Color, speed: f32) {
    for i in 0..count {
        let a = i as f32 * 2.399 + pos.x * 0.03;
        let v = vec2(a.cos(), a.sin()) * speed * (0.45 + (i % 5) as f32 * 0.14);
        spawn_particle(
            particles,
            pos,
            v,
            0.30 + (i % 4) as f32 * 0.08,
            1.5 + (i % 3) as f32,
            color,
            80.0,
            2.0,
        );
    }
}

fn spawn_ring(
    rings: &mut Vec<Ring>,
    pos: Vec2,
    start_radius: f32,
    max_radius: f32,
    life: f32,
    color: Color,
    thickness: f32,
) {
    if let Some(r) = rings.iter_mut().find(|r| !r.active) {
        r.active = true;
        r.pos = pos;
        r.radius = start_radius;
        r.max_radius = max_radius;
        r.life = life;
        r.max_life = life;
        r.color = color;
        r.thickness = thickness;
        r.fill = false;
    }
}

fn spawn_floater(floaters: &mut Vec<Floater>, pos: Vec2, text: &str, color: Color, size: f32) {
    if let Some(f) = floaters.iter_mut().find(|f| !f.active) {
        f.active = true;
        f.pos = pos;
        f.vel = vec2(0.0, -24.0);
        f.life = 0.75;
        f.max_life = 0.75;
        f.color = color;
        f.size = size;
        f.text.clear();
        f.text.push_str(text);
    }
}

fn spawn_number(floaters: &mut Vec<Floater>, pos: Vec2, value: i32, color: Color, size: f32) {
    if let Some(f) = floaters.iter_mut().find(|f| !f.active) {
        f.active = true;
        f.pos = pos;
        f.vel = vec2(0.0, -24.0);
        f.life = 0.75;
        f.max_life = 0.75;
        f.color = color;
        f.size = size;
        f.text.clear();
        let _ = write!(f.text, "{}", value);
    }
}

pub struct Game {
    pub screen: Screen,
    pub class: Class,
    pub player: Player,
    pub meta: Meta,
    pub rng: Rng,
    pub enemies: Vec<Enemy>,
    pub ally: Option<Ally>,
    pub projectiles: Vec<Projectile>,
    pub skeletons: Vec<Skeleton>,
    pub props: Vec<Prop>,
    pub pickups: Vec<Pickup>,
    pub particles: Vec<Particle>,
    pub floaters: Vec<Floater>,
    pub toasts: Vec<Toast>,
    pub items: Vec<Item>,
    pub attacks: Vec<AttackDef>,
    pub equipped_attacks: Vec<EquippedAttack>,
    pub level_choices: [LevelChoice; 3],
    pub pending_replace: Option<usize>,
    pub beams: Vec<BeamFx>,
    pub meteors: Vec<Meteor>,
    pub floor: u32,
    pub wave: u32,
    pub waves: u32,
    pub cleared: bool,
    pub portal: Vec2,
    pub modifier: Modifier,
    pub time: f32,
    pub shake: f32,
    pub hitstop: f32,
    /// Cadencia del daño de los pinchos del suelo.
    pub spike_t: f32,
    /// Presentación del guardián: tiempo restante y datos para el cartel.
    pub boss_intro_t: f32,
    pub boss_intro_name: String,
    pub boss_intro_desc: String,
    pub boss_intro_color: Color,
    pub boss_intro_skin: usize,
    /// Transición de piso: cuenta atrás y si hay un descenso esperando.
    pub floor_transition_t: f32,
    pub floor_transition_pending: bool,
    /// Fundido de entrada del piso nuevo.
    pub floor_arrival_t: f32,
    pub cards: [BuffDef; 3],
    pub rerolls: u32,
    pub buff_stacks: [u32; BUFF_COUNT],
    pub menu_index: usize,
    pub kills: u32,
    pub run_gold: u32,
    pub souls_earned: u32,
    pub endless: bool,
    pub next_enemy_id: u32,
    pub view: View,
    pub demo_input: DemoInput,
    pub dev_mode: bool,
    pub dev_build: String,
    pub dev_note: String,
    pub dev_note_t: f32,
    pub show_debug: bool,
    pub fps: f32,
    pub frame_ms: f32,
    pub wave_timer: f32,
    pub skeleton_respawn_t: f32,
    pub victory_pending: bool,
    pub rooms: Vec<Room>,
    pub room_index: usize,
    pub keys: u32,
    /// Llaves negras de bóveda: no abren puertas normales.
    pub special_keys: u32,
    pub debt: u32,
    pub echo_claimed: bool,
    pub room_voice: Option<EnemyVoice>,
    /// 0 = puertas abiertas, 1 = puertas selladas por el combate.
    pub door_seal: f32,
    pub door_slam_t: f32,
    pub mines: Vec<Mine>,
    pub hazards: Vec<Hazard>,
    pub bosses_used: [bool; 12],
    pub map_open: bool,
    pub map_t: f32,
    pub room_fade: f32,
    pub door_msg_t: f32,
    pub portal_t: f32,
    pub rings: Vec<Ring>,
    pub shop_offers: Vec<ShopOffer>,
    pub levelup_t: f32,
    pub stats_return: Screen,
    pub settings_return: Screen,
    pub inventory_return: Screen,
    pub shop_tab: usize,
    pub locker_class: usize,
    pub forge_class: usize,
    pub forge_return: Screen,
    pub editing_name: bool,
    pub name_buffer: String,
    pub name_t: f32,
    pub menu_repeat: f32,
    pub music_on: bool,
    pub obstacles: Vec<Obstacle>,
    pub audio_row: usize,
    pub audio_return: Screen,
    pub pacts_taken: u32,
    pub taken_pacts: Vec<Pact>,
    pub floor_element: Element,
    pub merchant_t: f32,
    pub rest_offers: [Pact; 2],
    pub library_tab: usize,
    pub library_page: usize,
    pub settings_row: usize,
    pub rebinding: bool,
}

impl Game {
    pub fn new(meta: Meta) -> Self {
        let dev_mode = std::env::var("CRIPTA_DEV")
            .map(|v| v == "1")
            .unwrap_or(false);
        let dev_build = std::env::var("CRIPTA_BUILD").unwrap_or_default();
        let dev_note = std::env::var("CRIPTA_NOTE").unwrap_or_default();
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            ^ 0x9E37_79B9_7F4A_7C15;
        let music_on = meta.music;
        let mut g = Self {
            screen: Screen::Menu,
            class: Class::Knight,
            player: Player::new(Class::Knight, &meta),
            meta,
            rng: Rng::new(seed),
            enemies: Vec::with_capacity(32),
            ally: None,
            projectiles: Vec::with_capacity(128),
            skeletons: Vec::with_capacity(8),
            props: Vec::with_capacity(12),
            pickups: Vec::with_capacity(128),
            particles: (0..MAX_PARTICLES).map(|_| Particle::empty()).collect(),
            floaters: (0..MAX_FLOATERS)
                .map(|_| Floater {
                    active: false,
                    pos: Vec2::ZERO,
                    vel: Vec2::ZERO,
                    life: 0.0,
                    max_life: 1.0,
                    color: C_WHITE,
                    size: 8.0,
                    text: String::with_capacity(16),
                })
                .collect(),
            toasts: Vec::with_capacity(4),
            items: Vec::with_capacity(32),
            attacks: attacks::catalog(),
            equipped_attacks: Vec::with_capacity(MAX_EQUIPPED),
            level_choices: [LevelChoice::Buff(buffs::POOL[0]); 3],
            pending_replace: None,
            beams: Vec::with_capacity(16),
            meteors: Vec::with_capacity(12),
            floor: 0,
            wave: 0,
            waves: 2,
            cleared: false,
            portal: vec2(240.0, 128.0),
            modifier: Modifier::None,
            time: 0.0,
            shake: 0.0,
            hitstop: 0.0,
            spike_t: 0.0,
            boss_intro_t: 0.0,
            boss_intro_name: String::new(),
            boss_intro_desc: String::new(),
            boss_intro_color: C_RED,
            boss_intro_skin: 0,
            floor_transition_t: 0.0,
            floor_transition_pending: false,
            floor_arrival_t: 0.0,
            cards: [buffs::POOL[0]; 3],
            rerolls: 1,
            buff_stacks: [0; BUFF_COUNT],
            menu_index: 0,
            kills: 0,
            run_gold: 0,
            souls_earned: 0,
            endless: false,
            next_enemy_id: 1,
            view: View::default(),
            demo_input: DemoInput::default(),
            dev_mode,
            dev_build,
            dev_note,
            dev_note_t: 0.0,
            show_debug: true,
            fps: 60.0,
            frame_ms: 16.6,
            wave_timer: 0.0,
            skeleton_respawn_t: 0.0,
            victory_pending: false,
            rooms: Vec::with_capacity(9),
            room_index: 0,
            keys: 0,
            special_keys: 0,
            debt: 0,
            echo_claimed: false,
            room_voice: None,
            door_seal: 0.0,
            door_slam_t: 0.0,
            mines: Vec::with_capacity(16),
            hazards: Vec::with_capacity(32),
            bosses_used: [false; 12],
            map_open: false,
            map_t: 0.0,
            room_fade: 0.0,
            door_msg_t: 0.0,
            portal_t: 0.0,
            rings: (0..96)
                .map(|_| Ring {
                    active: false,
                    pos: Vec2::ZERO,
                    radius: 0.0,
                    max_radius: 1.0,
                    life: 0.0,
                    max_life: 1.0,
                    color: C_WHITE,
                    thickness: 1.0,
                    fill: false,
                })
                .collect(),
            shop_offers: Vec::with_capacity(4),
            levelup_t: 1.0,
            stats_return: Screen::Menu,
            settings_return: Screen::Menu,
            inventory_return: Screen::Paused,
            shop_tab: 0,
            locker_class: 0,
            forge_class: 0,
            forge_return: Screen::Menu,
            editing_name: false,
            name_buffer: String::new(),
            name_t: 0.0,
            menu_repeat: 0.0,
            music_on,
            obstacles: Vec::with_capacity(8),
            audio_row: 0,
            audio_return: Screen::Menu,
            pacts_taken: 0,
            taken_pacts: Vec::new(),
            floor_element: Element::Bone,
            merchant_t: 0.0,
            rest_offers: [Pact::Iron, Pact::Time],
            library_tab: 0,
            library_page: 0,
            settings_row: 0,
            rebinding: false,
        };
        g.dev_note_t = if dev_mode { 7.0 } else { 0.0 };
        g
    }

    pub fn start_run(&mut self, class: Class) {
        self.class = class;
        self.player = Player::new(class, &self.meta);
        // Contrato de la taquilla: un aliado espectral y su bonus pasivo.
        self.ally = if self.meta.ticket > 0 && (self.meta.ticket as usize) < save::TICKETS.len() {
            match self.meta.ticket {
                1 => self.player.armor += 1.0,
                2 => self.player.regen += 0.4,
                3 => {
                    self.player.gold_mult += 0.20;
                    self.player.luck += 0.08;
                }
                4 => self.player.damage *= 1.05,
                _ => {}
            }
            Some(Ally::new(
                self.meta.ticket,
                self.player.pos + vec2(-16.0, 8.0),
            ))
        } else {
            None
        };
        self.enemies.clear();
        self.projectiles.clear();
        self.skeletons.clear();
        self.props.clear();
        self.pickups.clear();
        self.items.clear();
        self.equipped_attacks.clear();
        self.equipped_attacks
            .push(EquippedAttack::new(attacks::starter(class)));
        self.pending_replace = None;
        self.beams.clear();
        self.meteors.clear();
        self.toasts.clear();
        for p in &mut self.particles {
            p.active = false;
        }
        for f in &mut self.floaters {
            f.active = false;
        }
        self.buff_stacks = [0; BUFF_COUNT];
        self.floor = 0;
        self.kills = 0;
        self.run_gold = 0;
        self.souls_earned = 0;
        self.endless = false;
        self.rerolls = 1;
        self.next_enemy_id = 1;
        self.time = 0.0;
        self.shake = 0.0;
        self.hitstop = 0.0;
        self.boss_intro_t = 0.0;
        self.floor_transition_t = 0.0;
        self.floor_transition_pending = false;
        self.floor_arrival_t = 0.0;
        self.pacts_taken = 0;
        self.taken_pacts.clear();
        self.merchant_t = 0.0;
        self.obstacles.clear();
        self.mines.clear();
        self.hazards.clear();
        self.door_seal = 0.0;
        self.door_slam_t = 0.0;
        self.room_voice = None;
        self.special_keys = 0;
        self.debt = self.meta.debt;
        self.echo_claimed = false;
        self.bosses_used = [false; 12];
        self.screen = Screen::Playing;
        self.next_floor();
    }

    pub fn next_floor(&mut self) {
        self.floor += 1;
        self.wave = 0;
        self.cleared = false;
        self.victory_pending = false;
        self.boss_intro_t = 0.0;
        self.portal = vec2(240.0, 128.0);
        self.modifier = self.roll_modifier();
        self.enemies.clear();
        self.projectiles.clear();
        self.skeletons.clear();
        self.props.clear();
        self.pickups.clear();
        self.obstacles.clear();
        self.mines.clear();
        self.hazards.clear();
        self.door_seal = 0.0;
        self.door_slam_t = 0.0;
        self.room_voice = None;
        for r in &mut self.rings {
            r.active = false;
        }
        for f in &mut self.floaters {
            f.active = false;
        }
        self.skeleton_respawn_t = 0.0;
        self.wave_timer = 0.0;
        self.portal_t = 0.0;
        self.player.pos = vec2(240.0, 198.0);
        if let Some(a) = &mut self.ally {
            a.pos = self.player.pos + vec2(-16.0, 8.0);
            a.attack_t = 0.6;
        }
        self.player.invuln = 1.0;
        self.player.attack_t = 0.0;
        self.player.dash_t = 0.0;
        self.player.kill_speed_t = 0.0;
        if self.floor > 1 {
            let heal = self.player.max_hp * 0.20;
            self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
        }
        self.generate_dungeon();
        self.generate_shop_offers();
        let start = self.room_index;
        self.enter_room(start, 4);
        if self.class == Class::Necro {
            self.summon_all();
        }
        let c = self.modifier_color();
        let m = self.modifier_name();
        if self.modifier != Modifier::None {
            self.toast(&format!("Piso {} - {}", self.floor, m), c);
        } else {
            self.toast(&format!("Piso {}", self.floor), C_WHITE);
        }
        if self.floor == 1 {
            self.toast(
                "WASD mover   Espacio esquiva   Tab mapa   Esc pausa",
                Color::from_rgba(150, 200, 220, 255),
            );
            if self.debt > 0 {
                self.toast(
                    &format!(
                        "El mercader te espera en la entrada: debes {} de oro",
                        self.debt
                    ),
                    C_RED,
                );
            }
            if self.meta.echoes > 0 {
                self.toast("Un eco de una partida muerta vaga por la entrada", C_CYAN);
            }
        }
    }

    fn room_at(&self, gx: i32, gy: i32) -> Option<usize> {
        self.rooms.iter().position(|r| r.gx == gx && r.gy == gy)
    }

    fn current_kind(&self) -> RoomKind {
        self.rooms
            .get(self.room_index)
            .map(|r| r.kind)
            .unwrap_or(RoomKind::Combat)
    }

    /// El oro pesa: a partir de 90 monedas la carga ralentiza al jugador y
    /// atrae espíritus de codicia. Satura a las 350.
    fn gold_weight(&self) -> f32 {
        let raw = ((self.player.gold as f32 - 90.0) / 260.0).clamp(0.0, 1.0);
        // El mercader negocia con el peso: su oro pesa la mitad.
        if self.player.class == Class::Merchant {
            raw * 0.5
        } else {
            raw
        }
    }

    fn bind_down(&self, action: usize) -> bool {
        key_from_code(self.meta.binds.codes[action])
            .map(is_key_down)
            .unwrap_or(false)
    }

    fn bind_pressed(&self, action: usize) -> bool {
        key_from_code(self.meta.binds.codes[action])
            .map(is_key_pressed)
            .unwrap_or(false)
    }

    /// Dificultad activa de la partida, leída del progreso guardado.
    fn difficulty(&self) -> Difficulty {
        Difficulty::from_index(self.meta.difficulty)
    }

    /// El cristal agrietado concede daño extra cuanto más roto está.
    fn crystal_bonus(&self) -> f32 {
        let frac = (self.player.hp / self.player.max_hp.max(1.0)).clamp(0.0, 1.0);
        (1.0 - frac) * 0.25
    }

    /// Límites de la sala actual. Cada sala puede ser pequeña, mediana o grande.
    fn arena(&self) -> (f32, f32, f32, f32) {
        self.rooms
            .get(self.room_index)
            .map(|r| r.size.bounds())
            .unwrap_or((ARENA_L, ARENA_R, ARENA_T, ARENA_B))
    }

    fn door_pos(&self, dir: usize) -> Vec2 {
        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        match dir {
            0 => vec2(cx, t),
            1 => vec2(r, cy),
            2 => vec2(cx, b),
            _ => vec2(l, cy),
        }
    }

    fn inward(dir: usize) -> Vec2 {
        match dir {
            0 => vec2(0.0, 1.0),
            1 => vec2(-1.0, 0.0),
            2 => vec2(0.0, -1.0),
            _ => vec2(1.0, 0.0),
        }
    }

    fn neighbors4(gx: i32, gy: i32) -> [(i32, i32); 4] {
        [(gx, gy - 1), (gx + 1, gy), (gx, gy + 1), (gx - 1, gy)]
    }

    fn generate_dungeon(&mut self) {
        self.rooms.clear();

        // Recorrido aleatorio sobre una rejilla de 5x5: las salas quedan
        // desordenadas, pero todas están conectadas como en una mazmorra.
        let start_cell = (1 + self.rng.usize(3) as i32, 1 + self.rng.usize(3) as i32);
        // Nueve salas como mínimo: entrada, guardián, tienda, descanso, tesoro,
        // bóveda y tres cámaras de combate con cofre.
        let target = (9 + (self.floor as usize / 2).min(4) + self.rng.usize(2)).min(15);
        let mut cells: Vec<(i32, i32)> = vec![start_cell];
        while cells.len() < target {
            let mut frontier: Vec<(i32, i32)> = Vec::new();
            for &(gx, gy) in &cells {
                for (nx, ny) in Self::neighbors4(gx, gy) {
                    if nx >= 0
                        && ny >= 0
                        && nx < GRID
                        && ny < GRID
                        && !cells.contains(&(nx, ny))
                        && !frontier.contains(&(nx, ny))
                    {
                        frontier.push((nx, ny));
                    }
                }
            }
            if frontier.is_empty() {
                break;
            }
            // Preferimos celdas con más vecinos: así salen bloques compactos
            // y no un pasillo interminable.
            let mut best = frontier[0];
            let mut best_n = -1;
            for &c in &frontier {
                let n = Self::neighbors4(c.0, c.1)
                    .iter()
                    .filter(|&&(nx, ny)| cells.contains(&(nx, ny)))
                    .count() as i32;
                if n > best_n {
                    best_n = n;
                    best = c;
                }
            }
            let pick = if self.rng.chance(0.72) {
                best
            } else {
                frontier[self.rng.usize(frontier.len())]
            };
            cells.push(pick);
        }

        for &(gx, gy) in &cells {
            let dist = (gx - start_cell.0).abs() + (gy - start_cell.1).abs();
            let elevation = if dist == 0 { 0 } else { ((dist % 4) as i8) - 1 };
            self.rooms.push(Room {
                gx,
                gy,
                kind: RoomKind::Combat,
                size: match self.rng.usize(14) {
                    0..=1 => RoomSize::Tiny,
                    2..=5 => RoomSize::Small,
                    6..=9 => RoomSize::Medium,
                    10..=12 => RoomSize::Large,
                    _ => RoomSize::Giant,
                },
                visited: false,
                cleared: false,
                used: false,
                pact_used: false,
                has_chest: false,
                chest_opened: false,
                doors: [false; 4],
                locked: [false; 4],
                map_anim: 1.0,
                elevation,
                vault_open: false,
                decor_seed: self.rng.next_u32(),
            });
        }
        for i in 0..self.rooms.len() {
            let (gx, gy) = (self.rooms[i].gx, self.rooms[i].gy);
            if let Some(j) = self.room_at(gx + 1, gy) {
                self.rooms[i].doors[1] = true;
                self.rooms[j].doors[3] = true;
            }
            if let Some(j) = self.room_at(gx, gy + 1) {
                self.rooms[i].doors[2] = true;
                self.rooms[j].doors[0] = true;
            }
        }

        let start = self.room_at(start_cell.0, start_cell.1).unwrap();
        self.rooms[start].kind = RoomKind::Start;
        self.rooms[start].size = RoomSize::Medium;

        // El guardián va lo más lejos posible de la entrada.
        let mut boss = start;
        let mut best_d = -1;
        for (i, room) in self.rooms.iter().enumerate() {
            if i == start {
                continue;
            }
            let d = (room.gx - start_cell.0).abs() + (room.gy - start_cell.1).abs();
            if d > best_d {
                best_d = d;
                boss = i;
            }
        }
        self.rooms[boss].kind = RoomKind::Boss;
        self.rooms[boss].size = RoomSize::Giant;

        // Tienda, descanso y tesoro en salas sin salidas alternativas si se puede.
        let mut candidates: Vec<usize> = (0..self.rooms.len())
            .filter(|&i| i != start && i != boss)
            .collect();
        for i in (1..candidates.len()).rev() {
            let j = self.rng.usize(i + 1);
            candidates.swap(i, j);
        }
        candidates.sort_by_key(|&i| {
            let doors = self.rooms[i].doors.iter().filter(|&&d| d).count();
            (doors != 1) as u8
        });
        let shop = candidates[0];
        let rest = candidates[1];
        let treasure = candidates[2];
        let vault = candidates[3];
        self.rooms[shop].kind = RoomKind::Shop;
        self.rooms[shop].size = RoomSize::Large;
        self.rooms[rest].kind = RoomKind::Rest;
        self.rooms[rest].size = RoomSize::Small;
        self.rooms[treasure].kind = RoomKind::Treasure;
        self.rooms[treasure].size = RoomSize::Large;
        self.rooms[vault].kind = RoomKind::Vault;
        self.rooms[vault].size = RoomSize::Medium;
        self.rooms[vault].elevation = -1;
        self.rooms[boss].elevation = 2;
        self.rooms[treasure].elevation = 1;

        // Los cofres guardan las llaves. Tres cofres por piso: una para el
        // guardián, otra para el tesoro y una de reserva.
        let mut chest_rooms: Vec<usize> = candidates
            .iter()
            .copied()
            .skip(4)
            .filter(|&i| self.rooms[i].kind == RoomKind::Combat)
            .collect();
        for i in (1..chest_rooms.len()).rev() {
            let j = self.rng.usize(i + 1);
            chest_rooms.swap(i, j);
        }
        for &i in chest_rooms.iter().take(3) {
            self.rooms[i].has_chest = true;
        }

        for d in 0..4 {
            self.rooms[boss].locked[d] = self.rooms[boss].doors[d];
            self.rooms[treasure].locked[d] = self.rooms[treasure].doors[d];
            self.rooms[vault].locked[d] = self.rooms[vault].doors[d];
        }
        for room in &mut self.rooms {
            if matches!(
                room.kind,
                RoomKind::Start
                    | RoomKind::Shop
                    | RoomKind::Rest
                    | RoomKind::Treasure
                    | RoomKind::Vault
            ) {
                room.cleared = true;
            }
        }
        self.rooms[start].visited = true;
        self.rooms[start].cleared = true;
        self.rooms[start].map_anim = 1.0;
        self.room_index = start;
        self.keys = 0;
        self.map_open = false;
        self.map_t = 0.0;
        self.floor_element = Element::ALL[self.rng.usize(Element::ALL.len())];
        self.obstacles.clear();
    }

    fn generate_shop_offers(&mut self) {
        self.shop_offers.clear();
        let full = self.player.hp >= self.player.max_hp - 0.01;
        if full {
            let value = 8.0 + self.floor as f32 * 1.5;
            self.shop_offers.push(ShopOffer {
                name: "Sello de piedra".to_owned(),
                desc: format!("+{:.0} de vida máxima.", value),
                cost: 60 + self.floor * 4,
                kind: ShopKind::Item(Item {
                    name: "Sello de piedra",
                    kind: ItemKind::MaxHp,
                    rarity: Rarity::Rare,
                    value,
                }),
                sold: false,
            });
        } else {
            self.shop_offers.push(ShopOffer {
                name: "Poción de vida".to_owned(),
                desc: "Recupera el 45% de tu vida.".to_owned(),
                cost: 35 + self.floor * 3,
                kind: ShopKind::Heal,
                sold: false,
            });
        }
        if self.floor >= 3 && self.rng.chance(0.12) {
            self.shop_offers.push(ShopOffer {
                name: "Llave negra".to_owned(),
                desc: "Abre una bóveda sellada. Muy codiciada.".to_owned(),
                cost: 320 + self.floor * 22,
                kind: ShopKind::SpecialKey,
                sold: false,
            });
        } else {
            self.shop_offers.push(ShopOffer {
                name: "Llave de hierro".to_owned(),
                desc: "Abre una puerta sellada de este piso.".to_owned(),
                cost: 60 + self.floor * 3,
                kind: ShopKind::Key,
                sold: false,
            });
        }
        let item = self.roll_item(None);
        self.shop_offers.push(ShopOffer {
            name: item.name.to_owned(),
            desc: item_short_desc(&item),
            cost: 70 + self.floor * 5,
            kind: ShopKind::Item(item),
            sold: false,
        });
        let cards = buffs::pick_cards(
            &mut self.rng,
            self.class,
            &self.buff_stacks,
            self.player.level,
        );
        let buff = cards[0];
        self.shop_offers.push(ShopOffer {
            name: buff.name.to_owned(),
            desc: buff.desc.to_owned(),
            cost: 100 + self.floor * 6,
            kind: ShopKind::Buff(buff),
            sold: false,
        });
    }

    pub fn enter_room(&mut self, idx: usize, from_dir: usize) {
        if idx >= self.rooms.len() {
            return;
        }
        let first_visit = !self.rooms[idx].visited;
        self.room_index = idx;
        self.rooms[idx].visited = true;
        if first_visit {
            self.rooms[idx].map_anim = 0.0;
        }
        let (l, r, t, b) = self.rooms[idx].size.bounds();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        self.player.pos = match from_dir {
            0 => vec2(cx, b - 26.0),
            1 => vec2(l + 26.0, cy),
            2 => vec2(cx, t + 26.0),
            3 => vec2(r - 26.0, cy),
            _ => vec2(cx, b - 32.0),
        };
        self.player.invuln = self.player.invuln.max(0.6);
        self.enemies.clear();
        self.projectiles.clear();
        self.props.clear();
        self.pickups.clear();
        self.obstacles.clear();
        self.wave_timer = 0.0;
        self.portal_t = 0.0;
        self.room_fade = 0.34;
        self.map_open = false;
        self.merchant_t = 0.0;
        for r in &mut self.rings {
            r.active = false;
        }
        for f in &mut self.floaters {
            f.active = false;
        }

        let kind = self.rooms[idx].kind;
        self.waves = match kind {
            RoomKind::Combat => {
                let base = if self.floor >= 6 {
                    3
                } else if self.floor >= 2 {
                    2
                } else {
                    1
                };
                base + self.difficulty().extra_wave()
            }
            RoomKind::Boss => 1,
            _ => 0,
        };
        self.cleared = self.rooms[idx].cleared;
        self.wave = if self.cleared { self.waves } else { 0 };

        for s in &mut self.skeletons {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            s.pos = self.player.pos + vec2(a.cos(), a.sin()) * 20.0;
        }

        if kind == RoomKind::Boss {
            self.portal = vec2(cx, cy - 8.0);
        }

        match kind {
            RoomKind::Rest => {
                self.spawn_rest_obstacles();
                if !self.rooms[idx].used {
                    let a = self.rng.usize(Pact::ALL.len());
                    let b = (a + 1 + self.rng.usize(Pact::ALL.len() - 1)) % Pact::ALL.len();
                    self.rest_offers = [Pact::ALL[a], Pact::ALL[b]];
                    self.rooms[idx].used = true;
                    self.rooms[idx].cleared = true;
                    self.cleared = true;
                    let heal = self.player.max_hp * 0.45;
                    self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                    spawn_ring(
                        &mut self.rings,
                        self.player.pos,
                        8.0,
                        46.0,
                        0.55,
                        C_GREEN,
                        2.0,
                    );
                    burst(&mut self.particles, self.player.pos, 14, C_GREEN, 55.0);
                    self.toast("Has descansado: recuperas vida", C_GREEN);
                } else {
                    self.toast("Ya has descansado en esta sala", C_CYAN);
                }
            }
            RoomKind::Shop => {
                if !self.rooms[idx].used {
                    self.generate_shop_offers();
                    self.rooms[idx].used = true;
                }
                self.toast("Mercader: pulsa 1, 2, 3 o 4 para comprar", C_GOLD);
            }
            RoomKind::Treasure => {
                if !self.rooms[idx].used {
                    self.spawn_chest(vec2(cx, cy - 4.0));
                }
            }
            RoomKind::Vault => {
                if !self.rooms[idx].used {
                    self.rooms[idx].used = true;
                    let item = self.roll_item(Some(Rarity::Legendary));
                    self.spawn_pickup(vec2(cx - 10.0, cy), PickupKind::Item(item));
                    let gold = ((25 + self.floor * 5) as f32
                        * self.player.gold_mult
                        * if self.modifier == Modifier::Greed {
                            1.5
                        } else {
                            1.0
                        }) as u32;
                    self.spawn_pickup(vec2(cx + 12.0, cy + 6.0), PickupKind::Gold(gold.max(1)));
                    self.spawn_pickup(
                        vec2(cx + 2.0, cy + 16.0),
                        PickupKind::Heal(18.0 + self.floor as f32 * 2.0),
                    );
                    if self.rng.chance(0.10) {
                        self.special_keys += 1;
                        self.toast("La bóveda guardaba otra llave negra", C_PURPLE);
                    }
                    burst(&mut self.particles, vec2(cx, cy), 24, C_PURPLE, 100.0);
                    spawn_ring(&mut self.rings, vec2(cx, cy), 8.0, 70.0, 0.7, C_PURPLE, 2.4);
                }
            }
            RoomKind::Combat => {
                if self.rooms[idx].has_chest && !self.rooms[idx].chest_opened {
                    let cpos = vec2(
                        self.rng.range(l + 34.0, r - 34.0),
                        self.rng.range(t + 34.0, b - 34.0),
                    );
                    self.spawn_chest(cpos);
                }
                if !self.cleared {
                    self.spawn_room_props();
                    self.spawn_obstacles();
                    self.spawn_wave();
                }
            }
            RoomKind::Boss => {
                if !self.cleared {
                    self.spawn_obstacles();
                    self.spawn_wave();
                }
            }
            RoomKind::Start => {}
        }

        if self.cleared {
            self.room_voice = None;
        }

        if first_visit {
            let name = if kind == RoomKind::Vault && self.rooms[idx].vault_open {
                "Bóveda abierta"
            } else {
                kind.name()
            };
            self.toast(name, kind.color());
            if kind == RoomKind::Boss {
                self.toast("El guardián despierta", C_RED);
            }
        }
    }

    fn try_use_door(&mut self, dir: usize, sfx: &mut Sfx) -> bool {
        let (gx, gy, has_door) = match self.rooms.get(self.room_index) {
            Some(r) => (r.gx, r.gy, r.doors[dir]),
            None => return false,
        };
        if !has_door {
            return false;
        }
        let dpos = self.door_pos(dir);
        if self.player.pos.distance(dpos) > 15.0 {
            return false;
        }
        if !self.cleared {
            if self.door_msg_t <= 0.0 {
                self.toast("Derrota a los enemigos para abrir las puertas", C_RED);
                self.door_msg_t = 1.6;
                sfx.play(SfxKind::Lock, 0.32);
            }
            self.player.pos += Self::inward(dir) * 12.0;
            return false;
        }
        let (ngx, ngy) = match dir {
            0 => (gx, gy - 1),
            1 => (gx + 1, gy),
            2 => (gx, gy + 1),
            _ => (gx - 1, gy),
        };
        let target = match self.room_at(ngx, ngy) {
            Some(t) => t,
            None => return false,
        };
        if self.rooms[target].kind == RoomKind::Vault && !self.rooms[target].vault_open {
            if self.special_keys > 0 {
                self.special_keys -= 1;
                self.rooms[target].vault_open = true;
                for d in 0..4 {
                    self.rooms[target].locked[d] = false;
                }
                self.toast("La llave negra abre la bóveda", C_PURPLE);
                sfx.play(SfxKind::Portal, 0.40);
                self.shake = self.shake.max(2.0);
            } else {
                if self.door_msg_t <= 0.0 {
                    self.toast("Bóveda sellada: necesitas una llave negra", C_PURPLE);
                    self.door_msg_t = 1.6;
                    sfx.play(SfxKind::Lock, 0.34);
                }
                self.player.pos += Self::inward(dir) * 12.0;
                return false;
            }
        }
        if self.rooms[target].locked.iter().any(|&l| l) {
            if self.keys > 0 {
                self.keys -= 1;
                for d in 0..4 {
                    self.rooms[target].locked[d] = false;
                }
                let target_kind = self.rooms[target].kind;
                let msg = if target_kind == RoomKind::Boss {
                    "La puerta del guardián se abre con tu llave"
                } else {
                    "La cámara del tesoro se abre con tu llave"
                };
                self.toast(msg, C_GOLD);
                sfx.play(SfxKind::Portal, 0.34);
                self.shake = self.shake.max(1.2);
            } else {
                if self.door_msg_t <= 0.0 {
                    let msg = if self.rooms[target].kind == RoomKind::Boss {
                        "Sello del guardián: consigue una llave en un cofre"
                    } else {
                        "Necesitas una llave para abrir esta puerta"
                    };
                    self.toast(msg, C_RED);
                    self.door_msg_t = 1.6;
                    sfx.play(SfxKind::Lock, 0.32);
                }
                self.player.pos += Self::inward(dir) * 12.0;
                return false;
            }
        }
        self.enter_room(target, dir);
        true
    }

    fn spawn_room_props(&mut self) {
        let (l, r, t, b) = self.arena();
        let mut count = (2 + self.floor / 3) as usize;
        if count > 6 {
            count = 6;
        }
        if self.modifier == Modifier::Greed {
            count += 1;
        }
        for _ in 0..count {
            let mut pos = vec2((l + r) * 0.5, (t + b) * 0.5);
            for _ in 0..30 {
                let cand = vec2(
                    self.rng.range(l + 18.0, r - 18.0),
                    self.rng.range(t + 18.0, b - 18.0),
                );
                if cand.distance(self.player.pos) > 44.0
                    && cand.distance(vec2((l + r) * 0.5, (t + b) * 0.5)) > 34.0
                    && self.props.iter().all(|p| p.pos.distance(cand) > 24.0)
                    && self
                        .obstacles
                        .iter()
                        .all(|o| o.pos.distance(cand) > o.radius + 12.0)
                {
                    pos = cand;
                    break;
                }
            }
            let kind = match self.rng.usize(5) {
                0 => PropKind::Barrel,
                1 | 2 => PropKind::Crate,
                3 => PropKind::Urn,
                _ => PropKind::Statue,
            };
            let hp = if kind == PropKind::Statue { 2.0 } else { 1.0 };
            self.props.push(Prop {
                pos,
                kind,
                hp,
                hit_flash: 0.0,
                dead: false,
                bob: self.rng.range(0.0, 6.28),
                opened: false,
                open_t: 0.0,
            });
        }
    }

    fn spawn_chest(&mut self, pos: Vec2) {
        self.props.push(Prop {
            pos,
            kind: PropKind::Chest,
            hp: 9999.0,
            hit_flash: 0.0,
            dead: false,
            bob: 0.0,
            opened: false,
            open_t: 0.0,
        });
    }

    /// Pilares, rocas, huesos, braseros y pinchos. Las salas grandes pueden
    /// generar una ruina laberíntica con pasillos y huecos.
    fn spawn_obstacles(&mut self) {
        let (l, r, t, b) = self.arena();
        let boss_room = self.current_kind() == RoomKind::Boss;
        let size = self.rooms[self.room_index].size;
        let (count, max_r) = match size {
            RoomSize::Tiny => (self.rng.usize(2), 3.6),
            RoomSize::Small => (1 + self.rng.usize(3), 4.5),
            RoomSize::Medium => (3 + self.rng.usize(3), 5.5),
            RoomSize::Large => (5 + self.rng.usize(4), 6.0),
            RoomSize::Giant => (7 + self.rng.usize(4), 6.5),
        };
        let count = if boss_room { 2 } else { count };
        for _ in 0..count {
            let mut pos = vec2((l + r) * 0.5, (t + b) * 0.5);
            let mut radius = 6.0;
            for _ in 0..40 {
                let cand = vec2(
                    self.rng.range(l + 24.0, r - 24.0),
                    self.rng.range(t + 24.0, b - 24.0),
                );
                if cand.distance(self.player.pos) < 58.0 {
                    continue;
                }
                if (0..4).any(|d| cand.distance(self.door_pos(d)) < 38.0) {
                    continue;
                }
                if self.props.iter().any(|p| p.pos.distance(cand) < 24.0) {
                    continue;
                }
                if self
                    .obstacles
                    .iter()
                    .any(|o| o.pos.distance(cand) < o.radius + 16.0)
                {
                    continue;
                }
                pos = cand;
                radius = self.rng.range(max_r - 1.5, max_r);
                break;
            }
            let kind = if boss_room {
                ObstacleKind::Brazier
            } else {
                match self.rng.usize(4) {
                    0 => ObstacleKind::Pillar,
                    1 => ObstacleKind::Rock,
                    2 => ObstacleKind::Bones,
                    _ => ObstacleKind::Brazier,
                }
            };
            self.obstacles.push(Obstacle {
                pos,
                radius,
                kind,
                bob: self.rng.range(0.0, 6.28),
            });
        }

        // Ruinas con pasillos: solo en salas grandes y nunca en la del jefe.
        if size == RoomSize::Large && !boss_room && self.rng.chance(0.30) {
            let cx = (l + r) * 0.5;
            let cy = (t + b) * 0.5;
            let gap = self.rng.usize(4);
            for row in 0..2 {
                let y = cy - 26.0 + row as f32 * 52.0;
                for k in 0..5 {
                    if k == gap {
                        continue;
                    }
                    let x = cx - 52.0 + k as f32 * 26.0;
                    let p = vec2(x, y);
                    if (0..4).any(|d| p.distance(self.door_pos(d)) < 42.0)
                        || p.distance(self.player.pos) < 54.0
                    {
                        continue;
                    }
                    self.obstacles.push(Obstacle {
                        pos: p,
                        radius: 5.0,
                        kind: ObstacleKind::Pillar,
                        bob: self.rng.range(0.0, 6.28),
                    });
                }
            }
        }

        // Pinchos: suelo que duele, pero deja pasar.
        if !boss_room {
            let patches = match size {
                RoomSize::Tiny => 0,
                RoomSize::Small => self.rng.usize(2),
                RoomSize::Medium => 1 + self.rng.usize(2),
                RoomSize::Large => 2 + self.rng.usize(2),
                RoomSize::Giant => 3 + self.rng.usize(2),
            };
            for _ in 0..patches {
                for _ in 0..30 {
                    let cand = vec2(
                        self.rng.range(l + 30.0, r - 30.0),
                        self.rng.range(t + 30.0, b - 30.0),
                    );
                    if cand.distance(self.player.pos) < 56.0
                        || (0..4).any(|d| cand.distance(self.door_pos(d)) < 44.0)
                        || self
                            .obstacles
                            .iter()
                            .any(|o| o.pos.distance(cand) < o.radius + 14.0)
                    {
                        continue;
                    }
                    self.obstacles.push(Obstacle {
                        pos: cand,
                        radius: self.rng.range(9.0, 14.0),
                        kind: ObstacleKind::Spikes,
                        bob: self.rng.range(0.0, 6.28),
                    });
                    break;
                }
            }
        }
    }

    /// La sala de descanso es más pequeña e íntima: dos braseros junto a la
    /// cama y unos huesos secos en las esquinas.
    fn spawn_rest_obstacles(&mut self) {
        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        for (dx, kind) in [
            (-52.0f32, ObstacleKind::Brazier),
            (52.0, ObstacleKind::Brazier),
        ] {
            self.obstacles.push(Obstacle {
                pos: vec2(cx + dx, cy + 24.0),
                radius: 4.5,
                kind,
                bob: self.rng.range(0.0, 6.28),
            });
        }
        for (dx, dy) in [
            (-66.0f32, -42.0f32),
            (66.0, -42.0),
            (-66.0, 52.0),
            (66.0, 52.0),
        ] {
            self.obstacles.push(Obstacle {
                pos: vec2(cx + dx, cy + dy),
                radius: 5.0,
                kind: ObstacleKind::Bones,
                bob: self.rng.range(0.0, 6.28),
            });
        }
    }

    /// Abre el cofre más cercano. Los cofres de las salas de combate dan una
    /// llave; el de la cámara del tesoro da un objeto y oro.
    fn try_open_chest(&mut self, sfx: &mut Sfx) -> bool {
        let mut best = 36.0 * 36.0;
        let mut idx = None;
        for (i, p) in self.props.iter().enumerate() {
            if p.kind != PropKind::Chest || p.opened {
                continue;
            }
            let d = p.pos.distance_squared(self.player.pos);
            if d < best {
                best = d;
                idx = Some(i);
            }
        }
        let i = match idx {
            Some(i) => i,
            None => return false,
        };
        self.props[i].opened = true;
        self.props[i].open_t = 0.0;
        let pos = self.props[i].pos;
        burst(&mut self.particles, pos, 18, C_GOLD, 85.0);
        spawn_ring(&mut self.rings, pos, 6.0, 38.0, 0.5, C_GOLD, 2.0);
        self.shake = self.shake.max(1.4);
        sfx.play(SfxKind::Portal, 0.30);

        if self.current_kind() == RoomKind::Treasure {
            if let Some(room) = self.rooms.get_mut(self.room_index) {
                room.used = true;
            }
            let item = self.roll_item(Some(if self.floor >= 5 {
                Rarity::Epic
            } else {
                Rarity::Rare
            }));
            self.spawn_pickup(pos + vec2(0.0, 6.0), PickupKind::Item(item));
            let gold = ((5 + self.floor) as f32
                * self.player.gold_mult
                * if self.modifier == Modifier::Greed {
                    1.5
                } else {
                    1.0
                }) as u32;
            self.spawn_pickup(pos + vec2(-8.0, 4.0), PickupKind::Gold(gold.max(1)));
            self.spawn_pickup(
                pos + vec2(8.0, 4.0),
                PickupKind::Heal(10.0 + self.floor as f32 * 2.0),
            );
            if self.rng.chance(0.15) {
                self.special_keys += 1;
                self.toast("Entre el tesoro había una llave negra", C_PURPLE);
            }
            self.toast("La cámara del tesoro te recompensa", C_PURPLE);
        } else {
            self.keys += 1;
            if self.rng.chance(0.05) {
                self.special_keys += 1;
                self.toast("El cofre también guardaba una llave negra", C_PURPLE);
            }
            if let Some(room) = self.rooms.get_mut(self.room_index) {
                room.chest_opened = true;
            }
            let gold = ((1 + self.floor / 3) as f32 * self.player.gold_mult) as u32;
            self.spawn_pickup(pos + vec2(0.0, 4.0), PickupKind::Gold(gold.max(1)));
            self.toast("Has encontrado una llave oxidada", C_GOLD);
        }
        true
    }

    fn pact_pos(&self, i: usize) -> Vec2 {
        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        vec2(cx + if i == 0 { -76.0 } else { 76.0 }, cy + 26.0)
    }

    fn debt_pos(&self) -> Vec2 {
        let (l, r, t, b) = self.arena();
        vec2((l + r) * 0.5 + 64.0, (t + b) * 0.5 + 20.0)
    }

    fn echo_pos(&self) -> Vec2 {
        let (l, r, t, b) = self.arena();
        vec2((l + r) * 0.5 - 64.0, (t + b) * 0.5 + 20.0)
    }

    /// Acepta uno de los dos pactos de la sala de descanso.
    fn try_take_pact(&mut self, sfx: &mut Sfx) -> bool {
        if self.current_kind() != RoomKind::Rest {
            return false;
        }
        if self
            .rooms
            .get(self.room_index)
            .map(|r| r.pact_used)
            .unwrap_or(true)
        {
            return false;
        }
        let mut idx = None;
        for i in 0..2 {
            if self.player.pos.distance(self.pact_pos(i)) < 30.0 {
                idx = Some(i);
                break;
            }
        }
        let i = match idx {
            Some(i) => i,
            None => return false,
        };
        let pact = self.rest_offers[i];
        self.apply_pact(pact);
        if let Some(room) = self.rooms.get_mut(self.room_index) {
            room.pact_used = true;
        }
        self.pacts_taken += 1;
        self.taken_pacts.push(pact);
        self.toast(&format!("{}: {}", pact.name(), pact.desc()), pact.color());
        let pos = self.pact_pos(i);
        burst(&mut self.particles, pos, 20, pact.color(), 90.0);
        spawn_ring(&mut self.rings, pos, 8.0, 52.0, 0.6, pact.color(), 2.2);
        self.shake = self.shake.max(2.2);
        sfx.play(SfxKind::Boss, 0.30);
        true
    }

    /// El mercader aparece en la entrada de la siguiente partida si quedó
    /// deuda. Cobra todo el oro posible y, si no alcanza, se lleva vida.
    fn try_pay_debt(&mut self, sfx: &mut Sfx) -> bool {
        if self.current_kind() != RoomKind::Start || self.debt == 0 {
            return false;
        }
        if self.player.pos.distance(self.debt_pos()) > 30.0 {
            return false;
        }
        let paid = self.player.gold.min(self.debt);
        self.player.gold -= paid;
        self.debt -= paid;
        if self.debt == 0 {
            self.meta.debt = 0;
            self.toast("Deuda saldada. El mercader sonríe", C_GOLD);
        } else {
            let interest = (self.player.max_hp * 0.18)
                .min(self.player.hp - 1.0)
                .max(0.0);
            self.player.hp -= interest;
            self.meta.debt = self.debt;
            self.toast(
                &format!(
                    "Aún debes {}. El mercader se lleva parte de tu cristal",
                    self.debt
                ),
                C_RED,
            );
        }
        let dpos = self.debt_pos();
        burst(&mut self.particles, dpos, 16, C_GOLD, 80.0);
        spawn_ring(&mut self.rings, dpos, 6.0, 40.0, 0.5, C_GOLD, 2.0);
        sfx.play(SfxKind::Pickup, 0.30);
        true
    }

    /// El eco de una partida anterior concede una bendición menor una vez.
    fn try_claim_echo(&mut self, sfx: &mut Sfx) -> bool {
        if self.current_kind() != RoomKind::Start || self.meta.echoes == 0 || self.echo_claimed {
            return false;
        }
        if self.player.pos.distance(self.echo_pos()) > 30.0 {
            return false;
        }
        self.echo_claimed = true;
        match self.meta.echo_class {
            0 => self.player.armor += 2.0,
            1 => self.player.crit = (self.player.crit + 0.05).min(0.85),
            2 => self.player.aoe += 4.0,
            3 => self.player.speed += 5.0,
            _ => {
                self.player.skeleton_cap += 1;
                self.summon_all();
            }
        }
        let pos = self.echo_pos();
        burst(&mut self.particles, pos, 22, C_CYAN, 90.0);
        spawn_ring(&mut self.rings, pos, 8.0, 48.0, 0.6, C_CYAN, 2.2);
        self.toast(
            &format!("El eco del piso {} te bendice", self.meta.echo_floor.max(1)),
            C_CYAN,
        );
        sfx.play(SfxKind::Level, 0.28);
        true
    }

    fn apply_pact(&mut self, pact: Pact) {
        match pact {
            Pact::Iron => {
                self.player.armor += 5.0;
                self.player.speed *= 0.92;
            }
            Pact::Fire => {
                self.player.damage *= 1.30;
                self.player.max_hp *= 0.85;
                self.player.hp = self.player.hp.min(self.player.max_hp);
            }
            Pact::Greed => {
                self.player.gold_mult *= 1.45;
                self.player.max_hp *= 0.90;
                self.player.hp = self.player.hp.min(self.player.max_hp);
            }
            Pact::Blood => {
                self.player.lifesteal += 0.15;
                self.player.max_hp *= 0.82;
                self.player.hp = self.player.hp.min(self.player.max_hp);
            }
            Pact::Time => {
                self.player.attack_speed *= 1.18;
                self.player.speed *= 0.90;
            }
        }
        self.refresh_skeleton_damage();
    }

    fn roll_modifier(&mut self) -> Modifier {
        if self.floor < 2 {
            return Modifier::None;
        }
        if self.rng.chance(0.45) {
            match self.rng.usize(3) {
                0 => Modifier::Fog,
                1 => Modifier::Darkness,
                _ => Modifier::Greed,
            }
        } else {
            Modifier::None
        }
    }

    fn modifier_name(&self) -> &'static str {
        match self.modifier {
            Modifier::None => "",
            Modifier::Fog => "Niebla",
            Modifier::Darkness => "Oscuridad",
            Modifier::Greed => "Codicia",
        }
    }

    fn modifier_color(&self) -> Color {
        match self.modifier {
            Modifier::None => C_WHITE,
            Modifier::Fog => Color::from_rgba(160, 190, 210, 255),
            Modifier::Darkness => Color::from_rgba(170, 110, 210, 255),
            Modifier::Greed => C_GOLD,
        }
    }

    fn spawn_wave(&mut self) {
        self.wave += 1;
        self.wave_timer = 0.0;
        let boss = self.current_kind() == RoomKind::Boss;
        let mut count = (4 + self.floor / 2) as usize;
        if count > 9 {
            count = 9;
        }
        if self.modifier == Modifier::Greed {
            count += 1;
        }
        count += self.difficulty().extra_spawns();
        if boss {
            self.spawn_enemy(EnemyKind::Boss, false, false);
            if let Some(e) = self.enemies.iter().find(|e| e.boss) {
                self.boss_intro_name = e.name.clone();
                self.boss_intro_desc = boss_intro_desc(e.attack, e.element);
                self.boss_intro_color = e.tint;
                self.boss_intro_skin = e.boss_skin.min(11);
                self.boss_intro_t = 3.2;
                self.shake = self.shake.max(6.0);
            }
            count = (3 + self.floor / 6).min(6) as usize;
        }
        for i in 0..count {
            let kind = self.pick_enemy_kind();
            let extra_elite = !boss
                && self.floor > 1
                && i == 0
                && self.rng.chance(self.difficulty().extra_elite_chance());
            let elite = (!boss && self.floor % 4 == 0 && self.wave == 1 && i == 0) || extra_elite;
            let shiny = !elite && !boss && self.rng.chance(0.018);
            self.spawn_enemy(kind, elite, shiny);
        }
        // El oro pesado atrae espíritus de codicia.
        if !boss && self.gold_weight() > 0.40 && self.rng.chance(0.30 + self.gold_weight() * 0.45) {
            self.spawn_enemy(EnemyKind::Wraith, false, false);
            if let Some(e) = self.enemies.last_mut() {
                e.name = "Espíritu de codicia".to_owned();
                e.tint = C_GOLD;
                e.hp *= 1.5;
                e.max_hp = e.hp;
                e.speed *= 1.15;
            }
        }
        self.room_voice = if boss {
            Some(EnemyVoice::Roar)
        } else {
            self.enemies.iter().find(|e| !e.dead).map(|e| e.voice)
        };
        if self.wave > 1 {
            self.toast(&format!("Oleada {}", self.wave), C_GOLD);
        }
    }

    fn pick_enemy_kind(&mut self) -> EnemyKind {
        let pool: &[EnemyKind] = if self.floor <= 1 {
            &[EnemyKind::Slime, EnemyKind::Bat]
        } else if self.floor == 2 {
            &[EnemyKind::Slime, EnemyKind::Bat, EnemyKind::Brute]
        } else if self.floor <= 4 {
            &[
                EnemyKind::Slime,
                EnemyKind::Bat,
                EnemyKind::Brute,
                EnemyKind::Cultist,
                EnemyKind::Bomber,
            ]
        } else if self.floor <= 6 {
            &[
                EnemyKind::Slime,
                EnemyKind::Bat,
                EnemyKind::Brute,
                EnemyKind::Cultist,
                EnemyKind::Bomber,
                EnemyKind::Wraith,
                EnemyKind::Sentinel,
            ]
        } else {
            &EnemyKind::NORMAL
        };
        pool[self.rng.usize(pool.len())]
    }

    fn base_stats(kind: EnemyKind) -> (f32, f32, f32, f32, f32, f32) {
        match kind {
            EnemyKind::Slime => (26.0, 5.5, 28.0, 7.0, 17.0, 1.3),
            EnemyKind::Bat => (15.0, 4.4, 61.0, 6.0, 15.0, 0.9),
            EnemyKind::Brute => (60.0, 10.0, 20.0, 12.0, 21.0, 1.7),
            EnemyKind::Cultist => (22.0, 5.5, 24.0, 7.0, 145.0, 1.9),
            EnemyKind::Wraith => (24.0, 6.6, 32.0, 8.0, 155.0, 2.2),
            EnemyKind::Bomber => (19.0, 7.7, 27.0, 7.0, 115.0, 2.4),
            EnemyKind::Sentinel => (42.0, 7.7, 23.0, 9.0, 115.0, 2.1),
            EnemyKind::Eye => (20.0, 5.5, 25.0, 8.0, 165.0, 2.0),
            EnemyKind::Boss => (360.0, 18.0, 27.0, 24.0, 30.0, 1.35),
        }
    }

    fn spawn_enemy(&mut self, kind: EnemyKind, elite: bool, shiny: bool) {
        let (l, r, t, b) = self.arena();
        let pos = if kind == EnemyKind::Boss {
            vec2((l + r) * 0.5, t + 54.0)
        } else {
            let mut p = vec2((l + r) * 0.5, (t + b) * 0.5);
            for _ in 0..40 {
                let cand = vec2(
                    self.rng.range(l + 14.0, r - 14.0),
                    self.rng.range(t + 14.0, b - 14.0),
                );
                if cand.distance(self.player.pos) > 72.0
                    && self
                        .obstacles
                        .iter()
                        .all(|o| o.pos.distance(cand) > o.radius + 10.0)
                {
                    p = cand;
                    break;
                }
            }
            p
        };
        self.spawn_enemy_at(kind, elite, shiny, pos);
    }

    /// La vida enemiga sube por rangos según el daño del jugador. No usa el
    /// número exacto: así una mejora puntual no dispara la dificultad, pero
    /// un piso medio tampoco se resuelve de un solo golpe.
    fn player_power_bracket(&self) -> f32 {
        let d = self.player.damage;
        match d {
            d if d < 14.0 => 1.00,
            d if d < 24.0 => 1.16,
            d if d < 38.0 => 1.34,
            d if d < 60.0 => 1.55,
            d if d < 90.0 => 1.80,
            d if d < 130.0 => 2.08,
            d if d < 185.0 => 2.40,
            d if d < 260.0 => 2.76,
            d if d < 360.0 => 3.16,
            d if d < 500.0 => 3.60,
            _ => 4.10,
        }
    }

    fn spawn_enemy_at(&mut self, kind: EnemyKind, elite: bool, shiny: bool, pos: Vec2) {
        let (mut hp, mut dmg, mut speed, mut radius, _range, mut cd) = Self::base_stats(kind);
        let boss = kind == EnemyKind::Boss;
        let diff = self.difficulty();
        let hp_scale = 1.22f32.powf((self.floor - 1) as f32);
        let dmg_scale = 1.18f32.powf((self.floor - 1) as f32);
        hp *= hp_scale;
        dmg *= dmg_scale;
        // La dificultad elegida se aplica antes de los modificadores de piso.
        hp *= if boss {
            diff.boss_hp()
        } else {
            diff.enemy_hp()
        };
        dmg *= if boss {
            diff.boss_damage()
        } else {
            diff.enemy_damage()
        };
        speed *= diff.enemy_speed();
        cd *= diff.enemy_cooldown();
        // Escalado por rangos de daño del jugador. Los guardianes reciben solo
        // una parte para que sigan siendo peleas largas pero no eternas.
        let bracket = self.player_power_bracket();
        hp *= if boss {
            1.0 + (bracket - 1.0) * 0.65
        } else {
            bracket
        };
        if boss {
            cd *= 0.62;
            speed *= 1.10;
            // Los guardianes ganan vida y pegada extra con cada piso.
            hp *= 1.0 + self.floor as f32 * 0.012;
            dmg *= 1.0 + self.floor as f32 * 0.008;
        }
        if boss && self.floor % 10 == 0 {
            hp *= 1.8;
            dmg *= 1.35;
        }
        if elite {
            hp *= 4.0;
            dmg *= 1.55;
            radius *= 1.2;
            speed *= 0.95;
        }
        if shiny {
            hp *= 0.50;
            speed *= 1.6;
            radius *= 0.9;
        }

        // Cada guardián de la partida usa una piel distinta.
        let boss_skin = if boss {
            let available: Vec<usize> = (0..12).filter(|&i| !self.bosses_used[i]).collect();
            let pick = if available.is_empty() {
                self.bosses_used = [false; 12];
                self.rng.usize(12)
            } else {
                available[self.rng.usize(available.len())]
            };
            self.bosses_used[pick] = true;
            pick
        } else {
            0
        };

        // El piso tiene un elemento dominante; el resto de enemigos pueden
        // pertenecer a otros elementos para que nunca se repita la misma sala.
        let element = if boss || self.rng.chance(0.62) {
            self.floor_element
        } else {
            Element::ALL[self.rng.usize(Element::ALL.len())]
        };
        let base_name = if boss {
            EnemyKind::boss_titles()[boss_skin % 10]
        } else {
            kind.variants()[self.rng.usize(3)]
        };
        let name = format!("{} {}", base_name, element.adjective());
        let tint = if boss {
            element.color()
        } else {
            let c = element.color();
            Color::new(
                c.r + (1.0 - c.r) * 0.55,
                c.g + (1.0 - c.g) * 0.55,
                c.b + (1.0 - c.b) * 0.55,
                1.0,
            )
        };
        let attack = if boss {
            match (boss_skin + (self.floor / 2) as usize) % 6 {
                0 => EnemyAttack::Burst,
                1 => EnemyAttack::Summon,
                2 => EnemyAttack::Charge,
                3 => EnemyAttack::Orbit,
                4 => EnemyAttack::Spread,
                _ => EnemyAttack::Teleport,
            }
        } else if element == Element::Void {
            EnemyAttack::Teleport
        } else {
            kind.attack()
        };
        let voice = if boss {
            EnemyVoice::Roar
        } else {
            element.voice()
        };

        let life = if shiny { 14.0 } else { 9999.0 };
        self.enemies.push(Enemy {
            id: self.next_enemy_id,
            kind,
            pos,
            vel: Vec2::ZERO,
            hp,
            max_hp: hp,
            damage: dmg,
            speed,
            radius,
            attack_t: self.rng.range(0.3, 1.1),
            attack_cd: cd,
            hit_flash: 0.0,
            elite,
            shiny,
            boss,
            slow_t: 0.0,
            burn_t: 0.0,
            burn_dps: 0.0,
            special_t: 3.0,
            life,
            bob: self.rng.range(0.0, 6.28),
            dead: false,
            name,
            attack,
            element,
            voice,
            tint,
            state_t: 0.0,
            charge_dir: Vec2::ZERO,
            teleport_to: Vec2::ZERO,
            summoned: false,
            boss_skin,
            signature_t: (3.4 - self.floor as f32 * 0.04).max(1.6) + self.rng.range(0.0, 1.0),
            cast_t: 0.0,
            spawn_t: if boss { 1.1 } else { 0.45 },
            death_t: 0.0,
            walk_t: self.rng.range(0.0, 6.28),
            swing_t: 0.0,
            jump_wind: 0.0,
            jump_t: 0.0,
            jump_from: pos,
            jump_to: pos,
            jump_cd: if boss { 2.6 } else { 0.0 },
            stuck_t: 0.0,
            last_pos: pos,
        });
        self.next_enemy_id += 1;
    }

    fn summon_all(&mut self) {
        while self.skeletons.len() < self.player.skeleton_cap as usize {
            self.spawn_skeleton();
        }
    }

    /// Con muchos esqueletos cada uno pega menos: el daño total sigue subiendo,
    /// pero con rendimientos decrecientes. Evita que una legión borre la sala.
    fn skeleton_share(count: usize) -> f32 {
        let n = count.max(1) as f32;
        1.0 / (0.70 + 0.30 * n)
    }

    /// La duración también se reparte: una legión numerosa se deshace antes.
    fn skeleton_life(count: usize) -> f32 {
        let n = count.max(1) as f32;
        (14.0 / (1.0 + 0.055 * (n - 1.0))).max(7.0)
    }

    fn skeleton_mult(&self) -> f32 {
        self.player.skeleton_dmg * (1.0 + 0.35 * self.buff_stacks[BuffId::BoneDmg as usize] as f32)
            + self.player.skeleton_inherit
    }

    /// Daño individual cuando hay `count` esqueletos vivos. El total sube con
    /// cada invocación, pero repartido para que una legión no borre la sala.
    fn skeleton_damage_for(&self, count: usize) -> f32 {
        self.player.damage * self.skeleton_mult() * Self::skeleton_share(count)
    }

    fn spawn_skeleton(&mut self) {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        let pos = self.player.pos + vec2(a.cos(), a.sin()) * 20.0;
        let count = self.skeletons.len() + 1;
        self.skeletons.push(Skeleton {
            pos,
            vel: Vec2::ZERO,
            damage: self.skeleton_damage_for(count),
            life: Self::skeleton_life(count),
            attack_t: 0.0,
            bob: self.rng.range(0.0, 6.28),
        });
    }

    fn refresh_skeleton_damage(&mut self) {
        let dmg = self.skeleton_damage_for(self.skeletons.len());
        for s in &mut self.skeletons {
            s.damage = dmg;
        }
    }

    fn toast(&mut self, text: &str, color: Color) {
        if self.toasts.len() >= 4 {
            self.toasts.remove(0);
        }
        self.toasts.push(Toast {
            text: text.to_string(),
            color,
            t: 3.0,
        });
    }

    pub fn shake_vec(&self) -> Vec2 {
        if self.shake <= 0.05 {
            Vec2::ZERO
        } else {
            let t = self.time * 52.0;
            vec2(
                (t.sin() * 0.6 + (t * 1.7).sin() * 0.4) * self.shake,
                ((t * 1.3).cos() * 0.6 + (t * 2.1).cos() * 0.4) * self.shake,
            )
        }
    }

    pub fn mouse_virtual(&self) -> Vec2 {
        vec2(
            (self.view.mx - self.view.ox) / self.view.scale.max(0.001),
            (self.view.my - self.view.oy) / self.view.scale.max(0.001),
        )
    }

    /// Durante la presentación del guardián la cámara viaja del héroe al jefe
    /// y hace un empuje suave. Devuelve el punto al que mirar y el zoom.
    pub fn camera_focus(&self) -> Option<(Vec2, f32)> {
        if self.boss_intro_t <= 0.0 {
            return None;
        }
        let boss = self.enemies.iter().find(|e| e.boss && !e.dead)?;
        let total = 3.2;
        let progress = (1.0 - self.boss_intro_t.clamp(0.0, total) / total).clamp(0.0, 1.0);
        let smooth = |x: f32| {
            let x = x.clamp(0.0, 1.0);
            x * x * (3.0 - 2.0 * x)
        };
        // Entra hacia el jefe y vuelve al héroe antes de que acabe el cartel.
        let k = smooth(progress / 0.30) * (1.0 - smooth((progress - 0.65) / 0.35));
        Some((self.player.pos.lerp(boss.pos, k), 1.0 + 0.08 * k))
    }
}

impl Game {
    pub fn update(&mut self, dt: f32, sfx: &mut Sfx) {
        if self.dev_note_t > 0.0 {
            self.dev_note_t -= dt;
        }
        self.update_toasts(dt);
        self.room_fade = (self.room_fade - dt).max(0.0);
        self.floor_arrival_t = (self.floor_arrival_t - dt).max(0.0);
        self.door_msg_t = (self.door_msg_t - dt).max(0.0);
        self.map_t += dt;
        self.name_t += dt;
        for r in &mut self.rooms {
            if r.map_anim < 1.0 {
                r.map_anim = (r.map_anim + dt * 2.6).min(1.0);
            }
        }
        if is_key_pressed(KeyCode::F3) {
            self.show_debug = !self.show_debug;
        }
        if !self.editing_name && !self.rebinding && is_key_pressed(KeyCode::M) {
            let on = sfx.toggle_music();
            self.meta.music = on;
            self.music_on = on;
            save::save(&self.meta);
            self.toast(
                if on {
                    "Música activada"
                } else {
                    "Música desactivada"
                },
                C_CYAN,
            );
        }
        match self.screen {
            Screen::Menu => self.update_menu(dt, sfx),
            Screen::Playing => {
                if self.boss_intro_t > 0.0 {
                    // Presentación del guardián: el mundo se congela y la
                    // cámara late. Se puede saltar con espacio.
                    self.boss_intro_t -= dt;
                    let beat = (self.boss_intro_t * 6.0).sin().abs();
                    self.shake = self.shake.max(1.0 + beat * 2.4);
                    if is_key_pressed(KeyCode::Space)
                        || is_key_pressed(KeyCode::Enter)
                        || is_mouse_button_pressed(MouseButton::Left)
                    {
                        self.boss_intro_t = 0.0;
                        self.shake = 3.0;
                        sfx.play(SfxKind::Click, 0.2);
                    }
                } else if self.floor_transition_t > 0.0 {
                    // Descenso de piso: fundido breve, saltable con espacio.
                    self.floor_transition_t -= dt;
                    if is_key_pressed(KeyCode::Space)
                        || is_key_pressed(KeyCode::Enter)
                        || is_mouse_button_pressed(MouseButton::Left)
                    {
                        self.floor_transition_t = 0.0;
                    }
                    if self.floor_transition_t <= 0.0 {
                        self.floor_transition_t = 0.0;
                        if self.floor_transition_pending {
                            self.floor_transition_pending = false;
                            self.next_floor();
                            self.floor_arrival_t = 0.30;
                        }
                    }
                } else if self.map_open {
                    if self.bind_pressed(5)
                        || is_key_pressed(KeyCode::Escape)
                        || is_mouse_button_pressed(MouseButton::Right)
                        || (is_mouse_button_pressed(MouseButton::Left)
                            && map_close_rect().contains(self.mouse_virtual()))
                    {
                        self.map_open = false;
                        sfx.play(SfxKind::Click, 0.2);
                    }
                } else if self.bind_pressed(5) {
                    self.map_open = true;
                    sfx.play(SfxKind::Click, 0.25);
                } else if self.bind_pressed(6) || is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Paused;
                    sfx.play(SfxKind::Click, 0.3);
                } else {
                    self.update_playing(dt, sfx);
                }
            }
            Screen::LevelUp => self.update_levelup(dt, sfx),
            Screen::Paused => self.update_pause(dt, sfx),
            Screen::Dead => {
                if is_key_pressed(KeyCode::R) {
                    let c = self.class;
                    self.start_run(c);
                } else if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Menu;
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    let m = self.mouse_virtual();
                    if dead_retry_rect().contains(m) {
                        let c = self.class;
                        self.start_run(c);
                    } else if dead_menu_rect().contains(m) {
                        self.screen = Screen::Menu;
                    }
                }
            }
            Screen::Victory => {
                if is_key_pressed(KeyCode::C) {
                    self.endless = true;
                    self.screen = Screen::Playing;
                    self.next_floor();
                } else if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Menu;
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    let m = self.mouse_virtual();
                    if victory_endless_rect().contains(m) {
                        self.endless = true;
                        self.screen = Screen::Playing;
                        self.next_floor();
                    } else if victory_menu_rect().contains(m) {
                        self.screen = Screen::Menu;
                    }
                }
            }
            Screen::Shop => self.update_shop(sfx),
            Screen::Locker => self.update_locker(sfx),
            Screen::Audio => self.update_audio(sfx),
            Screen::Library => self.update_library(sfx),
            Screen::Settings => self.update_settings(sfx),
            Screen::Inventory => self.update_inventory(sfx),
            Screen::Tickets => self.update_tickets(sfx),
            Screen::Forge => self.update_forge(sfx),
            Screen::Stats => {
                if is_key_pressed(KeyCode::Escape)
                    || is_mouse_button_pressed(MouseButton::Left)
                        && stats_back_rect().contains(self.mouse_virtual())
                {
                    self.screen = self.stats_return;
                    sfx.play(SfxKind::Click, 0.2);
                }
            }
        }
    }

    fn update_pause(&mut self, _dt: f32, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Playing;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        if is_key_pressed(KeyCode::Q) {
            self.end_run(false);
            return;
        }
        if is_key_pressed(KeyCode::I) {
            self.inventory_return = Screen::Paused;
            self.screen = Screen::Inventory;
            sfx.play(SfxKind::Click, 0.3);
            return;
        }
        if is_key_pressed(KeyCode::O) {
            self.settings_return = Screen::Paused;
            self.rebinding = false;
            self.screen = Screen::Settings;
            sfx.play(SfxKind::Click, 0.3);
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let m = self.mouse_virtual();
            if pause_continue_rect().contains(m) {
                self.screen = Screen::Playing;
                sfx.play(SfxKind::Click, 0.3);
            } else if pause_stats_rect().contains(m) {
                self.stats_return = Screen::Paused;
                self.screen = Screen::Stats;
                sfx.play(SfxKind::Click, 0.3);
            } else if pause_inventory_rect().contains(m) {
                self.inventory_return = Screen::Paused;
                self.screen = Screen::Inventory;
                sfx.play(SfxKind::Click, 0.3);
            } else if pause_settings_rect().contains(m) {
                self.settings_return = Screen::Paused;
                self.rebinding = false;
                self.screen = Screen::Settings;
                sfx.play(SfxKind::Click, 0.3);
            } else if pause_abandon_rect().contains(m) {
                self.end_run(false);
            } else if pause_audio_rect().contains(m) {
                self.open_audio(Screen::Paused);
                sfx.play(SfxKind::Click, 0.3);
            }
        }
    }

    fn update_inventory(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::I)
            || (is_mouse_button_pressed(MouseButton::Left)
                && inventory_back_rect().contains(self.mouse_virtual()))
        {
            self.screen = self.inventory_return;
            sfx.play(SfxKind::Click, 0.2);
        }
    }

    fn update_tickets(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::T)
            || (is_mouse_button_pressed(MouseButton::Left)
                && tickets_back_rect().contains(self.mouse_virtual()))
        {
            self.screen = Screen::Locker;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let m = self.mouse_virtual();
            for i in 0..save::TICKETS.len() {
                if !tickets_row_rect(i).contains(m) {
                    continue;
                }
                let unlocked = i == 0 || (self.meta.owned_tickets & (1 << i)) != 0;
                if unlocked {
                    self.meta.ticket = i as u8;
                    save::save(&self.meta);
                    self.toast(
                        &format!("Contrato equipado: {}", save::TICKETS[i].0),
                        C_PURPLE,
                    );
                    sfx.play(SfxKind::Level, 0.2);
                } else {
                    self.toast("Ese contrato aún está bloqueado", C_RED);
                    sfx.play(SfxKind::Click, 0.2);
                }
                return;
            }
        }
    }

    fn open_forge(&mut self, from: Screen) {
        self.forge_return = from;
        self.forge_class = self.menu_index.min(Class::ALL.len() - 1);
        self.screen = Screen::Forge;
    }

    fn cycle_forge_class(&mut self, delta: i32, sfx: &mut Sfx) {
        let n = Class::ALL.len() as i32;
        self.forge_class = ((self.forge_class as i32 + delta + n) % n) as usize;
        sfx.play(SfxKind::Click, 0.18);
    }

    fn forge_action(&mut self, index: usize, sfx: &mut Sfx) {
        let class = Class::ALL[self.forge_class.min(Class::ALL.len() - 1)];
        let Some(def) = weapons::get(class, index) else {
            return;
        };
        if self.meta.weapon_for(class) == index {
            self.toast("Esa arma ya está equipada", C_GOLD);
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        if self.meta.owns_weapon(class, index) {
            self.meta.set_weapon(class, index);
            save::save(&self.meta);
            self.toast(&format!("Arma equipada: {}", def.name), class.color());
            sfx.play(SfxKind::Level, 0.22);
            return;
        }
        if self.meta.best_floor < def.req_floor {
            self.toast(
                &format!("Necesitas llegar al piso {} para forjarla", def.req_floor),
                C_RED,
            );
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        if self.meta.souls < def.cost {
            self.toast(
                &format!("Faltan {} almas", def.cost - self.meta.souls),
                C_RED,
            );
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        self.meta.souls -= def.cost;
        self.meta.unlock_weapon(class, index);
        self.meta.set_weapon(class, index);
        save::save(&self.meta);
        self.toast(&format!("Forjada y equipada: {}", def.name), C_GOLD);
        sfx.play(SfxKind::Level, 0.35);
    }

    fn update_forge(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::F)
            || (is_mouse_button_pressed(MouseButton::Left)
                && forge_back_rect().contains(self.mouse_virtual()))
        {
            self.screen = self.forge_return;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            self.cycle_forge_class(-1, sfx);
        }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            self.cycle_forge_class(1, sfx);
        }
        for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3]
            .iter()
            .enumerate()
        {
            if is_key_pressed(*key) {
                self.forge_action(i + 1, sfx);
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let m = self.mouse_virtual();
            if forge_class_minus().contains(m) {
                self.cycle_forge_class(-1, sfx);
                return;
            }
            if forge_class_plus().contains(m) {
                self.cycle_forge_class(1, sfx);
                return;
            }
            for i in 0..3 {
                if forge_button_rect(i).contains(m) {
                    self.forge_action(i + 1, sfx);
                    return;
                }
            }
        }
    }

    fn open_audio(&mut self, from: Screen) {
        self.audio_return = from;
        self.audio_row = 0;
        self.screen = Screen::Audio;
    }

    fn apply_audio_volumes(&mut self, sfx: &mut Sfx) {
        sfx.set_volumes(self.meta.music_vol, self.meta.sfx_vol, self.meta.enemy_vol);
        save::save(&self.meta);
    }

    fn update_audio(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            self.screen = self.audio_return;
            save::save(&self.meta);
            sfx.play(SfxKind::Click, 0.25);
            return;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.audio_row = (self.audio_row + 2) % 3;
            sfx.play(SfxKind::Click, 0.15);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.audio_row = (self.audio_row + 1) % 3;
            sfx.play(SfxKind::Click, 0.15);
        }
        let mut delta = 0i16;
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            delta -= 5;
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            delta += 5;
        }
        if delta != 0 {
            self.adjust_audio(self.audio_row, delta, sfx);
        }

        let m = self.mouse_virtual();
        if is_mouse_button_pressed(MouseButton::Left) {
            if audio_back_rect().contains(m) {
                self.screen = self.audio_return;
                save::save(&self.meta);
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            for i in 0..3 {
                let r = audio_row_rect(i);
                if audio_minus_rect(i).contains(m) {
                    self.audio_row = i;
                    self.adjust_audio(i, -5, sfx);
                    return;
                }
                if audio_plus_rect(i).contains(m) {
                    self.audio_row = i;
                    self.adjust_audio(i, 5, sfx);
                    return;
                }
                if audio_bar_rect(i).contains(m) {
                    self.audio_row = i;
                    let t = ((m.x - audio_bar_rect(i).x) / audio_bar_rect(i).w).clamp(0.0, 1.0);
                    self.set_audio(i, (t * 100.0).round() as u8, sfx);
                    return;
                }
                if r.contains(m) {
                    self.audio_row = i;
                    sfx.play(SfxKind::Click, 0.15);
                    return;
                }
            }
        }
    }

    fn adjust_audio(&mut self, row: usize, delta: i16, sfx: &mut Sfx) {
        let current = match row {
            0 => self.meta.music_vol,
            1 => self.meta.sfx_vol,
            _ => self.meta.enemy_vol,
        };
        let v = (current as i16 + delta).clamp(0, 100) as u8;
        self.set_audio(row, v, sfx);
    }

    fn set_audio(&mut self, row: usize, value: u8, sfx: &mut Sfx) {
        match row {
            0 => self.meta.music_vol = value,
            1 => self.meta.sfx_vol = value,
            _ => self.meta.enemy_vol = value,
        }
        self.apply_audio_volumes(sfx);
        if row == 1 {
            sfx.play(SfxKind::Click, 0.25);
        } else if row == 2 {
            sfx.play_enemy(EnemyVoice::Growl, 0.35);
        }
    }

    fn draw_audio(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.015, 0.012, 0.03, 0.88));
        ui.panel(72.0, 36.0, 336.0, 204.0, with_alpha(C_CYAN, 0.65));
        ui.text_center_shadow("Audio", 58.0, 13.0, C_WHITE);
        ui.text_center(
            "Música, efectos y criaturas por separado",
            74.0,
            6.0,
            Color::from_rgba(215, 210, 230, 255),
        );
        let labels = ["Música", "Efectos", "Enemigos"];
        let values = [self.meta.music_vol, self.meta.sfx_vol, self.meta.enemy_vol];
        let colors = [C_PURPLE, C_CYAN, C_RED];
        let m = self.mouse_virtual();
        for i in 0..3 {
            let r = audio_row_rect(i);
            let selected = self.audio_row == i;
            let hover = r.contains(m);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if selected || hover {
                    Color::new(0.10, 0.09, 0.14, 0.98)
                } else {
                    Color::new(0.055, 0.05, 0.075, 0.95)
                },
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if selected { 1.5 } else { 0.8 },
                if selected {
                    colors[i]
                } else {
                    Color::new(0.30, 0.28, 0.38, 0.9)
                },
            );
            ui.text_bold(labels[i], r.x + 8.0, r.y + 18.0, 7.0, C_WHITE, true);
            let bar = audio_bar_rect(i);
            ui.rect(
                bar.x,
                bar.y,
                bar.w,
                bar.h,
                Color::new(0.02, 0.018, 0.03, 1.0),
            );
            ui.rect(
                bar.x,
                bar.y,
                bar.w * values[i] as f32 / 100.0,
                bar.h,
                colors[i],
            );
            ui.rect_lines(bar.x, bar.y, bar.w, bar.h, 0.8, with_alpha(colors[i], 0.55));
            ui.text_right(
                &format!("{}", values[i]),
                r.x + 246.0,
                r.y + 18.0,
                7.0,
                C_GOLD,
                true,
            );
            ui_button(ui, audio_minus_rect(i), "-", C_CYAN, m);
            ui_button(ui, audio_plus_rect(i), "+", C_CYAN, m);
        }
        ui_button(ui, audio_back_rect(), "Volver (Esc)", C_CYAN, m);
        ui.text_center(
            "Flechas ajustar   ·   M silenciar música",
            226.0,
            5.8,
            Color::from_rgba(180, 175, 200, 255),
        );
    }

    fn update_menu(&mut self, dt: f32, sfx: &mut Sfx) {
        self.time += dt;
        let up_held = is_key_down(KeyCode::W)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::A)
            || is_key_down(KeyCode::Left);
        let down_held = is_key_down(KeyCode::S)
            || is_key_down(KeyCode::Down)
            || is_key_down(KeyCode::D)
            || is_key_down(KeyCode::Right);
        let fresh_up = is_key_pressed(KeyCode::W)
            || is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::A)
            || is_key_pressed(KeyCode::Left);
        let fresh_down = is_key_pressed(KeyCode::S)
            || is_key_pressed(KeyCode::Down)
            || is_key_pressed(KeyCode::D)
            || is_key_pressed(KeyCode::Right);
        if up_held || down_held {
            let mut step = false;
            if fresh_up || fresh_down {
                self.menu_repeat = 0.28;
                step = true;
            } else {
                self.menu_repeat -= dt;
                if self.menu_repeat <= 0.0 {
                    self.menu_repeat = 0.08;
                    step = true;
                }
            }
            if step {
                let delta = if up_held { -1 } else { 1 };
                self.menu_index = ((self.menu_index as i32 + delta + Class::ALL.len() as i32)
                    % Class::ALL.len() as i32) as usize;
                sfx.play(SfxKind::Click, 0.18);
            }
        } else {
            self.menu_repeat = 0.0;
        }
        if is_key_pressed(KeyCode::Key1) {
            self.buy_upgrade(0, sfx);
        }
        if is_key_pressed(KeyCode::Key2) {
            self.buy_upgrade(1, sfx);
        }
        if is_key_pressed(KeyCode::Key3) {
            self.buy_upgrade(2, sfx);
        }
        if is_key_pressed(KeyCode::F) {
            self.open_forge(Screen::Menu);
            sfx.play(SfxKind::Click, 0.25);
            return;
        }
        let m = self.mouse_virtual();
        for i in 0..Class::ALL.len() {
            let r = menu_class_rect(i);
            if r.contains(m) {
                if self.menu_index != i {
                    self.menu_index = i;
                    sfx.play(SfxKind::Click, 0.10);
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    sfx.play(SfxKind::Click, 0.22);
                }
            }
        }
        for i in 0..3 {
            if upgrade_rect(i).contains(m) && is_mouse_button_pressed(MouseButton::Left) {
                self.buy_upgrade(i, sfx);
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            if menu_play_rect().contains(m) {
                let c = Class::ALL[self.menu_index];
                sfx.play(SfxKind::Click, 0.4);
                self.start_run(c);
                return;
            }
            if menu_action_rect(0).contains(m) {
                self.screen = Screen::Shop;
                self.shop_tab = 0;
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
            if menu_action_rect(1).contains(m) {
                self.screen = Screen::Locker;
                self.locker_class = self.menu_index;
                self.name_buffer = self.meta.name.clone();
                self.editing_name = false;
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
            if menu_action_rect(2).contains(m) {
                self.open_forge(Screen::Menu);
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
            if menu_action_rect(3).contains(m) {
                self.screen = Screen::Library;
                self.library_page = 0;
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
            if menu_action_rect(4).contains(m) {
                self.settings_return = Screen::Menu;
                self.screen = Screen::Settings;
                self.rebinding = false;
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
            if menu_action_rect(5).contains(m) {
                self.stats_return = Screen::Menu;
                self.screen = Screen::Stats;
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
            if menu_action_rect(6).contains(m) {
                self.open_audio(Screen::Menu);
                sfx.play(SfxKind::Click, 0.35);
                return;
            }
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            let c = Class::ALL[self.menu_index];
            sfx.play(SfxKind::Click, 0.4);
            self.start_run(c);
        }
    }

    fn update_shop(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        let m = self.mouse_virtual();
        if is_mouse_button_pressed(MouseButton::Left) {
            if shop_back_rect().contains(m) {
                self.screen = Screen::Menu;
                sfx.play(SfxKind::Click, 0.2);
                return;
            }
            for i in 0..3 {
                if shop_tab_rect(i).contains(m) {
                    self.shop_tab = i;
                    sfx.play(SfxKind::Click, 0.2);
                    return;
                }
            }
            for i in 0..8 {
                if shop_item_rect(i).contains(m) {
                    self.buy_cosmetic(i, sfx);
                    return;
                }
            }
        }
    }

    fn buy_cosmetic(&mut self, idx: usize, sfx: &mut Sfx) {
        let (cost, owned, current) = match self.shop_tab {
            0 => (
                save::SKINS[idx].1,
                self.meta.owns_skin(idx),
                self.meta.skin as usize,
            ),
            1 => (
                save::HATS[idx].1,
                self.meta.owns_hat(idx),
                self.meta.hat as usize,
            ),
            _ => (
                save::MASKS[idx].1,
                self.meta.owns_mask(idx),
                self.meta.mask as usize,
            ),
        };
        if idx == current {
            return;
        }
        if owned {
            match self.shop_tab {
                0 => self.meta.skin = idx as u8,
                1 => self.meta.hat = idx as u8,
                _ => self.meta.mask = idx as u8,
            }
            save::save(&self.meta);
            self.toast("Cosmético equipado", C_GREEN);
            sfx.play(SfxKind::Pickup, 0.25);
            return;
        }
        if let Some((floor, kills)) = save::cosmetic_requirement(self.shop_tab, idx) {
            if self.meta.best_floor < floor || self.meta.total_kills < kills {
                let mut need = Vec::new();
                if self.meta.best_floor < floor {
                    need.push(format!("piso {}", floor));
                }
                if self.meta.total_kills < kills {
                    need.push(format!("{} bajas", kills));
                }
                self.toast(&format!("Desafío pendiente: {}", need.join(" y ")), C_RED);
                sfx.play(SfxKind::Lock, 0.32);
                return;
            }
        }
        if self.meta.souls < cost {
            self.toast("Faltan almas para comprar esto", C_RED);
            sfx.play(SfxKind::Hurt, 0.2);
            return;
        }
        self.meta.souls -= cost;
        let bit = 1u32 << idx;
        match self.shop_tab {
            0 => {
                self.meta.owned_skins |= bit;
                self.meta.skin = idx as u8;
            }
            1 => {
                self.meta.owned_hats |= bit;
                self.meta.hat = idx as u8;
            }
            _ => {
                self.meta.owned_masks |= bit;
                self.meta.mask = idx as u8;
            }
        }
        save::save(&self.meta);
        self.toast("Comprado y equipado", C_GOLD);
        sfx.play(SfxKind::Level, 0.3);
    }

    fn buy_shop_offer(&mut self, i: usize, sfx: &mut Sfx) {
        self.buy_shop_offer_inner(i, sfx, false);
    }

    fn buy_shop_offer_credit(&mut self, i: usize, sfx: &mut Sfx) {
        self.buy_shop_offer_inner(i, sfx, true);
    }

    fn buy_shop_offer_inner(&mut self, i: usize, sfx: &mut Sfx, credit: bool) {
        let (cost, kind, sold) = match self.shop_offers.get(i) {
            Some(o) => (o.cost, o.kind, o.sold),
            None => return,
        };
        if sold {
            self.toast("Ya está vendido", C_CYAN);
            return;
        }
        if matches!(kind, ShopKind::Heal) && self.player.hp >= self.player.max_hp - 0.01 {
            self.toast("Tu vida ya está completa", C_CYAN);
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        let surcharge = (cost as f32 * 1.30).ceil() as u32;
        if credit {
            if self.player.gold >= cost {
                self.toast("Tienes oro suficiente: no necesitas fiar", C_CYAN);
                return;
            }
            let limit = 140 + self.floor * 18;
            if self.debt + surcharge > limit {
                self.toast("El mercader no te fía tanto", C_RED);
                sfx.play(SfxKind::Hurt, 0.2);
                return;
            }
            self.debt += surcharge;
            self.meta.debt = self.debt;
            self.toast(
                &format!("Comprado a crédito. Debes {} de oro", self.debt),
                C_PURPLE,
            );
        } else {
            if self.player.gold < cost {
                self.toast("No te alcanza. Pulsa F para comprar a crédito", C_RED);
                sfx.play(SfxKind::Hurt, 0.2);
                return;
            }
            self.player.gold -= cost;
        }
        self.shop_offers[i].sold = true;
        match kind {
            ShopKind::Heal => {
                let heal = self.player.max_hp * 0.45;
                self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                spawn_number(
                    &mut self.floaters,
                    self.player.pos + vec2(0.0, -16.0),
                    heal.round() as i32,
                    C_GREEN,
                    8.0,
                );
            }
            ShopKind::Item(item) => self.apply_item(item),
            ShopKind::Buff(def) => self.apply_buff(def.id),
            ShopKind::Key => {
                self.keys += 1;
                self.toast("Has comprado una llave", C_GOLD);
            }
            ShopKind::SpecialKey => {
                self.special_keys += 1;
                self.toast("Has comprado una llave negra", C_PURPLE);
            }
        }
        spawn_ring(
            &mut self.rings,
            self.player.pos,
            6.0,
            34.0,
            0.4,
            if credit { C_PURPLE } else { C_GOLD },
            2.0,
        );
        sfx.play(SfxKind::Level, 0.3);
        if !credit {
            self.toast("Compra realizada", C_GOLD);
        }
    }

    fn update_locker(&mut self, sfx: &mut Sfx) {
        if self.editing_name {
            while let Some(c) = get_char_pressed() {
                if !c.is_control() && self.name_buffer.chars().count() < 14 {
                    self.name_buffer.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.name_buffer.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                self.commit_name(sfx);
            } else if is_key_pressed(KeyCode::Escape) {
                self.name_buffer = self.meta.name.clone();
                self.editing_name = false;
            }
            // Un clic fuera del campo guarda y devuelve el control sin bloquear
            // el resto de la taquilla.
            let m = self.mouse_virtual();
            if is_mouse_button_pressed(MouseButton::Left) && !locker_name_rect().contains(m) {
                self.commit_name(sfx);
            }
        }
        if !self.editing_name {
            if is_key_pressed(KeyCode::Escape) {
                self.screen = Screen::Menu;
                sfx.play(SfxKind::Click, 0.2);
                return;
            }
            if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                self.cycle_class(-1, sfx);
            }
            if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                self.cycle_class(1, sfx);
            }
            if is_key_pressed(KeyCode::N) {
                self.name_buffer = self.meta.name.clone();
                self.editing_name = true;
                self.name_t = 0.0;
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            if is_key_pressed(KeyCode::T) {
                self.screen = Screen::Tickets;
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            if is_key_pressed(KeyCode::F) {
                self.open_forge(Screen::Locker);
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
        }
        let m = self.mouse_virtual();
        if is_mouse_button_pressed(MouseButton::Left) {
            if locker_back_rect().contains(m) {
                self.screen = Screen::Menu;
                sfx.play(SfxKind::Click, 0.2);
                return;
            }
            if locker_class_minus().contains(m) {
                self.cycle_class(-1, sfx);
                return;
            }
            if locker_class_plus().contains(m) {
                self.cycle_class(1, sfx);
                return;
            }
            for row in 0..3 {
                if locker_minus(row).contains(m) {
                    self.cycle_cosmetic(row, -1, sfx);
                    return;
                }
                if locker_plus(row).contains(m) {
                    self.cycle_cosmetic(row, 1, sfx);
                    return;
                }
            }
            if locker_name_rect().contains(m) {
                self.name_buffer = self.meta.name.clone();
                self.editing_name = true;
                self.name_t = 0.0;
                sfx.play(SfxKind::Click, 0.25);
            }
            if locker_shop_rect().contains(m) {
                self.screen = Screen::Shop;
                self.shop_tab = 0;
                sfx.play(SfxKind::Click, 0.25);
            }
            if locker_tickets_rect().contains(m) {
                self.screen = Screen::Tickets;
                sfx.play(SfxKind::Click, 0.25);
            }
            if locker_forge_rect().contains(m) {
                self.open_forge(Screen::Locker);
                sfx.play(SfxKind::Click, 0.25);
            }
        }
    }

    fn commit_name(&mut self, sfx: &mut Sfx) {
        let clean: String = self.name_buffer.trim().chars().take(14).collect::<String>();
        self.meta.name = if clean.is_empty() {
            "Aventurero".to_owned()
        } else {
            clean
        };
        self.name_buffer = self.meta.name.clone();
        self.editing_name = false;
        save::save(&self.meta);
        self.toast("Nombre guardado", C_GREEN);
        sfx.play(SfxKind::Level, 0.25);
    }

    fn update_library(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        let pages = library_pages(self.library_tab);
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Q)
        {
            self.library_page = if self.library_page == 0 {
                pages - 1
            } else {
                self.library_page - 1
            };
            sfx.play(SfxKind::Click, 0.16);
        }
        if is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::D)
            || is_key_pressed(KeyCode::E)
        {
            self.library_page = (self.library_page + 1) % pages;
            sfx.play(SfxKind::Click, 0.16);
        }
        let m = self.mouse_virtual();
        if is_mouse_button_pressed(MouseButton::Left) {
            if library_back_rect().contains(m) {
                self.screen = Screen::Menu;
                sfx.play(SfxKind::Click, 0.2);
                return;
            }
            for i in 0..4 {
                if library_tab_rect(i).contains(m) {
                    self.library_tab = i;
                    self.library_page = 0;
                    sfx.play(SfxKind::Click, 0.2);
                    return;
                }
            }
            if library_prev_rect().contains(m) {
                self.library_page = if self.library_page == 0 {
                    pages - 1
                } else {
                    self.library_page - 1
                };
                sfx.play(SfxKind::Click, 0.16);
                return;
            }
            if library_next_rect().contains(m) {
                self.library_page = (self.library_page + 1) % pages;
                sfx.play(SfxKind::Click, 0.16);
            }
        }
    }

    fn update_settings(&mut self, sfx: &mut Sfx) {
        if self.rebinding {
            if is_key_pressed(KeyCode::Escape) {
                self.rebinding = false;
                self.toast("Asignación cancelada", C_CYAN);
                sfx.play(SfxKind::Click, 0.2);
                return;
            }
            if let Some(key) = get_last_key_pressed() {
                let code = key as u16;
                if let Some(other) = self.meta.binds.duplicate(self.settings_row, code) {
                    self.toast(
                        &format!(
                            "Esa tecla ya está asignada a {}",
                            save::KEY_ACTION_NAMES[other]
                        ),
                        C_RED,
                    );
                    sfx.play(SfxKind::Lock, 0.35);
                } else {
                    self.meta.binds.set(self.settings_row, code);
                    save::save(&self.meta);
                    self.rebinding = false;
                    self.toast("Tecla asignada", C_GREEN);
                    sfx.play(SfxKind::Level, 0.25);
                }
            }
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.screen = self.settings_return;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        let m = self.mouse_virtual();
        if is_mouse_button_pressed(MouseButton::Left) {
            if settings_back_rect().contains(m) {
                self.screen = self.settings_return;
                sfx.play(SfxKind::Click, 0.2);
                return;
            }
            if settings_reset_rect().contains(m) {
                self.meta.binds.reset();
                save::save(&self.meta);
                self.toast("Teclas restablecidas", C_CYAN);
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            if settings_audio_rect().contains(m) {
                self.open_audio(Screen::Settings);
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            if settings_transition_rect().contains(m) {
                self.meta.floor_transitions = !self.meta.floor_transitions;
                save::save(&self.meta);
                self.toast(
                    if self.meta.floor_transitions {
                        "Transición de piso activada"
                    } else {
                        "Transición de piso desactivada"
                    },
                    C_CYAN,
                );
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            if settings_difficulty_rect().contains(m) {
                self.meta.difficulty = (self.meta.difficulty + 1) % 3;
                save::save(&self.meta);
                let d = self.difficulty();
                self.toast(&format!("Dificultad: {}", d.name()), d.color());
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
            for i in 0..save::KEY_ACTION_COUNT {
                if settings_row_rect(i).contains(m) {
                    self.settings_row = i;
                    self.rebinding = true;
                    sfx.play(SfxKind::Click, 0.2);
                    return;
                }
            }
        }
    }

    fn cycle_class(&mut self, delta: i32, sfx: &mut Sfx) {
        let n = Class::ALL.len() as i32;
        self.locker_class = ((self.locker_class as i32 + delta + n) % n) as usize;
        self.menu_index = self.locker_class;
        sfx.play(SfxKind::Click, 0.18);
    }

    fn cycle_cosmetic(&mut self, row: usize, delta: i32, sfx: &mut Sfx) {
        let owned: Vec<usize> = match row {
            0 => (0..save::SKINS.len())
                .filter(|&i| self.meta.owns_skin(i))
                .collect(),
            1 => (0..save::HATS.len())
                .filter(|&i| self.meta.owns_hat(i))
                .collect(),
            _ => (0..save::MASKS.len())
                .filter(|&i| self.meta.owns_mask(i))
                .collect(),
        };
        if owned.is_empty() {
            return;
        }
        let current = match row {
            0 => self.meta.skin as usize,
            1 => self.meta.hat as usize,
            _ => self.meta.mask as usize,
        };
        let pos = owned.iter().position(|&i| i == current).unwrap_or(0);
        let next = (pos as i32 + delta).rem_euclid(owned.len() as i32) as usize;
        let chosen = owned[next] as u8;
        match row {
            0 => self.meta.skin = chosen,
            1 => self.meta.hat = chosen,
            _ => self.meta.mask = chosen,
        }
        save::save(&self.meta);
        sfx.play(SfxKind::Click, 0.18);
    }

    fn buy_upgrade(&mut self, kind: usize, sfx: &mut Sfx) {
        let level = match kind {
            0 => self.meta.hp_lv,
            1 => self.meta.dmg_lv,
            _ => self.meta.luck_lv,
        };
        let cost = save::upgrade_cost(kind, level);
        let req = save::upgrade_requirement(kind, level);
        if self.meta.best_floor < req {
            self.toast(&format!("Desafío pendiente: llega al piso {}", req), C_RED);
            sfx.play(SfxKind::Lock, 0.32);
            return;
        }
        if self.meta.souls >= cost {
            self.meta.souls -= cost;
            match kind {
                0 => self.meta.hp_lv += 1,
                1 => self.meta.dmg_lv += 1,
                _ => self.meta.luck_lv += 1,
            }
            save::save(&self.meta);
            self.toast("Mejora comprada", C_GOLD);
            sfx.play(SfxKind::Level, 0.3);
        } else {
            self.toast("Faltan almas", C_RED);
            sfx.play(SfxKind::Hurt, 0.2);
        }
    }

    fn update_levelup(&mut self, dt: f32, sfx: &mut Sfx) {
        self.levelup_t += dt;
        if self.pending_replace.is_some() {
            self.update_replace(sfx);
            return;
        }
        if is_key_pressed(KeyCode::Key1) {
            self.choose_level_choice(0, sfx);
            return;
        } else if is_key_pressed(KeyCode::Key2) {
            self.choose_level_choice(1, sfx);
            return;
        } else if is_key_pressed(KeyCode::Key3) {
            self.choose_level_choice(2, sfx);
            return;
        }
        let m = self.mouse_virtual();
        if is_mouse_button_pressed(MouseButton::Left) {
            for i in 0..3 {
                if card_rect(i).contains(m) {
                    self.choose_level_choice(i, sfx);
                    return;
                }
            }
            if levelup_reroll_rect().contains(m) && self.rerolls > 0 {
                self.rerolls -= 1;
                self.roll_level_choices();
                self.levelup_t = 0.0;
                sfx.play(SfxKind::Click, 0.3);
                return;
            }
            if levelup_stats_rect().contains(m) {
                self.stats_return = Screen::LevelUp;
                self.screen = Screen::Stats;
                sfx.play(SfxKind::Click, 0.25);
                return;
            }
        }
        if is_key_pressed(KeyCode::R) && self.rerolls > 0 {
            self.rerolls -= 1;
            self.roll_level_choices();
            self.levelup_t = 0.0;
            sfx.play(SfxKind::Click, 0.3);
        }
    }

    fn new_attack_candidates(&self) -> Vec<usize> {
        self.attacks
            .iter()
            .enumerate()
            .filter(|(i, a)| {
                a.class == self.class && !self.equipped_attacks.iter().any(|e| e.def == *i)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn upgrade_attack_candidates(&self) -> Vec<usize> {
        self.equipped_attacks
            .iter()
            .enumerate()
            .filter(|(_, e)| e.level < self.attacks[e.def].max_level)
            .map(|(i, _)| i)
            .collect()
    }

    /// Mezcla ataques nuevos, subidas de nivel y mejoras de estadísticas. Al
    /// menos una de las tres opciones siempre es un ataque si queda alguno.
    fn roll_level_choices(&mut self) {
        let new = self.new_attack_candidates();
        let up = self.upgrade_attack_candidates();
        let buff_cards = buffs::pick_cards(
            &mut self.rng,
            self.class,
            &self.buff_stacks,
            self.player.level,
        );
        let mut out: Vec<LevelChoice> = Vec::with_capacity(3);
        let mut buff_i = 0usize;

        if !new.is_empty() || !up.is_empty() {
            if !up.is_empty() && (new.is_empty() || self.rng.chance(0.45)) {
                let slot = up[self.rng.usize(up.len())];
                out.push(LevelChoice::UpgradeAttack(slot));
            } else {
                let def = new[self.rng.usize(new.len())];
                out.push(LevelChoice::NewAttack(def));
            }
        }

        while out.len() < 3 {
            let free_new: Vec<usize> = new
                .iter()
                .copied()
                .filter(|&d| {
                    !out.iter()
                        .any(|c| matches!(c, LevelChoice::NewAttack(x) if *x == d))
                })
                .collect();
            let free_up: Vec<usize> = up
                .iter()
                .copied()
                .filter(|&s| {
                    !out.iter()
                        .any(|c| matches!(c, LevelChoice::UpgradeAttack(x) if *x == s))
                })
                .collect();
            let roll = self.rng.f32();
            if (!free_new.is_empty() || !free_up.is_empty()) && roll < 0.68 {
                if !free_up.is_empty() && (free_new.is_empty() || roll < 0.30) {
                    let slot = free_up[self.rng.usize(free_up.len())];
                    out.push(LevelChoice::UpgradeAttack(slot));
                } else {
                    let def = free_new[self.rng.usize(free_new.len())];
                    out.push(LevelChoice::NewAttack(def));
                }
            } else if buff_i < buff_cards.len() {
                out.push(LevelChoice::Buff(buff_cards[buff_i]));
                buff_i += 1;
            } else if !free_new.is_empty() {
                let def = free_new[self.rng.usize(free_new.len())];
                out.push(LevelChoice::NewAttack(def));
            } else if !free_up.is_empty() {
                let slot = free_up[self.rng.usize(free_up.len())];
                out.push(LevelChoice::UpgradeAttack(slot));
            } else {
                break;
            }
        }
        while out.len() < 3 {
            out.push(LevelChoice::Buff(buff_cards[0]));
        }
        self.level_choices = [out[0], out[1], out[2]];
    }

    pub fn choose_level_choice(&mut self, i: usize, sfx: &mut Sfx) {
        if i >= 3 {
            return;
        }
        match self.level_choices[i] {
            LevelChoice::NewAttack(def) => {
                if self.equipped_attacks.len() >= MAX_EQUIPPED {
                    // No hay hueco: se abre el menú de sustitución.
                    self.pending_replace = Some(def);
                    sfx.play(SfxKind::Click, 0.25);
                    return;
                }
                self.equipped_attacks.push(EquippedAttack::new(def));
            }
            LevelChoice::UpgradeAttack(slot) => {
                if let Some(e) = self.equipped_attacks.get_mut(slot) {
                    let max = self.attacks[e.def].max_level;
                    e.level = (e.level + 1).min(max);
                }
            }
            LevelChoice::Buff(def) => self.apply_buff(def.id),
        }
        self.finish_levelup(sfx);
    }

    fn update_replace(&mut self, sfx: &mut Sfx) {
        if is_key_pressed(KeyCode::Escape) {
            self.pending_replace = None;
            sfx.play(SfxKind::Click, 0.2);
            return;
        }
        let m = self.mouse_virtual();
        let click = is_mouse_button_pressed(MouseButton::Left);
        for i in 0..self.equipped_attacks.len() {
            let keyed = match i {
                0 => is_key_pressed(KeyCode::Key1),
                1 => is_key_pressed(KeyCode::Key2),
                2 => is_key_pressed(KeyCode::Key3),
                3 => is_key_pressed(KeyCode::Key4),
                4 => is_key_pressed(KeyCode::Key5),
                _ => is_key_pressed(KeyCode::Key6),
            };
            if keyed || (click && replace_rect(i).contains(m)) {
                if let Some(def) = self.pending_replace.take() {
                    self.equipped_attacks[i] = EquippedAttack::new(def);
                    self.finish_levelup(sfx);
                }
                return;
            }
        }
    }

    fn finish_levelup(&mut self, sfx: &mut Sfx) {
        self.pending_replace = None;
        self.screen = Screen::Playing;
        sfx.play(SfxKind::Level, 0.35);
        self.check_level_up();
    }

    /// Sustitución de ataque usada por el modo grabación cuando la mochila
    /// de seis ataques ya está llena.
    pub fn demo_resolve_replace(&mut self, slot: usize, sfx: &mut Sfx) {
        if let Some(def) = self.pending_replace.take() {
            if let Some(e) = self.equipped_attacks.get_mut(slot) {
                *e = EquippedAttack::new(def);
                self.finish_levelup(sfx);
            }
        }
    }

    /// Red de seguridad del modo grabación: si un enemigo queda fuera del
    /// alcance o atrapado en la geometría, la sala se cierra igual para que
    /// la guía no se quede bloqueada. Solo se usa desde `demo.rs`.
    pub fn demo_force_clear(&mut self) {
        for e in &mut self.enemies {
            if !e.dead {
                e.hp = 0.0;
            }
        }
    }

    fn apply_buff(&mut self, id: BuffId) {
        let idx = id as usize;
        self.buff_stacks[idx] += 1;
        apply_buff_to_player(&mut self.player, id, self.class);
        if id == BuffId::Legion && self.class == Class::Necro {
            self.summon_all();
        }
        self.refresh_skeleton_damage();
    }

    /// Líneas de "ahora › después" que se muestran en cada carta de mejora.
    /// El valor futuro se pinta en verde en la interfaz.
    fn buff_preview(&self, id: BuffId) -> Vec<(String, String)> {
        let mut p = self.player.clone();
        apply_buff_to_player(&mut p, id, self.class);
        let mut out: Vec<(String, String)> = Vec::new();
        macro_rules! add {
            ($label:expr, $a:expr, $b:expr, $pct:expr) => {
                if ($a - $b).abs() > 0.0001 {
                    if $pct {
                        out.push((
                            $label.to_owned(),
                            format!("{:.0}% › {:.0}%", $a * 100.0, $b * 100.0),
                        ));
                    } else {
                        out.push(($label.to_owned(), format!("{:.1} › {:.1}", $a, $b)));
                    }
                }
            };
        }
        add!("Daño", self.player.damage, p.damage, false);
        add!("Vida máxima", self.player.max_hp, p.max_hp, false);
        add!("Velocidad", self.player.speed, p.speed, false);
        add!("Ataques/s", self.player.attack_speed, p.attack_speed, false);
        add!("Alcance", self.player.range, p.range, false);
        add!("Crítico", self.player.crit, p.crit, true);
        add!("Daño crítico", self.player.crit_mult, p.crit_mult, false);
        add!("Armadura", self.player.armor, p.armor, false);
        add!("Bloqueo", self.player.block, p.block, true);
        add!("Regeneración", self.player.regen, p.regen, false);
        add!("Robo de vida", self.player.lifesteal, p.lifesteal, true);
        add!("Espinas", self.player.thorns, p.thorns, true);
        add!("Reflejo", self.player.reflect, p.reflect, true);
        add!("Área", self.player.aoe, p.aoe, false);
        add!("Experiencia", self.player.xp_mult, p.xp_mult, false);
        add!("Oro", self.player.gold_mult, p.gold_mult, false);
        add!("Suerte", self.player.luck, p.luck, true);
        add!("Ejecución", self.player.execute, p.execute, true);
        if (self.player.dash_cd_max - p.dash_cd_max).abs() > 0.0001 {
            out.push((
                "Recarga de esquiva".to_owned(),
                format!("{:.1}s › {:.1}s", self.player.dash_cd_max, p.dash_cd_max),
            ));
        }
        if p.skeleton_cap != self.player.skeleton_cap {
            out.push((
                "Esqueletos".to_owned(),
                format!("{} › {}", self.player.skeleton_cap, p.skeleton_cap),
            ));
        }
        if p.extra_projectiles != self.player.extra_projectiles {
            out.push((
                "Proyectiles".to_owned(),
                format!(
                    "{} › {}",
                    1 + self.player.extra_projectiles,
                    1 + p.extra_projectiles
                ),
            ));
        }
        if p.pierce != self.player.pierce {
            out.push((
                "Perforación".to_owned(),
                format!("{} › {}", self.player.pierce, p.pierce),
            ));
        }
        if p.chain != self.player.chain {
            out.push((
                "Rebotes".to_owned(),
                format!("{} › {}", self.player.chain, p.chain),
            ));
        }
        if p.shockwave != self.player.shockwave {
            out.push((
                "Ondas de choque".to_owned(),
                format!("{} › {}", self.player.shockwave, p.shockwave),
            ));
        }
        if p.bloodlust != self.player.bloodlust {
            out.push((
                "Impulso al matar".to_owned(),
                format!("{} › {}", self.player.bloodlust, p.bloodlust),
            ));
        }
        if id == BuffId::BoneDmg {
            let stacks = self.buff_stacks[BuffId::BoneDmg as usize] as f32;
            let a = self.player.skeleton_dmg * (1.0 + 0.35 * stacks);
            let b = self.player.skeleton_dmg * (1.0 + 0.35 * (stacks + 1.0));
            out.push((
                "Daño de esqueletos".to_owned(),
                format!("{:.1} › {:.1}", a, b),
            ));
        }
        if id == BuffId::BoneInherit {
            add!(
                "Herencia de esqueletos",
                self.player.skeleton_inherit,
                p.skeleton_inherit,
                true
            );
        }
        if id == BuffId::BoneExplode {
            out.push((
                "Explosión al morir".to_owned(),
                if self.player.skeleton_explode {
                    "Activada › Activada".to_owned()
                } else {
                    "No › Sí".to_owned()
                },
            ));
        }
        if id == BuffId::Legion {
            out.push((
                "Esqueletos máximos".to_owned(),
                format!("{} › {}", self.player.skeleton_cap, p.skeleton_cap),
            ));
        }
        out
    }

    fn check_level_up(&mut self) {
        if self.player.xp >= self.player.xp_next {
            self.player.xp -= self.player.xp_next;
            self.player.level += 1;
            self.player.xp_next = 20.0 * (self.player.level as f32).powf(1.35);
            let heal = self.player.max_hp * 0.15;
            self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
            self.cards = buffs::pick_cards(
                &mut self.rng,
                self.class,
                &self.buff_stacks,
                self.player.level,
            );
            self.roll_level_choices();
            self.rerolls = 1;
            self.levelup_t = 0.0;
            self.screen = Screen::LevelUp;
        }
    }

    fn update_playing(&mut self, dt: f32, sfx: &mut Sfx) {
        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            return;
        }
        self.time += dt;
        // Las puertas se cierran de golpe al entrar en una sala con enemigos y
        // se abren al limpiarla. La animación es rápida, como un portazo.
        let sealing =
            !self.cleared && matches!(self.current_kind(), RoomKind::Combat | RoomKind::Boss);
        let target_seal = if sealing { 1.0 } else { 0.0 };
        let prev_seal = self.door_seal;
        let speed = if target_seal > self.door_seal {
            5.5
        } else {
            3.2
        };
        let delta = target_seal - self.door_seal;
        if delta.abs() > 0.001 {
            self.door_seal = (self.door_seal + delta.signum() * speed * dt).clamp(0.0, 1.0);
        } else {
            self.door_seal = target_seal;
        }
        if sealing && prev_seal < 0.94 && self.door_seal >= 0.94 {
            self.door_slam_t = 0.30;
            self.shake = self.shake.max(3.0);
            sfx.play(SfxKind::Lock, 0.30);
        }
        self.door_slam_t = (self.door_slam_t - dt).max(0.0);
        sfx.set_floor(self.floor);
        sfx.set_motif(if self.cleared { None } else { self.room_voice });
        self.shake *= 0.86;
        let (al, ar, at, ab) = self.arena();

        let p = &mut self.player;
        p.attack_t -= dt;
        p.dash_cd -= dt;
        p.invuln -= dt;
        p.hit_flash -= dt;
        p.kill_speed_t -= dt;
        p.slow_t -= dt;
        if p.regen > 0.0 {
            p.hp = (p.hp + p.regen * dt).min(p.max_hp);
        }

        let mut dir = Vec2::ZERO;
        if self.demo_input.active {
            dir = self.demo_input.dir;
        } else {
            if self.bind_down(0) {
                dir.y -= 1.0;
            }
            if self.bind_down(1) {
                dir.y += 1.0;
            }
            if self.bind_down(2) {
                dir.x -= 1.0;
            }
            if self.bind_down(3) {
                dir.x += 1.0;
            }
        }
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
            self.player.facing = dir;
        }

        let dash_pressed = if self.demo_input.active {
            self.demo_input.dash
        } else {
            self.bind_pressed(4)
        };
        if dash_pressed && self.player.dash_cd <= 0.0 {
            let d = if dir.length_squared() > 0.0 {
                dir
            } else {
                self.player.facing
            };
            self.player.dash_dir = if d.length_squared() > 0.0 {
                d.normalize()
            } else {
                vec2(1.0, 0.0)
            };
            self.player.dash_t = 0.14;
            self.player.dash_cd = self.player.dash_cd_max;
            self.player.invuln = self.player.invuln.max(0.22);
            sfx.play(SfxKind::Dash, 0.25);
            burst(
                &mut self.particles,
                self.player.pos,
                8,
                self.class.color(),
                45.0,
            );
        }

        let mut dash_ended = false;
        if self.player.dash_t > 0.0 {
            self.player.pos += self.player.dash_dir * 300.0 * dt;
            self.player.dash_t -= dt;
            if self.player.dash_t <= 0.0 {
                dash_ended = true;
            }
            if self.rng.chance(0.7) {
                spawn_particle(
                    &mut self.particles,
                    self.player.pos,
                    vec2(0.0, 0.0),
                    0.25,
                    2.0,
                    with_alpha(self.class.color(), 0.5),
                    0.0,
                    0.0,
                );
            }
        } else {
            let weight = self.gold_weight();
            let speed = if self.player.kill_speed_t > 0.0 {
                self.player.speed * 1.30
            } else {
                self.player.speed
            } * if self.player.slow_t > 0.0 { 0.62 } else { 1.0 }
                * (1.0 - 0.30 * weight);
            self.player.pos += dir * speed * dt;
        }
        // Balanceo de caminata: solo avanza si de verdad se mueve.
        self.player.moving = dir.length_squared() > 0.0 || self.player.dash_t > 0.0;
        self.player.swing_t = (self.player.swing_t - dt).max(0.0);
        if self.player.moving {
            let pace = if self.player.dash_t > 0.0 { 18.0 } else { 9.5 };
            self.player.walk_t += dt * pace;
        } else {
            self.player.walk_t += dt * 1.6;
        }
        if dash_ended && self.player.dash_burst > 0 {
            let radius = 36.0 + self.player.dash_burst as f32 * 6.0;
            let dmg = self.player.damage * 1.1 * (1.0 + self.crystal_bonus());
            self.nova_burst(radius, dmg, C_CYAN);
        }
        self.player.pos.x = self.player.pos.x.clamp(al + 7.0, ar - 7.0);
        self.player.pos.y = self.player.pos.y.clamp(at + 7.0, ab - 7.0);
        let mut pp = self.player.pos;
        push_out_of_obstacles(&mut pp, 6.0, &self.obstacles);
        self.player.pos = pp;

        // Pinchos: daño periódico si te quedas encima.
        self.spike_t -= dt;
        if self.spike_t <= 0.0 {
            let ppos = self.player.pos;
            let on_spikes = self
                .obstacles
                .iter()
                .any(|o| o.kind == ObstacleKind::Spikes && ppos.distance(o.pos) < o.radius + 2.0);
            if on_spikes {
                let dmg = 3.0 + self.floor as f32 * 0.9;
                let (taken, _) = hurt_player(
                    &mut self.player,
                    dmg,
                    &mut self.rng,
                    &mut self.floaters,
                    &mut self.particles,
                    &mut self.shake,
                    sfx,
                    ppos,
                );
                if taken > 0.0 {
                    spawn_ring(&mut self.rings, ppos, 4.0, 18.0, 0.25, C_RED, 1.6);
                    self.spike_t = 0.85;
                }
            }
        }

        if self.current_kind() == RoomKind::Shop {
            self.merchant_t += dt;
            if self.near_merchant() {
                if let Some(i) = self.demo_input.buy {
                    self.buy_shop_offer(i, sfx);
                    self.demo_input.buy = None;
                }
                for i in 0..4 {
                    if is_key_pressed(match i {
                        0 => KeyCode::Key1,
                        1 => KeyCode::Key2,
                        2 => KeyCode::Key3,
                        _ => KeyCode::Key4,
                    }) {
                        self.buy_shop_offer(i, sfx);
                    }
                }
                let m = self.mouse_virtual();
                if is_mouse_button_pressed(MouseButton::Left) {
                    for i in 0..4 {
                        if shop_offer_rect(i).contains(m) {
                            self.buy_shop_offer(i, sfx);
                        }
                    }
                }
                if is_key_pressed(KeyCode::F) || is_mouse_button_pressed(MouseButton::Right) {
                    for i in 0..4 {
                        if shop_offer_rect(i).contains(m) {
                            self.buy_shop_offer_credit(i, sfx);
                        }
                    }
                }
            }
        }

        let interact_pressed = if self.demo_input.active {
            self.demo_input.interact
        } else {
            self.bind_pressed(7)
        };
        if interact_pressed
            && !self.try_open_chest(sfx)
            && !self.try_take_pact(sfx)
            && !self.try_pay_debt(sfx)
        {
            self.try_claim_echo(sfx);
        }

        for dir in 0..4 {
            if self.try_use_door(dir, sfx) {
                return;
            }
        }

        let fog = if self.modifier == Modifier::Fog {
            0.85
        } else {
            1.0
        };
        let range = self.player.range * fog;
        let mut target: Option<Vec2> = None;
        let mut best = range * range;
        for e in &self.enemies {
            if e.dead {
                continue;
            }
            let d = e.pos.distance_squared(self.player.pos);
            if d < best {
                best = d;
                target = Some(e.pos);
            }
        }
        if target.is_none() {
            for p in &self.props {
                if p.dead {
                    continue;
                }
                let d = p.pos.distance_squared(self.player.pos);
                if d < best {
                    best = d;
                    target = Some(p.pos);
                }
            }
        }
        if let Some(t) = target {
            let to = t - self.player.pos;
            if to.length_squared() > 0.01 {
                self.player.facing = safe_norm(to);
            }
            if self.player.attack_t <= 0.0 {
                self.player_attack(t, sfx);
                let weight = self.gold_weight();
                self.player.attack_t =
                    1.0 / (self.player.attack_speed * (1.0 - 0.12 * weight)).max(0.1);
            }
        }

        self.update_skeletons(dt, sfx);
        self.update_ally(dt, sfx);
        self.update_attacks(dt, sfx);
        self.update_enemies(dt, sfx);
        self.update_projectiles(dt, sfx);
        self.update_orbit(dt, sfx);
        self.update_mines(dt, sfx);
        self.update_hazards(dt, sfx);
        self.update_field_effects(dt, sfx);
        self.update_props(dt, sfx);
        self.update_pickups(dt, sfx);
        self.update_particles(dt);
        self.update_floaters(dt);
        self.update_rings(dt);
        self.cleanup_dead(dt, sfx);
        if self.screen != Screen::Playing {
            return;
        }
        self.check_floor_clear(dt, sfx);
        if self.screen == Screen::Playing {
            self.check_level_up();
        }
        if self.player.hp <= 0.0 && self.screen == Screen::Playing {
            if self.demo_input.active {
                // La guía no puede terminar en muerte: se recupera con el
                // cristal agrietado para seguir mostrando el piso.
                self.player.hp = self.player.max_hp * 0.45;
                self.player.invuln = 1.2;
                burst(&mut self.particles, self.player.pos, 18, C_CYAN, 80.0);
                self.shake = 6.0;
                self.toast("El cristal resiste", C_CYAN);
            } else {
                self.player.hp = 0.0;
                burst(&mut self.particles, self.player.pos, 24, C_RED, 120.0);
                self.shake = 10.0;
                self.end_run(false);
            }
        }
    }

    fn player_attack(&mut self, target: Vec2, sfx: &mut Sfx) {
        self.player.swing_t = 0.16;
        let crit = self.rng.chance(self.player.crit);
        let mut dmg = self.player.damage * (1.0 + self.crystal_bonus());
        if crit {
            dmg *= self.player.crit_mult;
        }
        let class = self.class;
        let mut lifesteal = 0.0;
        let mut hit_any = false;

        if class == Class::Knight || class == Class::Assassin {
            let range = self.player.range;
            let dir = safe_norm(target - self.player.pos);
            for i in 0..self.enemies.len() {
                let (epos, erad, ehp, emax) = {
                    let e = &self.enemies[i];
                    (e.pos, e.radius, e.hp, e.max_hp)
                };
                if self.enemies[i].dead {
                    continue;
                }
                let to = epos - self.player.pos;
                if to.length() <= range + erad && safe_norm(to).dot(dir) > 0.10 {
                    let mut d = dmg;
                    if class == Class::Assassin && ehp / emax.max(1.0) < 0.30 {
                        d *= 1.0 + self.player.execute;
                    }
                    self.enemies[i].hp -= d;
                    self.enemies[i].hit_flash = 0.12;
                    lifesteal += d * self.player.lifesteal;
                    hit_any = true;
                    spawn_number(
                        &mut self.floaters,
                        epos + vec2(0.0, -12.0),
                        d.round() as i32,
                        if crit { C_GOLD } else { C_WHITE },
                        8.0,
                    );
                    burst(&mut self.particles, epos, 4, C_RED, 55.0);
                    spawn_ring(
                        &mut self.rings,
                        epos,
                        4.0,
                        if crit { 22.0 } else { 16.0 },
                        0.22,
                        if crit { C_GOLD } else { C_WHITE },
                        1.5,
                    );
                }
            }
            for i in 0..self.props.len() {
                if self.props[i].dead {
                    continue;
                }
                let to = self.props[i].pos - self.player.pos;
                if to.length() <= range + 8.0 && safe_norm(to).dot(dir) > 0.0 {
                    self.props[i].hp -= dmg;
                    self.props[i].hit_flash = 0.15;
                    hit_any = true;
                }
            }
            let base = dir.y.atan2(dir.x);
            for k in 0..6 {
                let a = base + (k as f32 - 2.5) * 0.16;
                spawn_particle(
                    &mut self.particles,
                    self.player.pos + vec2(a.cos(), a.sin()) * 14.0,
                    vec2(a.cos(), a.sin()) * 55.0,
                    0.16,
                    2.0,
                    if crit { C_GOLD } else { C_WHITE },
                    0.0,
                    3.0,
                );
            }
            self.player.attack_count += 1;
            if self.player.shockwave > 0 && self.player.attack_count % 5 == 0 {
                let radius = 52.0 + self.player.shockwave as f32 * 10.0;
                for i in 0..self.enemies.len() {
                    if self.enemies[i].dead {
                        continue;
                    }
                    if self.enemies[i].pos.distance(self.player.pos) < radius {
                        self.enemies[i].hp -= dmg * 1.4;
                        self.enemies[i].hit_flash = 0.15;
                        hit_any = true;
                    }
                }
                burst(
                    &mut self.particles,
                    self.player.pos,
                    18,
                    Color::from_rgba(230, 220, 170, 255),
                    120.0,
                );
                spawn_ring(
                    &mut self.rings,
                    self.player.pos,
                    10.0,
                    radius,
                    0.38,
                    Color::from_rgba(230, 220, 170, 255),
                    2.4,
                );
                self.shake = self.shake.max(4.0);
            }
            if hit_any {
                sfx.play(if crit { SfxKind::Crit } else { SfxKind::Hit }, 0.35);
                self.shake = self.shake.max(1.2);
            }
        } else {
            let dir = safe_norm(target - self.player.pos);
            let count = 1 + self.player.extra_projectiles;
            let spread = 0.12;
            let speed = match class {
                Class::Archer => 330.0,
                Class::Mage => 225.0,
                Class::Necro => 255.0,
                _ => 280.0,
            };
            let color = match class {
                Class::Archer => C_GREEN,
                Class::Mage => Color::from_rgba(255, 150, 60, 255),
                Class::Necro => Color::from_rgba(150, 240, 170, 255),
                _ => C_WHITE,
            };
            for k in 0..count {
                let angle = if count == 1 {
                    0.0
                } else {
                    (k as f32 - (count - 1) as f32 * 0.5) * spread
                };
                let v = rotate(dir, angle) * speed;
                self.projectiles.push(Projectile {
                    pos: self.player.pos + rotate(dir, angle) * 10.0,
                    vel: v,
                    damage: dmg,
                    radius: if class == Class::Mage { 4.2 } else { 2.6 },
                    life: 0.62,
                    friendly: true,
                    color,
                    pierce: self.player.pierce,
                    crit,
                    aoe: if class == Class::Mage {
                        self.player.aoe
                    } else {
                        0.0
                    },
                    slow: 0.0,
                    burn: if class == Class::Mage {
                        self.player.burn_on_hit
                    } else {
                        0.0
                    },
                    chain: self.player.chain,
                    homing: self.player.homing,
                    hits: [0; 8],
                    hit_count: 0,
                });
            }
            self.player.attack_count += 1;
            sfx.play(SfxKind::Shoot, 0.22);
        }

        if self.player.nova > 0 && self.player.attack_count % 5 == 0 {
            let radius = 44.0 + self.player.nova as f32 * 8.0;
            self.nova_burst(radius, dmg * 1.2, Color::from_rgba(180, 130, 255, 255));
        }
        if self.player.mine > 0 && self.player.attack_count % 3 == 0 {
            self.spawn_mine(dmg);
        }
        if self.player.bolas > 0 && self.player.attack_count % 4 == 0 {
            let dir = safe_norm(target - self.player.pos);
            let n = (1 + self.player.bolas).min(4);
            for k in 0..n {
                let angle = (k as f32 - (n - 1) as f32 * 0.5) * 0.30;
                self.projectiles.push(Projectile {
                    pos: self.player.pos + rotate(dir, angle) * 12.0,
                    vel: rotate(dir, angle) * 150.0,
                    damage: dmg * 0.9,
                    radius: 5.0,
                    life: 1.2,
                    friendly: true,
                    color: Color::from_rgba(226, 196, 126, 255),
                    pierce: 2,
                    crit,
                    aoe: 0.0,
                    slow: 1.0,
                    burn: 0.0,
                    chain: 0,
                    homing: 0.0,
                    hits: [0; 8],
                    hit_count: 0,
                });
            }
            sfx.play(SfxKind::Shoot, 0.25);
        }

        if lifesteal > 0.0 {
            self.player.hp = (self.player.hp + lifesteal).min(self.player.max_hp);
        }
    }

    fn nearest_enemy_pos(&self) -> Option<Vec2> {
        let mut best = f32::MAX;
        let mut out = None;
        for e in &self.enemies {
            if e.dead {
                continue;
            }
            let d = e.pos.distance_squared(self.player.pos);
            if d < best {
                best = d;
                out = Some(e.pos);
            }
        }
        out
    }

    /// Los ataques equipados se lanzan solos, todos a la vez. Cada uno lleva su
    /// propia recarga y su nivel escala daño, radio y frecuencia.
    fn update_attacks(&mut self, dt: f32, sfx: &mut Sfx) {
        for b in &mut self.beams {
            b.life -= dt;
        }
        self.beams.retain(|b| b.life > 0.0);
        for m in &mut self.meteors {
            m.t -= dt;
        }
        let ready: Vec<Meteor> = self
            .meteors
            .iter()
            .filter(|m| m.t <= 0.0)
            .copied()
            .collect();
        for m in ready {
            let mut hit = false;
            for e in &mut self.enemies {
                if !e.dead && e.pos.distance(m.pos) < m.radius + e.radius {
                    e.hp -= m.damage;
                    e.hit_flash = 0.18;
                    hit = true;
                }
            }
            burst(&mut self.particles, m.pos, 22, m.color, 130.0);
            spawn_ring(&mut self.rings, m.pos, 6.0, m.radius, 0.45, m.color, 2.6);
            if hit {
                self.shake = self.shake.max(2.4);
                sfx.play(SfxKind::Hit, 0.25);
            }
        }
        self.meteors.retain(|m| m.t > 0.0);

        let mut fired = Vec::new();
        for (slot, e) in self.equipped_attacks.iter_mut().enumerate() {
            e.cd -= dt;
            if e.cd <= 0.0 {
                let def = &self.attacks[e.def];
                let level = e.level;
                e.cd = attacks::level_cd(def.base_cd, level);
                fired.push((slot, e.def, level));
            }
        }
        for (_slot, def_idx, level) in fired {
            let (kind, color, base_damage, base_radius, element) = {
                let d = &self.attacks[def_idx];
                (d.kind, d.color, d.base_damage, d.base_radius, d.element)
            };
            let dmg = self.player.damage
                * attacks::level_damage(base_damage, level)
                * (1.0 + self.crystal_bonus());
            // Las salas se sienten más amplias si los efectos no ocupan media
            // pantalla: se reduce el radio visual y de impacto.
            let radius = attacks::level_radius(base_radius, level) * 0.74;
            let target = self
                .nearest_enemy_pos()
                .unwrap_or(self.player.pos + self.player.facing * 46.0);
            let dir = safe_norm(target - self.player.pos);
            let slow = if matches!(element, Element::Frost | Element::Venom) {
                1.0
            } else {
                0.0
            };
            let burn = if matches!(element, Element::Ember | Element::Blood) {
                dmg * 0.18
            } else {
                0.0
            };
            match kind {
                AttackKind::Bolt => {
                    self.projectiles.push(Projectile {
                        pos: self.player.pos + dir * 10.0,
                        vel: dir * 260.0,
                        damage: dmg,
                        radius: 3.2,
                        life: 1.2,
                        friendly: true,
                        color,
                        pierce: 0,
                        crit: false,
                        aoe: 0.0,
                        slow,
                        burn,
                        chain: 0,
                        homing: 2.4,
                        hits: [0; 8],
                        hit_count: 0,
                    });
                }
                AttackKind::Chain => {
                    self.projectiles.push(Projectile {
                        pos: self.player.pos + dir * 10.0,
                        vel: dir * 235.0,
                        damage: dmg,
                        radius: 3.4,
                        life: 1.1,
                        friendly: true,
                        color,
                        pierce: 0,
                        crit: false,
                        aoe: 0.0,
                        slow,
                        burn,
                        chain: 1 + level / 2,
                        homing: 1.0,
                        hits: [0; 8],
                        hit_count: 0,
                    });
                }
                AttackKind::Swarm => {
                    let n = 2 + level / 2;
                    for k in 0..n {
                        let a = (k as f32 - (n - 1) as f32 * 0.5) * 0.22;
                        let v = rotate(dir, a) * 210.0;
                        self.projectiles.push(Projectile {
                            pos: self.player.pos + rotate(dir, a) * 10.0,
                            vel: v,
                            damage: dmg,
                            radius: 2.4,
                            life: 1.4,
                            friendly: true,
                            color,
                            pierce: 0,
                            crit: false,
                            aoe: 0.0,
                            slow,
                            burn,
                            chain: 0,
                            homing: 3.2,
                            hits: [0; 8],
                            hit_count: 0,
                        });
                    }
                }
                AttackKind::Bolas => {
                    let n = 1 + level / 2;
                    for k in 0..n {
                        let a = (k as f32 - (n - 1) as f32 * 0.5) * 0.28;
                        self.projectiles.push(Projectile {
                            pos: self.player.pos + rotate(dir, a) * 12.0,
                            vel: rotate(dir, a) * 165.0,
                            damage: dmg,
                            radius: 5.0,
                            life: 1.3,
                            friendly: true,
                            color,
                            pierce: 2,
                            crit: false,
                            aoe: 0.0,
                            slow: 1.0,
                            burn,
                            chain: 0,
                            homing: 0.0,
                            hits: [0; 8],
                            hit_count: 0,
                        });
                    }
                }
                AttackKind::Beam => {
                    let len = radius;
                    for e in &mut self.enemies {
                        if e.dead {
                            continue;
                        }
                        let rel = e.pos - self.player.pos;
                        let t = rel.dot(dir).clamp(0.0, len);
                        let closest = self.player.pos + dir * t;
                        if e.pos.distance(closest) < 9.0 + e.radius {
                            e.hp -= dmg;
                            e.hit_flash = 0.16;
                        }
                    }
                    self.beams.push(BeamFx {
                        pos: self.player.pos,
                        dir,
                        length: len,
                        width: 4.5,
                        life: 0.16,
                        max_life: 0.16,
                        color,
                    });
                    self.shake = self.shake.max(1.6);
                    sfx.play(SfxKind::Shoot, 0.2);
                }
                AttackKind::Meteor => {
                    self.meteors.push(Meteor {
                        pos: target,
                        t: 0.55,
                        max_t: 0.55,
                        radius,
                        damage: dmg,
                        color,
                    });
                }
                AttackKind::Drain => {
                    let mut best = f32::MAX;
                    let mut hit = None;
                    for (i, e) in self.enemies.iter().enumerate() {
                        if e.dead {
                            continue;
                        }
                        let d = e.pos.distance_squared(self.player.pos);
                        if d < best && d < radius * radius {
                            best = d;
                            hit = Some(i);
                        }
                    }
                    if let Some(i) = hit {
                        self.enemies[i].hp -= dmg;
                        self.enemies[i].hit_flash = 0.16;
                        let heal = dmg * 0.35;
                        self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                        let epos = self.enemies[i].pos;
                        spawn_ring(&mut self.rings, epos, 4.0, 16.0, 0.25, color, 1.6);
                        spawn_floater(
                            &mut self.floaters,
                            self.player.pos + vec2(0.0, -18.0),
                            &format!("+{:.0}", heal),
                            C_GREEN,
                            8.0,
                        );
                        sfx.play(SfxKind::Hit, 0.16);
                    }
                }
                AttackKind::Mine | AttackKind::Spikes => {
                    self.mines.push(Mine {
                        pos: if kind == AttackKind::Spikes {
                            target
                        } else {
                            self.player.pos
                        },
                        t: if kind == AttackKind::Spikes {
                            0.55
                        } else {
                            1.1
                        },
                        damage: dmg * if kind == AttackKind::Spikes { 1.4 } else { 1.8 },
                        radius,
                        color,
                    });
                    if self.mines.len() > 32 {
                        self.mines.remove(0);
                    }
                }
                AttackKind::Trail => {
                    self.hazards.push(Hazard {
                        pos: self.player.pos,
                        radius,
                        life: 1.5,
                        max_life: 1.5,
                        dps: dmg * 1.6,
                        tick: 0.0,
                        pull: 0.0,
                        color,
                        kind: HazardKind::Trail,
                    });
                    if self.hazards.len() > 48 {
                        self.hazards.remove(0);
                    }
                }
                AttackKind::Vortex => {
                    self.hazards.push(Hazard {
                        pos: target,
                        radius,
                        life: 2.0,
                        max_life: 2.0,
                        dps: dmg * 0.9,
                        tick: 0.0,
                        pull: 90.0 + level as f32 * 22.0,
                        color,
                        kind: HazardKind::Vortex,
                    });
                    spawn_ring(&mut self.rings, target, 8.0, radius, 0.35, color, 2.0);
                }
                AttackKind::Aura | AttackKind::Orbit => {
                    let mut hit = false;
                    for e in &mut self.enemies {
                        if !e.dead && e.pos.distance(self.player.pos) < radius + e.radius {
                            e.hp -= dmg;
                            e.hit_flash = 0.10;
                            hit = true;
                        }
                    }
                    if hit {
                        spawn_ring(
                            &mut self.rings,
                            self.player.pos,
                            radius * 0.55,
                            radius,
                            0.26,
                            with_alpha(color, 0.75),
                            1.6,
                        );
                        sfx.play(SfxKind::Hit, 0.10);
                    }
                }
                AttackKind::Nova | AttackKind::Shockwave => {
                    self.nova_burst(radius, dmg, color);
                    sfx.play(SfxKind::Break, 0.18);
                }
                AttackKind::Mend => {
                    let heal = self.player.max_hp * (0.035 + 0.010 * level as f32);
                    if self.player.hp < self.player.max_hp {
                        self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                        spawn_ring(
                            &mut self.rings,
                            self.player.pos,
                            6.0,
                            radius,
                            0.4,
                            color,
                            1.8,
                        );
                        spawn_floater(
                            &mut self.floaters,
                            self.player.pos + vec2(0.0, -18.0),
                            &format!("+{:.0}", heal),
                            C_GREEN,
                            8.0,
                        );
                        sfx.play(SfxKind::Level, 0.12);
                    }
                }
            }
        }
    }

    fn nova_burst(&mut self, radius: f32, dmg: f32, color: Color) {
        let ppos = self.player.pos;
        for e in &mut self.enemies {
            if !e.dead && e.pos.distance(ppos) < radius + e.radius {
                e.hp -= dmg;
                e.hit_flash = 0.14;
            }
        }
        burst(&mut self.particles, ppos, 20, color, 130.0);
        spawn_ring(&mut self.rings, ppos, 8.0, radius, 0.42, color, 2.4);
        self.shake = self.shake.max(2.6);
    }

    fn spawn_mine(&mut self, dmg: f32) {
        self.mines.push(Mine {
            pos: self.player.pos,
            t: 1.15,
            damage: dmg * 1.9,
            radius: 38.0,
            color: Color::from_rgba(190, 130, 255, 255),
        });
        if self.mines.len() > 24 {
            self.mines.remove(0);
        }
    }

    fn update_mines(&mut self, dt: f32, sfx: &mut Sfx) {
        let mut exploded = false;
        for m in &mut self.mines {
            m.t -= dt;
            if m.t <= 0.0 {
                let (pos, radius, damage, color) = (m.pos, m.radius, m.damage, m.color);
                for e in &mut self.enemies {
                    if !e.dead && e.pos.distance(pos) < radius + e.radius {
                        e.hp -= damage;
                        e.hit_flash = 0.16;
                    }
                }
                burst(&mut self.particles, pos, 18, color, 120.0);
                spawn_ring(&mut self.rings, pos, 6.0, radius, 0.42, color, 2.2);
                self.shake = self.shake.max(2.2);
                exploded = true;
            }
        }
        if exploded {
            self.mines.retain(|m| m.t > 0.0);
            sfx.play(SfxKind::Break, 0.24);
        }
    }

    fn update_hazards(&mut self, dt: f32, sfx: &mut Sfx) {
        let enemies = &mut self.enemies;
        let hazards = &mut self.hazards;
        let particles = &mut self.particles;
        let mut burned = false;
        for h in hazards.iter_mut() {
            h.life -= dt;
            h.tick -= dt;
            if h.pull > 0.0 {
                for e in enemies.iter_mut() {
                    if e.dead {
                        continue;
                    }
                    let d = h.pos - e.pos;
                    let dist = d.length();
                    if dist > 2.0 && dist < h.radius * 1.9 {
                        let resist = if e.boss { 0.22 } else { 1.0 };
                        let k = (h.pull * resist * dt / dist).min(0.35);
                        e.pos += d * k;
                    }
                }
            }
            if h.tick <= 0.0 {
                h.tick = 0.25;
                let tick_dmg = h.dps * 0.25;
                for e in enemies.iter_mut() {
                    if e.dead {
                        continue;
                    }
                    if e.pos.distance(h.pos) < h.radius + e.radius {
                        e.hp -= tick_dmg;
                        e.hit_flash = 0.06;
                        burned = true;
                    }
                }
                if h.kind == HazardKind::Vortex {
                    for k in 0..3 {
                        let a = self.time * 3.0 + k as f32 * std::f32::consts::TAU / 3.0;
                        spawn_particle(
                            particles,
                            h.pos + vec2(a.cos(), a.sin()) * h.radius * 0.6,
                            vec2(-a.sin(), a.cos()) * 22.0,
                            0.3,
                            1.4,
                            with_alpha(h.color, 0.65),
                            0.0,
                            1.0,
                        );
                    }
                }
            }
        }
        self.hazards.retain(|h| h.life > 0.0);
        if burned && self.rng.chance(0.25) {
            sfx.play(SfxKind::Hit, 0.08);
        }
    }

    fn update_field_effects(&mut self, dt: f32, sfx: &mut Sfx) {
        let ppos = self.player.pos;
        let dmg = self.player.damage * (1.0 + self.crystal_bonus());

        // Rastro de brasas: una estela ardiente que queda en el suelo.
        if self.player.trail > 0 {
            self.player.trail_cd -= dt;
            if self.player.trail_cd <= 0.0 {
                self.player.trail_cd = 0.26;
                self.hazards.push(Hazard {
                    pos: ppos,
                    radius: 7.0 + self.player.trail as f32 * 1.6,
                    life: 1.5,
                    max_life: 1.5,
                    dps: dmg * 0.20 * self.player.trail as f32,
                    tick: 0.0,
                    pull: 0.0,
                    color: Color::from_rgba(255, 132, 52, 255),
                    kind: HazardKind::Trail,
                });
                if self.hazards.len() > 42 {
                    self.hazards.remove(0);
                }
            }
        }

        // Vórtice: un remolino periódico que arrastra a los enemigos al centro.
        if self.player.vortex > 0 {
            self.player.vortex_cd -= dt;
            if self.player.vortex_cd <= 0.0 {
                self.player.vortex_cd = (7.2 - self.player.vortex as f32 * 0.45).max(3.4);
                self.hazards.push(Hazard {
                    pos: ppos,
                    radius: 32.0 + self.player.vortex as f32 * 5.0,
                    life: 2.1,
                    max_life: 2.1,
                    dps: dmg * 0.26,
                    tick: 0.0,
                    pull: 120.0 + self.player.vortex as f32 * 26.0,
                    color: C_PURPLE,
                    kind: HazardKind::Vortex,
                });
                spawn_ring(
                    &mut self.rings,
                    ppos,
                    10.0,
                    40.0 + self.player.vortex as f32 * 5.0,
                    0.35,
                    C_PURPLE,
                    2.0,
                );
                sfx.play(SfxKind::Portal, 0.16);
            }
        }

        // Aura: daño constante a todo lo que se acerque demasiado.
        if self.player.aura > 0 {
            self.player.aura_tick -= dt;
            if self.player.aura_tick <= 0.0 {
                self.player.aura_tick = 0.65;
                let radius = 28.0 + self.player.aura as f32 * 6.5;
                let tick_dmg = dmg * 0.16 * self.player.aura as f32;
                let mut hit = false;
                for e in &mut self.enemies {
                    if !e.dead && e.pos.distance(ppos) < radius + e.radius {
                        e.hp -= tick_dmg;
                        e.hit_flash = 0.08;
                        hit = true;
                    }
                }
                if hit {
                    spawn_ring(
                        &mut self.rings,
                        ppos,
                        radius * 0.55,
                        radius,
                        0.28,
                        with_alpha(C_PURPLE, 0.65),
                        1.4,
                    );
                    sfx.play(SfxKind::Hit, 0.10);
                }
            }
        }

        // Remiendo: pulsos lentos de curación mientras sigas con vida.
        if self.player.mend > 0 {
            self.player.mend_cd -= dt;
            if self.player.mend_cd <= 0.0 {
                self.player.mend_cd = (9.0 - self.player.mend as f32 * 0.7).max(4.5);
                let heal = self.player.max_hp * (0.012 + 0.006 * self.player.mend as f32);
                if self.player.hp < self.player.max_hp {
                    self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                    spawn_ring(&mut self.rings, ppos, 6.0, 28.0, 0.4, C_GREEN, 1.8);
                    spawn_floater(
                        &mut self.floaters,
                        ppos + vec2(0.0, -18.0),
                        &format!("+{:.0}", heal),
                        C_GREEN,
                        8.0,
                    );
                    sfx.play(SfxKind::Pickup, 0.14);
                }
            }
        }
    }

    fn update_orbit(&mut self, dt: f32, sfx: &mut Sfx) {
        if self.player.orbit == 0 {
            return;
        }
        self.player.orbit_cd -= dt;
        if self.player.orbit_cd > 0.0 {
            return;
        }
        self.player.orbit_cd = 0.42;
        let radius = 25.0 + self.player.orbit as f32 * 3.5;
        let dmg = self.player.damage * 0.35 * (1.0 + self.crystal_bonus());
        let mut hit = false;
        for e in &mut self.enemies {
            if !e.dead && e.pos.distance(self.player.pos) < radius + e.radius {
                e.hp -= dmg;
                e.hit_flash = 0.12;
                hit = true;
            }
        }
        if hit {
            sfx.play(SfxKind::Hit, 0.18);
        }
    }

    fn update_ally(&mut self, dt: f32, sfx: &mut Sfx) {
        let Some(ally) = &mut self.ally else {
            return;
        };
        ally.orbit_t += dt;
        let ang = ally.orbit_t * 0.9 + ally.kind as f32 * 1.4;
        let target = self.player.pos + vec2(ang.cos() * 22.0, ang.sin() * 12.0 - 4.0);
        ally.pos += (target - ally.pos) * (dt * 4.0).min(1.0);
        ally.bob += dt * 5.0;
        ally.swing_t = (ally.swing_t - dt).max(0.0);
        ally.attack_t -= dt;
        if ally.attack_t > 0.0 {
            return;
        }
        let kind = ally.kind;
        let apos = ally.pos;
        let color = ally.color();
        match kind {
            1 => {
                // Escudero: tajo al enemigo más cercano.
                let mut best = 80.0 * 80.0;
                let mut hit = None;
                for (i, e) in self.enemies.iter().enumerate() {
                    if e.dead {
                        continue;
                    }
                    let d = e.pos.distance_squared(apos);
                    if d < best {
                        best = d;
                        hit = Some(i);
                    }
                }
                if let Some(i) = hit {
                    let dmg = self.player.damage * 0.70;
                    let epos = self.enemies[i].pos;
                    self.enemies[i].hp -= dmg;
                    self.enemies[i].hit_flash = 0.12;
                    spawn_number(
                        &mut self.floaters,
                        epos + vec2(0.0, -12.0),
                        dmg.round() as i32,
                        color,
                        6.0,
                    );
                    burst(&mut self.particles, epos, 10, color, 90.0);
                    spawn_ring(&mut self.rings, epos, 4.0, 16.0, 0.22, color, 1.6);
                    sfx.play(SfxKind::Hit, 0.10);
                }
                ally.attack_t = 1.0;
                ally.swing_t = 0.22;
            }
            2 => {
                // Farolero: cura periódica.
                let heal = self.player.max_hp * 0.025;
                if self.player.hp < self.player.max_hp {
                    self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                    spawn_number(
                        &mut self.floaters,
                        self.player.pos + vec2(0.0, -18.0),
                        heal.round() as i32,
                        C_GREEN,
                        6.0,
                    );
                    burst(&mut self.particles, self.player.pos, 8, color, 50.0);
                }
                ally.attack_t = 2.2;
                ally.swing_t = 0.28;
            }
            3 => {
                // Cuervo: picotea y suelta una moneda de vez en cuando.
                if self.rng.chance(0.55) {
                    let mut best = 120.0 * 120.0;
                    let mut hit = None;
                    for (i, e) in self.enemies.iter().enumerate() {
                        if e.dead {
                            continue;
                        }
                        let d = e.pos.distance_squared(apos);
                        if d < best {
                            best = d;
                            hit = Some(i);
                        }
                    }
                    if let Some(i) = hit {
                        let epos = self.enemies[i].pos;
                        self.enemies[i].hp -= self.player.damage * 0.25;
                        self.enemies[i].hit_flash = 0.10;
                        burst(&mut self.particles, epos, 6, color, 70.0);
                        self.pickups.push(Pickup {
                            pos: epos,
                            vel: vec2(self.rng.range(-10.0, 10.0), -20.0),
                            kind: PickupKind::Gold(1 + self.floor / 4),
                            life: 14.0,
                            bob: 0.0,
                        });
                    }
                }
                ally.attack_t = 2.6;
                ally.swing_t = 0.22;
            }
            4 => {
                // Verdugo: saeta espectral instantánea.
                let mut best = 170.0 * 170.0;
                let mut hit = None;
                for (i, e) in self.enemies.iter().enumerate() {
                    if e.dead {
                        continue;
                    }
                    let d = e.pos.distance_squared(apos);
                    if d < best {
                        best = d;
                        hit = Some(i);
                    }
                }
                if let Some(i) = hit {
                    let dmg = self.player.damage * 0.55;
                    let epos = self.enemies[i].pos;
                    self.enemies[i].hp -= dmg;
                    self.enemies[i].hit_flash = 0.14;
                    spawn_number(
                        &mut self.floaters,
                        epos + vec2(0.0, -12.0),
                        dmg.round() as i32,
                        color,
                        6.0,
                    );
                    let dir = safe_norm(epos - apos);
                    self.beams.push(BeamFx {
                        pos: apos,
                        dir,
                        length: apos.distance(epos),
                        width: 1.4,
                        life: 0.16,
                        max_life: 0.16,
                        color,
                    });
                    burst(&mut self.particles, epos, 8, color, 80.0);
                    sfx.play(SfxKind::Hit, 0.12);
                }
                ally.attack_t = 1.5;
                ally.swing_t = 0.20;
            }
            _ => {
                ally.attack_t = 1.0;
            }
        }
    }

    fn update_skeletons(&mut self, dt: f32, sfx: &mut Sfx) {
        self.skeleton_respawn_t -= dt;
        if self.class != Class::Necro {
            return;
        }
        let player_pos = self.player.pos;
        let time = self.time;
        let explode_flag = self.player.skeleton_explode;
        let mut died = false;
        {
            let enemies = &mut self.enemies;
            let skeletons = &mut self.skeletons;
            let particles = &mut self.particles;
            let floaters = &mut self.floaters;
            let mut i = 0;
            while i < skeletons.len() {
                let expired = {
                    let s = &mut skeletons[i];
                    s.life -= dt;
                    s.attack_t -= dt;
                    s.life <= 0.0
                };
                if expired {
                    let pos = skeletons[i].pos;
                    let dmg = skeletons[i].damage;
                    burst(particles, pos, 8, C_GREEN, 60.0);
                    if explode_flag {
                        for e in enemies.iter_mut() {
                            if !e.dead && e.pos.distance(pos) < 32.0 {
                                e.hp -= dmg * 2.5;
                                e.hit_flash = 0.12;
                            }
                        }
                        burst(
                            particles,
                            pos,
                            12,
                            Color::from_rgba(205, 255, 205, 255),
                            95.0,
                        );
                    }
                    skeletons.swap_remove(i);
                    died = true;
                    continue;
                }

                let mut best = f32::MAX;
                let mut target = None;
                for e in enemies.iter() {
                    if e.dead {
                        continue;
                    }
                    let d = e.pos.distance_squared(skeletons[i].pos);
                    if d < best {
                        best = d;
                        target = Some(e.pos);
                    }
                }
                if let Some(tpos) = target {
                    let to = tpos - skeletons[i].pos;
                    let dist = to.length();
                    if dist > 15.0 {
                        skeletons[i].vel = safe_norm(to) * 88.0;
                    } else {
                        skeletons[i].vel *= 0.7;
                        if skeletons[i].attack_t <= 0.0 {
                            let mut bi = None;
                            let mut bd = f32::MAX;
                            for (ei, e) in enemies.iter().enumerate() {
                                if e.dead {
                                    continue;
                                }
                                let d = e.pos.distance_squared(skeletons[i].pos);
                                if d < bd {
                                    bd = d;
                                    bi = Some(ei);
                                }
                            }
                            if let Some(ei) = bi {
                                if bd < 30.0 * 30.0 {
                                    let dmg = skeletons[i].damage;
                                    enemies[ei].hp -= dmg;
                                    enemies[ei].hit_flash = 0.10;
                                    let epos = enemies[ei].pos;
                                    spawn_number(
                                        floaters,
                                        epos + vec2(0.0, -10.0),
                                        dmg.round() as i32,
                                        C_GREEN,
                                        8.0,
                                    );
                                    burst(particles, epos, 2, C_GREEN, 35.0);
                                    skeletons[i].attack_t = 0.8;
                                    sfx.play(SfxKind::Hit, 0.10);
                                }
                            }
                        }
                    }
                } else {
                    let a = skeletons[i].bob + time * 1.6;
                    let orbit = player_pos + vec2(a.cos(), a.sin()) * 26.0;
                    let to = orbit - skeletons[i].pos;
                    if to.length() > 3.0 {
                        skeletons[i].vel = safe_norm(to) * 70.0;
                    } else {
                        skeletons[i].vel *= 0.7;
                    }
                }
                let svel = skeletons[i].vel;
                skeletons[i].pos += svel * dt;
                skeletons[i].pos.x = skeletons[i].pos.x.clamp(ARENA_L + 6.0, ARENA_R - 6.0);
                skeletons[i].pos.y = skeletons[i].pos.y.clamp(ARENA_T + 6.0, ARENA_B - 6.0);
                i += 1;
            }
        }
        if died {
            self.skeleton_respawn_t = 1.2;
            // Al morir uno, el resto recupera su cuota de daño.
            self.refresh_skeleton_damage();
        }
        if self.skeleton_respawn_t <= 0.0
            && self.skeletons.len() < self.player.skeleton_cap as usize
        {
            self.spawn_skeleton();
            self.skeleton_respawn_t = 1.2;
        }
    }

    fn update_enemies(&mut self, dt: f32, sfx: &mut Sfx) {
        let player_pos = self.player.pos;
        let floor = self.floor;
        let dark = if self.modifier == Modifier::Darkness {
            1.2
        } else {
            1.0
        };
        let (al, ar, at, ab) = self.arena();
        let obstacles = self.obstacles.clone();
        let mut summons: Vec<(EnemyKind, Vec2)> = Vec::new();
        let enemies = &mut self.enemies;
        let projectiles = &mut self.projectiles;
        let particles = &mut self.particles;
        let floaters = &mut self.floaters;
        let player = &mut self.player;
        let rng = &mut self.rng;
        let shake = &mut self.shake;
        let rings = &mut self.rings;

        for i in 0..enemies.len() {
            if enemies[i].dead {
                continue;
            }
            enemies[i].spawn_t -= dt;
            enemies[i].walk_t += dt * (enemies[i].speed * 0.06).max(1.2);
            if enemies[i].spawn_t > 0.0 {
                continue;
            }
            enemies[i].hit_flash -= dt;
            enemies[i].attack_t -= dt;
            enemies[i].special_t -= dt;
            enemies[i].signature_t -= dt;
            enemies[i].cast_t = (enemies[i].cast_t - dt).max(0.0);
            enemies[i].swing_t = (enemies[i].swing_t - dt).max(0.0);
            enemies[i].burn_t -= dt;
            if enemies[i].burn_t > 0.0 {
                enemies[i].hp -= enemies[i].burn_dps * dt;
                if rng.chance(0.10) {
                    spawn_particle(
                        particles,
                        enemies[i].pos + vec2(rng.range(-4.0, 4.0), rng.range(-7.0, 0.0)),
                        vec2(0.0, -30.0),
                        0.35,
                        1.5,
                        Color::from_rgba(255, 140, 50, 255),
                        0.0,
                        1.0,
                    );
                }
                if enemies[i].hp <= 0.0 {
                    continue;
                }
            }
            if enemies[i].shiny {
                enemies[i].life -= dt;
                if enemies[i].life <= 0.0 {
                    let pos = enemies[i].pos;
                    burst(particles, pos, 8, C_CYAN, 60.0);
                    enemies[i].dead = true;
                    continue;
                }
            }

            let to = player_pos - enemies[i].pos;
            let dist = to.length();
            let n = safe_norm(to);
            let slow = if enemies[i].slow_t > 0.0 { 0.55 } else { 1.0 };
            let speed = enemies[i].speed * dark * slow;
            let element = enemies[i].element;
            let edmg = enemies[i].damage
                * match element {
                    Element::Ember => 1.20,
                    Element::Blood => 1.10,
                    Element::Void => 1.10,
                    _ => 1.0,
                };
            let shot_slow = match element {
                Element::Frost => 1.4,
                Element::Venom => 0.8,
                _ => 0.0,
            };
            let color = element.color();
            let mut move_dir = Vec2::ZERO;
            let mut contact = false;
            let mut dashing = false;
            // Salto de guardián: aviso, vuelo y onda de impacto. Mientras está
            // en el aire ignora obstáculos y no puede ser empujado por ellos.
            let mut jumping = false;
            let mut jump_pos: Option<Vec2> = None;
            if enemies[i].boss {
                if enemies[i].jump_t > 0.0 {
                    jumping = true;
                    enemies[i].jump_t -= dt;
                    let max_t = 0.62;
                    let k = 1.0 - (enemies[i].jump_t / max_t).clamp(0.0, 1.0);
                    let e = k * k * (3.0 - 2.0 * k);
                    jump_pos = Some(enemies[i].jump_from.lerp(enemies[i].jump_to, e));
                    if enemies[i].jump_t <= 0.0 {
                        let landing = enemies[i].jump_to;
                        spawn_ring(rings, landing, 8.0, 62.0, 0.5, color, 3.0);
                        spawn_ring(rings, landing, 4.0, 34.0, 0.34, C_GOLD, 2.0);
                        burst(particles, landing, 18, color, 110.0);
                        *shake = (*shake).max(8.0);
                        sfx.play(SfxKind::Boss, 0.30);
                        for k in 0..10 {
                            let a = k as f32 * std::f32::consts::TAU / 10.0 + rng.range(0.0, 0.2);
                            projectiles.push(enemy_shot(
                                landing,
                                vec2(a.cos(), a.sin()) * 112.0,
                                edmg * 0.72,
                                color,
                                4.2,
                                2.8,
                                0.0,
                                shot_slow,
                                0.0,
                            ));
                        }
                        enemies[i].jump_cd = 4.0 + rng.range(0.0, 1.4);
                    }
                } else if enemies[i].jump_wind > 0.0 {
                    jumping = true;
                    enemies[i].jump_wind -= dt;
                    if enemies[i].jump_wind <= 0.0 {
                        enemies[i].jump_from = enemies[i].pos;
                        enemies[i].jump_t = 0.62;
                        burst(particles, enemies[i].pos, 10, color, 70.0);
                    }
                } else {
                    enemies[i].jump_cd -= dt;
                    let moved = enemies[i].pos.distance(enemies[i].last_pos);
                    if moved < 1.0 && dist > 36.0 {
                        enemies[i].stuck_t += dt;
                    } else {
                        enemies[i].stuck_t = (enemies[i].stuck_t - dt * 2.0).max(0.0);
                    }
                    let want_jump = (enemies[i].jump_cd <= 0.0 && dist > 44.0 && dist < 250.0)
                        || enemies[i].stuck_t > 0.60;
                    if want_jump {
                        enemies[i].jump_wind = 0.52;
                        enemies[i].stuck_t = 0.0;
                        let mut target = player_pos;
                        target.x = target.x.clamp(al + 20.0, ar - 20.0);
                        target.y = target.y.clamp(at + 20.0, ab - 20.0);
                        enemies[i].jump_to = target;
                        spawn_ring(rings, target, 4.0, 28.0, 0.6, color, 2.0);
                        sfx.play_enemy(enemies[i].voice, 0.22);
                    }
                }
            }
            if jumping {
                move_dir = Vec2::ZERO;
                contact = false;
            }

            let shots_before = projectiles.len();
            if !jumping {
                match enemies[i].attack {
                    EnemyAttack::Melee => {
                        move_dir = n;
                        contact = dist < enemies[i].radius + 14.0;
                    }
                    EnemyAttack::SlowAura => {
                        move_dir = n * 0.8;
                        contact = dist < enemies[i].radius + 12.0;
                        if dist < 82.0 {
                            player.slow_t = player.slow_t.max(0.25);
                            if rng.chance(0.12) {
                                let a = rng.range(0.0, std::f32::consts::TAU);
                                spawn_particle(
                                    particles,
                                    enemies[i].pos + vec2(a.cos(), a.sin()) * enemies[i].radius,
                                    vec2(0.0, -8.0),
                                    0.4,
                                    1.6,
                                    with_alpha(color, 0.5),
                                    0.0,
                                    0.6,
                                );
                            }
                        }
                    }
                    EnemyAttack::Charge => {
                        if enemies[i].state_t > 0.0 {
                            enemies[i].state_t -= dt;
                            dashing = true;
                            contact = dist < enemies[i].radius + 15.0;
                        } else if enemies[i].special_t <= 0.0 && dist < 190.0 {
                            enemies[i].special_t = enemies[i].attack_cd + 0.9;
                            enemies[i].state_t = 0.26;
                            enemies[i].charge_dir = n;
                            sfx.play_enemy(enemies[i].voice, 0.22);
                        } else {
                            move_dir = n * 0.65;
                        }
                    }
                    EnemyAttack::Bolt => {
                        move_dir = if dist < 95.0 {
                            -n
                        } else if dist > 145.0 {
                            n
                        } else {
                            vec2(-n.y, n.x) * 0.6
                        };
                        if enemies[i].attack_t <= 0.0 && dist < 200.0 {
                            projectiles.push(enemy_shot(
                                enemies[i].pos,
                                n * 135.0,
                                edmg,
                                color,
                                3.5,
                                2.6,
                                0.0,
                                shot_slow,
                                0.0,
                            ));
                            enemies[i].attack_t = enemies[i].attack_cd;
                            sfx.play_enemy(enemies[i].voice, 0.18);
                            sfx.play(SfxKind::Shoot, 0.12);
                        }
                    }
                    EnemyAttack::Spread => {
                        move_dir = if dist < 90.0 {
                            -n
                        } else if dist > 150.0 {
                            n
                        } else {
                            vec2(-n.y, n.x) * 0.5
                        };
                        if enemies[i].attack_t <= 0.0 && dist < 210.0 {
                            let base = n.y.atan2(n.x);
                            for k in 0..5 {
                                let a = base + (k as f32 - 2.0) * 0.16;
                                projectiles.push(enemy_shot(
                                    enemies[i].pos,
                                    vec2(a.cos(), a.sin()) * 125.0,
                                    edmg * 0.8,
                                    color,
                                    3.0,
                                    2.4,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                            enemies[i].attack_t = enemies[i].attack_cd;
                            sfx.play_enemy(enemies[i].voice, 0.18);
                            sfx.play(SfxKind::Shoot, 0.12);
                        }
                    }
                    EnemyAttack::Homing => {
                        move_dir = if dist < 110.0 {
                            -n
                        } else if dist > 170.0 {
                            n
                        } else {
                            vec2(-n.y, n.x) * 0.5
                        };
                        if enemies[i].attack_t <= 0.0 && dist < 220.0 {
                            projectiles.push(enemy_shot(
                                enemies[i].pos,
                                n * 85.0,
                                edmg,
                                color,
                                4.0,
                                4.0,
                                2.2,
                                shot_slow,
                                0.0,
                            ));
                            enemies[i].attack_t = enemies[i].attack_cd;
                            sfx.play_enemy(enemies[i].voice, 0.20);
                            sfx.play(SfxKind::Shoot, 0.12);
                        }
                    }
                    EnemyAttack::Bomb => {
                        move_dir = if dist < 80.0 {
                            -n
                        } else if dist > 120.0 {
                            n
                        } else {
                            vec2(-n.y, n.x) * 0.7
                        };
                        if enemies[i].attack_t <= 0.0 && dist < 180.0 {
                            projectiles.push(enemy_shot(
                                enemies[i].pos,
                                n * 95.0,
                                edmg * 1.1,
                                color,
                                5.0,
                                1.8,
                                0.0,
                                0.0,
                                30.0,
                            ));
                            enemies[i].attack_t = enemies[i].attack_cd;
                            sfx.play_enemy(enemies[i].voice, 0.22);
                        }
                    }
                    EnemyAttack::Summon => {
                        move_dir = if dist < 120.0 { -n * 0.5 } else { n * 0.5 };
                        if enemies[i].special_t <= 0.0 {
                            enemies[i].special_t = 6.0;
                            for k in 0..2 {
                                let a = rng.range(0.0, std::f32::consts::TAU)
                                    + k as f32 * std::f32::consts::PI;
                                summons.push((
                                    if rng.chance(0.5) {
                                        EnemyKind::Bat
                                    } else {
                                        EnemyKind::Wraith
                                    },
                                    enemies[i].pos + vec2(a.cos(), a.sin()) * 26.0,
                                ));
                            }
                            spawn_ring(rings, enemies[i].pos, 6.0, 34.0, 0.5, color, 2.0);
                            sfx.play_enemy(enemies[i].voice, 0.30);
                        }
                        if enemies[i].attack_t <= 0.0 && dist < 200.0 {
                            projectiles.push(enemy_shot(
                                enemies[i].pos,
                                n * 110.0,
                                edmg,
                                color,
                                4.0,
                                2.4,
                                0.0,
                                shot_slow,
                                0.0,
                            ));
                            enemies[i].attack_t = enemies[i].attack_cd;
                        }
                    }
                    EnemyAttack::Teleport => {
                        contact = dist < enemies[i].radius + 12.0;
                        if enemies[i].state_t > 0.0 {
                            // Aviso de teletransporte: se queda quieto, marca el
                            // destino y da tiempo a reaccionar.
                            enemies[i].state_t -= dt;
                            move_dir = Vec2::ZERO;
                            let p = enemies[i].teleport_to;
                            if rng.chance(0.35) {
                                let a = rng.range(0.0, std::f32::consts::TAU);
                                spawn_particle(
                                    particles,
                                    p + vec2(a.cos(), a.sin()) * rng.range(6.0, 20.0),
                                    vec2(0.0, -14.0),
                                    0.35,
                                    1.6,
                                    with_alpha(color, 0.75),
                                    0.0,
                                    1.0,
                                );
                            }
                            if enemies[i].state_t <= 0.0 {
                                enemies[i].pos = p;
                                burst(particles, p, 16, color, 80.0);
                                spawn_ring(rings, p, 4.0, 26.0, 0.35, color, 2.0);
                                sfx.play_enemy(enemies[i].voice, 0.25);
                                for k in 0..4 {
                                    let a2 = k as f32 * std::f32::consts::TAU / 4.0;
                                    projectiles.push(enemy_shot(
                                        p,
                                        vec2(a2.cos(), a2.sin()) * 105.0,
                                        edmg * 0.7,
                                        color,
                                        3.0,
                                        2.0,
                                        0.0,
                                        shot_slow,
                                        0.0,
                                    ));
                                }
                            }
                        } else {
                            move_dir = n * 0.35;
                            if enemies[i].special_t <= 0.0 {
                                enemies[i].special_t = 3.6 + rng.range(0.0, 1.2);
                                enemies[i].state_t = 0.65;
                                let a = rng.range(0.0, std::f32::consts::TAU);
                                let d = rng.range(42.0, 64.0);
                                let mut p = player_pos + vec2(a.cos(), a.sin()) * d;
                                p.x = p.x.clamp(al + 10.0, ar - 10.0);
                                p.y = p.y.clamp(at + 10.0, ab - 10.0);
                                enemies[i].teleport_to = p;
                                burst(particles, enemies[i].pos, 8, color, 55.0);
                                spawn_ring(rings, enemies[i].pos, 4.0, 20.0, 0.50, color, 1.5);
                                spawn_ring(rings, p, 3.0, 24.0, 0.65, color, 1.8);
                                sfx.play_enemy(enemies[i].voice, 0.18);
                            }
                        }
                    }
                    EnemyAttack::Orbit => {
                        move_dir = if dist > 120.0 { n * 0.7 } else { -n * 0.2 };
                        contact = dist < enemies[i].radius + 12.0;
                        if enemies[i].attack_t <= 0.0 {
                            for k in 0..6 {
                                let a = k as f32 * std::f32::consts::TAU / 6.0 + self.time;
                                let p = enemies[i].pos + vec2(a.cos(), a.sin()) * 12.0;
                                let v = vec2(-a.sin(), a.cos()) * 85.0 + n * 20.0;
                                projectiles.push(enemy_shot(
                                    p,
                                    v,
                                    edmg * 0.7,
                                    color,
                                    3.0,
                                    2.4,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                            enemies[i].attack_t = enemies[i].attack_cd;
                            sfx.play_enemy(enemies[i].voice, 0.18);
                        }
                    }
                    EnemyAttack::Burst => {
                        move_dir = n * 0.8;
                        contact = dist < 36.0;
                        if enemies[i].special_t <= 0.0 {
                            enemies[i].special_t = 3.6;
                            let count = if enemies[i].boss { 10 } else { 6 };
                            for k in 0..count {
                                let a = k as f32 * std::f32::consts::TAU / count as f32
                                    + rng.range(0.0, 0.25);
                                projectiles.push(enemy_shot(
                                    enemies[i].pos,
                                    vec2(a.cos(), a.sin()) * 100.0,
                                    edmg * 0.75,
                                    color,
                                    4.0,
                                    3.0,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                            *shake = (*shake).max(if enemies[i].boss { 6.0 } else { 2.5 });
                            sfx.play_enemy(enemies[i].voice, 0.30);
                            sfx.play(SfxKind::Boss, 0.22);
                        }
                    }
                }

                // Ataque de firma de los guardianes: cada piel tiene un patrón
                // propio y mucho más difícil de esquivar que el ataque común.
                if enemies[i].boss && enemies[i].signature_t <= 0.0 {
                    enemies[i].signature_t =
                        (4.0 - floor as f32 * 0.07).max(2.0) + rng.range(0.0, 0.8);
                    enemies[i].cast_t = 0.42;
                    let skin = enemies[i].boss_skin % 12;
                    let origin = enemies[i].pos;
                    let base = n.y.atan2(n.x);
                    let t = self.time;
                    match skin {
                        0 => {
                            // Rey calavera: corona de lanzas de hueso.
                            for k in 0..14 {
                                let a = k as f32 * std::f32::consts::TAU / 14.0 + t * 0.3;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 96.0,
                                    edmg * 0.62,
                                    color,
                                    3.6,
                                    3.0,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                        }
                        1 => {
                            // Bruto cornado: pisotón que abre una onda de choque.
                            for k in 0..10 {
                                let a = k as f32 * std::f32::consts::TAU / 10.0;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 108.0,
                                    edmg * 0.70,
                                    color,
                                    4.6,
                                    3.0,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                            spawn_ring(rings, origin, 7.0, 64.0, 0.55, color, 2.6);
                            *shake = (*shake).max(8.0);
                        }
                        2 => {
                            // Espectro alado: llama espectros y lanza lamentos.
                            for k in 0..2 {
                                let a = rng.range(0.0, std::f32::consts::TAU);
                                summons.push((
                                    if k == 0 {
                                        EnemyKind::Wraith
                                    } else {
                                        EnemyKind::Bat
                                    },
                                    origin + vec2(a.cos(), a.sin()) * 28.0,
                                ));
                            }
                            for k in 0..3 {
                                let a = base + (k as f32 - 1.0) * 0.32;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 82.0,
                                    edmg * 0.66,
                                    color,
                                    4.2,
                                    4.0,
                                    2.6,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                        }
                        3 => {
                            // Orbe vidente: espiral de miradas.
                            for k in 0..16 {
                                let a = base + k as f32 * 0.42 + t * 0.4;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 104.0,
                                    edmg * 0.56,
                                    color,
                                    3.2,
                                    2.8,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                        }
                        4 => {
                            // Serpiente: escupe bombas de veneno en abanico.
                            for k in 0..5 {
                                let a = base + (k as f32 - 2.0) * 0.26;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 88.0,
                                    edmg * 0.75,
                                    color,
                                    5.0,
                                    1.9,
                                    0.0,
                                    1.2,
                                    26.0,
                                ));
                            }
                        }
                        5 => {
                            // Coloso: anillo lento y pesado de piedra.
                            for k in 0..12 {
                                let a = k as f32 * std::f32::consts::TAU / 12.0 + 0.2;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 74.0,
                                    edmg * 0.80,
                                    color,
                                    5.4,
                                    3.2,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                            *shake = (*shake).max(6.5);
                        }
                        6 => {
                            // Obelisco: cuatro rayos de runas.
                            for d in 0..4 {
                                let a = d as f32 * std::f32::consts::FRAC_PI_2 + t * 0.2;
                                let perp = vec2(-a.sin(), a.cos());
                                for j in -1..=1 {
                                    let p = origin + perp * (j as f32 * 9.0);
                                    projectiles.push(enemy_shot(
                                        p,
                                        vec2(a.cos(), a.sin()) * 152.0,
                                        edmg * 0.58,
                                        color,
                                        2.8,
                                        2.4,
                                        0.0,
                                        shot_slow,
                                        0.0,
                                    ));
                                }
                            }
                        }
                        7 => {
                            // Reina araña: telaraña que ralentiza y crías.
                            for k in 0..7 {
                                let a = base + (k as f32 - 3.0) * 0.20;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 92.0,
                                    edmg * 0.55,
                                    color,
                                    3.8,
                                    3.0,
                                    0.0,
                                    1.7,
                                    0.0,
                                ));
                            }
                            for _ in 0..2 {
                                let a = rng.range(0.0, std::f32::consts::TAU);
                                summons
                                    .push((EnemyKind::Bat, origin + vec2(a.cos(), a.sin()) * 24.0));
                            }
                        }
                        8 => {
                            // Juez del farol: cruz de luz desde el farol.
                            for k in 0..8 {
                                let a = k as f32 * std::f32::consts::FRAC_PI_4 + t * 0.25;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 126.0,
                                    edmg * 0.66,
                                    color,
                                    3.4,
                                    3.0,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                        }
                        9 => {
                            // Ídolo doble: dos abanicos opuestos.
                            for side in 0..2 {
                                let b = base + if side == 0 { 0.0 } else { std::f32::consts::PI };
                                for k in 0..5 {
                                    let a = b + (k as f32 - 2.0) * 0.22;
                                    projectiles.push(enemy_shot(
                                        origin,
                                        vec2(a.cos(), a.sin()) * 100.0,
                                        edmg * 0.58,
                                        color,
                                        3.4,
                                        2.8,
                                        0.0,
                                        shot_slow,
                                        0.0,
                                    ));
                                }
                            }
                        }
                        10 => {
                            // Boca del vacío: atrae y luego estalla.
                            if dist > 42.0 {
                                player.pos += n * 9.0;
                            }
                            for k in 0..12 {
                                let a = k as f32 * std::f32::consts::TAU / 12.0 - t * 0.3;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 90.0,
                                    edmg * 0.72,
                                    color,
                                    4.2,
                                    3.2,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                            spawn_ring(rings, origin, 6.0, 48.0, 0.5, color, 2.4);
                        }
                        _ => {
                            // Mano astral: volea rápida y precisa.
                            for k in 0..5 {
                                let a = base + (k as f32 - 2.0) * 0.18;
                                projectiles.push(enemy_shot(
                                    origin,
                                    vec2(a.cos(), a.sin()) * 176.0,
                                    edmg * 0.62,
                                    color,
                                    2.8,
                                    2.2,
                                    0.0,
                                    shot_slow,
                                    0.0,
                                ));
                            }
                        }
                    }
                    *shake = (*shake).max(3.5);
                    sfx.play_enemy(enemies[i].voice, 0.28);
                    sfx.play(SfxKind::Boss, 0.18);
                }
            }

            // Cualquier disparo (ataque común o firma) marca el gesto del
            // enemigo: brazos y arma se echan hacia delante.
            if projectiles.len() > shots_before {
                enemies[i].swing_t = if enemies[i].boss { 0.30 } else { 0.22 };
            }

            if !jumping && contact && enemies[i].attack_t <= 0.0 {
                let dmg = enemies[i].damage;
                let (taken, blocked) = hurt_player(
                    player, dmg, rng, floaters, particles, shake, sfx, player_pos,
                );
                if taken > 0.0 {
                    spawn_ring(rings, player_pos, 5.0, 22.0, 0.26, C_RED, 1.8);
                    if player.retaliate > 0 {
                        let count = 6 + player.retaliate * 2;
                        for k in 0..count {
                            let a = k as f32 * std::f32::consts::TAU / count as f32;
                            projectiles.push(Projectile {
                                pos: player_pos,
                                vel: vec2(a.cos(), a.sin()) * 190.0,
                                damage: player.damage * 0.6,
                                radius: 3.0,
                                life: 0.55,
                                friendly: true,
                                color: Color::from_rgba(230, 140, 255, 255),
                                pierce: 1,
                                crit: false,
                                aoe: 0.0,
                                slow: 0.0,
                                burn: 0.0,
                                chain: 0,
                                homing: 0.0,
                                hits: [0; 8],
                                hit_count: 0,
                            });
                        }
                    }
                } else if blocked {
                    spawn_ring(rings, player_pos, 5.0, 26.0, 0.30, C_CYAN, 1.8);
                }
                if taken > 0.0 && player.thorns > 0.0 {
                    enemies[i].hp -= taken * player.thorns;
                }
                if blocked && player.reflect > 0.0 {
                    enemies[i].hp -= dmg * player.reflect;
                }
                enemies[i].attack_t = enemies[i].attack_cd * 0.8;
                enemies[i].swing_t = if enemies[i].boss { 0.30 } else { 0.22 };
            }

            if jumping {
                enemies[i].vel = Vec2::ZERO;
            } else if dashing {
                enemies[i].vel = enemies[i].charge_dir * 330.0;
            } else if move_dir.length_squared() > 0.0 {
                enemies[i].vel = safe_norm(move_dir) * speed;
            } else {
                enemies[i].vel *= 0.85;
            }
            let evel = enemies[i].vel;
            enemies[i].pos += evel * dt;
            enemies[i].pos.x = enemies[i]
                .pos
                .x
                .clamp(al + enemies[i].radius, ar - enemies[i].radius);
            enemies[i].pos.y = enemies[i]
                .pos
                .y
                .clamp(at + enemies[i].radius, ab - enemies[i].radius);
            let mut ep = jump_pos.unwrap_or(enemies[i].pos);
            if !enemies[i].boss {
                push_out_of_obstacles(&mut ep, enemies[i].radius, &obstacles);
            }
            enemies[i].pos = ep;
            enemies[i].last_pos = ep;
            let erad = enemies[i].radius;
            if dist < erad + 7.0 {
                enemies[i].pos -= n * (erad + 7.0 - dist) * 0.5;
            }
        }

        for i in 0..enemies.len() {
            for j in (i + 1)..enemies.len() {
                let a = enemies[i].pos;
                let b = enemies[j].pos;
                let ra = enemies[i].radius;
                let rb = enemies[j].radius;
                let d = b - a;
                let dist = d.length();
                if dist > 0.001 && dist < ra + rb {
                    let push = (ra + rb - dist) * 0.5;
                    let n = d / dist;
                    let mi = if enemies[i].boss { 0.15 } else { 1.0 };
                    let mj = if enemies[j].boss { 0.15 } else { 1.0 };
                    enemies[i].pos -= n * push * mi;
                    enemies[j].pos += n * push * mj;
                }
            }
        }

        for (kind, pos) in summons {
            if self.enemies.len() < 30 {
                self.spawn_enemy_at(kind, false, false, pos);
                if let Some(e) = self.enemies.last_mut() {
                    e.summoned = true;
                    e.life = 9999.0;
                }
            }
        }
    }

    fn update_projectiles(&mut self, dt: f32, sfx: &mut Sfx) {
        let enemies = &mut self.enemies;
        let projectiles = &mut self.projectiles;
        let props = &mut self.props;
        let particles = &mut self.particles;
        let floaters = &mut self.floaters;
        let player = &mut self.player;
        let rng = &mut self.rng;
        let shake = &mut self.shake;
        let rings = &mut self.rings;
        let obstacles = self.obstacles.clone();

        for pi in 0..projectiles.len() {
            if projectiles[pi].life <= 0.0 {
                continue;
            }
            if !projectiles[pi].friendly && projectiles[pi].homing > 0.0 {
                let vel = projectiles[pi].vel;
                let want = safe_norm(player.pos - projectiles[pi].pos) * vel.length();
                let k = (projectiles[pi].homing * dt).min(1.0);
                projectiles[pi].vel = vel + (want - vel) * k;
            }
            let pvel = projectiles[pi].vel;
            projectiles[pi].pos += pvel * dt;
            projectiles[pi].life -= dt;
            let ppos = projectiles[pi].pos;

            if obstacles
                .iter()
                .any(|o| ppos.distance(o.pos) < projectiles[pi].radius + o.radius)
            {
                projectiles[pi].life = 0.0;
                burst(particles, ppos, 4, projectiles[pi].color, 35.0);
                continue;
            }

            if projectiles[pi].friendly {
                let mut life = projectiles[pi].life;
                let mut pierce = projectiles[pi].pierce;
                let mut hits = projectiles[pi].hits;
                let mut hit_count = projectiles[pi].hit_count;
                let pr = projectiles[pi].radius;
                let pdmg = projectiles[pi].damage;
                let paoe = projectiles[pi].aoe;
                let pslow = projectiles[pi].slow;
                let pburn = projectiles[pi].burn;
                let pchain = projectiles[pi].chain;
                let pcrit = projectiles[pi].crit;
                let mut hit_something = false;

                for ei in 0..enemies.len() {
                    if enemies[ei].dead {
                        continue;
                    }
                    if hits[..hit_count as usize].contains(&enemies[ei].id) {
                        continue;
                    }
                    let dx = enemies[ei].pos.x - ppos.x;
                    let dy = enemies[ei].pos.y - ppos.y;
                    let rr = pr + enemies[ei].radius;
                    if dx * dx + dy * dy > rr * rr {
                        continue;
                    }
                    let mut d = pdmg;
                    if enemies[ei].hp / enemies[ei].max_hp.max(1.0) < 0.30 {
                        d *= 1.0 + player.execute;
                    }
                    enemies[ei].hp -= d;
                    enemies[ei].hit_flash = 0.12;
                    player.hp = (player.hp + d * player.lifesteal).min(player.max_hp);
                    if pslow > 0.0 {
                        enemies[ei].slow_t = 1.5;
                    }
                    if pburn > 0.0 {
                        enemies[ei].burn_t = 3.0;
                        enemies[ei].burn_dps = pburn;
                    }
                    let epos = enemies[ei].pos;
                    spawn_number(
                        floaters,
                        epos + vec2(0.0, -10.0),
                        d.round() as i32,
                        if pcrit { C_GOLD } else { C_WHITE },
                        8.0,
                    );
                    burst(particles, epos, 3, C_RED, 50.0);
                    spawn_ring(
                        rings,
                        epos,
                        pr,
                        (pr + 7.0 + paoe * 0.25).max(11.0),
                        0.22,
                        if pcrit { C_GOLD } else { C_WHITE },
                        1.4,
                    );
                    if paoe > 0.0 {
                        for ej in 0..enemies.len() {
                            if ej == ei || enemies[ej].dead {
                                continue;
                            }
                            if enemies[ej].pos.distance(epos) < paoe {
                                enemies[ej].hp -= d * 0.22;
                                enemies[ej].hit_flash = 0.10;
                            }
                        }
                        burst(
                            particles,
                            epos,
                            10,
                            Color::from_rgba(255, 150, 60, 255),
                            90.0,
                        );
                        spawn_ring(
                            rings,
                            epos,
                            pr,
                            paoe,
                            0.34,
                            Color::from_rgba(255, 150, 60, 255),
                            2.2,
                        );
                    }
                    if pchain > 0 {
                        let mut bi = None;
                        let mut bd = 90.0 * 90.0;
                        for ej in 0..enemies.len() {
                            if ej == ei || enemies[ej].dead {
                                continue;
                            }
                            let dd = enemies[ej].pos.distance_squared(epos);
                            if dd < bd {
                                bd = dd;
                                bi = Some(ej);
                            }
                        }
                        if let Some(ej) = bi {
                            enemies[ej].hp -= d * 0.55;
                            enemies[ej].hit_flash = 0.10;
                            let e2 = enemies[ej].pos;
                            for k in 0..5 {
                                let t = k as f32 / 4.0;
                                spawn_particle(
                                    particles,
                                    epos.lerp(e2, t),
                                    Vec2::ZERO,
                                    0.18,
                                    1.5,
                                    C_CYAN,
                                    0.0,
                                    0.0,
                                );
                            }
                        }
                    }
                    if hit_count < 8 {
                        hits[hit_count as usize] = enemies[ei].id;
                        hit_count += 1;
                    }
                    hit_something = true;
                    if pierce > 0 {
                        pierce -= 1;
                    } else {
                        life = 0.0;
                        break;
                    }
                }

                if life > 0.0 {
                    for prop in props.iter_mut() {
                        if prop.dead || prop.kind == PropKind::Chest {
                            continue;
                        }
                        if prop.pos.distance(ppos) < pr + 7.0 {
                            prop.hp -= pdmg;
                            prop.hit_flash = 0.15;
                            life = 0.0;
                            burst(
                                particles,
                                prop.pos,
                                4,
                                Color::from_rgba(150, 110, 70, 255),
                                45.0,
                            );
                            spawn_ring(
                                rings,
                                prop.pos,
                                3.0,
                                16.0,
                                0.20,
                                Color::from_rgba(190, 150, 100, 255),
                                1.4,
                            );
                            break;
                        }
                    }
                }

                projectiles[pi].life = life;
                projectiles[pi].pierce = pierce;
                projectiles[pi].hits = hits;
                projectiles[pi].hit_count = hit_count;
                if hit_something {
                    sfx.play(if pcrit { SfxKind::Crit } else { SfxKind::Hit }, 0.20);
                }
            } else {
                let pr = projectiles[pi].radius;
                let dmg = projectiles[pi].damage;
                let pslow = projectiles[pi].slow;
                let paoe = projectiles[pi].aoe;
                let pcol = projectiles[pi].color;
                if player.pos.distance(ppos) < pr + 7.0 {
                    projectiles[pi].life = 0.0;
                    let (taken, _) =
                        hurt_player(player, dmg, rng, floaters, particles, shake, sfx, ppos);
                    if taken > 0.0 && player.retaliate > 0 {
                        let count = 6 + player.retaliate * 2;
                        for k in 0..count {
                            let a = k as f32 * std::f32::consts::TAU / count as f32;
                            projectiles.push(Projectile {
                                pos: player.pos,
                                vel: vec2(a.cos(), a.sin()) * 190.0,
                                damage: player.damage * 0.6,
                                radius: 3.0,
                                life: 0.55,
                                friendly: true,
                                color: Color::from_rgba(230, 140, 255, 255),
                                pierce: 1,
                                crit: false,
                                aoe: 0.0,
                                slow: 0.0,
                                burn: 0.0,
                                chain: 0,
                                homing: 0.0,
                                hits: [0; 8],
                                hit_count: 0,
                            });
                        }
                    }
                    if pslow > 0.0 {
                        player.slow_t = player.slow_t.max(pslow);
                    }
                    if paoe > 0.0 {
                        spawn_ring(rings, ppos, pr, paoe, 0.34, pcol, 2.2);
                        burst(particles, ppos, 12, pcol, 90.0);
                        *shake = (*shake).max(3.0);
                    } else {
                        spawn_ring(rings, ppos, 4.0, 20.0, 0.26, pcol, 1.8);
                    }
                }
            }
        }
        projectiles.retain(|p| p.life > 0.0);
    }

    fn update_props(&mut self, dt: f32, sfx: &mut Sfx) {
        let mut any = false;
        for i in 0..self.props.len() {
            self.props[i].hit_flash -= dt;
            if self.props[i].kind == PropKind::Chest {
                if self.props[i].opened && self.props[i].open_t < 1.0 {
                    self.props[i].open_t = (self.props[i].open_t + dt * 2.6).min(1.0);
                    if self.rng.chance(0.25) {
                        spawn_particle(
                            &mut self.particles,
                            self.props[i].pos + vec2(self.rng.range(-6.0, 6.0), -4.0),
                            vec2(0.0, -22.0),
                            0.5,
                            1.5,
                            C_GOLD,
                            0.0,
                            0.5,
                        );
                    }
                }
                continue;
            }
            if self.props[i].hp <= 0.0 && !self.props[i].dead {
                self.props[i].dead = true;
                any = true;
                let pos = self.props[i].pos;
                let kind = self.props[i].kind;
                let color = match kind {
                    PropKind::Barrel => Color::from_rgba(150, 105, 65, 255),
                    PropKind::Crate => Color::from_rgba(135, 95, 60, 255),
                    PropKind::Chest => C_GOLD,
                    PropKind::Urn => Color::from_rgba(150, 100, 200, 255),
                    PropKind::Statue => Color::from_rgba(190, 195, 205, 255),
                };
                burst(&mut self.particles, pos, 12, color, 80.0);
                self.shake = self.shake.max(1.5);
                sfx.play(SfxKind::Break, 0.28);
                let greed = if self.modifier == Modifier::Greed {
                    1.5
                } else {
                    1.0
                };
                let gold = ((1 + self.floor / 3) as f32 * self.player.gold_mult * greed) as u32;
                self.spawn_pickup(pos, PickupKind::Gold(gold.max(1)));
                if self.rng.chance(0.30) {
                    let xp = (4.0 + self.floor as f32 * 1.5) * self.player.xp_mult;
                    self.spawn_pickup(pos + vec2(0.0, 4.0), PickupKind::Xp(xp));
                }
                if self.rng.chance(0.10) {
                    self.spawn_pickup(pos, PickupKind::Heal(8.0 + self.floor as f32 * 2.0));
                }
                let item_chance = if kind == PropKind::Statue { 0.25 } else { 0.05 };
                if self.rng.chance(item_chance + self.player.luck * 0.03) {
                    let item = self.roll_item(None);
                    self.spawn_pickup(pos, PickupKind::Item(item));
                }
            }
        }
        if any {
            self.props.retain(|p| !p.dead);
        }
    }

    fn update_pickups(&mut self, dt: f32, sfx: &mut Sfx) {
        for i in (0..self.pickups.len()).rev() {
            self.pickups[i].life -= dt;
            self.pickups[i].bob += dt * 4.0;
            let to = self.player.pos - self.pickups[i].pos;
            let dist = to.length();
            if dist < 70.0 {
                self.pickups[i].vel += safe_norm(to) * 420.0 * dt;
            }
            self.pickups[i].vel *= (1.0 - 4.0 * dt).max(0.0);
            let pvel = self.pickups[i].vel;
            self.pickups[i].pos += pvel * dt;
            self.pickups[i].pos.x = self.pickups[i].pos.x.clamp(ARENA_L, ARENA_R);
            self.pickups[i].pos.y = self.pickups[i].pos.y.clamp(ARENA_T, ARENA_B);
            if dist < 13.0 {
                let kind = self.pickups[i].kind;
                self.pickups.swap_remove(i);
                match kind {
                    PickupKind::Gold(n) => {
                        self.player.gold += n;
                        self.run_gold += n;
                        sfx.play(SfxKind::Pickup, 0.16);
                    }
                    PickupKind::Xp(v) => {
                        self.player.xp += v;
                        sfx.play(SfxKind::Pickup, 0.09);
                    }
                    PickupKind::Heal(v) => {
                        self.player.hp = (self.player.hp + v).min(self.player.max_hp);
                        spawn_number(
                            &mut self.floaters,
                            self.player.pos + vec2(0.0, -14.0),
                            v.round() as i32,
                            C_GREEN,
                            8.0,
                        );
                        sfx.play(SfxKind::Pickup, 0.22);
                    }
                    PickupKind::Item(item) => {
                        self.apply_item(item);
                    }
                }
            } else if self.pickups[i].life <= 0.0 {
                self.pickups.swap_remove(i);
            }
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for p in &mut self.particles {
            if !p.active {
                continue;
            }
            p.life -= dt;
            if p.life <= 0.0 {
                p.active = false;
                continue;
            }
            p.vel.y += p.gravity * dt;
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.pos += p.vel * dt;
        }
    }

    fn update_rings(&mut self, dt: f32) {
        for r in &mut self.rings {
            if !r.active {
                continue;
            }
            r.life -= dt;
            if r.life <= 0.0 {
                r.active = false;
                continue;
            }
            r.radius += (r.max_radius - r.radius) * (dt * 10.0).min(1.0);
        }
    }

    fn update_floaters(&mut self, dt: f32) {
        for f in &mut self.floaters {
            if !f.active {
                continue;
            }
            f.life -= dt;
            if f.life <= 0.0 {
                f.active = false;
                continue;
            }
            f.pos += f.vel * dt;
            f.vel.y += 14.0 * dt;
        }
    }

    fn cleanup_dead(&mut self, dt: f32, sfx: &mut Sfx) {
        let mut boss_died = false;
        for e in &mut self.enemies {
            if e.dead {
                e.death_t -= dt;
            }
        }
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp <= 0.0 && !self.enemies[i].dead {
                self.enemies[i].dead = true;
                self.enemies[i].death_t = if self.enemies[i].boss { 0.9 } else { 0.30 };
                let pos = self.enemies[i].pos;
                let elite = self.enemies[i].elite;
                let shiny = self.enemies[i].shiny;
                let boss = self.enemies[i].boss;
                let color = if boss {
                    C_RED
                } else if shiny {
                    C_CYAN
                } else if elite {
                    C_GOLD
                } else {
                    C_RED
                };
                burst(
                    &mut self.particles,
                    pos,
                    if boss { 30 } else { 10 },
                    color,
                    if boss { 150.0 } else { 80.0 },
                );
                self.kills += 1;
                self.player.kills += 1;
                self.shake = self.shake.max(if boss { 12.0 } else { 2.0 });
                sfx.play(SfxKind::Die, 0.22);
                self.drop_loot(pos, elite, shiny, boss);
                if self.player.bloodlust > 0 {
                    self.player.kill_speed_t = 2.0;
                }
                if boss {
                    boss_died = true;
                }
            }
            i += 1;
        }
        self.enemies.retain(|e| !e.dead || e.death_t > 0.0);
        if boss_died {
            self.toast("Guardián caído", C_GOLD);
            self.shake = 10.0;
            spawn_ring(
                &mut self.rings,
                vec2(240.0, 128.0),
                8.0,
                120.0,
                0.8,
                C_GOLD,
                2.5,
            );
        }
    }

    fn drop_loot(&mut self, pos: Vec2, elite: bool, shiny: bool, boss: bool) {
        let xp = (8.0 + self.floor as f32 * 2.0) * self.player.xp_mult;
        self.spawn_pickup(pos, PickupKind::Xp(xp));
        let greed = if self.modifier == Modifier::Greed {
            1.5
        } else {
            1.0
        };
        let mut gold = ((1 + self.floor / 3) as f32 * self.player.gold_mult * greed) as u32;
        if elite {
            gold *= 3;
        }
        if boss {
            gold *= 8;
        }
        // El oro ya no cae de cada cadáver: hay que elegir qué saquear.
        if boss || elite || self.rng.chance(0.55) {
            self.spawn_pickup(pos, PickupKind::Gold(gold.max(1)));
        }
        let chance = 0.06 + self.player.luck * 0.04;
        if boss {
            let key_chance = (0.35 + self.player.luck * 0.15).min(0.75);
            if self.rng.chance(key_chance) {
                self.special_keys += 1;
                self.toast("El guardián deja una llave negra", C_PURPLE);
            }
            let item = self.roll_item(Some(Rarity::Legendary));
            self.spawn_pickup(pos + vec2(-6.0, 0.0), PickupKind::Item(item));
            let item2 = self.roll_item(Some(Rarity::Rare));
            self.spawn_pickup(pos + vec2(6.0, 0.0), PickupKind::Item(item2));
        } else if shiny {
            let item = self.roll_item(Some(Rarity::Epic));
            self.spawn_pickup(pos, PickupKind::Item(item));
        } else if elite {
            if self.rng.chance(0.08) {
                self.special_keys += 1;
                self.toast("El élite llevaba una llave negra", C_PURPLE);
            }
            let item = self.roll_item(Some(Rarity::Rare));
            self.spawn_pickup(pos, PickupKind::Item(item));
        } else if self.rng.chance(chance) {
            let item = self.roll_item(None);
            self.spawn_pickup(pos, PickupKind::Item(item));
        }
    }

    fn spawn_pickup(&mut self, pos: Vec2, kind: PickupKind) {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        let speed = self.rng.range(15.0, 55.0);
        self.pickups.push(Pickup {
            pos,
            vel: vec2(a.cos(), a.sin()) * speed,
            kind,
            life: 30.0,
            bob: self.rng.range(0.0, 6.28),
        });
    }

    fn roll_rarity(&mut self) -> Rarity {
        let shift = (self.player.luck * 12.0).clamp(0.0, 32.0);
        let mut roll = (self.rng.f32() * 100.0 - shift).max(0.0);
        let table = [
            (Rarity::Common, 50.0),
            (Rarity::Uncommon, 30.0),
            (Rarity::Rare, 14.0),
            (Rarity::Epic, 5.0),
            (Rarity::Legendary, 1.0),
        ];
        for (r, w) in table {
            if roll < w {
                return r;
            }
            roll -= w;
        }
        Rarity::Legendary
    }

    fn roll_item(&mut self, force: Option<Rarity>) -> Item {
        let idx = self.rng.usize(ITEMS.len());
        let (mut name, mut kind, mut base) = ITEMS[idx];
        if matches!(kind, ItemKind::SkeletonCap) && self.class != Class::Necro {
            kind = ItemKind::MaxHp;
            name = "Placa de hierro";
            base = 10.0;
        }
        let rarity = match force {
            Some(r) => r,
            None => self.roll_rarity(),
        };
        let rmult = match rarity {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.3,
            Rarity::Rare => 1.7,
            Rarity::Epic => 2.2,
            Rarity::Legendary => 3.0,
        };
        let value = base * (1.0 + self.floor as f32 * 0.10) * rmult;
        Item {
            name,
            kind,
            rarity,
            value,
        }
    }

    fn apply_item(&mut self, item: Item) {
        match item.kind {
            ItemKind::Damage => self.player.damage += item.value,
            ItemKind::MaxHp => {
                self.player.max_hp += item.value;
                self.player.hp += item.value;
            }
            ItemKind::Speed => self.player.speed += item.value,
            ItemKind::Crit => self.player.crit = (self.player.crit + item.value).min(0.85),
            ItemKind::Lifesteal => self.player.lifesteal += item.value,
            ItemKind::Thorns => self.player.thorns += item.value,
            ItemKind::Armor => self.player.armor += item.value,
            ItemKind::SkeletonCap => {
                self.player.skeleton_cap += item.value.round().max(1.0) as u32;
                self.summon_all();
            }
            ItemKind::Regen => self.player.regen += item.value,
            ItemKind::AllStats => {
                let m = 1.0 + item.value;
                self.player.damage *= m;
                self.player.max_hp *= m;
                self.player.speed *= m;
                self.player.attack_speed *= m;
                self.player.armor *= m;
                self.player.regen *= m;
                self.player.lifesteal *= m;
                self.player.thorns *= m;
                self.player.luck *= m;
                self.player.xp_mult *= m;
                self.player.gold_mult *= m;
                self.player.aoe *= m;
                self.player.range *= m;
                self.player.hp =
                    (self.player.hp + self.player.max_hp * 0.10).min(self.player.max_hp);
            }
            ItemKind::GlassCannon => {
                self.player.damage *= 1.0 + item.value;
                self.player.max_hp *= 0.85;
                self.player.hp = self.player.hp.min(self.player.max_hp);
            }
            ItemKind::CrystalHeart => {
                self.player.max_hp += item.value;
                self.player.hp += item.value;
                self.player.regen += 0.5;
            }
            ItemKind::Orbit => {
                self.player.orbit += item.value.round().max(1.0) as u32;
            }
            ItemKind::Homing => {
                self.player.homing = (self.player.homing + item.value * 0.5).min(5.0);
            }
            ItemKind::Chain => {
                self.player.chain += item.value.round().max(1.0) as u32;
            }
            ItemKind::Nova => {
                self.player.nova += item.value.round().max(1.0) as u32;
            }
            ItemKind::Mine => {
                self.player.mine += item.value.round().max(1.0) as u32;
            }
            ItemKind::Retaliate => {
                self.player.retaliate += item.value.round().max(1.0) as u32;
            }
            ItemKind::DashBurst => {
                self.player.dash_burst += item.value.round().max(1.0) as u32;
            }
            ItemKind::Vortex => {
                self.player.vortex += item.value.round().max(1.0) as u32;
            }
            ItemKind::Bolas => {
                self.player.bolas += item.value.round().max(1.0) as u32;
            }
            ItemKind::Trail => {
                self.player.trail += item.value.round().max(1.0) as u32;
            }
            ItemKind::Aura => {
                self.player.aura += item.value.round().max(1.0) as u32;
            }
            ItemKind::Mend => {
                self.player.mend += item.value.round().max(1.0) as u32;
            }
        }
        self.refresh_skeleton_damage();
        self.items.push(item);
        let c = item.rarity.color();
        self.toast(&format!("{} [{}]", item.name, item.rarity.name()), c);
    }

    fn check_floor_clear(&mut self, dt: f32, sfx: &mut Sfx) {
        if self.enemies.is_empty() {
            if self.wave < self.waves {
                self.wave_timer += dt;
                if self.wave_timer > 0.8 {
                    self.spawn_wave();
                }
            } else if !self.cleared {
                self.cleared = true;
                if let Some(room) = self.rooms.get_mut(self.room_index) {
                    room.cleared = true;
                }
                if self.current_kind() == RoomKind::Boss {
                    self.toast("Portal abierto. Acércate para descender", C_CYAN);
                    spawn_ring(&mut self.rings, self.portal, 10.0, 70.0, 0.7, C_CYAN, 2.0);
                } else {
                    self.toast("Sala despejada. Puertas abiertas", C_GREEN);
                }
                sfx.play(SfxKind::Portal, 0.40);
            }
        }
        if self.cleared && self.current_kind() == RoomKind::Boss {
            let d = self.player.pos.distance(self.portal);
            // La guía termina al caer el segundo guardián: en el piso 2 no se
            // encadena otro descenso. En el piso 1 sí debe bajar normalmente.
            let demo_end = self.demo_input.active && self.floor >= 2;
            if d < 30.0 && !demo_end {
                self.portal_t += dt;
                if self.portal_t > 0.55 {
                    if self.floor % 10 == 0 && !self.endless {
                        self.end_run(true);
                    } else if self.meta.floor_transitions {
                        self.floor_transition_t = 0.85;
                        self.floor_transition_pending = true;
                        sfx.play(SfxKind::Portal, 0.5);
                    } else {
                        self.next_floor();
                    }
                    return;
                }
            } else {
                self.portal_t = 0.0;
            }
            if is_key_pressed(KeyCode::Q)
                && self.player.gold >= 30
                && self.player.hp < self.player.max_hp
            {
                self.player.gold -= 30;
                let heal = self.player.max_hp * 0.30;
                self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                self.toast("Curado un 30% por 30 de oro", C_GREEN);
                sfx.play(SfxKind::Pickup, 0.30);
            }
        }
    }

    fn end_run(&mut self, victory: bool) {
        self.souls_earned = self.floor * 8 + self.run_gold / 2;
        self.meta.souls += self.souls_earned;
        self.meta.best_floor = self.meta.best_floor.max(self.floor);
        self.meta.runs += 1;
        self.meta.total_kills += self.kills;
        // La deuda viaja con el jugador aunque muera: el mercader la cobrará
        // en la entrada de la siguiente partida.
        self.meta.debt = self.debt;
        if !victory {
            // Cada muerte deja un eco con la clase y el piso alcanzado.
            self.meta.echoes = (self.meta.echoes + 1).min(99);
            self.meta.echo_class = match self.class {
                Class::Knight => 0,
                Class::Archer => 1,
                Class::Mage => 2,
                Class::Assassin => 3,
                Class::Necro => 4,
                Class::Cleric => 5,
                Class::Merchant => 6,
            };
            self.meta.echo_floor = self.floor;
        }
        save::save(&self.meta);
        self.screen = if victory {
            Screen::Victory
        } else {
            Screen::Dead
        };
    }

    fn update_toasts(&mut self, dt: f32) {
        for t in &mut self.toasts {
            t.t -= dt;
        }
        self.toasts.retain(|t| t.t > 0.0);
    }

    pub fn draw_world(&self, s: &Sprites) {
        if self.screen == Screen::Menu {
            draw_texture(&s.bg, 0.0, 0.0, C_WHITE);
            return;
        }

        self.draw_chamber();
        self.draw_room_shell(s);
        self.draw_room_decor(s);

        let (l, r, t, _b) = self.arena();
        for gx in [l + 18.0, r - 18.0] {
            let pulse = 1.0 + (self.time * 6.0 + gx).sin() * 0.08;
            let sz = 64.0 * pulse;
            draw_texture_ex(
                &s.glow_white,
                gx - sz * 0.5,
                t + 6.0 - sz * 0.5,
                Color::new(0.42, 0.56, 1.0, 0.26),
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    ..Default::default()
                },
            );
        }

        if self.current_kind() == RoomKind::Boss {
            figures::draw_portal(self.portal, self.cleared, self.time);
        }

        for o in &self.obstacles {
            match o.kind {
                ObstacleKind::Pillar => {
                    sprites::draw_shadow(o.pos, 12.0, 10.0);
                    draw_rectangle(
                        o.pos.x - 5.0,
                        o.pos.y - 14.0,
                        10.0,
                        18.0,
                        Color::from_rgba(74, 68, 84, 255),
                    );
                    draw_rectangle(
                        o.pos.x - 7.0,
                        o.pos.y - 17.0,
                        14.0,
                        5.0,
                        Color::from_rgba(104, 96, 118, 255),
                    );
                    draw_rectangle_lines(
                        o.pos.x - 5.0,
                        o.pos.y - 14.0,
                        10.0,
                        18.0,
                        0.8,
                        Color::from_rgba(38, 34, 46, 255),
                    );
                }
                ObstacleKind::Rock => {
                    sprites::draw_shadow(o.pos, 14.0, 10.0);
                    draw_circle(
                        o.pos.x,
                        o.pos.y - 2.0,
                        o.radius,
                        Color::from_rgba(84, 80, 92, 255),
                    );
                    draw_circle(
                        o.pos.x - 2.0,
                        o.pos.y - 4.0,
                        o.radius * 0.55,
                        Color::from_rgba(112, 108, 122, 255),
                    );
                }
                ObstacleKind::Bones => {
                    sprites::draw_shadow(o.pos, 12.0, 6.0);
                    draw_line(
                        o.pos.x - 6.0,
                        o.pos.y + 1.0,
                        o.pos.x + 6.0,
                        o.pos.y - 3.0,
                        2.0,
                        Color::from_rgba(214, 208, 190, 255),
                    );
                    draw_line(
                        o.pos.x - 4.0,
                        o.pos.y - 4.0,
                        o.pos.x + 3.0,
                        o.pos.y + 3.0,
                        2.0,
                        Color::from_rgba(190, 184, 166, 255),
                    );
                    draw_circle(
                        o.pos.x + 6.0,
                        o.pos.y - 4.0,
                        2.6,
                        Color::from_rgba(226, 220, 202, 255),
                    );
                }
                ObstacleKind::Brazier => {
                    // Pilar astral: piedra, aro rúnico y cristal flotante.
                    let pulse = 0.85 + (self.time * 3.2 + o.bob).sin() * 0.15;
                    let sz = 44.0 * pulse;
                    draw_texture_ex(
                        &s.glow_white,
                        o.pos.x - sz * 0.5,
                        o.pos.y - 10.0 - sz * 0.5,
                        Color::new(0.52, 0.66, 1.0, 0.34),
                        DrawTextureParams {
                            dest_size: Some(vec2(sz, sz)),
                            ..Default::default()
                        },
                    );
                    let stone = Color::from_rgba(40, 38, 56, 255);
                    draw_rectangle(o.pos.x - 4.5, o.pos.y - 1.0, 9.0, 8.0, stone);
                    draw_rectangle(
                        o.pos.x - 6.5,
                        o.pos.y + 6.0,
                        13.0,
                        3.0,
                        Color::from_rgba(26, 24, 36, 255),
                    );
                    draw_rectangle_lines(
                        o.pos.x - 6.5,
                        o.pos.y + 6.0,
                        13.0,
                        3.0,
                        0.7,
                        with_alpha(C_CYAN, 0.42),
                    );
                    draw_line(
                        o.pos.x - 4.5,
                        o.pos.y - 1.0,
                        o.pos.x - 4.5,
                        o.pos.y + 6.0,
                        0.6,
                        with_alpha(C_CYAN, 0.50),
                    );
                    draw_line(
                        o.pos.x + 4.5,
                        o.pos.y - 1.0,
                        o.pos.x + 4.5,
                        o.pos.y + 6.0,
                        0.6,
                        with_alpha(C_CYAN, 0.50),
                    );
                    let cy = o.pos.y - 9.0 + (self.time * 2.2 + o.bob).sin() * 1.3;
                    draw_circle_lines(o.pos.x, cy, 6.0, 0.6, with_alpha(C_CYAN, 0.40));
                    draw_triangle(
                        vec2(o.pos.x, cy - 5.5),
                        vec2(o.pos.x - 3.6, cy),
                        vec2(o.pos.x, cy + 5.5),
                        with_alpha(C_CYAN, 0.95),
                    );
                    draw_triangle(
                        vec2(o.pos.x, cy - 5.5),
                        vec2(o.pos.x + 3.6, cy),
                        vec2(o.pos.x, cy + 5.5),
                        Color::from_rgba(150, 120, 255, 220),
                    );
                    draw_circle(o.pos.x, cy, 1.1, with_alpha(C_WHITE, 0.90));
                    let ang = self.time * 1.7 + o.bob;
                    draw_circle(
                        o.pos.x + ang.cos() * 8.0,
                        cy + ang.sin() * 4.0,
                        0.9,
                        with_alpha(C_CYAN, 0.75),
                    );
                }
                ObstacleKind::Spikes => {
                    // Placa de hierro con pinchos: se lee como peligro de suelo.
                    let pulse = 0.75 + (self.time * 3.6 + o.bob).sin() * 0.25;
                    draw_circle(
                        o.pos.x,
                        o.pos.y,
                        o.radius,
                        Color::from_rgba(30, 24, 30, 150),
                    );
                    draw_circle_lines(
                        o.pos.x,
                        o.pos.y,
                        o.radius,
                        0.7,
                        with_alpha(Color::from_rgba(170, 58, 64, 255), 0.55 * pulse),
                    );
                    for k in 0..6 {
                        let a = k as f32 * std::f32::consts::TAU / 6.0 + o.bob;
                        let bx = o.pos.x + a.cos() * o.radius * 0.45;
                        let by = o.pos.y + a.sin() * o.radius * 0.45;
                        let tx = o.pos.x + a.cos() * o.radius * 0.92;
                        let ty = o.pos.y + a.sin() * o.radius * 0.92;
                        let px = -a.sin() * 2.6;
                        let py = a.cos() * 2.6;
                        draw_triangle(
                            vec2(bx + px, by + py),
                            vec2(bx - px, by - py),
                            vec2(tx, ty),
                            Color::from_rgba(186, 182, 196, 255),
                        );
                        draw_triangle(
                            vec2(bx, by),
                            vec2(bx - px * 0.5, by - py * 0.5),
                            vec2(tx, ty),
                            with_alpha(Color::from_rgba(238, 232, 245, 255), 0.75),
                        );
                    }
                    draw_circle(o.pos.x, o.pos.y, 2.0, with_alpha(C_RED, 0.65 * pulse));
                }
            }
        }

        for p in &self.props {
            let bob = (self.time * 2.0 + p.bob).sin() * 0.5;
            if p.kind == PropKind::Chest {
                sprites::draw_shadow(p.pos, 14.0, 12.0);
                figures::draw_chest(p.pos + vec2(0.0, bob), p.opened, p.open_t, self.time, 1.0);
                if p.hit_flash > 0.0 {
                    draw_circle(
                        p.pos.x,
                        p.pos.y - 4.0 + bob,
                        12.0,
                        Color::new(1.0, 0.70, 0.60, 0.22),
                    );
                }
                continue;
            }
            let shadow_w = if p.kind == PropKind::Statue {
                16.0
            } else {
                13.0
            };
            sprites::draw_shadow(p.pos, shadow_w, 11.0);
            figures::draw_prop(
                p.kind,
                p.pos + vec2(0.0, bob * 0.4),
                p.hit_flash,
                self.time + p.bob,
                1.0,
            );
        }

        for m in &self.mines {
            let pulse = 0.7 + (self.time * (8.0 + (1.2 - m.t).max(0.0) * 12.0)).sin() * 0.3;
            draw_circle(
                m.pos.x,
                m.pos.y,
                m.radius * 0.30,
                with_alpha(m.color, 0.10 + pulse * 0.10),
            );
            draw_circle_lines(
                m.pos.x,
                m.pos.y,
                m.radius * 0.30 * pulse,
                0.9,
                with_alpha(m.color, 0.70),
            );
            draw_circle(m.pos.x, m.pos.y, 2.2, with_alpha(m.color, 0.95));
            for k in 0..5 {
                let a = k as f32 * std::f32::consts::TAU / 5.0 + self.time;
                draw_line(
                    m.pos.x,
                    m.pos.y,
                    m.pos.x + a.cos() * 5.0,
                    m.pos.y + a.sin() * 5.0,
                    0.8,
                    with_alpha(m.color, 0.65),
                );
            }
        }

        for h in &self.hazards {
            let fade = (h.life / h.max_life.max(0.01)).clamp(0.0, 1.0);
            match h.kind {
                HazardKind::Trail => {
                    let flick = 0.75 + (self.time * 11.0 + h.pos.x).sin() * 0.25;
                    draw_circle(
                        h.pos.x,
                        h.pos.y,
                        h.radius,
                        with_alpha(h.color, 0.10 * fade * flick),
                    );
                    for k in 0..3 {
                        let a = k as f32 * 2.1 + self.time * 2.0;
                        draw_circle(
                            h.pos.x + a.cos() * h.radius * 0.45,
                            h.pos.y + a.sin() * h.radius * 0.30,
                            1.1 + fade * 1.2,
                            with_alpha(h.color, 0.55 * fade * flick),
                        );
                    }
                }
                HazardKind::Vortex => {
                    let spin = self.time * 4.5;
                    draw_circle(h.pos.x, h.pos.y, h.radius, with_alpha(h.color, 0.07 * fade));
                    for k in 0..3 {
                        let r = h.radius * (0.32 + k as f32 * 0.24);
                        for j in 0..8 {
                            let a = j as f32 * std::f32::consts::TAU / 8.0
                                + spin * (1.0 + k as f32 * 0.25);
                            draw_circle(
                                h.pos.x + a.cos() * r,
                                h.pos.y + a.sin() * r,
                                0.9,
                                with_alpha(h.color, (0.65 - k as f32 * 0.12) * fade),
                            );
                        }
                    }
                    draw_circle(h.pos.x, h.pos.y, 2.0, with_alpha(C_WHITE, 0.75 * fade));
                }
            }
        }

        // Rayos de los ataques equipados: una línea breve y brillante.
        for b in &self.beams {
            let fade = (b.life / b.max_life.max(0.01)).clamp(0.0, 1.0);
            let end = b.pos + b.dir * b.length;
            draw_line(
                b.pos.x,
                b.pos.y,
                end.x,
                end.y,
                b.width * fade,
                with_alpha(b.color, 0.85 * fade),
            );
            draw_line(
                b.pos.x,
                b.pos.y,
                end.x,
                end.y,
                b.width * 0.35 * fade,
                with_alpha(C_WHITE, 0.90 * fade),
            );
        }

        // Meteoros: primero el aviso en el suelo, después la caída.
        for m in &self.meteors {
            let p = 1.0 - (m.t / m.max_t.max(0.01)).clamp(0.0, 1.0);
            draw_circle(
                m.pos.x,
                m.pos.y,
                m.radius,
                with_alpha(m.color, 0.08 + p * 0.12),
            );
            draw_circle_lines(
                m.pos.x,
                m.pos.y,
                m.radius * (0.55 + p * 0.45),
                1.1,
                with_alpha(m.color, 0.55 + p * 0.35),
            );
            let fall = 90.0 * (1.0 - p);
            let my = m.pos.y - fall;
            draw_line(
                m.pos.x,
                my - 26.0,
                m.pos.x,
                my,
                3.0,
                with_alpha(m.color, 0.65),
            );
            draw_circle(m.pos.x, my, 4.5 + p * 2.0, with_alpha(m.color, 0.95));
            draw_circle(m.pos.x, my, 2.0, with_alpha(C_WHITE, 0.9));
        }

        for p in &self.pickups {
            let bob = (self.time * 4.0 + p.bob).sin() * 1.5;
            match p.kind {
                PickupKind::Gold(_) => figures::draw_coin(p.pos, bob, self.time + p.bob),
                PickupKind::Xp(_) => figures::draw_gem(p.pos, bob, C_CYAN),
                PickupKind::Heal(_) => figures::draw_vial(p.pos, bob, self.time + p.bob),
                PickupKind::Item(item) => {
                    let c = item.rarity.color();
                    draw_circle(p.pos.x, p.pos.y + bob, 10.0, with_alpha(c, 0.14));
                    figures::draw_gem(p.pos + vec2(0.0, -1.0), bob, c);
                }
            }
        }

        for sk in &self.skeletons {
            let bob = (self.time * 8.0 + sk.bob).sin() * 1.2;
            sprites::draw_shadow(sk.pos, 12.0, 14.0);
            figures::draw_ally(1, sk.pos, Color::new(0.82, 0.80, 0.72, 1.0), bob, 0.0);
        }

        if let Some(a) = &self.ally {
            let bob = (a.bob).sin() * 1.4;
            let c = a.color();
            // Golpe: el aliado se lanza hacia su objetivo. El farolero lo hace
            // hacia el héroe, como si le tendiera la luz.
            let target = if a.kind == 2 {
                self.player.pos
            } else {
                self.nearest_enemy_pos()
                    .unwrap_or(self.player.pos + self.player.facing * 30.0)
            };
            let dir = safe_norm(target - a.pos);
            let swing = (a.swing_t / 0.28).clamp(0.0, 1.0) * std::f32::consts::PI;
            let apos = a.pos + dir * (swing.sin() * 3.0);
            sprites::draw_shadow(a.pos, 11.0, 12.0);
            draw_circle(a.pos.x, a.pos.y + bob, 10.0, with_alpha(c, 0.10));
            figures::draw_ally(a.kind as usize, apos, c, bob, swing);
            if a.kind == 2 {
                draw_circle(apos.x, apos.y + bob - 8.0, 2.0, with_alpha(C_GOLD, 0.75));
            }
        }

        for e in &self.enemies {
            let scale = if e.boss {
                1.7
            } else if e.elite {
                1.3
            } else {
                1.0
            };
            let bob = (self.time * if e.boss { 8.0 } else { 6.0 } + e.bob).sin()
                * if e.boss { 2.2 } else { 1.2 };
            let cast_shake = if e.cast_t > 0.0 {
                (e.cast_t * 52.0).sin() * 2.4
            } else {
                0.0
            };
            // Aparición: los que vuelan caen desde arriba; los que caminan
            // emergen del suelo entre polvo. Muerte: se encogen y se desvanecen.
            let spawn_max = if e.boss { 1.1 } else { 0.45 };
            let spawn_k = (e.spawn_t / spawn_max).clamp(0.0, 1.0);
            let burrow = !e.boss
                && matches!(
                    e.kind,
                    EnemyKind::Slime | EnemyKind::Brute | EnemyKind::Sentinel
                );
            let drop = if burrow {
                -spawn_k * spawn_k * 22.0
            } else {
                spawn_k * spawn_k * 76.0
            };
            let death_max = if e.boss { 0.9 } else { 0.30 };
            let death_k = if e.dead {
                (e.death_t / death_max).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let scale = scale * (0.55 + 0.45 * death_k);
            // Balanceo de caminata.
            let sway = (e.walk_t).sin() * if e.boss { 1.2 } else { 1.6 };
            // Golpe cuerpo a cuerpo: se impulsa hacia el jugador.
            let swing_k = (e.swing_t / if e.boss { 0.30 } else { 0.22 }).clamp(0.0, 1.0);
            let swing = (swing_k * std::f32::consts::PI).sin() * if e.boss { 4.5 } else { 2.8 };
            let to_player = safe_norm(self.player.pos - e.pos);
            let lunge = to_player * swing;
            // Salto de guardián: aviso en el suelo, sombra que se encoge y
            // cuerpo que se eleva en arco.
            let jump_k = if e.jump_t > 0.0 {
                1.0 - (e.jump_t / 0.62).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let jump_lift = (jump_k * std::f32::consts::PI).sin() * 34.0;
            if e.boss && e.jump_wind > 0.0 {
                let charge = 1.0 - (e.jump_wind / 0.52).clamp(0.0, 1.0);
                let p = e.jump_to;
                let pulse = 0.5 + 0.5 * (self.time * 16.0).sin();
                draw_circle(
                    p.x,
                    p.y,
                    10.0 + charge * 14.0,
                    Color::new(e.tint.r, e.tint.g, e.tint.b, 0.10 + charge * 0.14),
                );
                draw_circle_lines(
                    p.x,
                    p.y,
                    12.0 + charge * 12.0,
                    1.2,
                    Color::new(e.tint.r, e.tint.g, e.tint.b, 0.55 + pulse * 0.35),
                );
                for k in 0..6 {
                    let a = k as f32 * std::f32::consts::TAU / 6.0 - self.time * 1.8;
                    draw_line(
                        p.x + a.cos() * 20.0,
                        p.y + a.sin() * 20.0,
                        p.x + a.cos() * 9.0,
                        p.y + a.sin() * 9.0,
                        0.9,
                        Color::new(e.tint.r, e.tint.g, e.tint.b, 0.50),
                    );
                }
            }
            if burrow && spawn_k > 0.02 {
                draw_circle_lines(
                    e.pos.x,
                    e.pos.y + e.radius * 0.35,
                    e.radius * (1.1 + spawn_k * 0.9),
                    1.0,
                    with_alpha(e.tint, 0.28 * spawn_k),
                );
                for k in 0..4 {
                    let a = k as f32 * std::f32::consts::TAU / 4.0 + e.bob;
                    draw_circle(
                        e.pos.x + a.cos() * e.radius * 1.2,
                        e.pos.y + e.radius * 0.35 + a.sin() * 3.0,
                        1.0 + spawn_k * 1.2,
                        with_alpha(Color::from_rgba(150, 132, 116, 255), 0.35 * spawn_k),
                    );
                }
            }
            let shadow_w = if e.boss { 30.0 } else { 14.0 };
            if spawn_k < 0.85 {
                sprites::draw_shadow(e.pos, shadow_w * scale, shadow_w * scale * 0.6);
            } else {
                sprites::draw_shadow(
                    e.pos,
                    shadow_w * scale * (1.0 - jump_k * 0.45),
                    shadow_w * scale * (1.0 - jump_k * 0.45),
                );
            }
            // Telegrafía del ataque de firma: el guardián se hincha, tiembla
            // y dibuja un anillo de runas antes de desatar su patrón.
            if e.boss && !e.dead && e.signature_t < 1.15 {
                let charge = 1.0 - (e.signature_t / 1.15).clamp(0.0, 1.0);
                let pulse = 0.5 + 0.5 * (self.time * 18.0).sin();
                let ring = e.radius * (1.35 + charge * 0.75 + pulse * 0.10);
                draw_circle(
                    e.pos.x,
                    e.pos.y,
                    ring,
                    Color::new(e.tint.r, e.tint.g, e.tint.b, 0.06 + charge * 0.10),
                );
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::TAU / 8.0 + self.time * 2.2;
                    draw_circle(
                        e.pos.x + a.cos() * ring,
                        e.pos.y + a.sin() * ring,
                        1.2 + charge * 1.4,
                        Color::new(e.tint.r, e.tint.g, e.tint.b, 0.55 + charge * 0.40),
                    );
                }
            }
            let tint = if e.shiny {
                C_CYAN
            } else if e.elite {
                C_GOLD
            } else {
                e.tint
            };
            let appear_alpha = if burrow { 1.0 - spawn_k * 0.55 } else { 1.0 };
            let feet =
                e.pos + vec2(cast_shake + sway, e.radius * 0.38 + bob - drop - jump_lift) + lunge;
            figures::draw_enemy(
                feet,
                e.kind,
                e.radius,
                scale,
                tint,
                e.boss_skin,
                e.elite,
                e.shiny,
                e.boss,
                e.walk_t,
                e.swing_t,
                e.hit_flash,
                death_k * appear_alpha,
                self.time,
            );
            if !e.dead && e.spawn_t <= 0.0 && !e.boss {
                // Barra de vida pequeña sobre cada enemigo. Se coloca por
                // encima de la silueta según el arquetipo; los jefes usan la
                // barra grande del HUD y no llevan barra local.
                let w = if e.elite { 20.0 } else { 14.0 };
                let lift = match e.kind {
                    EnemyKind::Slime => 11.0,
                    EnemyKind::Bat => 16.0,
                    EnemyKind::Brute => 30.0,
                    EnemyKind::Cultist => 28.0,
                    EnemyKind::Wraith => 26.0,
                    EnemyKind::Bomber => 25.0,
                    EnemyKind::Sentinel => 29.0,
                    EnemyKind::Eye => 19.0,
                    EnemyKind::Boss => 0.0,
                };
                let y = feet.y - lift * scale - 3.0;
                draw_rectangle(
                    e.pos.x - w * 0.5 - 0.5,
                    y - 0.5,
                    w + 1.0,
                    3.5,
                    Color::new(0.03, 0.025, 0.05, 0.90),
                );
                draw_rectangle(
                    e.pos.x - w * 0.5,
                    y,
                    w * (e.hp / e.max_hp).clamp(0.0, 1.0),
                    2.5,
                    if e.boss {
                        C_RED
                    } else if e.elite {
                        C_GOLD
                    } else if e.hp / e.max_hp.max(1.0) < 0.35 {
                        Color::from_rgba(235, 110, 80, 255)
                    } else {
                        Color::from_rgba(210, 70, 82, 255)
                    },
                );
            }
            if e.shiny {
                let x = e.pos.x - 1.0;
                let y = e.pos.y - 8.0 * scale - 10.0;
                draw_rectangle(x, y, 2.0, 5.0, C_CYAN);
                draw_rectangle(x, y + 7.0, 2.0, 2.0, C_CYAN);
            }
        }

        // Alcance de los personajes a distancia. Se dibuja como un anillo
        // discontinuo para que no tape el suelo ni los enemigos.
        if self.player.range > 60.0
            && !self.cleared
            && matches!(self.current_kind(), RoomKind::Combat | RoomKind::Boss)
            && self.screen != Screen::Dead
            && self.screen != Screen::Victory
        {
            let rr = self.player.range;
            let pulse = 0.13 + (self.time * 2.2).sin() * 0.04;
            draw_circle_lines(
                self.player.pos.x,
                self.player.pos.y,
                rr,
                1.0,
                with_alpha(self.class.color(), pulse),
            );
            for k in 0..36 {
                if k % 2 == 0 {
                    let a0 = k as f32 * std::f32::consts::TAU / 36.0;
                    let a1 = (k + 1) as f32 * std::f32::consts::TAU / 36.0;
                    draw_line(
                        self.player.pos.x + a0.cos() * rr,
                        self.player.pos.y + a0.sin() * rr,
                        self.player.pos.x + a1.cos() * rr,
                        self.player.pos.y + a1.sin() * rr,
                        1.2,
                        with_alpha(self.class.color(), pulse + 0.08),
                    );
                }
            }
        }

        // En derrota y victoria el personaje ya no se dibuja: la escena queda
        // limpia detrás del panel de resultados.
        if self.screen != Screen::Dead && self.screen != Screen::Victory {
            // Balanceo de caminata: el sprite ya anima piernas y torso, así que
            // aquí solo se inclina el cuerpo en la dirección de avance.
            let lean = self.player.walk_t.cos() * if self.player.moving { 1.2 } else { 0.25 };
            // Ataque: los cuerpos a cuerpo se impulsan y los ranged retroceden.
            let punch = ((self.player.swing_t / 0.16).clamp(0.0, 1.0) * std::f32::consts::PI).sin();
            let lunge = if matches!(self.player.class, Class::Knight | Class::Assassin) {
                punch * 2.4
            } else {
                -punch * 1.5
            };
            let hero_pos = self.player.pos
                + vec2(
                    lean * self.player.facing.x + self.player.facing.x * lunge,
                    self.player.facing.y * lunge,
                );
            sprites::draw_shadow(self.player.pos, 14.0, 16.0);
            draw_circle(
                self.player.pos.x,
                self.player.pos.y + 8.0,
                8.0,
                with_alpha(self.class.color(), 0.15),
            );
            let skin_tint = if self.meta.skin == 0 {
                C_WHITE
            } else {
                skin_color(self.meta.skin)
            };
            let feet = hero_pos + vec2(0.0, 8.0);
            figures::draw_hero(
                feet,
                self.player.class,
                self.player.facing,
                self.player.walk_t,
                self.player.moving,
                self.player.swing_t,
                self.player.hit_flash,
                skin_tint,
                self.meta.hat,
                self.meta.mask,
                self.time,
            );
            draw_circle(
                self.player.pos.x + self.player.facing.x * 10.0,
                self.player.pos.y + self.player.facing.y * 10.0,
                1.5,
                with_alpha(C_WHITE, 0.55),
            );
            // Marcas de pacto orbitando al portador.
            let n = self.taken_pacts.len().max(1) as f32;
            for (i, pact) in self.taken_pacts.iter().enumerate() {
                let ang = self.time * 1.1 + i as f32 * std::f32::consts::TAU / n;
                let px = self.player.pos.x + ang.cos() * 17.0;
                let py = self.player.pos.y - 2.0 + ang.sin() * 9.0;
                draw_triangle(
                    vec2(px, py - 2.2),
                    vec2(px - 2.0, py),
                    vec2(px, py + 2.2),
                    with_alpha(pact.color(), 0.85),
                );
                draw_triangle(
                    vec2(px, py - 2.2),
                    vec2(px + 2.0, py),
                    vec2(px, py + 2.2),
                    with_alpha(pact.color(), 0.85),
                );
            }
            // Sigilos orbitales del objeto.
            if self.player.orbit > 0 {
                let n = self.player.orbit as f32;
                let radius = 25.0 + self.player.orbit as f32 * 3.5;
                for i in 0..self.player.orbit {
                    let ang = self.time * 2.4 + i as f32 * std::f32::consts::TAU / n;
                    let px = self.player.pos.x + ang.cos() * radius;
                    let py = self.player.pos.y + ang.sin() * radius * 0.62;
                    draw_circle(px, py, 3.4, with_alpha(C_PURPLE, 0.18));
                    draw_circle(px, py, 1.6, Color::from_rgba(210, 160, 255, 255));
                    draw_line(
                        self.player.pos.x,
                        self.player.pos.y,
                        px,
                        py,
                        0.4,
                        with_alpha(C_PURPLE, 0.22),
                    );
                }
            }
            // Aura persistente de los ataques equipados: un anillo tenue con
            // pulsos interiores para que se vea que está actuando aunque no
            // haya enemigos cerca.
            let mut aura_visual: Option<(f32, Color)> = None;
            for e in &self.equipped_attacks {
                let d = &self.attacks[e.def];
                if matches!(d.kind, AttackKind::Aura | AttackKind::Orbit) {
                    let r = attacks::level_radius(d.base_radius, e.level) * 0.74;
                    if aura_visual.map(|(best, _)| r > best).unwrap_or(true) {
                        aura_visual = Some((r, d.color));
                    }
                }
            }
            if let Some((radius, c)) = aura_visual {
                let pulse = 0.5 + 0.5 * (self.time * 3.2).sin();
                draw_circle_lines(
                    self.player.pos.x,
                    self.player.pos.y,
                    radius,
                    1.0,
                    with_alpha(c, 0.16 + pulse * 0.10),
                );
                for k in 0..3 {
                    let t = (self.time * 0.55 + k as f32 / 3.0) % 1.0;
                    let rr = radius * (0.22 + t * 0.74);
                    draw_circle_lines(
                        self.player.pos.x,
                        self.player.pos.y,
                        rr,
                        0.8,
                        with_alpha(c, (1.0 - t) * 0.22),
                    );
                }
                for k in 0..6 {
                    let a = k as f32 * std::f32::consts::TAU / 6.0 + self.time * 1.6;
                    let rr = radius * (0.45 + 0.12 * (self.time * 4.0 + k as f32).sin());
                    draw_circle(
                        self.player.pos.x + a.cos() * rr,
                        self.player.pos.y + a.sin() * rr * 0.68,
                        1.1,
                        with_alpha(c, 0.35 + pulse * 0.30),
                    );
                }
            }
        }

        for p in &self.projectiles {
            let dir = safe_norm(p.vel);
            if p.friendly {
                let r = p.radius.max(1.8);
                for k in 1..=2 {
                    let t = p.pos - dir * (k as f32 * 4.5);
                    draw_circle(t.x, t.y, r * 0.55, with_alpha(p.color, 0.20 / k as f32));
                }
                // Astilla de luz orientada hacia donde vuela.
                let tip = p.pos + dir * (r * 1.7);
                let back = p.pos - dir * (r * 1.15);
                let side = vec2(-dir.y, dir.x) * (r * 0.75);
                draw_triangle(tip, back + side, p.pos, p.color);
                draw_triangle(tip, back - side, p.pos, p.color);
                draw_circle(p.pos.x, p.pos.y, r * 0.45, with_alpha(C_WHITE, 0.90));
            } else {
                for k in 1..=2 {
                    let t = p.pos - dir * (k as f32 * 5.0);
                    draw_circle(
                        t.x,
                        t.y,
                        (p.radius * 0.7).max(1.0),
                        with_alpha(p.color, 0.16 / k as f32),
                    );
                }
                // Orbe sellado: núcleo oscuro y aro elemental.
                draw_circle(p.pos.x, p.pos.y, p.radius + 1.4, with_alpha(p.color, 0.18));
                draw_circle(
                    p.pos.x,
                    p.pos.y,
                    p.radius,
                    Color::new(0.05, 0.03, 0.08, 0.95),
                );
                draw_circle_lines(p.pos.x, p.pos.y, p.radius, 0.8, with_alpha(p.color, 0.95));
                draw_circle(p.pos.x, p.pos.y, p.radius * 0.38, with_alpha(p.color, 0.85));
            }
        }

        for p in &self.particles {
            if !p.active {
                continue;
            }
            let a = (p.life / p.max_life).clamp(0.0, 1.0);
            draw_rectangle(
                p.pos.x - p.size * 0.5,
                p.pos.y - p.size * 0.5,
                p.size,
                p.size,
                with_alpha(p.color, p.color.a * a),
            );
        }

        for r in &self.rings {
            if !r.active {
                continue;
            }
            let a = (r.life / r.max_life).clamp(0.0, 1.0);
            if r.fill {
                draw_circle(
                    r.pos.x,
                    r.pos.y,
                    r.radius,
                    with_alpha(r.color, r.color.a * a * 0.20),
                );
            }
            draw_circle_lines(
                r.pos.x,
                r.pos.y,
                r.radius,
                r.thickness,
                with_alpha(r.color, r.color.a * a * 0.85),
            );
            draw_circle_lines(
                r.pos.x,
                r.pos.y,
                (r.radius * 0.72).max(1.0),
                r.thickness * 0.7,
                with_alpha(r.color, r.color.a * a * 0.35),
            );
        }

        if self.room_fade > 0.0 {
            let a = (self.room_fade / 0.34).clamp(0.0, 1.0) * 0.92;
            draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.01, 0.008, 0.02, a));
        }
        // Feedback de daño: tinte rojo en los bordes y pulso con poca vida.
        if self.screen != Screen::Dead && self.screen != Screen::Victory {
            figures::draw_damage_vignette(
                self.player.hp / self.player.max_hp.max(1.0),
                self.player.hit_flash,
                self.time,
            );
        }
    }

    /// Cámara de la cripta: el suelo, los muros y un círculo ritual que cambia
    /// según la sala. Las salas flotan en un abismo oscuro en vez de ser
    /// celdas de un pasillo continuo.
    fn draw_chamber(&self) {
        let (l, r, t, b) = self.arena();
        let kind = self.current_kind();
        let accent = kind.color();
        let w = 10.0;

        // Abismo.
        draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.010, 0.009, 0.020, 1.0));
        // Polvo de estrellas en los márgenes: la cámara flota en un vacío.
        for (sx, sy, sz) in [
            (8.0, 70.0, 0.7),
            (21.0, 112.0, 0.9),
            (34.0, 158.0, 0.6),
            (13.0, 201.0, 0.8),
            (41.0, 232.0, 0.6),
            (444.0, 64.0, 0.8),
            (462.0, 106.0, 0.6),
            (452.0, 151.0, 1.0),
            (470.0, 196.0, 0.7),
            (437.0, 229.0, 0.6),
            (70.0, 236.0, 0.5),
            (120.0, 242.0, 0.7),
            (181.0, 233.0, 0.5),
            (240.0, 244.0, 0.8),
            (301.0, 236.0, 0.6),
            (361.0, 243.0, 0.7),
            (411.0, 234.0, 0.5),
        ] {
            let tw = 0.5 + (self.time * 1.3 + sx * 0.07 + sy * 0.03).sin() * 0.5;
            draw_circle(sx, sy, sz, with_alpha(C_WHITE, 0.12 + tw * 0.30));
        }

        // Suelo de losas.
        draw_rectangle(l, t, r - l, b - t, Color::from_rgba(23, 25, 39, 255));
        let tile: f32 = 32.0;
        let mut row = 0;
        let mut y = t;
        while y < b {
            let mut col = 0;
            let mut x = l;
            while x < r {
                if (row + col) % 2 == 0 {
                    draw_rectangle(
                        x,
                        y,
                        tile.min(r - x),
                        tile.min(b - y),
                        Color::from_rgba(27, 30, 46, 255),
                    );
                }
                x += tile;
                col += 1;
            }
            y += tile;
            row += 1;
        }
        let mut x = l;
        while x <= r + 0.5 {
            draw_line(x, t, x, b, 0.4, Color::new(0.04, 0.045, 0.075, 0.55));
            x += tile;
        }
        let mut y = t;
        while y <= b + 0.5 {
            draw_line(l, y, r, y, 0.4, Color::new(0.04, 0.045, 0.075, 0.55));
            y += tile;
        }

        // La cámara del guardián tiene un suelo ritual propio.
        if kind == RoomKind::Boss {
            let cx = (l + r) * 0.5;
            let cy = (t + b) * 0.5;
            let pulse = 0.70 + (self.time * 1.8).sin() * 0.30;
            draw_rectangle(l, t, r - l, b - t, Color::from_rgba(36, 14, 24, 48));
            for radius in [96.0f32, 70.0, 40.0] {
                draw_circle_lines(cx, cy, radius, 0.9, with_alpha(C_RED, 0.22 * pulse));
            }
            for k in 0..12 {
                let a = k as f32 * std::f32::consts::TAU / 12.0 + self.time * 0.05;
                let inner = 40.0;
                let outer = if k % 2 == 0 { 96.0 } else { 70.0 };
                draw_line(
                    cx + a.cos() * inner,
                    cy + a.sin() * inner,
                    cx + a.cos() * outer,
                    cy + a.sin() * outer,
                    0.7,
                    with_alpha(C_RED, 0.20 * pulse),
                );
                let rr = outer + 5.0;
                draw_circle(
                    cx + a.cos() * rr,
                    cy + a.sin() * rr,
                    1.4,
                    with_alpha(C_GOLD, 0.30 * pulse),
                );
            }
            draw_circle(cx, cy, 5.0, with_alpha(C_RED, 0.16 * pulse));
        }

        // Muros de piedra alrededor de la cámara.
        let (wall, wall_hi) = if kind == RoomKind::Boss {
            (
                Color::from_rgba(26, 12, 20, 255),
                Color::from_rgba(72, 30, 44, 255),
            )
        } else {
            (
                Color::from_rgba(15, 16, 26, 255),
                Color::from_rgba(44, 42, 62, 255),
            )
        };
        draw_rectangle(l - w, t - w, (r - l) + w * 2.0, w, wall);
        draw_rectangle(l - w, b, (r - l) + w * 2.0, w, wall);
        draw_rectangle(l - w, t, w, b - t, wall);
        draw_rectangle(r, t, w, b - t, wall);
        // Hiladas de ladrillo.
        let mut bx = l - w;
        while bx < r + w {
            draw_line(bx, t - w, bx, t, 0.5, with_alpha(wall_hi, 0.55));
            draw_line(bx, b, bx, b + w, 0.5, with_alpha(wall_hi, 0.55));
            bx += 16.0;
        }
        let mut by = t;
        while by < b {
            draw_line(l - w, by, l, by, 0.5, with_alpha(wall_hi, 0.55));
            draw_line(r, by, r + w, by, 0.5, with_alpha(wall_hi, 0.55));
            by += 12.0;
        }
        // Remate interior y exterior.
        draw_rectangle_lines(
            l - w,
            t - w,
            (r - l) + w * 2.0,
            (b - t) + w * 2.0,
            1.0,
            with_alpha(C_GOLD, 0.28),
        );
        draw_rectangle_lines(l, t, r - l, b - t, 1.0, with_alpha(accent, 0.42));

        // Esquinas con sigilo.
        for (cx, cy) in [
            (l + 9.0, t + 9.0),
            (r - 9.0, t + 9.0),
            (l + 9.0, b - 9.0),
            (r - 9.0, b - 9.0),
        ] {
            draw_triangle(
                vec2(cx, cy - 4.0),
                vec2(cx - 4.0, cy),
                vec2(cx, cy + 4.0),
                with_alpha(accent, 0.30),
            );
            draw_triangle(
                vec2(cx, cy - 4.0),
                vec2(cx + 4.0, cy),
                vec2(cx, cy + 4.0),
                with_alpha(accent, 0.30),
            );
        }

        // Círculo ritual central.
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        let rad = ((r - l).min(b - t) * 0.30).clamp(22.0, 56.0);
        draw_circle_lines(cx, cy, rad, 1.0, with_alpha(accent, 0.22));
        draw_circle_lines(cx, cy, rad * 0.70, 0.7, with_alpha(accent, 0.14));
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::FRAC_PI_4 + self.time * 0.05;
            let (sa, ca) = a.sin_cos();
            draw_line(
                cx + ca * rad * 0.86,
                cy + sa * rad * 0.86,
                cx + ca * rad * 1.04,
                cy + sa * rad * 1.04,
                1.0,
                with_alpha(accent, 0.30),
            );
        }
        draw_triangle(
            vec2(cx, cy - 7.0),
            vec2(cx - 7.0, cy),
            vec2(cx, cy + 7.0),
            with_alpha(accent, 0.16),
        );
        draw_triangle(
            vec2(cx, cy - 7.0),
            vec2(cx + 7.0, cy),
            vec2(cx, cy + 7.0),
            with_alpha(accent, 0.16),
        );
    }

    fn draw_room_shell(&self, _s: &Sprites) {
        let room = match self.rooms.get(self.room_index) {
            Some(r) => r,
            None => return,
        };
        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        let stone = Color::from_rgba(42, 40, 58, 255);
        let dark = Color::new(0.012, 0.010, 0.020, 0.98);
        for dir in 0..4 {
            if !room.doors[dir] {
                continue;
            }
            let locked = room.locked[dir];
            let closed = !self.cleared;
            let horizontal = dir == 0 || dir == 2;
            let p = match dir {
                0 => vec2(cx, t),
                1 => vec2(r, cy),
                2 => vec2(cx, b),
                _ => vec2(l, cy),
            };
            let half = 20.0;
            // Destino de la puerta: se muestra con un emblema, sin texto.
            let (ddx, ddy) = match dir {
                0 => (0, -1),
                1 => (1, 0),
                2 => (0, 1),
                _ => (-1, 0),
            };
            let target_kind = self
                .room_at(room.gx + ddx, room.gy + ddy)
                .map(|i| self.rooms[i].kind);

            // Paso oscuro y jambas de piedra.
            if horizontal {
                draw_rectangle(p.x - half, p.y - 6.0, half * 2.0, 12.0, dark);
                draw_rectangle(p.x - half - 3.0, p.y - 7.0, 3.0, 14.0, stone);
                draw_rectangle(p.x + half, p.y - 7.0, 3.0, 14.0, stone);
                draw_rectangle(p.x - half - 3.0, p.y - 7.0, half * 2.0 + 6.0, 2.0, stone);
                draw_line(
                    p.x - half,
                    p.y - 5.0,
                    p.x - half,
                    p.y + 5.0,
                    0.6,
                    with_alpha(C_GOLD, 0.30),
                );
                draw_line(
                    p.x + half,
                    p.y - 5.0,
                    p.x + half,
                    p.y + 5.0,
                    0.6,
                    with_alpha(C_GOLD, 0.30),
                );
            } else {
                draw_rectangle(p.x - 6.0, p.y - half, 12.0, half * 2.0, dark);
                draw_rectangle(p.x - 7.0, p.y - half - 3.0, 14.0, 3.0, stone);
                draw_rectangle(p.x - 7.0, p.y + half, 14.0, 3.0, stone);
                draw_rectangle(p.x - 7.0, p.y - half - 3.0, 2.0, half * 2.0 + 6.0, stone);
                draw_line(
                    p.x - 5.0,
                    p.y - half,
                    p.x + 5.0,
                    p.y - half,
                    0.6,
                    with_alpha(C_GOLD, 0.30),
                );
                draw_line(
                    p.x - 5.0,
                    p.y + half,
                    p.x + 5.0,
                    p.y + half,
                    0.6,
                    with_alpha(C_GOLD, 0.30),
                );
            }

            // Puertas de piedra con dos hojas deslizantes. Se cierran de golpe
            // al empezar el combate y se abren al limpiar la sala.
            let seal = self.door_seal.clamp(0.0, 1.0);
            let slam = self.door_slam_t.clamp(0.0, 0.30);
            let jolt = if slam > 0.0 {
                (slam * 52.0).sin() * slam * 5.0
            } else {
                0.0
            };
            let frame = Color::from_rgba(88, 79, 108, 255);
            let frame_hi = Color::from_rgba(138, 124, 162, 255);
            let leaf = Color::from_rgba(72, 62, 88, 255);
            let leaf_hi = Color::from_rgba(114, 100, 136, 255);
            let band = Color::from_rgba(170, 140, 92, 255);
            let seam = if locked {
                C_GOLD
            } else if closed {
                C_RED
            } else {
                C_CYAN
            };
            if horizontal {
                // Marco y hojas que entran desde las jambas.
                draw_rectangle(p.x - half - 3.0, p.y - 8.0, 3.0, 16.0, frame);
                draw_rectangle(p.x + half, p.y - 8.0, 3.0, 16.0, frame);
                draw_rectangle(p.x - half - 3.0, p.y - 8.0, half * 2.0 + 6.0, 2.0, frame_hi);
                let w = half * seal;
                if w > 0.5 {
                    draw_rectangle(p.x - half + jolt, p.y - 6.0, w, 12.0, leaf);
                    draw_rectangle(p.x - half + jolt, p.y - 6.0, w, 2.0, leaf_hi);
                    draw_line(
                        p.x - half + w * 0.5 + jolt,
                        p.y - 6.0,
                        p.x - half + w * 0.5 + jolt,
                        p.y + 6.0,
                        1.0,
                        band,
                    );
                    draw_rectangle(p.x - half + jolt, p.y + 4.0, w, 2.0, band);
                    draw_rectangle(p.x + half - w - jolt, p.y - 6.0, w, 12.0, leaf);
                    draw_rectangle(p.x + half - w - jolt, p.y - 6.0, w, 2.0, leaf_hi);
                    draw_line(
                        p.x + half - w * 0.5 - jolt,
                        p.y - 6.0,
                        p.x + half - w * 0.5 - jolt,
                        p.y + 6.0,
                        1.0,
                        band,
                    );
                    draw_rectangle(p.x + half - w - jolt, p.y + 4.0, w, 2.0, band);
                }
                if seal > 0.80 {
                    let a = ((seal - 0.80) / 0.20).clamp(0.0, 1.0);
                    draw_line(
                        p.x,
                        p.y - 6.0,
                        p.x,
                        p.y + 6.0,
                        1.0,
                        with_alpha(seam, 0.8 * a),
                    );
                }
            } else {
                draw_rectangle(p.x - 8.0, p.y - half - 3.0, 16.0, 3.0, frame);
                draw_rectangle(p.x - 8.0, p.y + half, 16.0, 3.0, frame);
                draw_rectangle(p.x - 8.0, p.y - half - 3.0, 2.0, half * 2.0 + 6.0, frame_hi);
                let h = half * seal;
                if h > 0.5 {
                    draw_rectangle(p.x - 6.0, p.y - half + jolt, 12.0, h, leaf);
                    draw_rectangle(p.x - 6.0, p.y - half + jolt, 2.0, h, leaf_hi);
                    draw_line(
                        p.x - 6.0,
                        p.y - half + h * 0.5 + jolt,
                        p.x + 6.0,
                        p.y - half + h * 0.5 + jolt,
                        1.0,
                        band,
                    );
                    draw_rectangle(p.x + 4.0, p.y - half + jolt, 2.0, h, band);
                    draw_rectangle(p.x - 6.0, p.y + half - h - jolt, 12.0, h, leaf);
                    draw_rectangle(p.x - 6.0, p.y + half - h - jolt, 2.0, h, leaf_hi);
                    draw_line(
                        p.x - 6.0,
                        p.y + half - h * 0.5 - jolt,
                        p.x + 6.0,
                        p.y + half - h * 0.5 - jolt,
                        1.0,
                        band,
                    );
                    draw_rectangle(p.x + 4.0, p.y + half - h - jolt, 2.0, h, band);
                }
                if seal > 0.80 {
                    let a = ((seal - 0.80) / 0.20).clamp(0.0, 1.0);
                    draw_line(
                        p.x - 6.0,
                        p.y,
                        p.x + 6.0,
                        p.y,
                        1.0,
                        with_alpha(seam, 0.8 * a),
                    );
                }
            }

            // Marcador de estado: candado dorado si falta una llave, runa roja
            // si el combate sella la puerta, flecha cian si el paso está libre.
            let pulse = 0.85 + (self.time * 3.0 + dir as f32).sin() * 0.15;
            if locked {
                let bob = (self.time * 2.4 + dir as f32).sin() * 0.8;
                let y = p.y - 10.0 + bob;
                draw_circle(p.x, y, 8.0 * pulse, with_alpha(C_GOLD, 0.10));
                // Arco y cuerpo del candado.
                draw_circle_lines(p.x, y - 2.2, 2.6, 1.0, with_alpha(C_GOLD, 0.90));
                draw_rectangle(
                    p.x - 1.1,
                    y - 0.8,
                    2.2,
                    1.6,
                    Color::from_rgba(20, 15, 8, 255),
                );
                draw_rectangle(p.x - 3.2, y, 6.4, 5.2, C_GOLD);
                draw_rectangle(
                    p.x - 2.4,
                    y + 0.8,
                    4.8,
                    3.6,
                    Color::from_rgba(70, 48, 12, 255),
                );
                draw_circle(p.x, y + 2.2, 1.1, C_GOLD);
                draw_rectangle(p.x - 0.4, y + 2.2, 0.8, 2.1, C_GOLD);
            } else if closed {
                draw_circle(p.x, p.y, 7.5, with_alpha(C_RED, 0.10));
                draw_circle_lines(p.x, p.y, 6.2, 1.0, with_alpha(C_RED, 0.80));
                for k in 0..6 {
                    let a = k as f32 * std::f32::consts::TAU / 6.0 + self.time * 0.6;
                    draw_circle(
                        p.x + a.cos() * 6.2,
                        p.y + a.sin() * 6.2,
                        0.7,
                        with_alpha(C_RED, 0.85),
                    );
                }
            } else {
                draw_circle(p.x, p.y, 6.0 * pulse, with_alpha(C_CYAN, 0.10));
                draw_circle_lines(p.x, p.y, 4.8, 0.8, with_alpha(C_CYAN, 0.50));
                // Flecha de paso hacia el interior.
                let tip = if dir == 0 {
                    vec2(p.x, p.y + 2.0)
                } else if dir == 2 {
                    vec2(p.x, p.y - 2.0)
                } else if dir == 1 {
                    vec2(p.x - 2.0, p.y)
                } else {
                    vec2(p.x + 2.0, p.y)
                };
                draw_line(p.x, p.y, tip.x, tip.y, 1.0, with_alpha(C_CYAN, 0.75));
            }

            // Emblema del destino sobre la pared, junto a la puerta.
            if let Some(tk) = target_kind {
                let off = if horizontal {
                    vec2(27.0, 0.0)
                } else {
                    vec2(0.0, -27.0)
                };
                draw_door_emblem(tk, p + off, self.time, 1.0);
            }
        }
    }

    /// Decoración procedural única de cada sala. La semilla cambia las losas,
    /// grietas, manchas de humedad, huesos, runas y estandartes, de modo que
    /// ninguna cámara se siente repetida. Se dibuja bajo props y enemigos.
    fn draw_room_ambience(&self) {
        let room = match self.rooms.get(self.room_index) {
            Some(r) => r,
            None => return,
        };
        let seed = room.decor_seed;
        let (l, r, t, b) = self.arena();
        let w = r - l;
        let h = b - t;
        let accent = room.kind.color();
        let stone_line = Color::new(0.40, 0.37, 0.50, 0.055);
        let crack = Color::new(0.015, 0.012, 0.025, 0.60);
        let crack_hi = Color::new(0.62, 0.58, 0.74, 0.075);

        // Losas: retícula irregular que sigue el tamaño de la sala.
        let cols = (w / 48.0).round().clamp(3.0, 9.0) as u32;
        let rows = (h / 42.0).round().clamp(2.0, 5.0) as u32;
        for iy in 1..rows {
            let y = t + iy as f32 * h / rows as f32;
            draw_line(l + 10.0, y, r - 10.0, y, 0.6, stone_line);
        }
        for ix in 1..cols {
            let x = l + ix as f32 * w / cols as f32 + (decor_rand(seed, ix) - 0.5) * 12.0;
            draw_line(x, t + 10.0, x, b - 10.0, 0.6, stone_line);
        }

        // Manchas de humedad y musgo: verdes y azuladas, siempre tenues.
        for i in 0..4u32 {
            let x = l + 20.0 + decor_rand(seed, 10 + i) * (w - 40.0);
            let y = t + 16.0 + decor_rand(seed, 20 + i) * (h - 32.0);
            let rad = 9.0 + decor_rand(seed, 30 + i) * 15.0;
            let tone = if i % 2 == 0 {
                Color::new(0.15, 0.30, 0.20, 1.0)
            } else {
                Color::new(0.13, 0.19, 0.30, 1.0)
            };
            draw_ellipse(x, y, rad, rad * 0.40, 0.0, with_alpha(tone, 0.09));
            draw_ellipse(
                x + rad * 0.28,
                y + 2.0,
                rad * 0.58,
                rad * 0.22,
                0.0,
                with_alpha(tone, 0.08),
            );
        }

        // Grietas quebradas: tres por sala, con un filo de luz en el borde.
        for c in 0..3u32 {
            let mut x = l + 24.0 + decor_rand(seed, 40 + c) * (w - 48.0);
            let mut y = t + 22.0 + decor_rand(seed, 50 + c) * (h - 44.0);
            let mut ang = decor_rand(seed, 60 + c) * std::f32::consts::TAU;
            for seg in 0..5u32 {
                let len = 4.0 + decor_rand(seed, 70 + c * 10 + seg) * 7.5;
                ang += (decor_rand(seed, 80 + c * 10 + seg) - 0.5) * 1.25;
                let nx = x + ang.cos() * len;
                let ny = y + ang.sin() * len * 0.55;
                draw_line(x, y, nx, ny, 1.0 - seg as f32 * 0.11, crack);
                draw_line(x, y + 0.7, nx, ny + 0.7, 0.5, crack_hi);
                x = nx;
                y = ny;
            }
        }

        // Huesos olvidados.
        let bone = Color::new(0.76, 0.71, 0.60, 0.20);
        for i in 0..3u32 {
            let x = l + 30.0 + decor_rand(seed, 100 + i) * (w - 60.0);
            let y = t + 32.0 + decor_rand(seed, 110 + i) * (h - 54.0);
            let ang = decor_rand(seed, 120 + i) * std::f32::consts::TAU;
            let dx = ang.cos() * 4.0;
            let dy = ang.sin() * 4.0;
            draw_line(x - dx, y - dy, x + dx, y + dy, 1.3, bone);
            draw_circle(x - dx, y - dy, 1.1, bone);
            draw_circle(x + dx, y + dy, 1.1, bone);
        }

        // Runas circulares: el sello de la cámara, casi borrado.
        let rx = l + w * (0.34 + decor_rand(seed, 130) * 0.32);
        let ry = t + h * 0.66;
        draw_circle_lines(rx, ry, 18.0, 0.7, with_alpha(accent, 0.10));
        draw_circle_lines(rx, ry, 12.0, 0.5, with_alpha(accent, 0.07));
        for k in 0..6 {
            let a = k as f32 * std::f32::consts::TAU / 6.0 + self.time * 0.06;
            draw_line(
                rx + a.cos() * 12.0,
                ry + a.sin() * 12.0,
                rx + a.cos() * 17.0,
                ry + a.sin() * 17.0,
                0.7,
                with_alpha(accent, 0.11),
            );
        }

        // Estandartes raídos en la pared superior, lejos de la puerta.
        for i in 0..2u32 {
            let x = l + w * (0.22 + i as f32 * 0.56) + (decor_rand(seed, 140 + i) - 0.5) * 16.0;
            let len = 15.0 + decor_rand(seed, 150 + i) * 8.0;
            let cloth = with_alpha(accent, 0.16);
            draw_line(
                x - 7.0,
                t + 3.0,
                x + 7.0,
                t + 3.0,
                1.1,
                with_alpha(C_GOLD, 0.30),
            );
            draw_rectangle(x - 6.0, t + 3.0, 12.0, len, cloth);
            draw_triangle(
                vec2(x - 6.0, t + 3.0 + len),
                vec2(x + 6.0, t + 3.0 + len),
                vec2(x, t + 7.0 + len),
                Color::new(0.01, 0.008, 0.02, 0.55),
            );
            draw_line(x, t + 5.0, x, t + 2.0 + len, 0.5, with_alpha(accent, 0.22));
        }

        // Velas en las esquinas inferiores: dos puntos cálidos y su halo.
        for i in 0..2u32 {
            let x = if i == 0 { l + 15.0 } else { r - 15.0 };
            let y = b - 16.0;
            let flick = 0.8 + (self.time * 9.0 + i as f32 * 2.0).sin() * 0.2;
            draw_circle(x, y - 4.0, 7.0, Color::new(1.0, 0.72, 0.32, 0.07 * flick));
            draw_rectangle(
                x - 1.6,
                y - 4.0,
                3.2,
                6.0,
                Color::new(0.86, 0.82, 0.70, 0.55),
            );
            draw_circle(x, y - 5.2, 1.1 * flick, Color::new(1.0, 0.82, 0.42, 0.85));
            draw_line(x, y + 2.0, x, y + 4.0, 0.7, with_alpha(C_GOLD, 0.30));
        }
    }

    fn draw_room_decor(&self, s: &Sprites) {
        self.draw_room_ambience();
        let kind = self.current_kind();
        let seed = self
            .rooms
            .get(self.room_index)
            .map(|r| r.decor_seed)
            .unwrap_or(0);
        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        match kind {
            RoomKind::Rest => {
                let sz = 86.0;
                draw_texture_ex(
                    &s.glow_white,
                    cx - sz * 0.5,
                    cy - sz * 0.5,
                    Color::new(0.35, 0.75, 0.95, 0.35),
                    DrawTextureParams {
                        dest_size: Some(vec2(sz, sz)),
                        ..Default::default()
                    },
                );
                sprites::draw_shadow(vec2(cx, cy + 2.0), 30.0, 18.0);
                figures::draw_bed(vec2(cx, cy + 4.0), self.time);

                // Dos altares de pacto: plinto de piedra y sigilo flotante.
                for i in 0..2 {
                    let p = self.pact_pos(i);
                    let pact = self.rest_offers[i];
                    let used = self.rooms[self.room_index].pact_used;
                    let pulse = 0.85 + (self.time * 3.0 + i as f32 * 1.7).sin() * 0.15;
                    let bob = (self.time * 2.2 + i as f32 * 2.0).sin() * 1.6;
                    let a = if used { 0.28 } else { 1.0 };
                    let sz = 48.0 * pulse;
                    draw_texture_ex(
                        &s.glow_white,
                        p.x - sz * 0.5,
                        p.y - 8.0 - sz * 0.5,
                        with_alpha(pact.color(), if used { 0.08 } else { 0.34 }),
                        DrawTextureParams {
                            dest_size: Some(vec2(sz, sz)),
                            ..Default::default()
                        },
                    );
                    // Círculo ritual en el suelo.
                    draw_ellipse(
                        p.x,
                        p.y + 7.0,
                        13.0,
                        4.0,
                        0.0,
                        with_alpha(pact.color(), 0.18 * a),
                    );
                    draw_circle_lines(
                        p.x,
                        p.y + 7.0,
                        11.0,
                        0.7,
                        with_alpha(pact.color(), 0.35 * a),
                    );
                    // Plinto de piedra.
                    let stone = Color::from_rgba(38, 34, 50, 255);
                    draw_rectangle(p.x - 6.0, p.y + 1.0, 12.0, 7.0, stone);
                    draw_rectangle(
                        p.x - 8.0,
                        p.y + 8.0,
                        16.0,
                        3.0,
                        Color::from_rgba(28, 25, 38, 255),
                    );
                    draw_rectangle_lines(
                        p.x - 8.0,
                        p.y + 8.0,
                        16.0,
                        3.0,
                        0.7,
                        with_alpha(pact.color(), 0.45 * a),
                    );
                    draw_line(
                        p.x - 6.0,
                        p.y + 1.0,
                        p.x - 6.0,
                        p.y + 8.0,
                        0.6,
                        with_alpha(pact.color(), 0.50 * a),
                    );
                    draw_line(
                        p.x + 6.0,
                        p.y + 1.0,
                        p.x + 6.0,
                        p.y + 8.0,
                        0.6,
                        with_alpha(pact.color(), 0.50 * a),
                    );
                    // Sigilo flotante.
                    let cy = p.y - 9.0 + bob;
                    draw_triangle(
                        vec2(p.x, cy - 6.0),
                        vec2(p.x - 4.5, cy),
                        vec2(p.x, cy + 6.0),
                        with_alpha(pact.color(), 0.95 * a),
                    );
                    draw_triangle(
                        vec2(p.x, cy - 6.0),
                        vec2(p.x + 4.5, cy),
                        vec2(p.x, cy + 6.0),
                        with_alpha(pact.color(), 0.55 * a),
                    );
                    draw_circle(p.x, cy, 1.3, with_alpha(C_WHITE, 0.85 * a));
                    if !used {
                        for k in 0..3 {
                            let ang = self.time * 1.6 + k as f32 * 2.094 + i as f32;
                            draw_circle(
                                p.x + ang.cos() * 9.0,
                                cy + ang.sin() * 4.0,
                                0.9,
                                with_alpha(pact.color(), 0.80),
                            );
                        }
                    }
                }
            }
            RoomKind::Shop => {
                // Alfombra granate bajo el mostrador.
                draw_ellipse(
                    cx,
                    cy - 8.0,
                    92.0,
                    30.0,
                    0.0,
                    Color::new(0.20, 0.055, 0.07, 0.55),
                );
                draw_ellipse(
                    cx,
                    cy - 8.0,
                    84.0,
                    26.0,
                    0.0,
                    Color::new(0.30, 0.08, 0.10, 0.45),
                );
                draw_circle_lines(cx, cy - 8.0, 27.0, 0.7, with_alpha(C_GOLD, 0.28));

                // Halo cálido de la tienda.
                let sz = 132.0;
                draw_texture_ex(
                    &s.glow,
                    cx - sz * 0.5,
                    (cy - 24.0) - sz * 0.5,
                    Color::new(1.0, 0.72, 0.35, 0.34),
                    DrawTextureParams {
                        dest_size: Some(vec2(sz, sz)),
                        ..Default::default()
                    },
                );

                // Estanterías en las esquinas superiores, con frascos y libros.
                for side in [-1.0f32, 1.0] {
                    let sx = cx + side * (r - l) * 0.34;
                    let sy = t + 24.0;
                    draw_rectangle(sx - 17.0, sy, 34.0, 40.0, Color::from_rgba(42, 28, 20, 255));
                    draw_rectangle_lines(
                        sx - 17.0,
                        sy,
                        34.0,
                        40.0,
                        1.0,
                        Color::from_rgba(112, 80, 48, 255),
                    );
                    for shelf in 0..2 {
                        let yy = sy + 13.0 + shelf as f32 * 13.0;
                        draw_rectangle(sx - 16.0, yy, 32.0, 2.0, Color::from_rgba(96, 66, 38, 255));
                        for k in 0..3 {
                            let bx = sx - 12.0 + k as f32 * 10.0;
                            let hh = 4.0
                                + decor_rand(seed, (side as i32 + 2) as u32 * 17 + shelf * 5 + k)
                                    * 4.0;
                            let tone = match (shelf + k) % 3 {
                                0 => Color::new(0.30, 0.62, 0.54, 1.0),
                                1 => Color::new(0.62, 0.36, 0.24, 1.0),
                                _ => Color::new(0.52, 0.44, 0.72, 1.0),
                            };
                            draw_rectangle(bx, yy - hh, 5.0, hh, tone);
                            draw_rectangle(bx, yy - hh, 5.0, 1.2, with_alpha(C_WHITE, 0.35));
                            draw_rectangle(
                                bx - 0.4,
                                yy - hh - 1.4,
                                5.8,
                                1.4,
                                Color::from_rgba(70, 52, 34, 255),
                            );
                        }
                    }
                    // Caja de monedas al pie de la estantería.
                    draw_rectangle(
                        sx - 13.0,
                        sy + 40.0,
                        26.0,
                        8.0,
                        Color::from_rgba(52, 34, 22, 255),
                    );
                    draw_rectangle_lines(
                        sx - 13.0,
                        sy + 40.0,
                        26.0,
                        8.0,
                        0.7,
                        with_alpha(C_GOLD, 0.55),
                    );
                    draw_circle(sx - 4.0, sy + 44.0, 1.4, C_GOLD);
                    draw_circle(sx + 3.0, sy + 44.0, 1.2, C_GOLD);
                }

                // Farolillos colgantes sobre el mostrador.
                for side in [-1.0f32, 1.0] {
                    let lx = cx + side * 58.0;
                    let ly = t + 12.0 + (self.time * 1.6 + side).sin() * 1.2;
                    draw_line(lx, t + 2.0, lx, ly - 4.0, 0.7, with_alpha(C_GOLD, 0.35));
                    draw_circle(lx, ly, 6.0, Color::new(1.0, 0.70, 0.28, 0.13));
                    draw_rectangle(
                        lx - 3.0,
                        ly - 3.0,
                        6.0,
                        7.0,
                        Color::new(1.0, 0.78, 0.38, 0.80),
                    );
                    draw_rectangle(
                        lx - 3.6,
                        ly - 4.2,
                        7.2,
                        1.4,
                        Color::from_rgba(88, 62, 30, 255),
                    );
                    draw_rectangle(
                        lx - 3.6,
                        ly + 4.0,
                        7.2,
                        1.4,
                        Color::from_rgba(88, 62, 30, 255),
                    );
                }

                // Mercader procedural detrás del mostrador. Se anima más cuando
                // el héroe está cerca y juega con una moneda.
                let excited = self.near_merchant();
                let bob = (self.time * 2.0).sin() * if excited { 1.5 } else { 1.0 };
                figures::draw_hero(
                    vec2(cx, cy - 20.0 + bob),
                    Class::Merchant,
                    vec2(0.0, 1.0),
                    self.time * 1.6,
                    false,
                    0.0,
                    0.0,
                    C_WHITE,
                    0,
                    0,
                    self.time,
                );
                // La moneda sube desde detrás del mostrador y vuelve a caer.
                if excited {
                    let ph = (self.time * 0.8) % 1.0;
                    let arc = (ph * std::f32::consts::TAU).sin().abs();
                    let spin = (ph * 2.0 * std::f32::consts::TAU).cos();
                    let coin_x = cx + 17.0;
                    let coin_y = cy - 24.0 - arc * 9.0;
                    let cw = 1.0 + spin.abs() * 1.5;
                    draw_ellipse(coin_x, coin_y, cw, 1.5, 0.0, C_GOLD);
                    draw_ellipse(
                        coin_x,
                        coin_y,
                        cw * 0.5,
                        0.9,
                        0.0,
                        with_alpha(C_WHITE, 0.80),
                    );
                    if ph > 0.92 {
                        draw_circle(coin_x, cy - 24.0, 2.2, with_alpha(C_WHITE, 0.45));
                    }
                }

                // Mostrador de madera con tablones, cortina y borde dorado.
                let top = cy - 31.0;
                draw_rectangle(
                    cx - 54.0,
                    top,
                    108.0,
                    20.0,
                    Color::from_rgba(58, 36, 22, 255),
                );
                draw_rectangle(
                    cx - 54.0,
                    top,
                    108.0,
                    4.0,
                    Color::from_rgba(96, 64, 36, 255),
                );
                draw_rectangle(cx - 54.0, top + 4.0, 108.0, 1.4, with_alpha(C_GOLD, 0.80));
                for k in 0..5 {
                    draw_line(
                        cx - 48.0 + k as f32 * 24.0,
                        top + 6.0,
                        cx - 48.0 + k as f32 * 24.0,
                        top + 20.0,
                        0.6,
                        Color::from_rgba(36, 22, 14, 255),
                    );
                }
                // Cortina frontal con pliegues.
                draw_rectangle(
                    cx - 50.0,
                    top + 10.0,
                    100.0,
                    12.0,
                    Color::from_rgba(74, 20, 28, 255),
                );
                for k in 0..7 {
                    draw_line(
                        cx - 46.0 + k as f32 * 15.0,
                        top + 10.0,
                        cx - 46.0 + k as f32 * 15.0,
                        top + 22.0,
                        0.8,
                        Color::from_rgba(104, 32, 42, 255),
                    );
                }
                draw_rectangle(cx - 50.0, top + 20.0, 100.0, 2.0, with_alpha(C_GOLD, 0.70));
                draw_rectangle_lines(
                    cx - 54.0,
                    top,
                    108.0,
                    22.0,
                    0.9,
                    Color::from_rgba(120, 84, 46, 255),
                );

                // Balanza y monedas sobre el mostrador.
                let scale_x = cx + 30.0;
                draw_line(
                    scale_x,
                    top - 6.0,
                    scale_x,
                    top,
                    0.9,
                    with_alpha(C_GOLD, 0.85),
                );
                draw_line(
                    scale_x - 7.0,
                    top - 6.0,
                    scale_x + 7.0,
                    top - 6.0,
                    0.9,
                    with_alpha(C_GOLD, 0.85),
                );
                for sx2 in [scale_x - 7.0, scale_x + 7.0] {
                    draw_line(
                        sx2,
                        top - 6.0,
                        sx2,
                        top - 3.0,
                        0.5,
                        with_alpha(C_GOLD, 0.60),
                    );
                    draw_circle(sx2, top - 2.6, 1.6, with_alpha(C_GOLD, 0.55));
                }
                for k in 0..3 {
                    draw_circle(
                        cx - 30.0 + k as f32 * 8.0,
                        top - 2.0,
                        1.3,
                        with_alpha(C_GOLD, 0.85),
                    );
                }
            }
            RoomKind::Treasure => {
                let sz = 96.0;
                draw_texture_ex(
                    &s.glow_white,
                    cx - sz * 0.5,
                    cy - sz * 0.5,
                    Color::new(0.72, 0.44, 0.92, 0.40),
                    DrawTextureParams {
                        dest_size: Some(vec2(sz, sz)),
                        ..Default::default()
                    },
                );
            }
            RoomKind::Boss => {
                let sz = 120.0;
                draw_texture_ex(
                    &s.glow_white,
                    cx - sz * 0.5,
                    (t + 84.0) - sz * 0.5,
                    Color::new(0.85, 0.20, 0.30, 0.32),
                    DrawTextureParams {
                        dest_size: Some(vec2(sz, sz)),
                        ..Default::default()
                    },
                );
            }
            RoomKind::Start => {
                // Dos balizas astrales flanquean la entrada.
                for bx in [cx - 92.0, cx + 92.0] {
                    let pulse = 0.85 + (self.time * 3.4 + bx).sin() * 0.15;
                    let sz = 62.0 * pulse;
                    draw_texture_ex(
                        &s.glow_white,
                        bx - sz * 0.5,
                        (cy - 30.0) - sz * 0.5,
                        Color::new(0.46, 0.62, 1.0, 0.42),
                        DrawTextureParams {
                            dest_size: Some(vec2(sz, sz)),
                            ..Default::default()
                        },
                    );
                    let stone = Color::from_rgba(40, 38, 56, 255);
                    draw_rectangle(bx - 4.0, cy - 22.0, 8.0, 12.0, stone);
                    draw_rectangle(
                        bx - 6.0,
                        cy - 10.0,
                        12.0,
                        3.0,
                        Color::from_rgba(26, 24, 36, 255),
                    );
                    draw_rectangle_lines(
                        bx - 6.0,
                        cy - 10.0,
                        12.0,
                        3.0,
                        0.7,
                        with_alpha(C_CYAN, 0.42),
                    );
                    draw_line(
                        bx - 4.0,
                        cy - 22.0,
                        bx - 4.0,
                        cy - 10.0,
                        0.6,
                        with_alpha(C_CYAN, 0.45),
                    );
                    draw_line(
                        bx + 4.0,
                        cy - 22.0,
                        bx + 4.0,
                        cy - 10.0,
                        0.6,
                        with_alpha(C_CYAN, 0.45),
                    );
                    let gy = cy - 28.0 + (self.time * 2.1 + bx).sin() * 1.4;
                    draw_circle_lines(bx, gy, 7.0, 0.6, with_alpha(C_CYAN, 0.35));
                    draw_triangle(
                        vec2(bx, gy - 6.5),
                        vec2(bx - 4.2, gy),
                        vec2(bx, gy + 6.5),
                        with_alpha(C_CYAN, 0.95),
                    );
                    draw_triangle(
                        vec2(bx, gy - 6.5),
                        vec2(bx + 4.2, gy),
                        vec2(bx, gy + 6.5),
                        Color::from_rgba(150, 120, 255, 220),
                    );
                    draw_circle(bx, gy, 1.2, with_alpha(C_WHITE, 0.90));
                }

                // Cobrador de deudas: el mercader espera junto a la entrada.
                if self.debt > 0 {
                    let p = self.debt_pos();
                    let bob = (self.time * 2.6).sin() * 1.2;
                    sprites::draw_shadow(p, 18.0, 18.0);
                    let feet = p + vec2(0.0, bob + 8.0);
                    figures::draw_hero(
                        feet,
                        Class::Merchant,
                        vec2(0.0, 1.0),
                        self.time * 1.6,
                        false,
                        0.0,
                        0.0,
                        Color::new(1.0, 0.86, 0.55, 0.95),
                        0,
                        0,
                        self.time,
                    );
                    draw_cosmetics_world(feet + vec2(0.0, -11.0), 1.0, 0, 0, C_GOLD);
                    let label = format!("Debes {}", self.debt);
                    draw_text_ex(
                        &label,
                        p.x - 18.0,
                        p.y - 20.0,
                        TextParams {
                            font_size: 8,
                            color: C_GOLD,
                            ..Default::default()
                        },
                    );
                }

                // Eco de una partida muerta.
                if self.meta.echoes > 0 && !self.echo_claimed {
                    let p = self.echo_pos();
                    let bob = (self.time * 1.8).sin() * 2.0;
                    let ghost_class =
                        Class::ALL[(self.meta.echo_class as usize).min(Class::ALL.len() - 1)];
                    draw_texture_ex(
                        &s.glow_white,
                        p.x - 24.0,
                        p.y - 24.0 + bob,
                        Color::new(0.35, 0.85, 1.0, 0.28),
                        DrawTextureParams {
                            dest_size: Some(vec2(48.0, 48.0)),
                            ..Default::default()
                        },
                    );
                    figures::draw_hero(
                        p + vec2(0.0, bob + 8.0),
                        ghost_class,
                        vec2(0.0, 1.0),
                        self.time * 1.2,
                        false,
                        0.0,
                        0.0,
                        Color::new(0.55, 0.95, 1.0, 0.62),
                        0,
                        0,
                        self.time,
                    );
                    draw_text_ex(
                        "Eco",
                        p.x - 8.0,
                        p.y - 20.0,
                        TextParams {
                            font_size: 8,
                            color: C_CYAN,
                            ..Default::default()
                        },
                    );
                }
            }
            _ => {}
        }
    }

    pub fn draw_world_text(&self, ui: &Ui) {
        if self.screen != Screen::Menu
            && self.screen != Screen::Dead
            && self.screen != Screen::Victory
            && self.boss_intro_t <= 0.0
        {
            // El rótulo del héroe se oculta cuando taparía el cofre o el
            // mostrador del mercader.
            let near_chest = self
                .props
                .iter()
                .any(|p| p.kind == PropKind::Chest && self.player.pos.distance(p.pos) < 44.0);
            if self.current_kind() != RoomKind::Shop && !near_chest {
                let name = &self.meta.name;
                let nw = ui.measure(name, 6.0, true);
                ui.text_shadow_world(
                    name,
                    self.player.pos.x - nw * 0.5,
                    self.player.pos.y - 23.0,
                    6.0,
                    Color::from_rgba(242, 238, 252, 235),
                    true,
                );
            }
        }
        for f in &self.floaters {
            if !f.active {
                continue;
            }
            let a = (f.life / f.max_life).clamp(0.0, 1.0);
            let w = ui.measure(&f.text, 6.2, true);
            ui.text_shadow_world(
                &f.text,
                f.pos.x - w * 0.5,
                f.pos.y,
                6.2,
                with_alpha(f.color, a),
                true,
            );
        }

        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;

        // El nombre del guardián se muestra en su barra del HUD, no sobre el
        // sprite: así no se pisa con los avisos de la sala.

        // Avisos de interacción.
        if self.screen == Screen::Playing {
            for p in &self.props {
                if p.kind == PropKind::Chest && !p.opened && self.player.pos.distance(p.pos) < 36.0
                {
                    let text = "E  Abrir cofre";
                    let w = ui.measure(text, 6.0, true);
                    ui.text_shadow_world(
                        text,
                        p.pos.x - w * 0.5,
                        p.pos.y - 22.0,
                        6.0,
                        C_GOLD,
                        true,
                    );
                }
            }
            if self.current_kind() == RoomKind::Rest && !self.rooms[self.room_index].pact_used {
                for i in 0..2 {
                    let p = self.pact_pos(i);
                    if self.player.pos.distance(p) < 34.0 {
                        let pact = self.rest_offers[i];
                        let text = format!("E  Aceptar {}", pact.name());
                        let w = ui.measure(&text, 5.8, true);
                        ui.text_shadow_world(
                            &text,
                            p.x - w * 0.5,
                            p.y - 24.0,
                            5.8,
                            pact.color(),
                            true,
                        );
                        let desc = pact.desc();
                        let dw = ui.measure(desc, 4.6, true);
                        ui.text_shadow_world(
                            desc,
                            p.x - dw * 0.5,
                            p.y - 35.0,
                            4.6,
                            with_alpha(pact.color(), 0.88),
                            true,
                        );
                    }
                }
            }
            if self.current_kind() == RoomKind::Shop {
                let near = self.player.pos.distance(vec2(cx, cy + 6.0)) < 88.0;
                if near {
                    let sold = self.shop_offers.iter().filter(|o| o.sold).count();
                    let cheapest = self
                        .shop_offers
                        .iter()
                        .filter(|o| !o.sold)
                        .map(|o| o.cost)
                        .min()
                        .unwrap_or(0);
                    let line = if sold >= self.shop_offers.len() {
                        "Te has llevado todo. Vuelve con más oro."
                    } else if self.player.hp >= self.player.max_hp - 0.01
                        && self
                            .shop_offers
                            .iter()
                            .any(|o| matches!(o.kind, ShopKind::Heal))
                    {
                        "Estás entero. La carne puede esperar."
                    } else if self.player.gold < cheapest {
                        "El oro abre más puertas que las llaves."
                    } else {
                        "Mira con calma. Nada de aquí es gratis."
                    };
                    let w = ui.measure(line, 5.8, true);
                    ui.text_shadow_world(line, cx - w * 0.5, cy - 46.0, 5.8, C_GOLD, true);
                }
            }
        }

        if self.current_kind() == RoomKind::Rest
            && self.rooms[self.room_index].used
            && self.screen == Screen::Playing
        {
            let text = if self.rooms[self.room_index].pact_used {
                "Sala de descanso agotada"
            } else {
                "Bien usado · Altares activos"
            };
            let w = ui.measure(text, 5.8, true);
            ui.text_shadow_world(
                text,
                cx - w * 0.5,
                t + 30.0,
                5.8,
                Color::from_rgba(150, 205, 220, 255),
                true,
            );
        }

        if self.cleared && self.current_kind() == RoomKind::Boss && self.screen == Screen::Playing {
            let d = self.player.pos.distance(self.portal);
            if d < 70.0 {
                let near = d < 30.0;
                let text = if near {
                    "Descendiendo..."
                } else {
                    "Acércate al portal"
                };
                let w = ui.measure(text, 6.6, true);
                ui.text_shadow_world(text, 240.0 - w * 0.5, 62.0, 6.6, C_CYAN, true);
                if self.player.gold >= 30 && self.player.hp < self.player.max_hp {
                    let t2 = "Q  Curar por 30 de oro";
                    let w2 = ui.measure(t2, 6.6, true);
                    ui.text_shadow_world(t2, 240.0 - w2 * 0.5, 73.0, 6.6, C_GREEN, true);
                }
            }
        }
    }

    pub fn draw_ui(&self, s: &Sprites, ui: &Ui) {
        draw_texture_ex(
            &s.vignette,
            ui.ox,
            ui.oy,
            C_WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VW * ui.scale, VH * ui.scale)),
                ..Default::default()
            },
        );
        match self.screen {
            Screen::Menu => self.draw_menu(s, ui),
            Screen::Playing => {
                self.draw_hud(s, ui);
                if self.current_kind() == RoomKind::Shop {
                    self.draw_shop_room(s, ui);
                }
                if self.map_open {
                    self.draw_map(ui);
                }
                if self.boss_intro_t > 0.0 {
                    self.draw_boss_intro(s, ui);
                }
                if self.floor_transition_t > 0.0 || self.floor_arrival_t > 0.0 {
                    self.draw_floor_transition(ui);
                }
            }
            Screen::LevelUp => self.draw_levelup(ui),
            Screen::Paused => self.draw_pause(ui),
            Screen::Dead => self.draw_dead(ui),
            Screen::Victory => self.draw_victory(ui),
            Screen::Shop => self.draw_shop(s, ui),
            Screen::Locker => self.draw_locker(s, ui),
            Screen::Stats => self.draw_stats(ui),
            Screen::Audio => self.draw_audio(ui),
            Screen::Library => self.draw_library(ui),
            Screen::Settings => self.draw_settings(ui),
            Screen::Inventory => self.draw_inventory(ui),
            Screen::Tickets => self.draw_tickets(ui),
            Screen::Forge => self.draw_forge(s, ui),
        }
        self.draw_toasts(ui);
        if self.dev_mode {
            self.draw_dev(ui);
        }
    }

    fn draw_hud(&self, _s: &Sprites, ui: &Ui) {
        // Barra superior: experiencia, piso, sala y oleada.
        ui.rect(0.0, 0.0, VW, 34.0, Color::new(0.020, 0.017, 0.038, 0.90));
        ui.rect(0.0, 33.0, VW, 1.0, with_alpha(C_GOLD, 0.26));
        ui.rect(0.0, 34.0, VW, 1.0, Color::new(0.0, 0.0, 0.0, 0.55));

        let room_label = match self.rooms.get(self.room_index) {
            Some(r) if r.kind == RoomKind::Vault && r.vault_open => "Bóveda abierta",
            _ => self.current_kind().name(),
        };
        ui.text_center_shadow(room_label, 13.0, 6.4, self.current_kind().color());
        if self.current_kind() == RoomKind::Combat || self.current_kind() == RoomKind::Boss {
            ui.text_center_shadow(
                &format!("Oleada {}/{}", self.wave.max(1), self.waves.max(1)),
                23.0,
                5.6,
                Color::from_rgba(228, 222, 240, 255),
            );
        } else if self.modifier != Modifier::None {
            ui.text_center_shadow(self.modifier_name(), 23.0, 5.6, self.modifier_color());
        }
        ui.text_shadow(
            &format!("Piso {}", self.floor),
            8.0,
            14.0,
            7.0,
            C_GOLD,
            true,
        );
        let diff = self.difficulty();
        let size_label = self
            .rooms
            .get(self.room_index)
            .map(|r| r.size.name())
            .unwrap_or("Mediana");
        ui.text_right_shadow(
            &format!("{} · {}", diff.name(), size_label),
            472.0,
            13.0,
            5.6,
            diff.color(),
            true,
        );

        // Experiencia arriba, siempre visible.
        let xf = (self.player.xp / self.player.xp_next.max(1.0)).clamp(0.0, 1.0);
        ui.rect(8.0, 27.0, 464.0, 5.0, Color::new(0.025, 0.021, 0.043, 0.96));
        ui.rect(8.0, 27.0, 464.0 * xf, 5.0, C_PURPLE);
        ui.rect(8.0, 27.0, 464.0 * xf, 1.5, with_alpha(C_WHITE, 0.22));
        for i in 1..10 {
            ui.rect(
                8.0 + i as f32 * 46.4,
                27.0,
                0.5,
                5.0,
                Color::new(0.010, 0.008, 0.020, 0.60),
            );
        }
        ui.text_shadow(
            &format!("Nv. {}", self.player.level),
            8.0,
            24.0,
            6.2,
            C_WHITE,
            true,
        );
        ui.text_right_shadow(
            &format!("{:.0}/{:.0} xp", self.player.xp, self.player.xp_next),
            472.0,
            24.0,
            5.6,
            Color::from_rgba(205, 198, 225, 255),
            false,
        );

        // Barra inferior: vida, esquiva, llaves, oro y ataques equipados.
        ui.rect(0.0, 238.0, VW, 32.0, Color::new(0.020, 0.017, 0.038, 0.92));
        ui.rect(0.0, 238.0, VW, 1.0, with_alpha(C_GOLD, 0.20));

        let hp_frac = (self.player.hp / self.player.max_hp.max(1.0)).clamp(0.0, 1.0);
        let hp_col = if hp_frac < 0.30 {
            C_RED
        } else if hp_frac < 0.55 {
            Color::from_rgba(226, 150, 70, 255)
        } else {
            Color::from_rgba(196, 68, 78, 255)
        };
        let sx = 16.0;
        let sy = 252.0;
        ui.triangle(
            vec2(sx, sy - 6.5),
            vec2(sx - 5.5, sy),
            vec2(sx, sy + 6.5),
            with_alpha(hp_col, 0.90),
        );
        ui.triangle(
            vec2(sx, sy - 6.5),
            vec2(sx + 5.5, sy),
            vec2(sx, sy + 6.5),
            with_alpha(hp_col, 0.90),
        );
        ui.triangle(
            vec2(sx, sy - 3.0),
            vec2(sx - 2.4, sy),
            vec2(sx, sy + 3.0),
            with_alpha(C_WHITE, 0.55),
        );
        ui.triangle(
            vec2(sx, sy - 3.0),
            vec2(sx + 2.4, sy),
            vec2(sx, sy + 3.0),
            with_alpha(C_WHITE, 0.55),
        );
        let bx = 28.0;
        let bw = 104.0;
        let segs = 10;
        let gap = 1.2;
        let seg_w = (bw - gap * (segs as f32 - 1.0)) / segs as f32;
        for i in 0..segs {
            let x = bx + i as f32 * (seg_w + gap);
            ui.rect(x, 245.0, seg_w, 9.0, Color::new(0.025, 0.021, 0.043, 0.96));
            let fill = (hp_frac * segs as f32 - i as f32).clamp(0.0, 1.0);
            if fill > 0.0 {
                ui.rect(x, 246.0, seg_w * fill, 7.0, hp_col);
                ui.rect(x, 246.0, seg_w * fill, 1.6, with_alpha(C_WHITE, 0.22));
            }
        }
        ui.text_shadow(
            &format!("{:.0}/{:.0}", self.player.hp, self.player.max_hp),
            138.0,
            253.0,
            6.6,
            C_WHITE,
            false,
        );
        let df = 1.0 - (self.player.dash_cd / self.player.dash_cd_max.max(0.1)).clamp(0.0, 1.0);
        ui.rect(
            28.0,
            259.0,
            72.0,
            3.2,
            Color::new(0.025, 0.021, 0.043, 0.96),
        );
        ui.rect(28.0, 259.0, 72.0 * df, 3.2, C_CYAN);
        ui.text(
            "Esquiva",
            104.0,
            262.0,
            5.2,
            Color::from_rgba(185, 220, 235, 255),
        );

        // Ataques equipados: seis huecos, con su icono y nivel.
        for i in 0..MAX_EQUIPPED {
            let x = 178.0 + i as f32 * 20.0;
            let y = 245.0;
            ui.rect(x, y, 18.0, 18.0, Color::new(0.035, 0.028, 0.055, 0.96));
            if let Some(e) = self.equipped_attacks.get(i) {
                let d = &self.attacks[e.def];
                ui.rect_lines(x, y, 18.0, 18.0, 0.8, with_alpha(d.color, 0.75));
                self.draw_attack_icon(
                    ui,
                    LevelChoice::NewAttack(e.def),
                    x + 9.0,
                    y + 8.0,
                    d.color,
                    0.95,
                );
                ui.text_right(
                    &format!("{}", e.level),
                    x + 17.0,
                    y + 17.0,
                    4.4,
                    with_alpha(C_WHITE, 0.9),
                    true,
                );
            } else {
                // Hueco vacío: panel grabado con muesca central.
                ui.rect_lines(x, y, 18.0, 18.0, 0.5, with_alpha(C_WHITE, 0.10));
                ui.rect(
                    x + 2.0,
                    y + 2.0,
                    14.0,
                    14.0,
                    Color::new(0.020, 0.016, 0.034, 0.90),
                );
                ui.rect_lines(
                    x + 2.0,
                    y + 2.0,
                    14.0,
                    14.0,
                    0.5,
                    Color::new(0.0, 0.0, 0.0, 0.65),
                );
                ui.rect(x + 6.5, y + 8.5, 5.0, 1.0, with_alpha(C_WHITE, 0.09));
            }
        }

        // Llaves.
        draw_ui_key(
            ui,
            vec2(314.0, 250.0),
            4.4,
            if self.keys > 0 {
                C_GOLD
            } else {
                Color::from_rgba(150, 146, 160, 255)
            },
            1.0,
        );
        ui.text_shadow(
            &format!("{}", self.keys),
            321.0,
            255.0,
            6.8,
            if self.keys > 0 {
                C_GOLD
            } else {
                Color::from_rgba(150, 146, 160, 255)
            },
            true,
        );
        draw_black_key_marker(ui, 342.0, 248.0, 0.80, 1.0);
        ui.text_shadow(
            &format!("{}", self.special_keys),
            349.0,
            255.0,
            6.8,
            if self.special_keys > 0 {
                C_PURPLE
            } else {
                Color::from_rgba(150, 146, 160, 255)
            },
            true,
        );

        // Oro y peso.
        let gold_text = format!("{}", self.player.gold);
        let gw = ui.measure(&gold_text, 7.6, true);
        draw_ui_coin(ui, vec2(458.0 - gw - 10.0, 250.0), 4.4, 1.0);
        ui.text_right_shadow(&gold_text, 468.0, 256.0, 7.6, C_GOLD, true);
        let weight = self.gold_weight();
        let info = if self.debt > 0 {
            format!("Deuda {}", self.debt)
        } else {
            format!("Obj. {}  ·  Peso {:.0}%", self.items.len(), weight * 100.0)
        };
        ui.text_right_shadow(
            &info,
            468.0,
            266.0,
            4.8,
            if self.debt > 0 {
                C_RED
            } else if weight > 0.55 {
                C_GOLD
            } else {
                Color::from_rgba(210, 204, 228, 255)
            },
            false,
        );

        self.draw_minimap(ui);

        if self.boss_intro_t <= 0.0 {
            if let Some(b) = self.enemies.iter().find(|e| e.boss && !e.dead) {
                ui.rect(
                    136.0,
                    38.0,
                    208.0,
                    10.0,
                    Color::new(0.020, 0.017, 0.038, 0.94),
                );
                ui.rect_lines(136.0, 38.0, 208.0, 10.0, 0.6, with_alpha(C_GOLD, 0.30));
                ui.rect(140.0, 40.0, 200.0, 6.0, Color::new(0.03, 0.025, 0.05, 0.92));
                ui.rect(
                    140.0,
                    40.0,
                    200.0 * (b.hp / b.max_hp).clamp(0.0, 1.0),
                    6.0,
                    C_RED,
                );
                ui.rect(
                    140.0,
                    40.0,
                    200.0 * (b.hp / b.max_hp).clamp(0.0, 1.0),
                    2.0,
                    with_alpha(C_WHITE, 0.20),
                );
                ui.text_center_shadow(&b.name, 39.2, 4.8, C_WHITE);
            }
        }
    }

    /// Brújula de la cripta: un disco de navegación con las salas exploradas
    /// como puntos de luz, no una rejilla de habitaciones.
    fn draw_minimap(&self, ui: &Ui) {
        if self.rooms.len() < 5 {
            return;
        }
        let cx = 430.0;
        let cy = 84.0;
        let rad = 30.0;
        ui.circle(cx, cy, rad, Color::new(0.012, 0.011, 0.026, 0.82));
        draw_circle_lines(
            ui.x(cx),
            ui.y(cy),
            ui.s(rad),
            ui.s(0.9),
            with_alpha(C_GOLD, 0.38),
        );
        draw_circle_lines(
            ui.x(cx),
            ui.y(cy),
            ui.s(rad - 4.0),
            ui.s(0.5),
            with_alpha(C_GOLD, 0.16),
        );
        for k in 0..4 {
            let a = k as f32 * std::f32::consts::FRAC_PI_2;
            let (sa, ca) = a.sin_cos();
            ui.line(
                cx + ca * (rad - 3.5),
                cy + sa * (rad - 3.5),
                cx + ca * (rad - 1.0),
                cy + sa * (rad - 1.0),
                0.8,
                with_alpha(C_GOLD, 0.45),
            );
        }
        ui.triangle(
            vec2(cx, cy - rad + 1.5),
            vec2(cx - 2.0, cy - rad + 4.5),
            vec2(cx + 2.0, cy - rad + 4.5),
            with_alpha(C_GOLD, 0.70),
        );

        let minx = self.rooms.iter().map(|r| r.gx).min().unwrap_or(0);
        let maxx = self.rooms.iter().map(|r| r.gx).max().unwrap_or(0);
        let miny = self.rooms.iter().map(|r| r.gy).min().unwrap_or(0);
        let maxy = self.rooms.iter().map(|r| r.gy).max().unwrap_or(0);
        let span = ((maxx - minx).max(maxy - miny)).max(1) as f32;
        let step = (rad - 8.0) / span;
        let ox = cx - (minx + maxx) as f32 * 0.5 * step;
        let oy = cy - (miny + maxy) as f32 * 0.5 * step;
        let node = |room: &Room| vec2(ox + room.gx as f32 * step, oy + room.gy as f32 * step);

        let adjacent_visited = |room: &Room| {
            [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                self.room_at(room.gx + dx, room.gy + dy)
                    .map(|j| self.rooms[j].visited)
                    .unwrap_or(false)
            })
        };

        // Hilos entre salas conocidas.
        for room in &self.rooms {
            if !room.visited && !adjacent_visited(room) {
                continue;
            }
            let a = node(room);
            for (dir, (dx, dy)) in [(1, (1, 0)), (2, (0, 1))] {
                if !room.doors[dir] {
                    continue;
                }
                if let Some(j) = self.room_at(room.gx + dx, room.gy + dy) {
                    let other = &self.rooms[j];
                    if !room.visited && !other.visited {
                        continue;
                    }
                    let b = node(other);
                    let known = room.visited && other.visited;
                    let prog = if known {
                        1.0
                    } else {
                        room.map_anim.clamp(0.0, 1.0)
                    };
                    let ex = a.x + (b.x - a.x) * prog;
                    let ey = a.y + (b.y - a.y) * prog;
                    ui.line(
                        a.x,
                        a.y,
                        ex,
                        ey,
                        if known { 0.8 } else { 0.5 },
                        if known {
                            with_alpha(C_GOLD, 0.30)
                        } else {
                            with_alpha(C_WHITE, 0.12)
                        },
                    );
                }
            }
        }

        for (i, room) in self.rooms.iter().enumerate() {
            let p = node(room);
            if !room.visited {
                if room.kind == RoomKind::Vault {
                    // La bóveda se anuncia como llave negra; abierta, como candado.
                    if room.vault_open {
                        draw_open_lock_marker(ui, p.x, p.y, 1.1, 0.90);
                    } else {
                        draw_black_key_marker(ui, p.x, p.y, 0.62, 0.95);
                    }
                } else if adjacent_visited(room) {
                    draw_circle_lines(
                        ui.x(p.x),
                        ui.y(p.y),
                        ui.s(1.5),
                        ui.s(0.5),
                        with_alpha(C_WHITE, 0.22),
                    );
                }
                continue;
            }
            let c = room.kind.color();
            ui.circle(p.x, p.y, 3.4, with_alpha(c, 0.16));
            ui.circle(p.x, p.y, 1.35, c);
            // Pequeños destellos de constelación alrededor de la sala.
            for k in 0..3 {
                let ang = self.time * 0.5 + k as f32 * 2.1 + p.x * 0.05;
                ui.circle(
                    p.x + ang.cos() * 3.4,
                    p.y + ang.sin() * 3.4,
                    0.35,
                    with_alpha(c, 0.50),
                );
            }
            if room.has_chest && !room.chest_opened {
                ui.circle(p.x + 1.8, p.y - 1.8, 0.8, C_GOLD);
            }
            if room.kind == RoomKind::Vault {
                if room.vault_open {
                    draw_open_lock_marker(ui, p.x, p.y, 1.0, 0.95);
                } else {
                    draw_black_key_marker(ui, p.x, p.y, 0.60, 0.95);
                }
            }
            if room.locked.iter().any(|&l| l) && room.kind != RoomKind::Vault {
                ui.circle(p.x - 1.9, p.y - 1.9, 0.8, C_GOLD);
            }
            if i == self.room_index {
                let pulse = 0.55 + (self.time * 4.0).sin() * 0.25;
                draw_circle_lines(
                    ui.x(p.x),
                    ui.y(p.y),
                    ui.s(3.4 + pulse * 1.2),
                    ui.s(0.9),
                    with_alpha(C_GOLD, 0.85),
                );
            }
        }
    }

    fn draw_menu(&self, s: &Sprites, ui: &Ui) {
        // Fondo de cripta: ladrillo, arco ritual, velas y niebla.
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.012, 0.010, 0.022, 0.90));
        draw_texture_ex(
            &s.glow_white,
            ui.x(80.0),
            ui.y(2.0),
            Color::new(0.36, 0.20, 0.60, 0.18),
            DrawTextureParams {
                dest_size: Some(vec2(ui.s(320.0), ui.s(250.0))),
                ..Default::default()
            },
        );
        let brick = Color::new(0.30, 0.26, 0.38, 0.055);
        for row in 0..16 {
            let y = row as f32 * 18.0;
            ui.line(0.0, y, VW, y, 0.5, brick);
            let off = if row % 2 == 0 { 0.0 } else { 18.0 };
            let mut x = off;
            while x < VW {
                ui.line(x, y, x, y + 18.0, 0.5, brick);
                x += 36.0;
            }
        }
        for k in 0..4 {
            draw_circle_lines(
                ui.x(240.0),
                ui.y(118.0),
                ui.s(118.0 - k as f32 * 12.0),
                ui.s(0.6),
                with_alpha(C_PURPLE, 0.05 + k as f32 * 0.012),
            );
        }
        for x in [6.0, 466.0] {
            ui.rect(x, 0.0, 8.0, VH, Color::new(0.028, 0.024, 0.042, 0.92));
            ui.rect_lines(x, 0.0, 8.0, VH, 0.6, with_alpha(C_GOLD, 0.16));
        }
        for (cx, ph) in [(12.0, 0.0), (468.0, 1.7)] {
            let flick = 0.82 + (self.time * 9.0 + ph).sin() * 0.18;
            draw_texture_ex(
                &s.glow_white,
                ui.x(cx - 22.0),
                ui.y(86.0),
                Color::new(1.0, 0.62, 0.22, 0.18 * flick),
                DrawTextureParams {
                    dest_size: Some(vec2(ui.s(44.0), ui.s(44.0))),
                    ..Default::default()
                },
            );
            ui.rect(
                cx - 1.5,
                112.0,
                3.0,
                13.0,
                Color::from_rgba(198, 188, 168, 255),
            );
            ui.circle(cx, 109.0, 1.9 * flick, C_GOLD);
        }
        for x in [66.0, 414.0] {
            for k in 0..8 {
                let y = k as f32 * 7.0;
                ui.rect_lines(
                    x - 1.5,
                    y,
                    3.0,
                    5.0,
                    0.4,
                    Color::new(0.30, 0.27, 0.36, 0.20),
                );
            }
        }
        draw_texture_ex(
            &s.glow_white,
            ui.x(-40.0),
            ui.y(196.0),
            Color::new(0.22, 0.16, 0.34, 0.22),
            DrawTextureParams {
                dest_size: Some(vec2(ui.s(560.0), ui.s(120.0))),
                ..Default::default()
            },
        );

        // Cabecera: emblema, logotipo, subtítulo, frase y progreso.
        ui.rect(
            14.0,
            10.0,
            34.0,
            34.0,
            Color::new(0.035, 0.028, 0.055, 0.96),
        );
        ui.rect_lines(14.0, 10.0, 34.0, 34.0, 1.0, with_alpha(C_GOLD, 0.55));
        let emblem = Color::from_rgba(230, 190, 90, 255);
        ui.line(26.0, 34.0, 26.0, 27.0, 1.2, emblem);
        ui.line(36.0, 34.0, 36.0, 27.0, 1.2, emblem);
        ui.line(26.0, 27.0, 31.0, 21.0, 1.2, emblem);
        ui.line(31.0, 21.0, 36.0, 27.0, 1.2, emblem);
        ui.circle(31.0, 28.0, 2.0, with_alpha(C_PURPLE, 0.35));
        ui.circle(31.0, 28.0, 1.2, emblem);
        ui.rect(30.4, 28.0, 1.2, 4.0, emblem);
        ui.text_shadow("CRIPTA", 53.0, 30.0, 20.0, C_GOLD, true);
        for k in 0..20 {
            let a = 1.0 - k as f32 / 20.0;
            ui.rect(
                54.0 + k as f32 * 4.2,
                36.0,
                3.4,
                1.2,
                with_alpha(C_GOLD, 0.55 * a),
            );
        }
        ui.text(
            "DESCIENDE · SAQUEA · SOBREVIVE",
            54.0,
            48.0,
            5.0,
            Color::from_rgba(220, 210, 235, 255),
        );

        let qi = ((self.time / 7.0) as usize) % MENU_QUOTES.len();
        let ti = ((self.time / 5.0) as usize + 2) % MENU_TIPS.len();
        ui.wrap_center(
            &format!("« {} »", MENU_QUOTES[qi]),
            245.0,
            12.0,
            168.0,
            5.6,
            7.0,
            Color::from_rgba(240, 232, 250, 255),
        );
        ui.wrap_center(
            MENU_TIPS[ti],
            245.0,
            29.0,
            168.0,
            4.8,
            5.8,
            Color::from_rgba(178, 170, 198, 255),
        );

        ui.text_right_shadow(&self.meta.name, 466.0, 10.0, 6.4, C_WHITE, true);
        let souls = format!("{} almas", self.meta.souls);
        let sw = ui.measure(&souls, 6.0, true);
        draw_ui_gem(ui, vec2(466.0 - sw - 8.0, 24.0), 3.2, C_PURPLE, 1.0);
        ui.text_right(&souls, 466.0, 21.0, 6.0, C_PURPLE, true);
        ui.text_right(
            &format!(
                "Mejor piso {}  ·  {} partidas  ·  {} bajas",
                self.meta.best_floor, self.meta.runs, self.meta.total_kills
            ),
            466.0,
            33.0,
            4.6,
            Color::from_rgba(178, 170, 198, 255),
            false,
        );
        ui.rect(14.0, 54.0, 452.0, 1.0, with_alpha(C_GOLD, 0.35));
        ui.rect(14.0, 55.0, 452.0, 1.0, Color::new(0.0, 0.0, 0.0, 0.45));

        // Columna de clases.
        ui.panel(10.0, 58.0, 166.0, 142.0, with_alpha(C_GOLD, 0.35));
        ui.text("CLASES", 18.0, 62.0, 5.2, with_alpha(C_GOLD, 0.90));
        let m = self.mouse_virtual();
        let tags = [
            "Tanque",
            "Tirador",
            "Arcana",
            "Sombra",
            "Nigromante",
            "Support",
            "Oro",
        ];
        let skin_tint = if self.meta.skin == 0 {
            C_WHITE
        } else {
            skin_color(self.meta.skin)
        };
        for (i, class) in Class::ALL.iter().enumerate() {
            let r = menu_class_rect(i);
            let selected = i == self.menu_index;
            let hover = r.contains(m);
            let accent = class.color();
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if selected {
                    Color::new(0.10, 0.085, 0.14, 0.98)
                } else if hover {
                    Color::new(0.075, 0.065, 0.105, 0.96)
                } else {
                    Color::new(0.045, 0.038, 0.065, 0.92)
                },
            );
            if selected {
                ui.rect(r.x, r.y, 2.5, r.h, accent);
            }
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if selected { 1.0 } else { 0.6 },
                if selected {
                    C_GOLD
                } else if hover {
                    with_alpha(accent, 0.80)
                } else {
                    Color::new(0.28, 0.25, 0.36, 0.85)
                },
            );
            let icon = vec2(r.x + 17.0, r.y + 9.5);
            draw_hero_preview(
                ui,
                *class,
                icon,
                17.0,
                skin_tint,
                self.meta.hat,
                self.meta.mask,
                self.time + i as f32,
            );
            ui.text_bold(
                class.name(),
                r.x + 31.0,
                r.y + 6.5,
                6.4,
                if selected { C_GOLD } else { C_WHITE },
                selected,
            );
            ui.text(
                tags[i],
                r.x + 31.0,
                r.y + 14.0,
                4.4,
                with_alpha(accent, 0.92),
            );
            if selected {
                ui.text_bold("›", r.x + r.w - 13.0, r.y + 12.0, 8.0, C_GOLD, true);
            }
        }

        // Ficha de la clase elegida.
        ui.panel(178.0, 58.0, 292.0, 138.0, with_alpha(C_GOLD, 0.35));
        let class = Class::ALL[self.menu_index.min(Class::ALL.len() - 1)];
        let accent = class.color();
        draw_texture_ex(
            &s.glow_white,
            ui.x(188.0),
            ui.y(74.0),
            Color::new(accent.r, accent.g, accent.b, 0.22),
            DrawTextureParams {
                dest_size: Some(vec2(ui.s(88.0), ui.s(88.0))),
                ..Default::default()
            },
        );
        let bob = (self.time * 2.6).sin() * 1.5;
        let icon = vec2(218.0, 110.0 + bob);
        draw_hero_preview(
            ui,
            class,
            icon,
            84.0,
            skin_tint,
            self.meta.hat,
            self.meta.mask,
            self.time,
        );
        // Pedestal de piedra bajo el héroe: losa biselada con runas doradas.
        let pxx = 218.0;
        let pyy = 124.0;
        let stone = Color::new(0.20, 0.18, 0.28, 1.0);
        let stone_hi = Color::new(0.34, 0.31, 0.45, 1.0);
        let stone_dk = Color::new(0.11, 0.09, 0.17, 1.0);
        ui.rect(pxx - 22.0, pyy, 44.0, 5.5, stone);
        ui.rect(pxx - 22.0, pyy, 44.0, 1.4, stone_hi);
        ui.rect(pxx - 22.0, pyy + 4.1, 44.0, 1.4, stone_dk);
        ui.rect_lines(
            pxx - 22.0,
            pyy,
            44.0,
            5.5,
            0.7,
            Color::new(0.05, 0.04, 0.09, 1.0),
        );
        for k in 0..3 {
            let rx = pxx - 14.0 + k as f32 * 11.0;
            ui.rect(
                rx,
                pyy + 1.8,
                5.0,
                1.2,
                with_alpha(C_GOLD, if k == 1 { 0.90 } else { 0.50 }),
            );
        }

        let titles = [
            "GUARDIÁN DE LA PUERTA",
            "CAZADOR DE LA NIEBLA",
            "TEJEDOR DE RUNAS",
            "FILO DE LA SOMBRA",
            "SEÑOR DE LOS HUESOS",
            "VOZ DEL SANTUARIO",
            "SEÑOR DEL MERCADO",
        ];
        let lore = [
            "Aguanta el golpe y devuélvelo con intereses. La primera puerta no perdona a los imprudentes.",
            "Mide la distancia, respira y dispara. Cada flecha bien colocada vale una sala.",
            "Convierte la habitación en una tormenta. El área limpia enjambres; los jefes exigen puntería.",
            "Entra por la sombra y sal antes de que te vean. El riesgo alto paga mejor.",
            "Los muertos caminan contigo. Deja que ellos reciban el primer golpe.",
            "La fe sostiene a los tuyos. Cura tus heridas y castiga a quien ose acercarse al santuario.",
            "Todo tiene un precio. Convierte cada baja en oro y deja que la suerte pague la diferencia.",
        ];
        ui.text_bold(class.name(), 252.0, 74.0, 11.0, C_WHITE, true);
        ui.text(titles[self.menu_index], 252.0, 86.0, 5.0, accent);
        ui.wrap_left(
            lore[self.menu_index],
            252.0,
            96.0,
            190.0,
            5.2,
            7.0,
            Color::from_rgba(212, 206, 228, 255),
        );

        let base = Player::new(class, &Meta::default());
        let stats = [
            ("VIDA", base.max_hp, 120.0),
            ("DAÑO", base.damage, 14.0),
            ("VEL.", base.speed, 110.0),
        ];
        for (si, (label, val, max)) in stats.iter().enumerate() {
            let y = 117.0 + si as f32 * 10.0;
            ui.text(label, 252.0, y, 4.6, Color::from_rgba(178, 170, 198, 255));
            ui.rect(
                286.0,
                y + 1.0,
                120.0,
                3.0,
                Color::new(0.035, 0.030, 0.055, 0.95),
            );
            let fill = 120.0 * (val / max).clamp(0.0, 1.0);
            ui.rect(286.0, y + 1.0, fill, 3.0, accent);
            ui.rect(286.0, y + 1.0, fill, 1.2, with_alpha(C_WHITE, 0.30));
            // Muescas de píxel para leer la proporción de un vistazo.
            for k in 1..5 {
                ui.rect(
                    286.0 + k as f32 * 24.0,
                    y + 0.4,
                    0.7,
                    4.2,
                    Color::new(0.012, 0.010, 0.024, 0.85),
                );
            }
            ui.rect_lines(
                286.0,
                y + 1.0,
                120.0,
                3.0,
                0.5,
                Color::new(0.012, 0.010, 0.024, 0.9),
            );
            ui.text_right(&format!("{:.0}", val), 448.0, y, 4.8, C_WHITE, false);
        }

        let pr = menu_play_rect();
        let hover = pr.contains(m);
        let pulse = 0.5 + 0.5 * (self.time * 3.0).sin();
        ui.rect(
            pr.x,
            pr.y,
            pr.w,
            pr.h,
            if hover {
                Color::new(0.16, 0.12, 0.06, 0.99)
            } else {
                Color::new(0.11, 0.085, 0.05, 0.98)
            },
        );
        ui.rect_lines(
            pr.x,
            pr.y,
            pr.w,
            pr.h,
            1.4,
            with_alpha(C_GOLD, 0.60 + 0.30 * pulse),
        );
        let label = "DESCENDER";
        let lw = ui.measure(label, 8.0, true);
        ui.text_bold(
            label,
            pr.x + (pr.w - lw) * 0.5,
            pr.y + 15.0,
            8.0,
            C_GOLD,
            true,
        );
        let hint = "Enter o clic para comenzar el descenso";
        let hw = ui.measure(hint, 4.6, false);
        ui.text(
            hint,
            350.0 - hw * 0.5,
            181.0,
            4.6,
            Color::from_rgba(170, 164, 190, 255),
        );

        // Accesos directos.
        let actions = [
            ("Tienda", C_GOLD),
            ("Taquilla", C_CYAN),
            ("Forja", C_GOLD),
            ("Biblioteca", C_PURPLE),
            ("Ajustes", C_GREEN),
            ("Estadísticas", C_WHITE),
            ("Audio", Color::from_rgba(232, 160, 92, 255)),
        ];
        for (i, (label, color)) in actions.iter().enumerate() {
            ui_button(ui, menu_action_rect(i), label, *color, m);
        }

        // Mejoras permanentes.
        let ups = [
            ("Vida máxima", "+10 de vida máxima", self.meta.hp_lv),
            ("Daño", "+5% de daño", self.meta.dmg_lv),
            ("Suerte", "+10% de suerte", self.meta.luck_lv),
        ];
        for (i, (name, desc, lv)) in ups.iter().enumerate() {
            let r = upgrade_rect(i);
            let cost = save::upgrade_cost(i, *lv);
            let req = save::upgrade_requirement(i, *lv);
            let req_ok = self.meta.best_floor >= req;
            let affordable = self.meta.souls >= cost && req_ok;
            ui.rect(r.x, r.y, r.w, r.h, Color::new(0.055, 0.046, 0.075, 0.96));
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                0.8,
                if affordable {
                    with_alpha(C_GOLD, 0.80)
                } else {
                    Color::new(0.30, 0.27, 0.37, 0.90)
                },
            );
            ui.text_bold(
                &format!("{} · Nv.{}", name, lv),
                r.x + 7.0,
                r.y + 10.0,
                5.6,
                C_WHITE,
                true,
            );
            ui.text_right(
                &format!("[{}]", i + 1),
                r.x + r.w - 7.0,
                r.y + 10.0,
                4.8,
                Color::from_rgba(200, 194, 218, 255),
                false,
            );
            let line = if req_ok {
                desc.to_string()
            } else {
                format!("Requiere piso {}", req)
            };
            ui.text(
                &line,
                r.x + 7.0,
                r.y + 20.0,
                4.6,
                if req_ok {
                    Color::from_rgba(200, 194, 218, 255)
                } else {
                    Color::from_rgba(235, 120, 125, 255)
                },
            );
            ui.text_right(
                &format!("{} almas", cost),
                r.x + r.w - 7.0,
                r.y + 20.0,
                5.2,
                if affordable {
                    C_GOLD
                } else {
                    Color::from_rgba(235, 120, 125, 255)
                },
                true,
            );
        }

        ui.text_center(
            "W/S elegir clase   ·   Enter descender   ·   1/2/3 comprar mejoras   ·   Esc pausa",
            262.0,
            4.8,
            Color::from_rgba(190, 184, 210, 255),
        );
    }

    fn draw_levelup(&self, ui: &Ui) {
        let a = (self.levelup_t / 0.18).clamp(0.0, 1.0);
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.02, 0.01, 0.04, 0.72 * a));
        // Modal compacto: estadísticas a la izquierda, opciones a la derecha.
        ui.panel(16.0, 20.0, 448.0, 230.0, with_alpha(C_GOLD, 0.55 * a));
        ui.text_center_shadow("Subiste de nivel", 36.0, 11.0, with_alpha(C_GOLD, a));
        ui.text_center(
            &format!(
                "Nivel {}   ·   Elige un ataque o una mejora   ·   Se lanzan solos",
                self.player.level
            ),
            48.0,
            5.8,
            with_alpha(Color::from_rgba(228, 222, 240, 255), a),
        );

        if let Some(def) = self.pending_replace {
            self.draw_replace_modal(ui, def, a);
            return;
        }

        let m = self.mouse_virtual();
        let hovered = (0..3).find(|&i| card_rect(i).contains(m));

        // Panel de estadísticas.
        ui.rect(
            24.0,
            56.0,
            160.0,
            178.0,
            with_alpha(Color::new(0.035, 0.028, 0.055, 0.98), a),
        );
        ui.rect_lines(
            24.0,
            56.0,
            160.0,
            178.0,
            0.7,
            with_alpha(C_PURPLE, 0.40 * a),
        );
        ui.text_bold("Personaje", 30.0, 68.0, 6.4, with_alpha(C_PURPLE, a), true);
        ui.line(30.0, 72.0, 178.0, 72.0, 0.5, with_alpha(C_PURPLE, 0.30 * a));
        let preview = hovered.map(|i| self.choice_preview(i)).unwrap_or_default();
        let stats = self.general_stat_rows();
        for (i, (label, value)) in stats.iter().enumerate() {
            let y = 83.0 + i as f32 * 10.2;
            ui.text(
                label,
                30.0,
                y,
                5.5,
                with_alpha(Color::from_rgba(190, 184, 210, 255), a),
            );
            if let Some((_, new_value)) = preview.iter().find(|(l, _)| l == label) {
                let cur = ui.measure(value, 5.5, false);
                let neww = ui.measure(new_value, 5.5, true);
                ui.text_right(
                    value,
                    178.0 - neww - 9.0,
                    y,
                    5.5,
                    with_alpha(Color::from_rgba(160, 154, 178, 255), a),
                    false,
                );
                ui.text_right(
                    "›",
                    178.0 - neww - 2.0,
                    y,
                    5.2,
                    with_alpha(C_GOLD, a),
                    false,
                );
                ui.text_right(new_value, 178.0, y, 5.5, with_alpha(C_GREEN, a), true);
                let _ = cur;
            } else {
                ui.text_right(value, 178.0, y, 5.5, with_alpha(C_WHITE, a), false);
            }
        }
        // Filas extra (daño/recarga/radio del ataque) que no son estadísticas
        // generales se muestran debajo, también en verde.
        let extra: Vec<&(String, String)> = preview
            .iter()
            .filter(|(l, _)| !stats.iter().any(|(s, _)| s == l))
            .collect();
        let mut ey = 190.0;
        for (label, value) in extra.iter().take(4) {
            ui.text(
                label,
                30.0,
                ey,
                5.2,
                with_alpha(Color::from_rgba(180, 174, 200, 255), a),
            );
            ui.text_right(value, 178.0, ey, 5.2, with_alpha(C_GREEN, a), true);
            ey += 9.5;
        }
        // Sin vista previa de carta, el hueco inferior muestra al personaje
        // de pie sobre la losa, vivo y animado.
        if extra.is_empty() {
            draw_hero_preview(
                ui,
                self.class,
                vec2(104.0, 210.0),
                46.0,
                if self.meta.skin == 0 {
                    C_WHITE
                } else {
                    skin_color(self.meta.skin)
                },
                self.meta.hat,
                self.meta.mask,
                self.time,
            );
        }

        // Opciones.
        for i in 0..3 {
            let choice = self.level_choices[i];
            let base = card_rect(i);
            let ca = ((self.levelup_t - i as f32 * 0.05) / 0.18).clamp(0.0, 1.0);
            if ca <= 0.0 {
                continue;
            }
            let r = Rect::new(base.x, base.y + (1.0 - ca) * 6.0, base.w, base.h);
            let hover = r.contains(m);
            let (accent, kind_label, name, desc, level_label) = self.choice_text(choice);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                with_alpha(
                    if hover {
                        Color::new(0.14, 0.115, 0.19, 0.98)
                    } else {
                        Color::new(0.075, 0.062, 0.105, 0.98)
                    },
                    ca,
                ),
            );
            ui.rect(r.x, r.y, 3.0, r.h, with_alpha(accent, ca));
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if hover { 1.4 } else { 0.8 },
                with_alpha(
                    if hover {
                        accent
                    } else {
                        with_alpha(accent, 0.55)
                    },
                    ca,
                ),
            );
            let num = format!("{}", i + 1);
            ui.rect(
                r.x + 7.0,
                r.y + 7.0,
                13.0,
                13.0,
                with_alpha(Color::new(0.03, 0.02, 0.05, 0.98), ca),
            );
            ui.rect_lines(
                r.x + 7.0,
                r.y + 7.0,
                13.0,
                13.0,
                0.7,
                with_alpha(accent, 0.85 * ca),
            );
            ui.text_bold(
                &num,
                r.x + 10.5,
                r.y + 17.0,
                6.2,
                with_alpha(C_WHITE, ca),
                true,
            );
            ui.text_bold(
                kind_label,
                r.x + 24.0,
                r.y + 15.0,
                5.2,
                with_alpha(accent, ca),
                true,
            );
            ui.text_right(
                &level_label,
                r.x + r.w - 8.0,
                r.y + 15.0,
                5.2,
                with_alpha(Color::from_rgba(210, 204, 228, 255), ca),
                false,
            );
            self.draw_attack_icon(ui, choice, r.x + 14.0, r.y + 34.0, accent, ca);
            ui.text_bold(
                &name,
                r.x + 28.0,
                r.y + 31.0,
                6.8,
                with_alpha(C_WHITE, ca),
                true,
            );
            ui.wrap_left(
                &desc,
                r.x + 28.0,
                r.y + 40.0,
                r.w - 36.0,
                5.2,
                6.2,
                with_alpha(Color::from_rgba(212, 206, 230, 255), ca),
            );
        }

        ui_button(
            ui,
            levelup_reroll_rect(),
            &format!("Cambiar opciones (R)  ·  {}", self.rerolls),
            C_CYAN,
            m,
        );
        ui_button(ui, levelup_stats_rect(), "Ver estadísticas", C_PURPLE, m);
    }

    fn draw_replace_modal(&self, ui: &Ui, def: usize, a: f32) {
        let d = &self.attacks[def];
        ui.text_center_shadow(
            "Tienes el máximo de ataques",
            66.0,
            8.4,
            with_alpha(C_GOLD, a),
        );
        ui.text_center(
            &format!("Elige qué ataque sustituir por «{}»", d.name),
            80.0,
            5.8,
            with_alpha(Color::from_rgba(228, 222, 240, 255), a),
        );
        let m = self.mouse_virtual();
        let hovered = (0..self.equipped_attacks.len()).find(|&i| replace_rect(i).contains(m));
        ui.rect(
            24.0,
            88.0,
            160.0,
            146.0,
            with_alpha(Color::new(0.035, 0.028, 0.055, 0.98), a),
        );
        ui.rect_lines(
            24.0,
            88.0,
            160.0,
            146.0,
            0.7,
            with_alpha(C_PURPLE, 0.40 * a),
        );
        ui.text_bold(
            "Nuevo ataque",
            30.0,
            100.0,
            6.2,
            with_alpha(C_GOLD, a),
            true,
        );
        self.draw_attack_icon(ui, LevelChoice::NewAttack(def), 34.0, 116.0, d.color, a);
        ui.text_bold(&d.name, 48.0, 114.0, 6.4, with_alpha(C_WHITE, a), true);
        ui.wrap_left(
            &d.desc,
            30.0,
            126.0,
            148.0,
            5.2,
            6.2,
            with_alpha(Color::from_rgba(212, 206, 230, 255), a),
        );
        ui.text(
            &format!("{}  ·  Nv. 1", d.kind.label()),
            30.0,
            160.0,
            5.2,
            with_alpha(d.color, a),
        );
        ui.line(
            30.0,
            170.0,
            178.0,
            170.0,
            0.5,
            with_alpha(C_PURPLE, 0.30 * a),
        );
        ui.text_bold("Sustituir", 30.0, 179.0, 5.8, with_alpha(C_RED, a), true);
        if let Some(i) = hovered {
            let ad = &self.attacks[self.equipped_attacks[i].def];
            ui.text_bold(&ad.name, 30.0, 189.0, 5.6, with_alpha(C_WHITE, a), true);
            ui.wrap_left(
                &ad.desc,
                30.0,
                197.0,
                148.0,
                4.8,
                5.6,
                with_alpha(Color::from_rgba(212, 206, 230, 255), a),
            );
        } else {
            ui.wrap_left(
                "Pasa el cursor por un ataque equipado para ver qué hace.",
                30.0,
                190.0,
                148.0,
                4.8,
                5.6,
                with_alpha(Color::from_rgba(170, 164, 190, 255), a),
            );
        }

        for (i, e) in self.equipped_attacks.iter().enumerate() {
            let r = replace_rect(i);
            let ad = &self.attacks[e.def];
            let hover = r.contains(m);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                with_alpha(
                    if hover {
                        Color::new(0.15, 0.11, 0.12, 0.98)
                    } else {
                        Color::new(0.075, 0.062, 0.105, 0.98)
                    },
                    a,
                ),
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if hover { 1.3 } else { 0.7 },
                with_alpha(
                    if hover {
                        C_RED
                    } else {
                        with_alpha(ad.color, 0.55)
                    },
                    a,
                ),
            );
            ui.text_bold(
                &format!("{}", i + 1),
                r.x + 7.0,
                r.y + 13.0,
                5.6,
                with_alpha(C_GOLD, a),
                true,
            );
            ui.text_bold(
                &ad.name,
                r.x + 18.0,
                r.y + 13.0,
                5.9,
                with_alpha(C_WHITE, a),
                true,
            );
            ui.text_right(
                &format!("Nv. {}", e.level),
                r.x + r.w - 7.0,
                r.y + 13.0,
                5.4,
                with_alpha(ad.color, a),
                false,
            );
            ui.text(
                &ad.kind.label(),
                r.x + 18.0,
                r.y + 21.0,
                4.8,
                with_alpha(Color::from_rgba(185, 180, 205, 255), a),
            );
        }
        ui.text_center(
            "1-6 o clic para sustituir   ·   Esc cancelar",
            245.0,
            5.2,
            with_alpha(Color::from_rgba(190, 184, 210, 255), a),
        );
    }

    fn general_stat_rows(&self) -> Vec<(String, String)> {
        let p = &self.player;
        vec![
            ("Daño".to_owned(), format!("{:.1}", p.damage)),
            ("Vida".to_owned(), format!("{:.0}", p.max_hp)),
            ("Velocidad".to_owned(), format!("{:.0}", p.speed)),
            ("Ataques/s".to_owned(), format!("{:.2}", p.attack_speed)),
            ("Alcance".to_owned(), format!("{:.0}", p.range)),
            ("Crítico".to_owned(), format!("{:.0}%", p.crit * 100.0)),
            ("Armadura".to_owned(), format!("{:.1}", p.armor)),
            ("Suerte".to_owned(), format!("{:.0}%", p.luck * 100.0)),
            ("Oro".to_owned(), format!("{:.0}%", p.gold_mult * 100.0)),
            ("Esquiva".to_owned(), format!("{:.1}s", p.dash_cd_max)),
        ]
    }

    fn choice_text(&self, choice: LevelChoice) -> (Color, &'static str, String, String, String) {
        match choice {
            LevelChoice::NewAttack(def) => {
                let d = &self.attacks[def];
                (
                    d.color,
                    "NUEVO ATAQUE",
                    d.name.clone(),
                    d.desc.clone(),
                    format!("{} · Nv. 1/{}", d.rarity.name(), d.max_level),
                )
            }
            LevelChoice::UpgradeAttack(slot) => {
                let e = &self.equipped_attacks[slot];
                let d = &self.attacks[e.def];
                (
                    d.color,
                    "SUBIR NIVEL",
                    d.name.clone(),
                    format!("Sube «{}» al nivel {}.", d.kind.label(), e.level + 1),
                    format!("{} · Nv. {} › {}", d.rarity.name(), e.level, e.level + 1),
                )
            }
            LevelChoice::Buff(def) => (
                def.rarity.color(),
                "MEJORA",
                def.name.to_owned(),
                def.desc.to_owned(),
                format!("{} · Máx. {}", def.rarity.name(), def.max_stacks),
            ),
        }
    }

    fn choice_preview(&self, i: usize) -> Vec<(String, String)> {
        match self.level_choices[i] {
            LevelChoice::Buff(def) => self.buff_preview(def.id),
            LevelChoice::NewAttack(def) => {
                let d = &self.attacks[def];
                vec![
                    (
                        "Daño del ataque".to_owned(),
                        format!("{:.1}", d.base_damage * 100.0),
                    ),
                    ("Recarga".to_owned(), format!("{:.2}s", d.base_cd)),
                    ("Radio".to_owned(), format!("{:.0}", d.base_radius)),
                    ("Elemento".to_owned(), format!("{:?}", d.element)),
                ]
            }
            LevelChoice::UpgradeAttack(slot) => {
                let e = &self.equipped_attacks[slot];
                let d = &self.attacks[e.def];
                let n = e.level + 1;
                vec![
                    ("Nivel".to_owned(), format!("{} › {}", e.level, n)),
                    (
                        "Daño".to_owned(),
                        format!(
                            "{:.0} › {:.0}",
                            attacks::level_damage(d.base_damage, e.level) * 100.0,
                            attacks::level_damage(d.base_damage, n) * 100.0
                        ),
                    ),
                    (
                        "Recarga".to_owned(),
                        format!(
                            "{:.2}s › {:.2}s",
                            attacks::level_cd(d.base_cd, e.level),
                            attacks::level_cd(d.base_cd, n)
                        ),
                    ),
                    (
                        "Radio".to_owned(),
                        format!(
                            "{:.0} › {:.0}",
                            attacks::level_radius(d.base_radius, e.level),
                            attacks::level_radius(d.base_radius, n)
                        ),
                    ),
                ]
            }
        }
    }

    fn draw_attack_icon(
        &self,
        ui: &Ui,
        choice: LevelChoice,
        x: f32,
        y: f32,
        color: Color,
        alpha: f32,
    ) {
        let c = with_alpha(color, alpha);
        let soft = with_alpha(color, 0.35 * alpha);
        let kind = match choice {
            LevelChoice::NewAttack(d) => self.attacks[d].kind,
            LevelChoice::UpgradeAttack(s) => self.attacks[self.equipped_attacks[s].def].kind,
            LevelChoice::Buff(def) => {
                draw_buff_icon(ui, def.id, x, y, color, alpha);
                return;
            }
        };
        match kind {
            AttackKind::Bolt => {
                ui.line(x - 4.0, y + 4.0, x + 4.0, y - 4.0, 1.2, c);
                ui.circle(x + 4.0, y - 4.0, 1.8, c);
            }
            AttackKind::Orbit => {
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(5.0), ui.s(0.8), c);
                ui.circle(x + 4.6, y - 2.0, 1.6, c);
            }
            AttackKind::Aura => {
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(5.5), ui.s(0.7), soft);
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(3.2), ui.s(0.8), c);
            }
            AttackKind::Nova => {
                for k in 0..4 {
                    let a = k as f32 * std::f32::consts::FRAC_PI_2;
                    ui.line(x, y, x + a.cos() * 6.0, y + a.sin() * 6.0, 1.0, c);
                }
                ui.circle(x, y, 1.8, c);
            }
            AttackKind::Chain => {
                ui.circle(x - 2.6, y, 2.4, c);
                ui.circle(x + 2.6, y, 2.4, soft);
                ui.line(x - 0.5, y, x + 0.5, y, 0.8, c);
            }
            AttackKind::Mine => {
                ui.circle(x, y, 3.0, c);
                for k in 0..5 {
                    let a = k as f32 * std::f32::consts::TAU / 5.0;
                    ui.line(x, y, x + a.cos() * 6.0, y + a.sin() * 6.0, 0.8, c);
                }
            }
            AttackKind::Trail => {
                ui.circle(x - 3.0, y + 2.5, 2.0, soft);
                ui.circle(x, y, 2.2, c);
                ui.circle(x + 3.0, y - 2.5, 1.6, soft);
            }
            AttackKind::Vortex => {
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(5.5), ui.s(0.8), soft);
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(3.4), ui.s(0.8), c);
                ui.circle(x, y, 1.3, c);
            }
            AttackKind::Bolas => {
                ui.circle(x - 3.5, y + 2.0, 2.0, c);
                ui.circle(x + 3.5, y + 2.0, 2.0, c);
                ui.circle(x, y - 3.0, 2.0, soft);
            }
            AttackKind::Mend => {
                ui.line(x - 5.0, y, x + 5.0, y, 1.5, c);
                ui.line(x, y - 5.0, x, y + 5.0, 1.5, c);
            }
            AttackKind::Beam => {
                ui.line(x - 6.0, y, x + 6.0, y, 1.8, c);
                ui.line(x - 6.0, y - 1.5, x + 6.0, y - 1.5, 0.6, soft);
            }
            AttackKind::Meteor => {
                ui.line(x + 1.0, y - 6.0, x + 4.0, y - 2.0, 1.4, soft);
                ui.circle(x, y + 1.0, 3.4, c);
            }
            AttackKind::Swarm => {
                ui.circle(x - 3.5, y - 2.0, 1.5, c);
                ui.circle(x + 3.0, y - 3.0, 1.5, soft);
                ui.circle(x, y + 2.5, 1.7, c);
            }
            AttackKind::Shockwave => {
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(3.0), ui.s(0.8), c);
                draw_circle_lines(ui.x(x), ui.y(y), ui.s(6.0), ui.s(0.7), soft);
            }
            AttackKind::Spikes => {
                ui.triangle(
                    vec2(x - 5.0, y + 4.0),
                    vec2(x - 2.0, y - 5.0),
                    vec2(x + 1.0, y + 4.0),
                    c,
                );
                ui.triangle(
                    vec2(x + 1.0, y + 4.0),
                    vec2(x + 4.0, y - 3.0),
                    vec2(x + 6.0, y + 4.0),
                    soft,
                );
            }
            AttackKind::Drain => {
                ui.circle(x, y + 1.5, 3.4, c);
                ui.triangle(
                    vec2(x, y - 6.0),
                    vec2(x - 3.2, y + 1.0),
                    vec2(x + 3.2, y + 1.0),
                    c,
                );
            }
        }
    }

    fn draw_pause(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.02, 0.01, 0.04, 0.80));
        ui.panel(112.0, 28.0, 256.0, 214.0, with_alpha(C_CYAN, 0.70));
        ui.text_center_shadow("Pausa", 54.0, 13.0, C_WHITE);
        ui.text_center(
            &format!("Piso {}   ·   Nivel {}", self.floor, self.player.level),
            72.0,
            6.8,
            C_GOLD,
        );
        ui.text_center(
            &format!(
                "Oro {}   ·   Bajas {}   ·   Objetos {}",
                self.player.gold,
                self.kills,
                self.items.len()
            ),
            86.0,
            6.4,
            Color::from_rgba(235, 230, 245, 255),
        );
        let m = self.mouse_virtual();
        ui_button(ui, pause_continue_rect(), "Continuar (Esc)", C_CYAN, m);
        ui_button(ui, pause_stats_rect(), "Estadísticas", C_PURPLE, m);
        ui_button(ui, pause_inventory_rect(), "Inventario (I)", C_GOLD, m);
        ui_button(
            ui,
            pause_settings_rect(),
            "Ajustes y teclas (O)",
            C_GREEN,
            m,
        );
        ui_button(ui, pause_audio_rect(), "Audio", C_CYAN, m);
        ui_button(ui, pause_abandon_rect(), "Abandonar partida (Q)", C_RED, m);
        ui.text_center(
            "Esc continuar   ·   M música   ·   F11 pantalla completa",
            236.0,
            5.4,
            Color::from_rgba(190, 184, 210, 255),
        );
    }

    fn draw_tickets(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.018, 0.014, 0.032, 0.96));
        ui.panel(22.0, 18.0, 436.0, 236.0, with_alpha(C_PURPLE, 0.55));
        ui.text_center_shadow("Contratos de aliado", 34.0, 11.0, C_PURPLE);
        ui.text_center(
            "Un aliado espectral te acompaña durante toda la partida",
            48.0,
            5.8,
            Color::from_rgba(212, 206, 230, 255),
        );
        let m = self.mouse_virtual();
        for i in 0..save::TICKETS.len() {
            let r = tickets_row_rect(i);
            let unlocked = i == 0 || (self.meta.owned_tickets & (1 << i)) != 0;
            let equipped = self.meta.ticket as usize == i;
            let hover = r.contains(m);
            let accent = match i {
                0 => Color::from_rgba(150, 145, 170, 255),
                1 => Color::from_rgba(150, 220, 220, 255),
                2 => Color::from_rgba(245, 214, 120, 255),
                3 => Color::from_rgba(230, 190, 90, 255),
                _ => Color::from_rgba(205, 90, 110, 255),
            };
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if equipped {
                    Color::new(0.12, 0.10, 0.17, 0.98)
                } else if hover {
                    Color::new(0.085, 0.072, 0.12, 0.98)
                } else {
                    Color::new(0.05, 0.043, 0.072, 0.96)
                },
            );
            ui.rect(r.x, r.y, 3.0, r.h, accent);
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                1.0,
                if equipped {
                    C_GOLD
                } else if hover {
                    with_alpha(accent, 0.85)
                } else {
                    Color::new(0.26, 0.24, 0.34, 0.95)
                },
            );
            draw_ticket_icon(ui, i, r.x + 18.0, r.y + r.h * 0.5, accent, 1.0);
            ui.text_bold(
                save::TICKETS[i].0,
                r.x + 34.0,
                r.y + 11.0,
                6.4,
                if unlocked {
                    C_WHITE
                } else {
                    Color::from_rgba(130, 125, 148, 255)
                },
                true,
            );
            ui.text(
                save::TICKETS[i].1,
                r.x + 34.0,
                r.y + 21.0,
                5.0,
                if unlocked {
                    Color::from_rgba(200, 194, 218, 255)
                } else {
                    Color::from_rgba(120, 115, 138, 255)
                },
            );
            let state = if equipped {
                "EQUIPADO"
            } else if unlocked {
                "EQUIPAR"
            } else {
                "BLOQUEADO"
            };
            ui.text_right(
                state,
                r.x + r.w - 8.0,
                r.y + 15.0,
                5.2,
                if equipped {
                    C_GOLD
                } else if unlocked {
                    accent
                } else {
                    Color::from_rgba(120, 115, 138, 255)
                },
                true,
            );
        }
        ui_button(ui, tickets_back_rect(), "Volver (Esc)", C_CYAN, m);
        ui.text_center(
            "El contrato elegido se aplica al empezar una partida nueva",
            236.0,
            5.2,
            Color::from_rgba(160, 155, 178, 255),
        );
    }

    fn draw_forge(&self, _s: &Sprites, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.016, 0.013, 0.028, 0.97));
        ui.panel(18.0, 14.0, 444.0, 242.0, with_alpha(C_GOLD, 0.55));
        let class = Class::ALL[self.forge_class.min(Class::ALL.len() - 1)];
        let accent = class.color();
        let m = self.mouse_virtual();
        ui_button(ui, forge_back_rect(), "Volver (Esc)", accent, m);
        ui.text_center_shadow("Forja de la cripta", 30.0, 10.0, C_GOLD);
        let souls = format!("{} almas", self.meta.souls);
        let sw = ui.measure(&souls, 6.0, true);
        draw_ui_gem(ui, vec2(456.0 - sw - 8.0, 24.0), 3.0, C_PURPLE, 1.0);
        ui.text_right(&souls, 456.0, 21.0, 6.0, C_PURPLE, true);

        draw_hero_preview(
            ui,
            class,
            vec2(158.0, 57.0),
            32.0,
            if self.meta.skin == 0 {
                C_WHITE
            } else {
                skin_color(self.meta.skin)
            },
            self.meta.hat,
            self.meta.mask,
            self.time,
        );
        ui_button(ui, forge_class_minus(), "‹", accent, m);
        ui_button(ui, forge_class_plus(), "›", accent, m);
        ui.text_center(class.name(), 59.0, 8.0, C_GOLD);
        ui.text_center(
            "Tres armas por personaje · el arma equipada se aplica al empezar",
            68.0,
            4.8,
            Color::from_rgba(178, 170, 198, 255),
        );

        for i in 0..3 {
            let r = forge_card_rect(i);
            let def = &weapons::class_weapons(class)[i];
            let owned = self.meta.owns_weapon(class, i + 1);
            let equipped = self.meta.weapon_for(class) == i + 1;
            let req_ok = self.meta.best_floor >= def.req_floor;
            let afford = self.meta.souls >= def.cost;
            let hover = r.contains(m);
            let active = owned || (req_ok && afford);
            let dim = Color::from_rgba(126, 120, 146, 255);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if equipped {
                    Color::new(0.115, 0.095, 0.075, 0.99)
                } else if hover {
                    Color::new(0.075, 0.065, 0.105, 0.98)
                } else {
                    Color::new(0.042, 0.036, 0.06, 0.96)
                },
            );
            ui.rect(r.x, r.y, r.w, 2.0, if owned { accent } else { dim });
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if equipped { 1.4 } else { 0.8 },
                if equipped {
                    C_GOLD
                } else if hover {
                    with_alpha(accent, 0.85)
                } else {
                    Color::new(0.27, 0.24, 0.35, 0.95)
                },
            );
            draw_weapon_icon(
                ui,
                class,
                i + 1,
                r.x + 70.0,
                r.y + 22.0,
                if owned { accent } else { dim },
                1.0,
            );
            ui_text_center_at(
                ui,
                def.name,
                r.x + r.w * 0.5,
                r.y + 42.0,
                5.8,
                if owned { C_WHITE } else { dim },
            );
            ui.wrap_center(
                def.desc,
                r.x + 70.0,
                r.y + 50.0,
                124.0,
                4.5,
                5.5,
                if owned {
                    Color::from_rgba(205, 198, 222, 255)
                } else {
                    Color::from_rgba(145, 138, 164, 255)
                },
            );
            for (ei, effect) in def.effects.iter().enumerate() {
                ui.text(
                    &format!("· {}", weapons::effect_label(*effect)),
                    r.x + 11.0,
                    r.y + 72.0 + ei as f32 * 7.0,
                    4.4,
                    if owned { accent } else { dim },
                );
            }
            let status = if owned {
                "Forjada".to_owned()
            } else {
                format!("{} almas", def.cost)
            };
            ui_text_center_at(
                ui,
                &status,
                r.x + r.w * 0.5,
                r.y + 104.0,
                5.2,
                if owned {
                    C_GREEN
                } else if afford {
                    C_GOLD
                } else {
                    C_RED
                },
            );
            if !owned && !req_ok {
                ui_text_center_at(
                    ui,
                    &format!("Requiere piso {}", def.req_floor),
                    r.x + r.w * 0.5,
                    r.y + 112.0,
                    4.4,
                    C_RED,
                );
            }
            let label = if equipped {
                "EQUIPADA"
            } else if owned {
                "EQUIPAR"
            } else if !req_ok {
                "BLOQUEADA"
            } else if !afford {
                "SIN ALMAS"
            } else {
                "FORJAR"
            };
            ui_button(
                ui,
                forge_button_rect(i),
                label,
                if active { accent } else { dim },
                m,
            );
            ui.text_right(
                &format!("{}", i + 1),
                r.x + r.w - 8.0,
                r.y + 11.0,
                4.4,
                Color::from_rgba(150, 144, 168, 255),
                false,
            );
        }
        ui.text_center(
            "Forja con almas · A/D cambia de personaje · 1-3 forja o equipa",
            244.0,
            4.8,
            Color::from_rgba(160, 155, 178, 255),
        );
    }

    fn draw_inventory(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.02, 0.015, 0.035, 0.95));
        ui.panel(20.0, 14.0, 440.0, 242.0, with_alpha(C_GOLD, 0.55));
        ui.text_center_shadow("Inventario", 30.0, 11.0, C_GOLD);
        ui.text_center(
            &format!(
                "{}   ·   Piso {}   ·   Nv. {}   ·   Oro {}",
                self.player.class.name(),
                self.floor,
                self.player.level,
                self.player.gold
            ),
            43.0,
            5.8,
            Color::from_rgba(225, 219, 238, 255),
        );
        let weapon_name = weapons::get(self.player.class, self.meta.weapon_for(self.player.class))
            .map(|w| w.name)
            .unwrap_or("Sin arma forjada");
        ui.text_right(
            &format!("Arma: {}", weapon_name),
            442.0,
            52.0,
            5.0,
            if weapons::get(self.player.class, self.meta.weapon_for(self.player.class)).is_some() {
                C_GOLD
            } else {
                Color::from_rgba(140, 134, 160, 255)
            },
            false,
        );

        // Ataques equipados.
        ui.text_bold("ATAQUES EQUIPADOS", 32.0, 59.0, 5.6, C_PURPLE, true);
        ui.line(32.0, 63.0, 448.0, 63.0, 0.5, with_alpha(C_PURPLE, 0.35));
        for i in 0..MAX_EQUIPPED {
            let y = 68.0 + i as f32 * 17.0;
            ui.rect(32.0, y, 416.0, 15.0, Color::new(0.05, 0.043, 0.075, 0.95));
            match self.equipped_attacks.get(i) {
                Some(e) => {
                    let d = &self.attacks[e.def];
                    ui.rect(32.0, y, 2.5, 15.0, d.color);
                    self.draw_attack_icon(
                        ui,
                        LevelChoice::NewAttack(e.def),
                        42.0,
                        y + 7.5,
                        d.color,
                        0.95,
                    );
                    ui.text_bold(&d.name, 52.0, y + 7.0, 5.8, C_WHITE, true);
                    ui.text_right(
                        &format!("Nv. {}/{}", e.level, d.max_level),
                        442.0,
                        y + 7.0,
                        5.4,
                        d.color,
                        false,
                    );
                    ui.text(
                        &format!("{}  ·  {:.1}s", d.kind.label(), d.base_cd),
                        170.0,
                        y + 7.0,
                        5.0,
                        Color::from_rgba(180, 174, 200, 255),
                    );
                }
                None => {
                    ui.rect_lines(32.0, y, 416.0, 15.0, 0.5, with_alpha(C_WHITE, 0.10));
                    ui.text(
                        "Hueco libre",
                        40.0,
                        y + 7.0,
                        5.2,
                        Color::from_rgba(130, 125, 148, 255),
                    );
                }
            }
        }

        // Objetos.
        ui.text_bold("OBJETOS", 32.0, 181.0, 5.6, C_GOLD, true);
        ui.line(32.0, 185.0, 448.0, 185.0, 0.5, with_alpha(C_GOLD, 0.30));
        if self.items.is_empty() {
            ui.text(
                "Sin objetos. Abre cofres, compra al mercader o derrota guardianes.",
                38.0,
                197.0,
                5.4,
                Color::from_rgba(160, 155, 178, 255),
            );
        } else {
            for (i, item) in self.items.iter().take(3).enumerate() {
                let y = 195.0 + i as f32 * 14.0;
                let c = item.rarity.color();
                draw_item_icon(ui, item.kind, 40.0, y - 2.0, c, 1.0);
                ui.text_bold(item.name, 48.0, y, 5.6, c, true);
                ui.text(
                    &format!("{}  ·  {:.1}", item.kind.label(), item.value),
                    178.0,
                    y,
                    5.2,
                    Color::from_rgba(195, 190, 212, 255),
                );
            }
            if self.items.len() > 3 {
                ui.text_right(
                    &format!("+{} objetos más", self.items.len() - 3),
                    442.0,
                    195.0,
                    5.2,
                    Color::from_rgba(160, 155, 178, 255),
                    false,
                );
            }
        }

        let m = self.mouse_virtual();
        ui_button(ui, inventory_back_rect(), "Volver (Esc)", C_CYAN, m);
    }

    fn draw_dead(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.10, 0.01, 0.03, 0.88));
        ui.text_center_shadow("Has caído", 58.0, 15.0, C_RED);
        ui.text_center(
            "La cripta reclama otro nombre.",
            76.0,
            6.8,
            Color::from_rgba(240, 212, 218, 255),
        );
        ui.panel(128.0, 92.0, 224.0, 96.0, with_alpha(C_RED, 0.70));
        ui.text_center(
            &format!("Piso alcanzado: {}", self.floor),
            110.0,
            6.8,
            C_WHITE,
        );
        ui.text_center(&format!("Bajas: {}", self.kills), 126.0, 6.8, C_WHITE);
        ui.text_center(
            &format!("Oro recogido: {}", self.run_gold),
            142.0,
            6.8,
            C_GOLD,
        );
        ui.text_center(
            &format!("Almas ganadas: +{}", self.souls_earned),
            158.0,
            6.8,
            C_PURPLE,
        );
        ui.text_center(
            &format!("Almas totales: {}", self.meta.souls),
            174.0,
            6.4,
            C_PURPLE,
        );
        let m = self.mouse_virtual();
        ui_button(ui, dead_retry_rect(), "Reintentar (R)", C_GOLD, m);
        ui_button(ui, dead_menu_rect(), "Menú (Esc)", C_CYAN, m);
    }

    fn draw_victory(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.06, 0.04, 0.01, 0.84));
        ui.text_center_shadow("Victoria", 54.0, 15.0, C_GOLD);
        ui.text_center(
            "El guardián del piso 10 ha caído.",
            74.0,
            6.8,
            Color::from_rgba(248, 230, 192, 255),
        );
        ui.panel(128.0, 90.0, 224.0, 96.0, with_alpha(C_GOLD, 0.75));
        ui.text_center(&format!("Piso: {}", self.floor), 108.0, 6.8, C_WHITE);
        ui.text_center(&format!("Bajas: {}", self.kills), 124.0, 6.8, C_WHITE);
        ui.text_center(
            &format!("Objetos: {}", self.items.len()),
            140.0,
            6.8,
            C_GOLD,
        );
        ui.text_center(
            &format!("Almas ganadas: +{}", self.souls_earned),
            156.0,
            6.8,
            C_PURPLE,
        );
        ui.text_center(
            &format!("Almas totales: {}", self.meta.souls),
            172.0,
            6.4,
            C_PURPLE,
        );
        let m = self.mouse_virtual();
        ui_button(
            ui,
            victory_endless_rect(),
            "Descender al infinito (C)",
            C_GOLD,
            m,
        );
        ui_button(ui, victory_menu_rect(), "Menú (Esc)", C_CYAN, m);
    }

    /// Cartel de presentación del guardián: foco oscuro, sprite grande y
    /// nombre. Congela la acción y se puede saltar con espacio.
    fn draw_boss_intro(&self, s: &Sprites, ui: &Ui) {
        let total = 3.2;
        let t = self.boss_intro_t.clamp(0.0, total);
        let elapsed = total - t;
        // Entrada rápida, salida suave.
        let fade = (elapsed / 0.30).clamp(0.0, 1.0) * (t / 0.45).clamp(0.0, 1.0);
        let c = self.boss_intro_color;
        let pulse = 0.75 + (elapsed * 5.2).sin().abs() * 0.25;

        ui.rect(
            0.0,
            0.0,
            VW,
            VH,
            Color::new(0.01, 0.006, 0.015, 0.86 * fade),
        );
        // Foco de cámara: halo y suelo iluminado.
        let glow = 190.0 + (elapsed * 2.0).sin() * 10.0;
        ui.texture_centered(
            &s.glow_white,
            vec2(240.0, 128.0),
            glow / s.glow_white.width(),
            with_alpha(c, 0.22 * fade * pulse),
        );
        // Bandas cinematográficas.
        let bar = 30.0 * fade;
        ui.rect(0.0, 0.0, VW, bar, Color::new(0.0, 0.0, 0.0, 0.94 * fade));
        ui.rect(
            0.0,
            VH - bar,
            VW,
            bar,
            Color::new(0.0, 0.0, 0.0, 0.94 * fade),
        );

        // El guardián entra desde abajo y crece, dibujado con el mismo
        // renderizador procedural que en combate (nada de texturas pixeladas).
        // Se usa una cámara con el mismo encuadre que la interfaz para que las
        // formas se dibujen en el espacio virtual de 480x270. La silueta oscura
        // con rim de color mantiene legible el nombre superpuesto.
        let rise = (1.0 - (elapsed / 0.45).clamp(0.0, 1.0)) * 26.0;
        let grow = 2.05 + (elapsed / 0.45).clamp(0.0, 1.0) * 0.35;
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VW, VH));
        // La cámara directa a pantalla invierte el eje Y; lo devolvemos al
        // convenio del juego (Y crece hacia abajo).
        cam.zoom.y = -cam.zoom.y;
        let vh = screen_height();
        cam.viewport = Some((
            ui.ox.round() as i32,
            (vh - (ui.oy + VH * ui.scale)).round() as i32,
            (VW * ui.scale).round().max(1.0) as i32,
            (VH * ui.scale).round().max(1.0) as i32,
        ));
        set_camera(&cam);
        let intro_pos = vec2(240.0, 140.0 + rise);
        // Halo de color alrededor de la silueta.
        figures::draw_enemy(
            intro_pos,
            EnemyKind::Boss,
            20.0,
            grow * 1.10,
            with_alpha(c, 0.62),
            self.boss_intro_skin.min(11),
            false,
            false,
            true,
            0.0,
            0.0,
            0.0,
            fade,
            self.time,
        );
        figures::draw_enemy(
            intro_pos,
            EnemyKind::Boss,
            20.0,
            grow,
            Color::new(
                0.045 + c.r * 0.16,
                0.040 + c.g * 0.16,
                0.055 + c.b * 0.16,
                1.0,
            ),
            self.boss_intro_skin.min(11),
            false,
            false,
            true,
            0.0,
            0.0,
            0.0,
            fade,
            self.time,
        );
        set_default_camera();

        // Nombre y descripción. La etiqueta va sobre la cabeza, fuera del
        // retrato, y el nombre queda bajo el puño para no tapar la silueta.
        let name_alpha = ((elapsed - 0.15) / 0.35).clamp(0.0, 1.0) * fade;
        ui.text_center("GUARDIÁN", 38.0, 6.0, with_alpha(c, 0.75 * name_alpha));
        ui.line(
            150.0,
            43.0,
            330.0,
            43.0,
            0.6,
            with_alpha(c, 0.45 * name_alpha),
        );
        ui.text_center_shadow(
            &self.boss_intro_name,
            196.0,
            11.5,
            with_alpha(C_WHITE, name_alpha),
        );
        ui.text_center(
            &self.boss_intro_desc,
            214.0,
            6.0,
            with_alpha(c, 0.92 * name_alpha),
        );
        ui.text_center(
            "ESPACIO  ·  SALTAR",
            VH - 22.0,
            4.8,
            with_alpha(C_WHITE, 0.40 * fade),
        );
    }

    /// Fundido corto entre pisos: piso destino, línea ritual y aviso de salto.
    fn draw_floor_transition(&self, ui: &Ui) {
        let total = 0.85;
        let (alpha, next_floor) = if self.floor_transition_t > 0.0 {
            let p = 1.0 - (self.floor_transition_t / total).clamp(0.0, 1.0);
            (p.powf(1.5), self.floor + 1)
        } else {
            let a = (self.floor_arrival_t / 0.30).clamp(0.0, 1.0);
            (a * 0.95, self.floor)
        };
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.008, 0.006, 0.014, alpha));
        if alpha < 0.15 {
            return;
        }
        let c = C_CYAN;
        let mid = VH * 0.5;
        // Línea que se cierra sobre el número.
        let spread = 60.0 + (1.0 - alpha) * 120.0;
        ui.line(
            240.0 - spread,
            mid - 18.0,
            240.0 - 26.0,
            mid - 18.0,
            0.7,
            with_alpha(c, 0.55 * alpha),
        );
        ui.line(
            240.0 + 26.0,
            mid - 18.0,
            240.0 + spread,
            mid - 18.0,
            0.7,
            with_alpha(c, 0.55 * alpha),
        );
        ui.text_center("DESCENSO", mid - 44.0, 5.4, with_alpha(c, 0.75 * alpha));
        ui.text_center_shadow(
            &format!("PISO {}", next_floor),
            mid - 10.0,
            14.0,
            with_alpha(C_WHITE, alpha),
        );
        let (line, line_color) = if next_floor % 10 == 0 {
            ("Algo enorme respira al otro lado...", C_GOLD)
        } else if next_floor > 1 && (next_floor - 1) % 10 == 0 {
            ("El guardián ha caído. La cripta se abre.", C_GOLD)
        } else {
            (
                "La cripta se hunde un nivel más",
                Color::from_rgba(196, 190, 214, 255),
            )
        };
        ui.text_center(line, mid + 12.0, 5.6, with_alpha(line_color, 0.85 * alpha));
        if self.floor_transition_t > 0.0 {
            ui.text_center(
                "ESPACIO  ·  SALTAR",
                mid + 34.0,
                4.8,
                with_alpha(C_WHITE, 0.38 * alpha),
            );
        }
    }

    fn draw_map(&self, ui: &Ui) {
        let a = (self.map_t * 4.0).clamp(0.0, 1.0);
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.01, 0.008, 0.02, 0.86 * a));
        ui.panel(112.0, 26.0, 256.0, 222.0, with_alpha(C_GOLD, 0.55));

        // Esquinas grabadas.
        for (x, y, sx, sy) in [
            (118.0, 32.0, 1.0, 1.0),
            (362.0, 32.0, -1.0, 1.0),
            (118.0, 242.0, 1.0, -1.0),
            (362.0, 242.0, -1.0, -1.0),
        ] {
            ui.line(x, y, x + sx * 10.0, y, 0.8, with_alpha(C_GOLD, 0.35));
            ui.line(x, y, x, y + sy * 10.0, 0.8, with_alpha(C_GOLD, 0.35));
        }

        ui.text_center_shadow("Carta de la cripta", 46.0, 10.0, C_GOLD);
        ui.text_center(
            &format!(
                "Piso {}   ·   Llaves: {}   ·   Negras: {}",
                self.floor, self.keys, self.special_keys
            ),
            60.0,
            6.2,
            if self.keys > 0 || self.special_keys > 0 {
                C_GOLD
            } else {
                Color::from_rgba(190, 184, 210, 255)
            },
        );

        // Rosa de los vientos y polvo de estrellas.
        let mcx = 240.0;
        let mcy = 140.0;
        draw_circle_lines(
            ui.x(mcx),
            ui.y(mcy),
            ui.s(62.0),
            ui.s(0.7),
            with_alpha(C_GOLD, 0.10),
        );
        draw_circle_lines(
            ui.x(mcx),
            ui.y(mcy),
            ui.s(47.0),
            ui.s(0.5),
            with_alpha(C_GOLD, 0.07),
        );
        for k in 0..8 {
            let ang = k as f32 * std::f32::consts::FRAC_PI_4;
            let (sa, ca) = ang.sin_cos();
            ui.line(
                mcx + ca * 44.0,
                mcy + sa * 44.0,
                mcx + ca * 62.0,
                mcy + sa * 62.0,
                0.5,
                with_alpha(C_GOLD, 0.10),
            );
        }
        for (sx, sy) in [
            (178.0, 88.0),
            (302.0, 94.0),
            (186.0, 202.0),
            (308.0, 198.0),
            (238.0, 78.0),
        ] {
            ui.circle(sx, sy, 0.6, with_alpha(C_WHITE, 0.35));
        }

        let minx = self.rooms.iter().map(|r| r.gx).min().unwrap_or(0);
        let maxx = self.rooms.iter().map(|r| r.gx).max().unwrap_or(0);
        let miny = self.rooms.iter().map(|r| r.gy).min().unwrap_or(0);
        let maxy = self.rooms.iter().map(|r| r.gy).max().unwrap_or(0);
        let cell = 32.0;
        let x0 = mcx - (maxx - minx) as f32 * cell * 0.5;
        let y0 = mcy - (maxy - miny) as f32 * cell * 0.5 + 2.0;
        // Cada nodo se desplaza un poco de la rejilla: el mapa parece una
        // constelación orgánica, no un plano cuadrado.
        let node = |room: &Room| {
            let h = (room.gx.wrapping_mul(73856093) ^ room.gy.wrapping_mul(19349663)) as u32;
            let jx = ((h & 0xFF) as f32 / 255.0 - 0.5) * 10.0;
            let jy = (((h >> 8) & 0xFF) as f32 / 255.0 - 0.5) * 10.0;
            vec2(
                x0 + (room.gx - minx) as f32 * cell + jx,
                y0 + (room.gy - miny) as f32 * cell + jy,
            )
        };
        let adjacent_visited = |room: &Room| {
            [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                self.room_at(room.gx + dx, room.gy + dy)
                    .map(|j| self.rooms[j].visited)
                    .unwrap_or(false)
            })
        };

        // Hilos de constelación.
        for room in &self.rooms {
            if !room.visited && !adjacent_visited(room) {
                continue;
            }
            let a = node(room);
            for (dir, (dx, dy)) in [(1, (1, 0)), (2, (0, 1))] {
                if !room.doors[dir] {
                    continue;
                }
                if let Some(j) = self.room_at(room.gx + dx, room.gy + dy) {
                    let other = &self.rooms[j];
                    if !room.visited && !other.visited {
                        continue;
                    }
                    let b = node(other);
                    let known = room.visited && other.visited;
                    let prog = if known {
                        1.0
                    } else {
                        room.map_anim.clamp(0.0, 1.0)
                    };
                    let ex = a.x + (b.x - a.x) * prog;
                    let ey = a.y + (b.y - a.y) * prog;
                    // Hilo ligeramente curvado, como una constelación dibujada
                    // a mano.
                    let dx = ex - a.x;
                    let dy = ey - a.y;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    let bow = ((room.gx * 31 + room.gy * 57 + dir as i32 * 13) as f32 * 0.7).sin()
                        * 3.2
                        * prog;
                    let cxp = (a.x + ex) * 0.5 - dy / len * bow;
                    let cyp = (a.y + ey) * 0.5 + dx / len * bow;
                    let lw = if known { 1.0 } else { 0.6 };
                    let lc = if known {
                        with_alpha(C_GOLD, 0.38)
                    } else {
                        with_alpha(C_WHITE, 0.12)
                    };
                    let mut prev = a;
                    for step in 1..=6 {
                        let t = step as f32 / 6.0;
                        let mt = 1.0 - t;
                        let px = mt * mt * a.x + 2.0 * mt * t * cxp + t * t * ex;
                        let py = mt * mt * a.y + 2.0 * mt * t * cyp + t * t * ey;
                        ui.line(prev.x, prev.y, px, py, lw, lc);
                        prev = vec2(px, py);
                    }
                }
            }
        }

        for (i, room) in self.rooms.iter().enumerate() {
            let p = node(room);
            let current = i == self.room_index;
            if !room.visited {
                if room.kind == RoomKind::Vault {
                    if room.vault_open {
                        draw_open_lock_marker(ui, p.x, p.y, 1.3, 0.95);
                    } else {
                        draw_black_key_marker(ui, p.x, p.y, 1.25, 1.0);
                    }
                    ui.text(
                        "Bóveda",
                        p.x + 8.0,
                        p.y + 2.0,
                        4.0,
                        with_alpha(C_PURPLE, 0.75),
                    );
                } else if adjacent_visited(room) {
                    draw_circle_lines(
                        ui.x(p.x),
                        ui.y(p.y),
                        ui.s(3.0),
                        ui.s(0.6),
                        with_alpha(C_WHITE, 0.20),
                    );
                    ui.circle(p.x, p.y, 0.7, with_alpha(C_WHITE, 0.28));
                }
                continue;
            }
            let c = room.kind.color();
            let anim = room.map_anim;
            let fill = Color::new(0.030, 0.025, 0.050, 0.97);
            // Nodo de constelación: un punto de luz, no un cuadro de mapa.
            ui.circle(p.x, p.y, 5.2, with_alpha(c, 0.07 + anim * 0.10));
            ui.circle(p.x, p.y, 2.2, c);
            ui.circle(p.x, p.y, 0.9, with_alpha(C_WHITE, 0.75));
            draw_circle_lines(
                ui.x(p.x),
                ui.y(p.y),
                ui.s(5.4),
                ui.s(if current { 1.2 } else { 0.6 }),
                if current {
                    with_alpha(C_GOLD, 0.95)
                } else {
                    with_alpha(c, 0.55)
                },
            );
            // Cuatro destellos giran alrededor del nodo, como una estrella.
            for k in 0..4 {
                let ang = self.map_t * 0.7 + k as f32 * std::f32::consts::FRAC_PI_2;
                let (sa, ca) = ang.sin_cos();
                ui.circle(p.x + ca * 7.2, p.y + sa * 7.2, 0.55, with_alpha(c, 0.55));
            }
            match room.kind {
                RoomKind::Start => {
                    ui.line(p.x - 2.0, p.y - 2.2, p.x - 2.0, p.y + 2.2, 0.6, c);
                    ui.line(p.x + 2.0, p.y - 2.2, p.x + 2.0, p.y + 2.2, 0.6, c);
                    ui.line(p.x - 2.0, p.y - 2.2, p.x + 2.0, p.y - 2.2, 0.6, c);
                    ui.circle(p.x, p.y + 1.0, 0.6, c);
                }
                RoomKind::Combat => {}
                RoomKind::Shop => {
                    draw_circle_lines(ui.x(p.x), ui.y(p.y), ui.s(2.4), ui.s(0.7), c);
                    ui.circle(p.x, p.y, 0.8, c);
                }
                RoomKind::Rest => {
                    ui.circle(p.x, p.y, 2.4, c);
                    ui.circle(p.x + 1.1, p.y - 0.8, 2.2, fill);
                }
                RoomKind::Treasure => {
                    draw_chest_marker(ui, p.x - 3.4, p.y - 3.0, 6.8, anim);
                }
                RoomKind::Vault => {
                    if room.vault_open {
                        draw_open_lock_marker(ui, p.x, p.y, 1.3, 0.95);
                    } else {
                        draw_black_key_marker(ui, p.x, p.y, 1.25, 1.0);
                    }
                }
                RoomKind::Boss => {
                    ui.circle(p.x, p.y - 0.4, 2.8, c);
                    ui.circle(p.x - 1.0, p.y - 0.7, 0.8, fill);
                    ui.circle(p.x + 1.0, p.y - 0.7, 0.8, fill);
                    ui.line(p.x - 1.6, p.y + 1.8, p.x + 1.6, p.y + 1.8, 0.6, c);
                }
            }
            if room.has_chest && !room.chest_opened && room.kind != RoomKind::Treasure {
                draw_chest_marker(ui, p.x + 3.0, p.y - 9.0, 6.0, anim);
            }
            if room.locked.iter().any(|&l| l) && room.kind != RoomKind::Vault {
                ui.circle(p.x + 6.6, p.y + 5.4, 1.2, C_GOLD);
                ui.rect(p.x + 6.1, p.y + 6.2, 1.0, 2.2, C_GOLD);
            }
            if current {
                let pulse = 0.5 + (self.map_t * 4.0).sin() * 0.25;
                draw_circle_lines(
                    ui.x(p.x),
                    ui.y(p.y),
                    ui.s(9.5),
                    ui.s(0.8),
                    with_alpha(C_GOLD, pulse),
                );
                ui.triangle(
                    vec2(p.x, p.y - 11.0),
                    vec2(p.x - 2.0, p.y - 14.5),
                    vec2(p.x + 2.0, p.y - 14.5),
                    with_alpha(C_GOLD, 0.90),
                );
            }
            if anim < 1.0 {
                draw_circle_lines(
                    ui.x(p.x),
                    ui.y(p.y),
                    ui.s(5.4 + (1.0 - anim) * 18.0),
                    ui.s(0.9),
                    with_alpha(c, (1.0 - anim) * 0.55),
                );
            }
        }

        // Leyenda de la carta.
        let kinds = [
            RoomKind::Start,
            RoomKind::Combat,
            RoomKind::Shop,
            RoomKind::Rest,
            RoomKind::Treasure,
            RoomKind::Vault,
            RoomKind::Boss,
        ];
        for (k, kind) in kinds.iter().enumerate() {
            let x = 122.0 + k as f32 * 34.0;
            let label_x = if *kind == RoomKind::Vault {
                draw_black_key_marker(ui, x, 219.0, 0.75, 0.95);
                x + 5.2
            } else {
                ui.circle(x, 220.0, 1.7, kind.color());
                x + 3.6
            };
            ui.text(
                kind.short(),
                label_x,
                222.0,
                3.6,
                Color::from_rgba(205, 198, 222, 255),
            );
        }
        ui.text_center(
            "La niebla ritual solo dibuja los hilos de las salas conocidas",
            208.0,
            4.6,
            with_alpha(Color::from_rgba(190, 184, 210, 255), 0.75),
        );

        ui_button(
            ui,
            map_close_rect(),
            "Cerrar (Tab)",
            C_CYAN,
            self.mouse_virtual(),
        );
    }

    /// El mercader solo muestra su lista cuando lo tienes cerca; así la
    /// interfaz no tapa al jugador cuando camina por la tienda.
    fn near_merchant(&self) -> bool {
        let (l, r, t, b) = self.arena();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        // Hay que estar delante del mostrador: así las tarjetas quedan debajo
        // del héroe y nunca lo tapan.
        self.player.pos.distance(vec2(cx, cy + 2.0)) < 56.0 && self.player.pos.y < cy + 6.0
    }

    fn draw_shop_room(&self, s: &Sprites, ui: &Ui) {
        let m = self.mouse_virtual();
        if !self.near_merchant() {
            let (l, r, t, b) = self.arena();
            let cx = (l + r) * 0.5;
            let cy = (t + b) * 0.5;
            if self.player.pos.distance(vec2(cx, cy + 2.0)) > 82.0 {
                ui.text_center_shadow(
                    "Acércate al mercader para ver su mercancía",
                    196.0,
                    6.0,
                    with_alpha(C_GOLD, 0.85),
                );
            }
            return;
        }
        // Lista de la tienda: cuatro ofertas, cada una con su icono y precio.
        for i in 0..4 {
            let offer = match self.shop_offers.get(i) {
                Some(o) => o,
                None => continue,
            };
            let r = shop_offer_rect(i);
            let hover = r.contains(m);
            let sold = offer.sold;
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if sold {
                    Color::new(0.035, 0.03, 0.045, 0.94)
                } else if hover {
                    Color::new(0.13, 0.10, 0.08, 0.96)
                } else {
                    Color::new(0.075, 0.06, 0.06, 0.94)
                },
            );
            ui.rect(
                r.x,
                r.y,
                r.w,
                2.0,
                if sold {
                    Color::new(0.3, 0.28, 0.34, 1.0)
                } else {
                    C_GOLD
                },
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if hover && !sold { 1.6 } else { 0.8 },
                if sold {
                    Color::new(0.28, 0.26, 0.32, 0.9)
                } else if hover {
                    C_GOLD
                } else {
                    with_alpha(C_GOLD, 0.55)
                },
            );
            let name_col = if sold {
                Color::from_rgba(130, 126, 140, 255)
            } else {
                C_WHITE
            };
            ui.text_bold(&offer.name, r.x + 6.0, r.y + 12.0, 5.2, name_col, true);
            draw_shop_icon(
                ui,
                s,
                offer.kind,
                vec2(r.x + r.w - 15.0, r.y + 9.0),
                0.72,
                sold,
            );
            ui.wrap_left(
                &offer.desc,
                r.x + 6.0,
                r.y + 21.0,
                r.w - 12.0,
                4.4,
                5.2,
                if sold {
                    Color::from_rgba(120, 116, 130, 255)
                } else {
                    Color::from_rgba(220, 214, 234, 255)
                },
            );
            let price = if sold {
                "Vendido".to_owned()
            } else {
                format!("{} de oro", offer.cost)
            };
            ui.text_bold(
                &price,
                r.x + 6.0,
                r.y + r.h - 5.0,
                5.4,
                if sold {
                    Color::from_rgba(120, 116, 130, 255)
                } else if self.player.gold >= offer.cost {
                    C_GOLD
                } else {
                    Color::from_rgba(228, 110, 115, 255)
                },
                true,
            );
            ui.text_right(
                &format!("[{}]", i + 1),
                r.x + r.w - 6.0,
                r.y + r.h - 5.0,
                5.2,
                Color::from_rgba(190, 184, 205, 255),
                false,
            );
        }
    }

    fn draw_shop(&self, s: &Sprites, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.02, 0.015, 0.035, 0.97));
        draw_texture_ex(
            &s.glow,
            ui.x(240.0 - 120.0),
            ui.y(-58.0),
            Color::new(0.96, 0.72, 0.30, 0.16),
            DrawTextureParams {
                dest_size: Some(vec2(ui.s(240.0), ui.s(240.0))),
                ..Default::default()
            },
        );
        ui.text_center_shadow("Tienda de almas", 22.0, 12.0, C_GOLD);
        ui.text_center(
            "Compra aspectos para tu personaje",
            34.0,
            5.8,
            Color::from_rgba(225, 219, 238, 255),
        );

        let m = self.mouse_virtual();
        ui_button(ui, shop_back_rect(), "Volver", C_CYAN, m);
        let souls = format!("{} almas", self.meta.souls);
        let sw = ui.measure(&souls, 7.0, true);
        draw_ui_gem(ui, vec2(468.0 - sw - 11.0, 18.0), 4.0, C_PURPLE, 1.0);
        ui.text_right_shadow(&souls, 468.0, 23.0, 7.0, C_PURPLE, true);

        let tabs = ["Colores", "Gorras", "Máscaras"];
        for (i, label) in tabs.iter().enumerate() {
            let r = shop_tab_rect(i);
            let selected = self.shop_tab == i;
            let hover = r.contains(m);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if selected {
                    Color::new(0.16, 0.13, 0.08, 0.98)
                } else if hover {
                    Color::new(0.10, 0.085, 0.12, 0.96)
                } else {
                    Color::new(0.06, 0.05, 0.08, 0.94)
                },
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                1.0,
                if selected {
                    C_GOLD
                } else {
                    Color::new(0.32, 0.29, 0.40, 0.95)
                },
            );
            let tw = ui.measure(label, 6.4, true);
            ui.text_bold(
                label,
                r.x + (r.w - tw) * 0.5,
                r.y + 12.0,
                6.4,
                if selected { C_GOLD } else { C_WHITE },
                true,
            );
        }

        for i in 0..8 {
            let r = shop_item_rect(i);
            let (name, cost, owned, equipped) = match self.shop_tab {
                0 => (
                    save::SKINS[i].0,
                    save::SKINS[i].1,
                    self.meta.owns_skin(i),
                    self.meta.skin as usize == i,
                ),
                1 => (
                    save::HATS[i].0,
                    save::HATS[i].1,
                    self.meta.owns_hat(i),
                    self.meta.hat as usize == i,
                ),
                _ => (
                    save::MASKS[i].0,
                    save::MASKS[i].1,
                    self.meta.owns_mask(i),
                    self.meta.mask as usize == i,
                ),
            };
            let req = save::cosmetic_requirement(self.shop_tab, i);
            let req_ok = req
                .map(|(floor, kills)| {
                    self.meta.best_floor >= floor && self.meta.total_kills >= kills
                })
                .unwrap_or(true);
            let hover = r.contains(m);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if equipped {
                    Color::new(0.075, 0.115, 0.085, 0.97)
                } else if hover {
                    Color::new(0.13, 0.11, 0.16, 0.98)
                } else {
                    Color::new(0.065, 0.055, 0.09, 0.96)
                },
            );
            ui.rect(
                r.x,
                r.y,
                r.w,
                2.0,
                if equipped {
                    C_GREEN
                } else if owned {
                    C_CYAN
                } else {
                    with_alpha(C_GOLD, 0.75)
                },
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                if hover { 1.6 } else { 0.8 },
                if equipped {
                    C_GREEN
                } else if hover {
                    C_GOLD
                } else {
                    Color::new(0.34, 0.31, 0.42, 0.95)
                },
            );
            let nw = ui.measure(name, 6.2, true);
            ui.text_bold(name, r.x + (r.w - nw) * 0.5, r.y + 12.0, 6.2, C_WHITE, true);

            let class = Class::ALL[self.menu_index.min(Class::ALL.len() - 1)];
            let tint = if self.shop_tab == 0 {
                if i == 0 {
                    C_WHITE
                } else {
                    skin_color(i as u8)
                }
            } else if self.meta.skin == 0 {
                C_WHITE
            } else {
                skin_color(self.meta.skin)
            };
            let icon = vec2(r.x + r.w * 0.5, r.y + 38.0);
            let hat = if self.shop_tab == 1 {
                i as u8
            } else {
                self.meta.hat
            };
            let mask = if self.shop_tab == 2 {
                i as u8
            } else {
                self.meta.mask
            };
            draw_hero_preview(ui, class, icon, 40.0, tint, hat, mask, self.time + i as f32);

            if !owned {
                let lx = r.x + r.w - 13.0;
                let ly = r.y + 10.0;
                let lock = with_alpha(C_GOLD, 0.85);
                ui.line(lx + 1.5, ly, lx + 1.5, ly - 2.0, 0.8, lock);
                ui.line(lx + 4.5, ly, lx + 4.5, ly - 2.0, 0.8, lock);
                ui.line(lx + 1.5, ly - 2.0, lx + 4.5, ly - 2.0, 0.8, lock);
                ui.rect(lx, ly, 6.0, 5.0, lock);
                ui.rect(
                    lx + 2.2,
                    ly + 1.6,
                    1.6,
                    2.0,
                    Color::new(0.06, 0.05, 0.08, 0.9),
                );
            }
            if !owned && !req_ok {
                if let Some((floor, kills)) = req {
                    let req_line = format!("Piso {} · {} bajas", floor, kills);
                    let rw = ui.measure(&req_line, 4.2, false);
                    ui.text(
                        &req_line,
                        r.x + (r.w - rw) * 0.5,
                        r.y + 60.5,
                        4.2,
                        Color::from_rgba(235, 120, 125, 255),
                    );
                }
            }
            let bottom = if equipped {
                "Equipado".to_owned()
            } else if owned {
                "Equipar".to_owned()
            } else if !req_ok {
                "Bloqueado".to_owned()
            } else {
                format!("{} almas", cost)
            };
            let bw = ui.measure(&bottom, 6.0, true);
            ui.text_bold(
                &bottom,
                r.x + (r.w - bw) * 0.5,
                r.y + r.h - 6.0,
                6.0,
                if equipped {
                    C_GREEN
                } else if owned {
                    C_CYAN
                } else if !req_ok {
                    Color::from_rgba(235, 120, 125, 255)
                } else if self.meta.souls >= cost {
                    C_GOLD
                } else {
                    Color::from_rgba(228, 110, 115, 255)
                },
                true,
            );
        }

        ui.text_center(
            "Gana almas jugando. Cada aspecto exige un desafío: piso y bajas.",
            246.0,
            5.8,
            Color::from_rgba(190, 184, 210, 255),
        );
    }

    fn draw_locker(&self, _s: &Sprites, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.02, 0.015, 0.035, 0.92));
        ui.text_center_shadow("Taquilla", 24.0, 12.0, C_CYAN);
        let m = self.mouse_virtual();
        ui_button(ui, locker_back_rect(), "Volver", C_CYAN, m);

        let class = Class::ALL[self.locker_class.min(Class::ALL.len() - 1)];
        ui.panel(24.0, 44.0, 184.0, 196.0, with_alpha(C_CYAN, 0.55));
        ui.text_bold(
            "Nombre",
            38.0,
            62.0,
            5.8,
            Color::from_rgba(190, 184, 210, 255),
            false,
        );
        let nr = locker_name_rect();
        ui.rect(
            nr.x,
            nr.y,
            nr.w,
            nr.h,
            if self.editing_name {
                Color::new(0.08, 0.11, 0.13, 0.98)
            } else {
                Color::new(0.055, 0.05, 0.08, 0.96)
            },
        );
        ui.rect_lines(
            nr.x,
            nr.y,
            nr.w,
            nr.h,
            1.0,
            if self.editing_name {
                C_CYAN
            } else {
                Color::new(0.34, 0.31, 0.42, 0.95)
            },
        );
        let shown = if self.editing_name {
            &self.name_buffer
        } else {
            &self.meta.name
        };
        let nw = ui.measure(shown, 7.0, true);
        ui.text_bold(
            shown,
            nr.x + (nr.w - nw) * 0.5,
            nr.y + 13.0,
            7.0,
            C_WHITE,
            true,
        );
        if self.editing_name && (self.name_t * 2.4).sin() > 0.0 {
            ui.rect(nr.x + (nr.w + nw) * 0.5 + 2.0, nr.y + 5.0, 1.0, 9.0, C_CYAN);
        }

        ui_button(ui, locker_class_minus(), "‹", C_CYAN, m);
        ui_button(ui, locker_class_plus(), "›", C_CYAN, m);
        let cn = class.name();
        let cw = ui.measure(cn, 8.0, true);
        ui.text_bold(
            cn,
            locker_class_minus().x + 20.0 + (108.0 - cw) * 0.5,
            109.0,
            8.0,
            C_GOLD,
            true,
        );
        ui.wrap_center(
            class.desc(),
            116.0,
            122.0,
            164.0,
            5.4,
            7.0,
            Color::from_rgba(210, 204, 226, 255),
        );

        let icon = vec2(116.0, 168.0);
        ui.circle(icon.x, icon.y, 38.0, with_alpha(class.color(), 0.10));
        let tint = if self.meta.skin == 0 {
            C_WHITE
        } else {
            skin_color(self.meta.skin)
        };
        draw_hero_preview(
            ui,
            class,
            icon,
            80.0,
            tint,
            self.meta.hat,
            self.meta.mask,
            self.time,
        );
        let hint = "N para cambiar el nombre";
        let hw = ui.measure(hint, 5.4, false);
        ui.text(
            hint,
            116.0 - hw * 0.5,
            232.0,
            5.4,
            Color::from_rgba(180, 175, 200, 255),
        );

        ui.panel(220.0, 44.0, 236.0, 196.0, with_alpha(C_CYAN, 0.35));
        let labels = ["Color", "Gorra", "Máscara"];
        for row in 0..3 {
            let y = 58.0 + row as f32 * 44.0;
            ui.rect(232.0, y, 212.0, 38.0, Color::new(0.055, 0.05, 0.075, 0.94));
            ui.rect_lines(
                232.0,
                y,
                212.0,
                38.0,
                0.8,
                Color::new(0.30, 0.28, 0.38, 0.95),
            );
            ui.text_bold(
                labels[row],
                238.0,
                y + 11.0,
                5.8,
                Color::from_rgba(190, 184, 210, 255),
                false,
            );
            let (name, owned_count) = match row {
                0 => (
                    save::SKINS[self.meta.skin as usize].0,
                    (0..save::SKINS.len())
                        .filter(|&i| self.meta.owns_skin(i))
                        .count(),
                ),
                1 => (
                    save::HATS[self.meta.hat as usize].0,
                    (0..save::HATS.len())
                        .filter(|&i| self.meta.owns_hat(i))
                        .count(),
                ),
                _ => (
                    save::MASKS[self.meta.mask as usize].0,
                    (0..save::MASKS.len())
                        .filter(|&i| self.meta.owns_mask(i))
                        .count(),
                ),
            };
            let vw = ui.measure(name, 6.8, true);
            ui.text_bold(name, 338.0 - vw * 0.5, y + 19.0, 6.8, C_WHITE, true);
            let ow = format!("{} comprados", owned_count);
            let oww = ui.measure(&ow, 5.2, false);
            ui.text(
                &ow,
                338.0 - oww * 0.5,
                y + 31.0,
                5.2,
                Color::from_rgba(160, 155, 178, 255),
            );
            ui_button(ui, locker_minus(row), "‹", C_CYAN, m);
            ui_button(ui, locker_plus(row), "›", C_CYAN, m);
        }
        ui_button(
            ui,
            locker_tickets_rect(),
            "Contratos de aliado (T)",
            C_PURPLE,
            m,
        );
        ui_button(ui, locker_forge_rect(), "Forja (F)", C_GOLD, m);
        ui_button(ui, locker_shop_rect(), "Ir a la tienda", C_GOLD, m);
    }

    fn draw_library(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.018, 0.014, 0.032, 0.96));
        ui.text_center_shadow("Biblioteca de la cripta", 22.0, 11.0, C_CYAN);
        ui.text_center(
            "Ataques, guardianes y criaturas del descenso",
            33.0,
            5.6,
            Color::from_rgba(205, 199, 222, 255),
        );
        let m = self.mouse_virtual();
        ui_button(ui, library_back_rect(), "Volver", C_CYAN, m);
        let tabs = ["Ataques", "Jefes", "Enemigos", "Objetos"];
        for (i, label) in tabs.iter().enumerate() {
            let r = library_tab_rect(i);
            let active = i == self.library_tab;
            let hover = r.contains(m);
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if active {
                    Color::new(0.10, 0.13, 0.16, 0.98)
                } else if hover {
                    Color::new(0.08, 0.07, 0.12, 0.98)
                } else {
                    Color::new(0.05, 0.045, 0.08, 0.95)
                },
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                1.0,
                if active {
                    C_GOLD
                } else if hover {
                    with_alpha(C_CYAN, 0.8)
                } else {
                    Color::new(0.28, 0.26, 0.36, 0.95)
                },
            );
            let tw = ui.measure(label, 6.2, active);
            ui.text_bold(
                label,
                r.x + (r.w - tw) * 0.5,
                r.y + 11.0,
                6.2,
                if active { C_GOLD } else { C_WHITE },
                active,
            );
        }

        let count = library_len(self.library_tab);
        let pages = library_pages(self.library_tab);
        let page = self.library_page.min(pages - 1);
        let start = page * 6;
        for slot in 0..6 {
            let idx = start + slot;
            if idx >= count {
                break;
            }
            let r = library_card_rect(slot);
            let (name, desc, accent) = match self.library_tab {
                0 => {
                    let (n, d) = LIBRARY_ATTACKS[idx];
                    (n, d, C_PURPLE)
                }
                1 => {
                    let (n, d) = LIBRARY_BOSSES[idx];
                    (n, d, C_RED)
                }
                2 => {
                    let (n, d) = LIBRARY_ENEMIES[idx];
                    (n, d, C_CYAN)
                }
                _ => {
                    let def = &buffs::POOL[idx];
                    (def.name, def.desc, def.rarity.color())
                }
            };
            ui.rect(r.x, r.y, r.w, r.h, Color::new(0.06, 0.052, 0.09, 0.97));
            ui.rect(r.x, r.y, 3.0, r.h, with_alpha(accent, 0.95));
            ui.rect_lines(r.x, r.y, r.w, r.h, 0.8, with_alpha(accent, 0.35));
            // Adorno inferior: tres runas pequeñas y una línea tenue llenan el
            // aire de la ficha sin competir con el texto.
            ui.rect(
                r.x + 9.0,
                r.y + r.h - 7.0,
                r.w - 18.0,
                0.5,
                with_alpha(accent, 0.18),
            );
            for k in 0..3 {
                let bx = r.x + r.w - 16.0 - k as f32 * 6.0;
                ui.rect(bx, r.y + r.h - 6.0, 2.5, 2.5, with_alpha(accent, 0.28));
            }
            ui.text_bold(name, r.x + 9.0, r.y + 14.0, 6.8, C_WHITE, true);
            if self.library_tab == 3 {
                let def = &buffs::POOL[idx];
                let tag = match def.class {
                    Some(c) => c.name().to_owned(),
                    None => def.rarity.name().to_owned(),
                };
                ui.text_right(&tag, r.x + r.w - 7.0, r.y + 14.0, 5.0, accent, false);
            } else {
                let tag = match self.library_tab {
                    0 => "ATAQUE",
                    1 => "GUARDIÁN",
                    _ => "CRIATURA",
                };
                ui.text_right(tag, r.x + r.w - 7.0, r.y + 14.0, 4.6, accent, false);
            }
            ui.wrap_center(
                desc,
                r.x + r.w * 0.5,
                r.y + 28.0,
                r.w - 18.0,
                5.8,
                9.0,
                Color::from_rgba(214, 208, 230, 255),
            );
        }

        let counter = format!("{} entradas   ·   Página {} de {}", count, page + 1, pages);
        ui.text_center(&counter, 232.0, 5.8, Color::from_rgba(190, 184, 210, 255));
        ui_button(ui, library_prev_rect(), "‹ Anterior", C_CYAN, m);
        ui_button(ui, library_next_rect(), "Siguiente ›", C_CYAN, m);
        ui.text_center(
            "A/D o Q/E cambian de página   ·   Esc vuelve al menú",
            258.0,
            5.4,
            Color::from_rgba(160, 155, 178, 255),
        );
    }

    fn draw_settings(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.018, 0.014, 0.032, 0.96));
        ui.text_center_shadow("Ajustes", 24.0, 12.0, C_CYAN);
        ui.text_center(
            "Reasigna las teclas del descenso a tu gusto",
            38.0,
            5.8,
            Color::from_rgba(205, 199, 222, 255),
        );
        let m = self.mouse_virtual();
        for i in 0..save::KEY_ACTION_COUNT {
            let r = settings_row_rect(i);
            let selected = self.rebinding && self.settings_row == i;
            let hover = r.contains(m) && !self.rebinding;
            ui.rect(
                r.x,
                r.y,
                r.w,
                r.h,
                if selected {
                    Color::new(0.13, 0.10, 0.06, 0.98)
                } else if hover {
                    Color::new(0.09, 0.08, 0.13, 0.98)
                } else {
                    Color::new(0.055, 0.048, 0.082, 0.96)
                },
            );
            ui.rect_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                1.0,
                if selected {
                    C_GOLD
                } else if hover {
                    with_alpha(C_CYAN, 0.8)
                } else {
                    Color::new(0.28, 0.26, 0.36, 0.95)
                },
            );
            ui.text(
                save::KEY_ACTION_NAMES[i],
                r.x + 9.0,
                r.y + 17.0,
                6.4,
                if selected { C_GOLD } else { C_WHITE },
            );
            let chip = if selected {
                "Pulsa una tecla...".to_owned()
            } else {
                key_label(self.meta.binds.codes[i]).to_owned()
            };
            let cw = ui.measure(&chip, 5.8, true).max(38.0) + 10.0;
            let cx = r.x + r.w - cw - 6.0;
            ui.rect(
                cx,
                r.y + 6.0,
                cw,
                16.0,
                if selected {
                    with_alpha(C_GOLD, 0.20)
                } else {
                    Color::new(0.03, 0.025, 0.05, 0.95)
                },
            );
            ui.rect_lines(
                cx,
                r.y + 6.0,
                cw,
                16.0,
                0.8,
                if selected {
                    C_GOLD
                } else {
                    with_alpha(C_CYAN, 0.45)
                },
            );
            let tw = ui.measure(&chip, 5.8, true);
            ui.text_bold(
                &chip,
                cx + (cw - tw) * 0.5,
                r.y + 17.0,
                5.8,
                if selected { C_GOLD } else { C_CYAN },
                true,
            );
        }

        ui_button(ui, settings_reset_rect(), "Restablecer", C_PURPLE, m);
        ui_button(ui, settings_audio_rect(), "Audio", C_GREEN, m);
        ui_button(ui, settings_back_rect(), "Volver", C_CYAN, m);
        ui_button(
            ui,
            settings_transition_rect(),
            if self.meta.floor_transitions {
                "Transición de piso: Sí"
            } else {
                "Transición de piso: No"
            },
            if self.meta.floor_transitions {
                C_GREEN
            } else {
                Color::from_rgba(150, 145, 170, 255)
            },
            m,
        );
        let diff = self.difficulty();
        ui_button(
            ui,
            settings_difficulty_rect(),
            &format!("Dificultad: {}", diff.name()),
            diff.color(),
            m,
        );
        let hint = if self.rebinding {
            "Pulsa la nueva tecla. Esc cancela. Las teclas repetidas se rechazan."
        } else {
            "Haz clic en una acción para reasignarla. La dificultad se guarda al instante."
        };
        ui.text_center(
            hint,
            244.0,
            5.6,
            if self.rebinding {
                C_GOLD
            } else {
                Color::from_rgba(190, 184, 210, 255)
            },
        );
        ui.text_center(diff.desc(), 256.0, 5.2, diff.color());
    }

    fn draw_stats(&self, ui: &Ui) {
        ui.rect(0.0, 0.0, VW, VH, Color::new(0.02, 0.015, 0.035, 0.93));
        ui.panel(40.0, 18.0, 400.0, 236.0, with_alpha(C_PURPLE, 0.60));
        ui.text_center_shadow("Estadísticas", 40.0, 12.0, C_PURPLE);
        ui.text_center(
            &format!(
                "{}   ·   {}   ·   Piso {}",
                self.meta.name,
                self.player.class.name(),
                self.floor
            ),
            54.0,
            6.2,
            Color::from_rgba(225, 219, 238, 255),
        );

        let p = &self.player;
        let col1 = [
            ("Daño", format!("{:.1}", p.damage)),
            ("Vida máxima", format!("{:.0}", p.max_hp)),
            ("Velocidad", format!("{:.0}", p.speed)),
            ("Ataques por segundo", format!("{:.2}", p.attack_speed)),
            ("Alcance", format!("{:.0}", p.range)),
            ("Crítico", format!("{:.0}%", p.crit * 100.0)),
            ("Daño crítico", format!("x{:.1}", p.crit_mult)),
            ("Armadura", format!("{:.1}", p.armor)),
            ("Bloqueo", format!("{:.0}%", p.block * 100.0)),
        ];
        let col2 = [
            ("Regeneración", format!("{:.1}/s", p.regen)),
            ("Robo de vida", format!("{:.0}%", p.lifesteal * 100.0)),
            ("Espinas", format!("{:.0}%", p.thorns * 100.0)),
            ("Reflejo", format!("{:.0}%", p.reflect * 100.0)),
            ("Área", format!("{:.0}", p.aoe)),
            ("Suerte", format!("{:.0}%", p.luck * 100.0)),
            ("Ejecución", format!("{:.0}%", p.execute * 100.0)),
            ("Recarga de esquiva", format!("{:.1}s", p.dash_cd_max)),
            ("Proyectiles", format!("{}", 1 + p.extra_projectiles)),
        ];
        for (i, (label, value)) in col1.iter().enumerate() {
            let y = 72.0 + i as f32 * 12.0;
            ui.text(label, 62.0, y, 6.0, Color::from_rgba(190, 184, 210, 255));
            ui.text_right(value, 218.0, y, 6.0, C_WHITE, true);
            ui.line(
                62.0,
                y + 3.0,
                218.0,
                y + 3.0,
                0.4,
                Color::new(0.24, 0.22, 0.32, 0.55),
            );
        }
        for (i, (label, value)) in col2.iter().enumerate() {
            let y = 72.0 + i as f32 * 12.0;
            ui.text(label, 252.0, y, 6.0, Color::from_rgba(190, 184, 210, 255));
            ui.text_right(value, 418.0, y, 6.0, C_WHITE, true);
            ui.line(
                252.0,
                y + 3.0,
                418.0,
                y + 3.0,
                0.4,
                Color::new(0.24, 0.22, 0.32, 0.55),
            );
        }

        // Ataques equipados: icono, nivel y nombre corto.
        ui.line(58.0, 180.0, 422.0, 180.0, 0.7, with_alpha(C_PURPLE, 0.40));
        ui.text(
            "Ataques",
            62.0,
            190.0,
            5.2,
            Color::from_rgba(190, 184, 210, 255),
        );
        for (i, e) in self.equipped_attacks.iter().take(MAX_EQUIPPED).enumerate() {
            let d = &self.attacks[e.def];
            let x = 104.0 + i as f32 * 53.0;
            self.draw_attack_icon(ui, LevelChoice::NewAttack(e.def), x, 187.0, d.color, 0.95);
            ui.text(&format!("Nv.{}", e.level), x + 8.0, 190.0, 4.8, d.color);
        }

        ui.text_center(
            &format!(
                "Oro {}   ·   Bajas {}   ·   Llaves {}   ·   Negras {}   ·   Objetos {}   ·   Mejoras {}",
                p.gold,
                self.kills,
                self.keys,
                self.special_keys,
                self.items.len(),
                self.buff_stacks.iter().filter(|&&n| n > 0).count()
            ),
            202.0,
            5.4,
            C_GOLD,
        );
        let echo_line = if self.meta.echoes > 0 {
            format!(
                "Ecos {}   ·   Piso del último eco {}   ·   Deuda {}",
                self.meta.echoes, self.meta.echo_floor, self.meta.debt
            )
        } else {
            format!(
                "Ecos 0   ·   Aún no hay ecos   ·   Deuda {}",
                self.meta.debt
            )
        };
        ui.text_center(
            &echo_line,
            221.0,
            5.4,
            if self.meta.debt > 0 { C_RED } else { C_CYAN },
        );
        let m = self.mouse_virtual();
        ui_button(ui, stats_back_rect(), "Volver (Esc)", C_CYAN, m);
    }

    fn draw_toasts(&self, ui: &Ui) {
        for (i, t) in self.toasts.iter().rev().enumerate() {
            let a = (t.t / 0.45).clamp(0.0, 1.0);
            let w = ui.measure(&t.text, 6.8, true);
            let x = VW - w - 14.0;
            let y = 228.0 - i as f32 * 13.0;
            ui.rect(
                x - 6.0,
                y - 9.0,
                w + 12.0,
                13.0,
                Color::new(0.02, 0.015, 0.04, 0.86 * a),
            );
            ui.rect(x - 6.0, y - 9.0, 2.0, 13.0, with_alpha(t.color, 0.75 * a));
            ui.text_shadow(&t.text, x, y, 6.8, with_alpha(t.color, a), true);
        }
    }

    fn draw_dev(&self, ui: &Ui) {
        if !self.show_debug {
            return;
        }
        let x = 292.0;
        let y = 58.0;
        let w = 180.0;
        let h = if self.dev_note.is_empty() { 62.0 } else { 72.0 };
        ui.rect(x, y, w, h, Color::new(0.02, 0.02, 0.03, 0.88));
        ui.rect_lines(x, y, w, h, 1.0, with_alpha(C_CYAN, 0.70));
        let mut yy = y + 11.0;
        ui.text(
            &format!("FPS {:.0}   {:.1} ms", self.fps, self.frame_ms),
            x + 5.0,
            yy,
            6.2,
            if self.fps >= 55.0 { C_GREEN } else { C_RED },
        );
        yy += 11.0;
        ui.text(
            &format!(
                "E {}  P {}  PA {}  PT {}",
                self.enemies.len(),
                self.projectiles.len(),
                self.particles.iter().filter(|p| p.active).count(),
                self.pickups.len()
            ),
            x + 5.0,
            yy,
            6.2,
            Color::from_rgba(215, 215, 230, 255),
        );
        yy += 11.0;
        ui.text(
            &format!(
                "Piso {}  Oleada {}/{}  T {:.0}s",
                self.floor, self.wave, self.waves, self.time
            ),
            x + 5.0,
            yy,
            6.2,
            Color::from_rgba(215, 215, 230, 255),
        );
        yy += 11.0;
        ui.text(
            &format!(
                "Build {}",
                if self.dev_build.is_empty() {
                    "-"
                } else {
                    self.dev_build.as_str()
                }
            ),
            x + 5.0,
            yy,
            6.2,
            Color::from_rgba(170, 215, 230, 255),
        );
        if !self.dev_note.is_empty() && self.dev_note_t > 0.0 {
            ui.text(
                &self.dev_note,
                x + 5.0,
                yy + 11.0,
                6.2,
                with_alpha(C_GOLD, self.dev_note_t.min(1.0)),
            );
        }
    }
}

fn apply_weapon(p: &mut Player, weapon: &weapons::WeaponDef) {
    for &effect in weapon.effects {
        match effect {
            WeaponEffect::Damage(v) => p.damage *= 1.0 + v,
            WeaponEffect::MaxHp(v) => {
                let add = p.max_hp * v;
                p.max_hp += add;
                p.hp = (p.hp + add).min(p.max_hp);
            }
            WeaponEffect::Speed(v) => p.speed *= 1.0 + v,
            WeaponEffect::AttackSpeed(v) => p.attack_speed *= 1.0 + v,
            WeaponEffect::Crit(v) => p.crit = (p.crit + v).min(0.85),
            WeaponEffect::CritMult(v) => p.crit_mult += v,
            WeaponEffect::Armor(v) => p.armor += v,
            WeaponEffect::Gold(v) => p.gold_mult += v,
            WeaponEffect::Luck(v) => p.luck += v,
            WeaponEffect::Aoe(v) => p.aoe *= 1.0 + v,
            WeaponEffect::Regen(v) => p.regen += v,
            WeaponEffect::Block(v) => p.block = (p.block + v).min(0.60),
            WeaponEffect::Execute(v) => p.execute += v,
            WeaponEffect::SkeletonCap(v) => p.skeleton_cap += v,
            WeaponEffect::SkeletonDmg(v) => p.skeleton_dmg *= 1.0 + v,
            WeaponEffect::SkeletonInherit(v) => p.skeleton_inherit += v,
            WeaponEffect::ExtraProjectiles(v) => p.extra_projectiles += v,
            WeaponEffect::Pierce(v) => p.pierce += v,
            WeaponEffect::Burn(v) => p.burn_on_hit += v,
            WeaponEffect::Chain(v) => p.chain += v,
            WeaponEffect::Shockwave(v) => p.shockwave += v,
        }
    }
}

fn apply_buff_to_player(p: &mut Player, id: BuffId, class: Class) {
    match id {
        BuffId::Fury => p.damage *= 1.20,
        BuffId::Vigor => {
            let add = p.max_hp * 0.20;
            p.max_hp += add;
            p.hp = (p.hp + add).min(p.max_hp);
        }
        BuffId::Swift => p.speed *= 1.12,
        BuffId::Haste => p.attack_speed *= 1.12,
        BuffId::Crit => p.crit = (p.crit + 0.08).min(0.75),
        BuffId::CritDmg => p.crit_mult += 0.40,
        BuffId::Armor => p.armor += 2.0,
        BuffId::Regen => p.regen += 0.8,
        BuffId::Lifesteal => p.lifesteal += 0.04,
        BuffId::Thorns => p.thorns += 0.25,
        BuffId::DashCd => p.dash_cd_max = (p.dash_cd_max * 0.80).max(0.6),
        BuffId::Luck => p.luck += 0.15,
        BuffId::Wisdom => p.xp_mult += 0.15,
        BuffId::Greed => p.gold_mult += 0.25,
        BuffId::Legion => {
            if class == Class::Necro {
                p.skeleton_cap += 2;
            }
        }
        BuffId::BoneDmg => {}
        BuffId::BoneExplode => p.skeleton_explode = true,
        BuffId::BoneInherit => p.skeleton_inherit += 0.25,
        BuffId::Block => p.block = (p.block + 0.12).min(0.60),
        BuffId::Reflect => p.reflect += 0.50,
        BuffId::Shockwave => p.shockwave += 1,
        BuffId::Multishot => p.extra_projectiles += 1,
        BuffId::Pierce => p.pierce += 1,
        BuffId::Aim => {
            p.range *= 1.15;
            p.crit = (p.crit + 0.10).min(0.75);
        }
        BuffId::Pyro => {
            p.aoe *= 1.35;
            p.burn_on_hit += 2.5;
        }
        BuffId::Chain => p.chain += 1,
        BuffId::Execute => p.execute += 0.30,
        BuffId::Bloodlust => p.bloodlust += 1,
        BuffId::Lethal => {
            p.crit = (p.crit + 0.12).min(0.75);
            p.crit_mult += 0.50;
        }
        BuffId::Crown => {
            p.damage *= 1.15;
            p.max_hp *= 1.15;
            p.speed *= 1.15;
            p.attack_speed *= 1.15;
            p.armor *= 1.15;
            p.regen *= 1.15;
            p.lifesteal *= 1.15;
            p.thorns *= 1.15;
            p.luck *= 1.15;
            p.xp_mult *= 1.15;
            p.gold_mult *= 1.15;
            p.aoe *= 1.15;
            p.range *= 1.15;
            p.hp = (p.hp + p.max_hp * 0.15).min(p.max_hp);
        }
        BuffId::Faith => {
            p.regen += 0.7;
            let add = p.max_hp * 0.08;
            p.max_hp += add;
            p.hp = (p.hp + add).min(p.max_hp);
        }
        BuffId::Sanctuary => {
            p.block = (p.block + 0.10).min(0.60);
            p.armor += 1.0;
        }
        BuffId::Commerce => {
            p.gold_mult += 0.25;
            p.luck += 0.10;
        }
    }
}

fn item_short_desc(item: &Item) -> String {
    match item.kind {
        ItemKind::Damage => format!("+{:.1} de daño.", item.value),
        ItemKind::MaxHp => format!("+{:.0} de vida máxima.", item.value),
        ItemKind::Speed => format!("+{:.1} de velocidad.", item.value),
        ItemKind::Crit => format!("+{:.0}% de crítico.", item.value * 100.0),
        ItemKind::Lifesteal => format!("+{:.0}% de robo de vida.", item.value * 100.0),
        ItemKind::Thorns => format!("+{:.0}% de daño devuelto.", item.value * 100.0),
        ItemKind::Armor => format!("+{:.1} de armadura.", item.value),
        ItemKind::SkeletonCap => format!("+{} esqueleto.", item.value.round().max(1.0)),
        ItemKind::Regen => format!("+{:.1} de vida por segundo.", item.value),
        ItemKind::AllStats => format!("+{:.0}% a todas las estadísticas.", item.value * 100.0),
        ItemKind::GlassCannon => format!("+{:.0}% de daño, -15% de vida.", item.value * 100.0),
        ItemKind::CrystalHeart => format!("+{:.0} de vida y regeneración.", item.value),
        ItemKind::Orbit => "Un sigilo orbita y corta a los enemigos.".to_owned(),
        ItemKind::Homing => "Tus proyectiles persiguen al enemigo.".to_owned(),
        ItemKind::Chain => "Los impactos saltan a otro enemigo.".to_owned(),
        ItemKind::Nova => "Cada 5 ataques estalla una nova de cristal.".to_owned(),
        ItemKind::Mine => "Dejas minas rituales al atacar.".to_owned(),
        ItemKind::Retaliate => "Al recibir daño, respondes con una ráfaga.".to_owned(),
        ItemKind::DashBurst => "Tu esquiva termina en una explosión astral.".to_owned(),
        ItemKind::Vortex => "Abre remolinos que arrastran y desgarran enemigos.".to_owned(),
        ItemKind::Bolas => "Lanza bolas pesadas que ralentizan y perforan.".to_owned(),
        ItemKind::Trail => "Deja una estela de brasas que quema al pisarla.".to_owned(),
        ItemKind::Aura => "Un aura de ceniza castiga a todo enemigo cercano.".to_owned(),
        ItemKind::Mend => "Pulsos de costura restauran tu vida con el tiempo.".to_owned(),
    }
}

fn hurt_player(
    player: &mut Player,
    dmg: f32,
    rng: &mut Rng,
    floaters: &mut Vec<Floater>,
    particles: &mut Vec<Particle>,
    shake: &mut f32,
    sfx: &mut Sfx,
    hit_pos: Vec2,
) -> (f32, bool) {
    if player.invuln > 0.0 || player.hp <= 0.0 {
        return (0.0, false);
    }
    if player.block > 0.0 && rng.chance(player.block) {
        player.invuln = 0.28;
        *shake = (*shake).max(2.0);
        spawn_floater(
            floaters,
            player.pos + vec2(0.0, -16.0),
            "Bloqueo",
            C_CYAN,
            8.0,
        );
        burst(particles, hit_pos, 4, C_CYAN, 40.0);
        sfx.play(SfxKind::Click, 0.30);
        return (0.0, true);
    }
    let reduction = (player.armor / (player.armor + 25.0)).min(0.75);
    let taken = (dmg * (1.0 - reduction)).max(1.0);
    player.hp -= taken;
    player.hit_flash = 0.18;
    player.invuln = 0.32;
    *shake = (*shake).max(3.5);
    spawn_number(
        floaters,
        player.pos + vec2(0.0, -18.0),
        taken.round() as i32,
        C_RED,
        8.0,
    );
    burst(particles, player.pos, 6, C_RED, 55.0);
    sfx.play(SfxKind::Hurt, 0.35);
    (taken, false)
}

fn draw_chest_marker(ui: &Ui, x: f32, y: f32, size: f32, alpha: f32) {
    let body = with_alpha(C_GOLD, alpha);
    let lid = with_alpha(Color::from_rgba(170, 116, 48, 255), alpha);
    let band = with_alpha(Color::from_rgba(64, 38, 16, 255), alpha);
    ui.rect(x, y + size * 0.22, size, size * 0.68, body);
    ui.rect(x, y, size, size * 0.26, lid);
    ui.rect(x, y + size * 0.44, size, size * 0.12, band);
}

/// Llave negra de bóveda: asta oscura, anillo y dientes. Se dibuja sobre el
/// mapa para señalar una cámara sellada que aún no se ha abierto.
/// Llave de bóveda: una llave de luz violeta, inconfundible en la carta.
/// Nunca se dibuja como un borrón negro ni como un círculo gigante.
fn draw_black_key_marker(ui: &Ui, x: f32, y: f32, k: f32, alpha: f32) {
    let body = with_alpha(Color::from_rgba(198, 168, 255, 255), alpha);
    let edge = with_alpha(Color::from_rgba(124, 92, 178, 255), alpha);
    let glint = with_alpha(Color::from_rgba(245, 235, 255, 255), alpha);

    // Halo pequeño y tenue, solo para despegar la llave del fondo.
    ui.circle(
        x - 1.4 * k,
        y - 1.8 * k,
        4.6 * k,
        with_alpha(Color::from_rgba(150, 100, 235, 255), 0.14 * alpha),
    );

    // Anilla de la llave, con doble trazo para que se lea a cualquier zoom.
    draw_circle_lines(
        ui.x(x - 1.6 * k),
        ui.y(y - 2.3 * k),
        ui.s(2.4 * k),
        ui.s(1.5 * k),
        edge,
    );
    draw_circle_lines(
        ui.x(x - 1.6 * k),
        ui.y(y - 2.3 * k),
        ui.s(1.7 * k),
        ui.s(0.9 * k),
        body,
    );
    // Hueco de la anilla.
    ui.circle(
        x - 1.6 * k,
        y - 2.3 * k,
        0.75 * k,
        with_alpha(Color::from_rgba(18, 14, 26, 255), alpha),
    );

    // Caña diagonal y dientes: la silueta clásica de una llave.
    let sx = x - 0.1 * k;
    let sy = y - 0.8 * k;
    let ex = x + 4.2 * k;
    let ey = y + 3.3 * k;
    ui.line(sx, sy, ex, ey, 2.5 * k, edge);
    ui.line(sx, sy, ex, ey, 1.4 * k, body);

    for (tx, ty, dx, dy) in [
        (x + 1.9 * k, y + 1.2 * k, 1.6 * k, -0.5 * k),
        (x + 2.9 * k, y + 2.1 * k, 1.6 * k, -0.5 * k),
    ] {
        ui.line(tx, ty, tx + dx, ty + dy, 2.0 * k, edge);
        ui.line(tx, ty, tx + dx, ty + dy, 1.0 * k, body);
    }

    // Destello en la punta.
    ui.line(
        ex - 0.9 * k,
        ey - 0.9 * k,
        ex + 0.9 * k,
        ey + 0.9 * k,
        0.5 * k,
        glint,
    );
    ui.line(
        ex + 0.9 * k,
        ey - 0.9 * k,
        ex - 0.9 * k,
        ey + 0.9 * k,
        0.5 * k,
        glint,
    );
}

/// Candado abierto: el arco se ha separado del cuerpo. Marca una bóveda ya
/// abierta en la carta de la cripta.
fn draw_open_lock_marker(ui: &Ui, x: f32, y: f32, k: f32, alpha: f32) {
    let body = with_alpha(C_PURPLE, alpha);
    let dark = with_alpha(Color::new(0.02, 0.015, 0.03, 1.0), alpha);
    ui.line(
        x - 1.8 * k,
        y - 0.4 * k,
        x - 1.8 * k,
        y - 2.6 * k,
        0.9 * k,
        body,
    );
    ui.line(
        x - 1.8 * k,
        y - 2.6 * k,
        x + 1.2 * k,
        y - 2.6 * k,
        0.9 * k,
        body,
    );
    ui.line(
        x + 1.2 * k,
        y - 2.6 * k,
        x + 2.2 * k,
        y - 1.6 * k,
        0.9 * k,
        body,
    );
    ui.rect(x - 2.6 * k, y - 0.4 * k, 5.4 * k, 3.6 * k, body);
    ui.rect(x - 0.8 * k, y + 0.4 * k, 1.6 * k, 1.6 * k, dark);
}

fn menu_class_rect(i: usize) -> Rect {
    Rect::new(14.0, 66.0 + i as f32 * 19.0, 158.0, 18.0)
}

fn menu_play_rect() -> Rect {
    Rect::new(252.0, 150.0, 196.0, 24.0)
}

fn menu_action_rect(i: usize) -> Rect {
    Rect::new(14.0 + i as f32 * 65.0, 201.0, 62.0, 18.0)
}

fn upgrade_rect(i: usize) -> Rect {
    Rect::new(14.0 + i as f32 * 152.0, 222.0, 148.0, 32.0)
}

fn card_rect(i: usize) -> Rect {
    Rect::new(192.0, 58.0 + i as f32 * 56.0, 264.0, 52.0)
}

fn replace_rect(i: usize) -> Rect {
    Rect::new(192.0, 88.0 + i as f32 * 24.0, 264.0, 22.0)
}

fn library_back_rect() -> Rect {
    Rect::new(12.0, 10.0, 58.0, 18.0)
}

fn library_tab_rect(i: usize) -> Rect {
    Rect::new(12.0 + i as f32 * 118.0, 34.0, 112.0, 16.0)
}

fn library_card_rect(i: usize) -> Rect {
    let col = i % 2;
    let row = i / 2;
    Rect::new(
        16.0 + col as f32 * 228.0,
        56.0 + row as f32 * 56.0,
        220.0,
        50.0,
    )
}

fn library_prev_rect() -> Rect {
    Rect::new(16.0, 242.0, 58.0, 18.0)
}

fn library_next_rect() -> Rect {
    Rect::new(406.0, 242.0, 58.0, 18.0)
}

fn settings_row_rect(i: usize) -> Rect {
    let col = i / 4;
    let row = i % 4;
    Rect::new(
        28.0 + col as f32 * 220.0,
        62.0 + row as f32 * 34.0,
        204.0,
        28.0,
    )
}

fn settings_reset_rect() -> Rect {
    Rect::new(118.0, 218.0, 86.0, 18.0)
}

fn settings_audio_rect() -> Rect {
    Rect::new(212.0, 218.0, 86.0, 18.0)
}

fn settings_transition_rect() -> Rect {
    Rect::new(28.0, 196.0, 240.0, 18.0)
}

fn settings_difficulty_rect() -> Rect {
    Rect::new(276.0, 196.0, 176.0, 18.0)
}

fn settings_back_rect() -> Rect {
    Rect::new(306.0, 218.0, 56.0, 18.0)
}

fn shop_back_rect() -> Rect {
    Rect::new(12.0, 10.0, 58.0, 18.0)
}

fn shop_tab_rect(i: usize) -> Rect {
    Rect::new(152.0 + i as f32 * 58.0, 40.0, 54.0, 16.0)
}

fn shop_item_rect(i: usize) -> Rect {
    let col = i % 4;
    let row = i / 4;
    Rect::new(
        20.0 + col as f32 * 112.0,
        64.0 + row as f32 * 78.0,
        104.0,
        72.0,
    )
}

fn shop_offer_rect(i: usize) -> Rect {
    // Fila inferior, por encima del HUD y sin tapar las puertas laterales ni
    // la puerta inferior de la sala del mercader.
    Rect::new(54.0 + i as f32 * 94.0, 160.0, 90.0, 56.0)
}

/// Ruido determinista para la decoración de las salas. La misma semilla da
/// siempre el mismo rincón, grieta o estandarte.
fn decor_rand(seed: u32, salt: u32) -> f32 {
    let mut x = seed
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(salt.wrapping_mul(0x85EB_CA6B));
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0xFFFF) as f32 / 65535.0
}

fn locker_back_rect() -> Rect {
    Rect::new(12.0, 10.0, 58.0, 18.0)
}

fn locker_name_rect() -> Rect {
    Rect::new(36.0, 68.0, 160.0, 20.0)
}

fn locker_class_minus() -> Rect {
    Rect::new(38.0, 94.0, 20.0, 20.0)
}

fn locker_class_plus() -> Rect {
    Rect::new(150.0, 94.0, 20.0, 20.0)
}

fn locker_minus(row: usize) -> Rect {
    Rect::new(276.0, 58.0 + row as f32 * 44.0 + 9.0, 18.0, 20.0)
}

fn locker_plus(row: usize) -> Rect {
    Rect::new(382.0, 58.0 + row as f32 * 44.0 + 9.0, 18.0, 20.0)
}

fn tickets_row_rect(i: usize) -> Rect {
    Rect::new(32.0, 60.0 + i as f32 * 31.0, 416.0, 28.0)
}

fn forge_back_rect() -> Rect {
    Rect::new(24.0, 22.0, 62.0, 16.0)
}

fn forge_class_minus() -> Rect {
    Rect::new(184.0, 48.0, 18.0, 18.0)
}

fn forge_class_plus() -> Rect {
    Rect::new(278.0, 48.0, 18.0, 18.0)
}

fn forge_card_rect(i: usize) -> Rect {
    Rect::new(24.0 + i as f32 * 148.0, 74.0, 140.0, 148.0)
}

fn forge_button_rect(i: usize) -> Rect {
    let r = forge_card_rect(i);
    Rect::new(r.x + 10.0, r.y + 120.0, r.w - 20.0, 18.0)
}

fn tickets_back_rect() -> Rect {
    Rect::new(30.0, 218.0, 86.0, 18.0)
}

fn locker_shop_rect() -> Rect {
    Rect::new(286.0, 214.0, 104.0, 18.0)
}

fn locker_tickets_rect() -> Rect {
    Rect::new(232.0, 188.0, 212.0, 18.0)
}

fn locker_forge_rect() -> Rect {
    Rect::new(396.0, 10.0, 74.0, 18.0)
}

fn stats_back_rect() -> Rect {
    Rect::new(180.0, 230.0, 120.0, 18.0)
}

fn map_close_rect() -> Rect {
    Rect::new(192.0, 228.0, 96.0, 16.0)
}

fn dead_retry_rect() -> Rect {
    Rect::new(128.0, 198.0, 108.0, 20.0)
}

fn dead_menu_rect() -> Rect {
    Rect::new(244.0, 198.0, 108.0, 20.0)
}

fn victory_endless_rect() -> Rect {
    Rect::new(112.0, 198.0, 120.0, 20.0)
}

fn victory_menu_rect() -> Rect {
    Rect::new(248.0, 198.0, 120.0, 20.0)
}

fn pause_continue_rect() -> Rect {
    Rect::new(132.0, 104.0, 216.0, 18.0)
}

fn pause_stats_rect() -> Rect {
    Rect::new(132.0, 126.0, 216.0, 18.0)
}

fn pause_inventory_rect() -> Rect {
    Rect::new(132.0, 148.0, 216.0, 18.0)
}

fn pause_settings_rect() -> Rect {
    Rect::new(132.0, 170.0, 216.0, 18.0)
}

fn pause_audio_rect() -> Rect {
    Rect::new(132.0, 192.0, 216.0, 18.0)
}

fn inventory_back_rect() -> Rect {
    Rect::new(180.0, 228.0, 120.0, 18.0)
}

fn audio_row_rect(i: usize) -> Rect {
    Rect::new(90.0, 82.0 + i as f32 * 36.0, 300.0, 30.0)
}

fn audio_bar_rect(i: usize) -> Rect {
    let r = audio_row_rect(i);
    Rect::new(r.x + 84.0, r.y + 11.0, 130.0, 8.0)
}

fn audio_minus_rect(i: usize) -> Rect {
    let r = audio_row_rect(i);
    Rect::new(r.x + 252.0, r.y + 6.0, 18.0, 18.0)
}

fn audio_plus_rect(i: usize) -> Rect {
    let r = audio_row_rect(i);
    Rect::new(r.x + 276.0, r.y + 6.0, 18.0, 18.0)
}

fn audio_back_rect() -> Rect {
    Rect::new(180.0, 196.0, 120.0, 18.0)
}

fn pause_abandon_rect() -> Rect {
    Rect::new(132.0, 214.0, 216.0, 18.0)
}

fn levelup_reroll_rect() -> Rect {
    Rect::new(192.0, 226.0, 126.0, 16.0)
}

fn levelup_stats_rect() -> Rect {
    Rect::new(328.0, 226.0, 128.0, 16.0)
}

/// Icono compacto de una mejora. Cada familia tiene una forma propia para
/// que la carta se lea de un vistazo.
fn draw_buff_icon(ui: &Ui, id: BuffId, x: f32, y: f32, color: Color, alpha: f32) {
    let c = with_alpha(color, alpha);
    let soft = with_alpha(color, 0.35 * alpha);
    match id {
        BuffId::Fury | BuffId::CritDmg | BuffId::Lethal => {
            ui.line(x - 4.5, y + 5.0, x + 4.5, y - 5.0, 1.4, c);
            ui.line(
                x - 1.0,
                y - 1.0,
                x + 1.5,
                y + 1.5,
                1.2,
                with_alpha(C_WHITE, 0.8 * alpha),
            );
            ui.line(x - 4.0, y + 2.0, x - 1.5, y + 4.5, 1.1, soft);
        }
        BuffId::Vigor => {
            ui.circle(x - 2.2, y - 1.2, 2.6, c);
            ui.circle(x + 2.2, y - 1.2, 2.6, c);
            ui.triangle(vec2(x - 4.7, y), vec2(x + 4.7, y), vec2(x, y + 5.6), c);
        }
        BuffId::Swift | BuffId::DashCd => {
            ui.triangle(
                vec2(x - 5.0, y - 4.5),
                vec2(x + 0.5, y),
                vec2(x - 5.0, y + 4.5),
                c,
            );
            ui.triangle(
                vec2(x + 0.0, y - 4.5),
                vec2(x + 5.5, y),
                vec2(x + 0.0, y + 4.5),
                soft,
            );
        }
        BuffId::Haste | BuffId::Multishot | BuffId::Pierce | BuffId::Aim => {
            ui.line(x - 5.0, y + 4.0, x + 4.0, y - 4.0, 1.1, c);
            ui.triangle(
                vec2(x + 5.5, y - 5.5),
                vec2(x + 2.0, y - 3.5),
                vec2(x + 3.5, y - 0.5),
                c,
            );
            ui.line(x - 4.0, y - 1.0, x + 0.5, y + 3.5, 0.8, soft);
        }
        BuffId::Crit | BuffId::Luck => {
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::FRAC_PI_2 + 0.78;
                ui.line(x, y, x + a.cos() * 6.0, y + a.sin() * 6.0, 1.0, c);
            }
            ui.circle(x, y, 1.6, with_alpha(C_WHITE, alpha));
        }
        BuffId::Armor | BuffId::Block | BuffId::Reflect | BuffId::Thorns | BuffId::Sanctuary => {
            ui.rect(x - 4.6, y - 5.6, 9.2, 7.0, c);
            ui.triangle(
                vec2(x - 4.6, y + 1.0),
                vec2(x + 4.6, y + 1.0),
                vec2(x, y + 6.0),
                c,
            );
            ui.line(
                x,
                y - 4.0,
                x,
                y + 4.0,
                0.7,
                with_alpha(C_WHITE, 0.55 * alpha),
            );
        }
        BuffId::Regen | BuffId::Lifesteal | BuffId::Bloodlust | BuffId::Faith => {
            ui.circle(x, y + 2.0, 4.2, c);
            ui.triangle(
                vec2(x - 4.2, y + 1.0),
                vec2(x + 4.2, y + 1.0),
                vec2(x, y - 7.5),
                c,
            );
            if id == BuffId::Lifesteal || id == BuffId::Bloodlust {
                ui.circle(x + 4.0, y + 4.0, 1.3, with_alpha(C_WHITE, 0.7 * alpha));
            }
        }
        BuffId::Wisdom => {
            ui.rect(x - 5.0, y - 5.0, 10.0, 10.0, c);
            ui.line(
                x,
                y - 5.0,
                x,
                y + 5.0,
                0.8,
                with_alpha(C_WHITE, 0.55 * alpha),
            );
            ui.line(
                x - 3.5,
                y - 2.0,
                x - 0.8,
                y - 2.0,
                0.6,
                with_alpha(C_WHITE, 0.45 * alpha),
            );
        }
        BuffId::Greed | BuffId::Commerce => {
            ui.circle(x, y, 5.4, c);
            draw_circle_lines(
                ui.x(x),
                ui.y(y),
                ui.s(3.2),
                ui.s(0.8),
                with_alpha(C_WHITE, 0.7 * alpha),
            );
            ui.circle(x, y, 1.0, with_alpha(C_WHITE, 0.8 * alpha));
        }
        BuffId::Legion | BuffId::BoneDmg | BuffId::BoneExplode | BuffId::BoneInherit => {
            ui.circle(x, y - 0.6, 4.2, c);
            ui.rect(x - 2.0, y + 2.2, 4.0, 2.4, c);
            ui.circle(x - 1.5, y - 1.2, 1.0, Color::new(0.03, 0.01, 0.02, alpha));
            ui.circle(x + 1.5, y - 1.2, 1.0, Color::new(0.03, 0.01, 0.02, alpha));
            ui.line(x - 1.0, y + 4.4, x - 1.0, y + 6.0, 0.7, c);
            ui.line(x + 1.0, y + 4.4, x + 1.0, y + 6.0, 0.7, c);
        }
        BuffId::Shockwave => {
            draw_circle_lines(ui.x(x), ui.y(y), ui.s(3.0), ui.s(0.8), c);
            draw_circle_lines(ui.x(x), ui.y(y), ui.s(6.0), ui.s(0.7), soft);
        }
        BuffId::Pyro => {
            ui.triangle(
                vec2(x, y - 6.0),
                vec2(x - 4.0, y + 4.0),
                vec2(x + 4.0, y + 4.0),
                c,
            );
            ui.triangle(
                vec2(x, y - 2.0),
                vec2(x - 1.8, y + 3.5),
                vec2(x + 1.8, y + 3.5),
                Color::new(1.0, 0.86, 0.52, alpha),
            );
        }
        BuffId::Chain => {
            ui.circle(x - 2.6, y, 2.4, c);
            ui.circle(x + 2.6, y, 2.4, soft);
            ui.line(x - 0.5, y, x + 0.5, y, 0.8, c);
        }
        BuffId::Execute => {
            ui.line(x - 3.0, y + 5.0, x + 3.0, y - 5.0, 1.5, c);
            ui.line(x - 5.0, y + 3.0, x + 1.0, y + 6.0, 0.9, soft);
            ui.circle(x + 3.0, y - 5.0, 1.0, with_alpha(C_WHITE, 0.8 * alpha));
        }
        BuffId::Crown => {
            ui.triangle(
                vec2(x - 5.5, y + 3.5),
                vec2(x - 5.5, y - 4.0),
                vec2(x - 1.5, y + 1.5),
                c,
            );
            ui.triangle(
                vec2(x - 2.0, y + 3.5),
                vec2(x, y - 6.0),
                vec2(x + 2.0, y + 3.5),
                soft,
            );
            ui.triangle(
                vec2(x + 5.5, y + 3.5),
                vec2(x + 5.5, y - 4.0),
                vec2(x + 1.5, y + 1.5),
                c,
            );
            ui.rect(x - 5.5, y + 3.0, 11.0, 2.2, c);
        }
    }
}

/// Icono de inventario para un objeto, con la misma familia visual que la
/// tienda pero en tamaño reducido.
fn draw_item_icon(ui: &Ui, kind: ItemKind, x: f32, y: f32, color: Color, alpha: f32) {
    let c = with_alpha(color, alpha);
    let soft = with_alpha(color, 0.40 * alpha);
    match kind {
        ItemKind::Damage | ItemKind::GlassCannon => {
            ui.line(x - 3.2, y + 4.0, x + 3.2, y - 4.0, 1.2, c);
            ui.line(x - 2.8, y + 1.2, x - 1.2, y + 2.8, 0.9, soft);
        }
        ItemKind::Crit => {
            ui.line(x - 4.0, y - 4.0, x + 4.0, y + 4.0, 1.0, c);
            ui.line(x + 4.0, y - 4.0, x - 4.0, y + 4.0, 1.0, c);
        }
        ItemKind::MaxHp | ItemKind::CrystalHeart => {
            ui.triangle(vec2(x, y - 5.0), vec2(x - 3.4, y), vec2(x, y + 5.0), c);
            ui.triangle(vec2(x, y - 5.0), vec2(x + 3.4, y), vec2(x, y + 5.0), soft);
        }
        ItemKind::Speed => {
            ui.triangle(
                vec2(x - 4.0, y - 3.5),
                vec2(x + 0.5, y),
                vec2(x - 4.0, y + 3.5),
                c,
            );
            ui.triangle(
                vec2(x + 0.5, y - 3.5),
                vec2(x + 5.0, y),
                vec2(x + 0.5, y + 3.5),
                soft,
            );
        }
        ItemKind::Lifesteal | ItemKind::Regen => {
            ui.circle(x, y + 1.5, 3.2, c);
            ui.triangle(
                vec2(x - 3.2, y + 1.0),
                vec2(x + 3.2, y + 1.0),
                vec2(x, y - 5.5),
                c,
            );
        }
        ItemKind::Thorns | ItemKind::Armor => {
            ui.rect(x - 3.6, y - 4.4, 7.2, 5.4, c);
            ui.triangle(
                vec2(x - 3.6, y + 0.8),
                vec2(x + 3.6, y + 0.8),
                vec2(x, y + 4.8),
                c,
            );
        }
        ItemKind::SkeletonCap => {
            ui.circle(x, y - 0.5, 3.4, c);
            ui.rect(x - 1.6, y + 1.8, 3.2, 2.0, c);
            ui.circle(x - 1.2, y - 1.0, 0.8, Color::new(0.03, 0.01, 0.02, alpha));
            ui.circle(x + 1.2, y - 1.0, 0.8, Color::new(0.03, 0.01, 0.02, alpha));
        }
        ItemKind::AllStats => {
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::FRAC_PI_2 + 0.78;
                ui.line(x, y, x + a.cos() * 5.0, y + a.sin() * 5.0, 0.9, c);
            }
            ui.circle(x, y, 1.4, with_alpha(C_WHITE, alpha));
        }
        ItemKind::Orbit | ItemKind::Vortex => {
            draw_circle_lines(ui.x(x), ui.y(y), ui.s(4.4), ui.s(0.8), c);
            ui.circle(x + 3.6, y - 2.0, 1.3, c);
        }
        ItemKind::Homing => {
            ui.line(x - 4.0, y + 3.0, x + 3.0, y - 3.0, 1.0, c);
            ui.triangle(
                vec2(x + 4.0, y - 4.0),
                vec2(x + 1.5, y - 2.5),
                vec2(x + 2.5, y),
                c,
            );
        }
        ItemKind::Chain => {
            ui.circle(x - 2.0, y, 1.9, c);
            ui.circle(x + 2.0, y, 1.9, soft);
        }
        ItemKind::Nova => {
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::FRAC_PI_2;
                ui.line(x, y, x + a.cos() * 5.0, y + a.sin() * 5.0, 0.9, c);
            }
            ui.circle(x, y, 1.2, c);
        }
        ItemKind::Mine => {
            ui.circle(x, y, 2.6, c);
            for k in 0..5 {
                let a = k as f32 * std::f32::consts::TAU / 5.0;
                ui.line(x, y, x + a.cos() * 5.2, y + a.sin() * 5.2, 0.7, c);
            }
        }
        ItemKind::Retaliate | ItemKind::DashBurst => {
            draw_circle_lines(ui.x(x), ui.y(y), ui.s(4.6), ui.s(0.8), soft);
            ui.line(x - 3.0, y + 3.0, x + 3.0, y - 3.0, 1.0, c);
        }
        ItemKind::Bolas => {
            ui.circle(x - 3.0, y + 2.0, 1.8, c);
            ui.circle(x + 3.0, y + 2.0, 1.8, c);
            ui.circle(x, y - 2.6, 1.8, soft);
        }
        ItemKind::Trail => {
            ui.circle(x - 3.0, y + 2.0, 1.7, soft);
            ui.circle(x, y, 2.0, c);
            ui.circle(x + 3.0, y - 2.0, 1.4, soft);
        }
        ItemKind::Aura => {
            draw_circle_lines(ui.x(x), ui.y(y), ui.s(4.8), ui.s(0.7), soft);
            draw_circle_lines(ui.x(x), ui.y(y), ui.s(2.8), ui.s(0.8), c);
        }
        ItemKind::Mend => {
            ui.line(x - 4.0, y, x + 4.0, y, 1.4, c);
            ui.line(x, y - 4.0, x, y + 4.0, 1.4, c);
        }
    }
}

/// Icono de un arma de la forja. La forma cambia por personaje y el índice
/// añade un detalle para distinguir las tres piezas.
fn draw_weapon_icon(ui: &Ui, class: Class, index: usize, x: f32, y: f32, color: Color, alpha: f32) {
    // Icono pixel art: escala entera de pantalla para que cada píxel cae limpio.
    let s = (ui.s(1.0) * 2.0).round().max(1.0);
    crate::pixel::draw_weapon_icon(class, index, ui.x(x), ui.y(y), color, alpha, s);
}

/// Icono de un contrato de la taquilla.
fn draw_ticket_icon(ui: &Ui, kind: usize, x: f32, y: f32, color: Color, alpha: f32) {
    let c = with_alpha(color, alpha);
    let soft = with_alpha(color, 0.35 * alpha);
    match kind {
        0 => {
            ui.circle(x, y, 5.2, soft);
            ui.line(x - 3.5, y + 3.5, x + 3.5, y - 3.5, 1.0, c);
        }
        1 => {
            ui.line(x - 3.6, y + 4.4, x + 3.2, y - 4.4, 1.5, c);
            ui.line(x - 4.0, y + 1.4, x - 1.4, y + 4.0, 1.1, soft);
            ui.circle(x + 3.2, y - 4.4, 1.0, with_alpha(C_WHITE, 0.85 * alpha));
        }
        2 => {
            ui.rect(x - 3.2, y - 4.2, 6.4, 8.4, c);
            ui.line(
                x - 3.2,
                y - 4.2,
                x + 3.2,
                y - 4.2,
                0.7,
                with_alpha(C_WHITE, 0.6 * alpha),
            );
            ui.circle(x, y, 1.4, with_alpha(C_WHITE, 0.9 * alpha));
            ui.line(x, y - 4.2, x, y - 6.2, 0.8, c);
        }
        3 => {
            ui.circle(x - 2.0, y - 2.4, 2.2, c);
            ui.circle(x + 2.0, y - 2.4, 2.2, c);
            ui.triangle(
                vec2(x - 4.2, y - 1.6),
                vec2(x + 4.2, y - 1.6),
                vec2(x, y + 5.0),
                c,
            );
        }
        _ => {
            ui.line(x - 4.0, y + 4.0, x + 4.0, y - 4.0, 1.2, c);
            ui.triangle(
                vec2(x + 5.0, y - 5.0),
                vec2(x + 2.2, y - 3.6),
                vec2(x + 3.8, y - 1.2),
                c,
            );
        }
    }
}

/// Describe el estilo de pelea del guardián para el cartel de presentación.
fn boss_intro_desc(attack: EnemyAttack, element: Element) -> String {
    let mech = match attack {
        EnemyAttack::Melee => "Embiste sin descanso",
        EnemyAttack::Bolt => "Dispara proyectiles precisos",
        EnemyAttack::Spread => "Abre abanicos de proyectiles",
        EnemyAttack::Homing => "Persigue con proyectiles",
        EnemyAttack::Charge => "Carga y arrolla",
        EnemyAttack::Bomb => "Siembra explosivos",
        EnemyAttack::SlowAura => "Ralentiza a quien se acerca",
        EnemyAttack::Summon => "Invoca secuaces",
        EnemyAttack::Teleport => "Se teletransporta y ataca",
        EnemyAttack::Orbit => "Rodea con órbitas letales",
        EnemyAttack::Burst => "Desata ráfagas circulares",
    };
    format!("{}  ·  {}", mech, element.adjective())
}

/// Emblema de destino de una puerta: solo forma y color, sin texto.
fn draw_door_emblem(kind: RoomKind, pos: Vec2, time: f32, alpha: f32) {
    let c = kind.color();
    let pulse = 0.82 + (time * 2.8 + pos.x * 0.05).sin() * 0.18;
    draw_circle(
        pos.x,
        pos.y,
        7.2,
        Color::new(0.02, 0.015, 0.03, 0.72 * alpha),
    );
    draw_circle_lines(pos.x, pos.y, 6.4, 0.7, with_alpha(c, 0.62 * alpha));
    draw_circle(pos.x, pos.y, 5.0 * pulse, with_alpha(c, 0.10 * alpha));
    match kind {
        RoomKind::Boss => {
            // Calavera.
            draw_circle(pos.x, pos.y - 0.4, 3.4, with_alpha(c, 0.95 * alpha));
            draw_circle(
                pos.x - 1.2,
                pos.y - 0.8,
                0.9,
                Color::new(0.03, 0.01, 0.02, alpha),
            );
            draw_circle(
                pos.x + 1.2,
                pos.y - 0.8,
                0.9,
                Color::new(0.03, 0.01, 0.02, alpha),
            );
            draw_rectangle(
                pos.x - 1.4,
                pos.y + 1.6,
                2.8,
                1.6,
                with_alpha(c, 0.95 * alpha),
            );
            draw_line(
                pos.x - 0.5,
                pos.y + 1.7,
                pos.x - 0.5,
                pos.y + 3.0,
                0.5,
                with_alpha(c, 0.8 * alpha),
            );
            draw_line(
                pos.x + 0.5,
                pos.y + 1.7,
                pos.x + 0.5,
                pos.y + 3.0,
                0.5,
                with_alpha(c, 0.8 * alpha),
            );
        }
        RoomKind::Shop => {
            // Moneda.
            draw_circle(pos.x, pos.y, 3.4, with_alpha(c, 0.95 * alpha));
            draw_circle_lines(pos.x, pos.y, 2.3, 0.6, Color::new(0.16, 0.11, 0.04, alpha));
            draw_circle(pos.x, pos.y, 0.9, Color::new(0.16, 0.11, 0.04, alpha));
        }
        RoomKind::Rest => {
            // Llama de campamento.
            draw_triangle(
                vec2(pos.x, pos.y - 3.6),
                vec2(pos.x - 2.8, pos.y + 2.6),
                vec2(pos.x + 2.8, pos.y + 2.6),
                with_alpha(c, 0.95 * alpha),
            );
            draw_triangle(
                vec2(pos.x, pos.y - 1.2),
                vec2(pos.x - 1.3, pos.y + 2.4),
                vec2(pos.x + 1.3, pos.y + 2.4),
                Color::new(1.0, 0.86, 0.52, alpha),
            );
        }
        RoomKind::Treasure => {
            // Rombo.
            draw_triangle(
                vec2(pos.x, pos.y - 4.0),
                vec2(pos.x - 3.4, pos.y),
                vec2(pos.x, pos.y + 4.0),
                with_alpha(c, 0.95 * alpha),
            );
            draw_triangle(
                vec2(pos.x, pos.y - 4.0),
                vec2(pos.x + 3.4, pos.y),
                vec2(pos.x, pos.y + 4.0),
                with_alpha(c, 0.60 * alpha),
            );
        }
        RoomKind::Vault => {
            // Cerradura.
            draw_circle_lines(pos.x, pos.y - 1.4, 1.8, 0.8, with_alpha(c, 0.95 * alpha));
            draw_rectangle(
                pos.x - 2.6,
                pos.y - 0.2,
                5.2,
                4.0,
                with_alpha(c, 0.95 * alpha),
            );
            draw_circle(pos.x, pos.y + 1.3, 0.8, Color::new(0.05, 0.02, 0.08, alpha));
        }
        RoomKind::Combat => {
            // Espadas cruzadas.
            draw_line(
                pos.x - 3.4,
                pos.y - 3.4,
                pos.x + 3.0,
                pos.y + 3.0,
                1.1,
                with_alpha(c, 0.90 * alpha),
            );
            draw_line(
                pos.x + 3.4,
                pos.y - 3.4,
                pos.x - 3.0,
                pos.y + 3.0,
                1.1,
                with_alpha(c, 0.90 * alpha),
            );
            draw_circle(pos.x, pos.y, 0.9, with_alpha(C_WHITE, 0.80 * alpha));
        }
        RoomKind::Start => {
            // Portal de entrada.
            draw_circle_lines(pos.x, pos.y, 3.6, 0.8, with_alpha(c, 0.90 * alpha));
            draw_circle(pos.x, pos.y, 1.6, with_alpha(c, 0.75 * alpha));
        }
    }
}

/// Texto centrado en un punto arbitrario de la pantalla.
fn ui_text_center_at(ui: &Ui, text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let w = ui.measure(text, size, false);
    ui.text(text, cx - w * 0.5, y, size, color);
}

fn ui_button(ui: &Ui, r: Rect, label: &str, accent: Color, m: Vec2) {
    let hover = r.contains(m);
    ui.rect(
        r.x,
        r.y,
        r.w,
        r.h,
        if hover {
            Color::new(0.14, 0.115, 0.17, 0.98)
        } else {
            Color::new(0.065, 0.055, 0.085, 0.96)
        },
    );
    ui.rect_lines(
        r.x,
        r.y,
        r.w,
        r.h,
        if hover { 1.5 } else { 0.9 },
        if hover {
            accent
        } else {
            with_alpha(accent, 0.60)
        },
    );
    let w = ui.measure(label, 6.4, true);
    ui.text_bold(
        label,
        r.x + (r.w - w) * 0.5,
        r.y + r.h * 0.5 + 2.4,
        6.4,
        if hover { accent } else { C_WHITE },
        true,
    );
}

/// Iconos propios para cada objeto de la tienda. No usan botones ni cajas:
/// cada mercancía tiene una silueta reconocible.
fn draw_shop_icon(ui: &Ui, _s: &Sprites, kind: ShopKind, c: Vec2, k: f32, dim: bool) {
    let a = if dim { 0.35 } else { 1.0 };
    match kind {
        ShopKind::Heal => {
            draw_ui_vial(ui, c, 5.2 * k, a);
        }
        ShopKind::Key => {
            draw_ui_key(ui, c, 5.2 * k, C_GOLD, a);
        }
        ShopKind::SpecialKey => {
            // Llave negra: asta oscura con dientes y un halo violeta.
            let col = with_alpha(Color::from_rgba(70, 44, 104, 255), a);
            let edge = with_alpha(C_PURPLE, a);
            ui.circle(
                c.x - 3.4 * k,
                c.y - 2.6 * k,
                3.2 * k,
                with_alpha(C_PURPLE, 0.18 * a),
            );
            draw_circle_lines(
                ui.x(c.x - 3.4 * k),
                ui.y(c.y - 2.6 * k),
                ui.s(2.6 * k),
                ui.s(0.9),
                edge,
            );
            ui.line(
                c.x - 1.4 * k,
                c.y - 1.6 * k,
                c.x + 4.6 * k,
                c.y + 3.4 * k,
                1.8 * k,
                col,
            );
            ui.line(
                c.x + 2.6 * k,
                c.y + 1.4 * k,
                c.x + 4.4 * k,
                c.y + 0.2 * k,
                1.4 * k,
                col,
            );
            ui.line(
                c.x + 4.0 * k,
                c.y + 2.8 * k,
                c.x + 5.6 * k,
                c.y + 1.6 * k,
                1.2 * k,
                col,
            );
        }
        ShopKind::Buff(_) => {
            let col = with_alpha(C_PURPLE, a);
            ui.circle(c.x, c.y, 8.0 * k, with_alpha(col, 0.20));
            ui.circle(c.x, c.y, 6.5 * k, with_alpha(col, 0.85));
            ui.circle(
                c.x,
                c.y,
                3.0 * k,
                with_alpha(Color::new(0.03, 0.02, 0.05, 1.0), a),
            );
            ui.line(
                c.x,
                c.y - 4.0 * k,
                c.x,
                c.y + 4.0 * k,
                1.2 * k,
                with_alpha(C_WHITE, a),
            );
        }
        ShopKind::Item(item) => {
            let col = with_alpha(item.rarity.color(), a);
            ui.circle(c.x, c.y, 9.0 * k, with_alpha(col, 0.16));
            match item.kind {
                ItemKind::Damage => {
                    ui.line(
                        c.x - 3.0 * k,
                        c.y + 6.0 * k,
                        c.x + 4.0 * k,
                        c.y - 5.0 * k,
                        1.8 * k,
                        col,
                    );
                    ui.line(
                        c.x - 1.0 * k,
                        c.y - 1.0 * k,
                        c.x + 2.0 * k,
                        c.y + 2.0 * k,
                        1.6 * k,
                        with_alpha(C_WHITE, a),
                    );
                    ui.line(
                        c.x - 4.0 * k,
                        c.y + 3.0 * k,
                        c.x - 1.0 * k,
                        c.y + 6.0 * k,
                        1.4 * k,
                        col,
                    );
                }
                ItemKind::MaxHp | ItemKind::CrystalHeart => {
                    // Sigilo de cristal, no un corazón.
                    let top = c.y - 7.0 * k;
                    let bot = c.y + 7.0 * k;
                    ui.triangle(
                        vec2(c.x, top),
                        vec2(c.x - 4.6 * k, c.y),
                        vec2(c.x, bot),
                        col,
                    );
                    ui.triangle(
                        vec2(c.x, top),
                        vec2(c.x + 4.6 * k, c.y),
                        vec2(c.x, bot),
                        with_alpha(col, 0.72),
                    );
                    ui.line(c.x, top, c.x, bot, 0.8 * k, with_alpha(C_WHITE, 0.55 * a));
                    ui.line(
                        c.x - 4.6 * k,
                        c.y,
                        c.x + 4.6 * k,
                        c.y,
                        0.7 * k,
                        with_alpha(C_WHITE, 0.35 * a),
                    );
                    if matches!(item.kind, ItemKind::CrystalHeart) {
                        ui.circle(
                            c.x + 5.2 * k,
                            c.y - 5.2 * k,
                            1.5 * k,
                            with_alpha(C_WHITE, a),
                        );
                        ui.circle(c.x - 5.0 * k, c.y + 4.4 * k, 1.0 * k, with_alpha(col, a));
                    }
                }
                ItemKind::Speed => {
                    ui.triangle(
                        vec2(c.x - 5.0 * k, c.y - 5.0 * k),
                        vec2(c.x + 1.0 * k, c.y),
                        vec2(c.x - 5.0 * k, c.y + 5.0 * k),
                        col,
                    );
                    ui.triangle(
                        vec2(c.x + 0.5 * k, c.y - 5.0 * k),
                        vec2(c.x + 6.5 * k, c.y),
                        vec2(c.x + 0.5 * k, c.y + 5.0 * k),
                        col,
                    );
                }
                ItemKind::Crit => {
                    ui.line(c.x, c.y - 7.0 * k, c.x, c.y + 7.0 * k, 1.4 * k, col);
                    ui.line(c.x - 7.0 * k, c.y, c.x + 7.0 * k, c.y, 1.4 * k, col);
                    ui.line(
                        c.x - 4.0 * k,
                        c.y - 4.0 * k,
                        c.x + 4.0 * k,
                        c.y + 4.0 * k,
                        1.0 * k,
                        col,
                    );
                    ui.line(
                        c.x + 4.0 * k,
                        c.y - 4.0 * k,
                        c.x - 4.0 * k,
                        c.y + 4.0 * k,
                        1.0 * k,
                        col,
                    );
                }
                ItemKind::Lifesteal => {
                    ui.circle(c.x, c.y + 2.0 * k, 4.0 * k, col);
                    ui.triangle(
                        vec2(c.x - 4.0 * k, c.y + 1.0 * k),
                        vec2(c.x + 4.0 * k, c.y + 1.0 * k),
                        vec2(c.x, c.y - 8.0 * k),
                        col,
                    );
                }
                ItemKind::Thorns | ItemKind::Armor => {
                    ui.rect(c.x - 5.0 * k, c.y - 6.0 * k, 10.0 * k, 8.0 * k, col);
                    ui.triangle(
                        vec2(c.x - 5.0 * k, c.y + 2.0 * k),
                        vec2(c.x + 5.0 * k, c.y + 2.0 * k),
                        vec2(c.x, c.y + 7.0 * k),
                        col,
                    );
                    if matches!(item.kind, ItemKind::Thorns) {
                        ui.line(c.x, c.y - 3.0 * k, c.x, c.y + 2.0 * k, 1.0 * k, C_WHITE);
                    }
                }
                ItemKind::SkeletonCap => {
                    ui.circle(c.x, c.y - 1.0 * k, 5.0 * k, col);
                    ui.rect(c.x - 3.5 * k, c.y + 1.0 * k, 7.0 * k, 4.0 * k, col);
                    ui.circle(
                        c.x - 2.0 * k,
                        c.y - 1.5 * k,
                        1.3 * k,
                        Color::new(0.03, 0.02, 0.05, 1.0),
                    );
                    ui.circle(
                        c.x + 2.0 * k,
                        c.y - 1.5 * k,
                        1.3 * k,
                        Color::new(0.03, 0.02, 0.05, 1.0),
                    );
                }
                ItemKind::Regen => {
                    ui.rect(c.x - 2.0 * k, c.y - 7.0 * k, 4.0 * k, 14.0 * k, col);
                    ui.rect(c.x - 7.0 * k, c.y - 2.0 * k, 14.0 * k, 4.0 * k, col);
                }
                ItemKind::AllStats => {
                    ui.triangle(
                        vec2(c.x, c.y - 8.0 * k),
                        vec2(c.x + 2.0 * k, c.y - 2.0 * k),
                        vec2(c.x - 2.0 * k, c.y - 2.0 * k),
                        col,
                    );
                    ui.triangle(
                        vec2(c.x, c.y + 8.0 * k),
                        vec2(c.x + 2.0 * k, c.y + 2.0 * k),
                        vec2(c.x - 2.0 * k, c.y + 2.0 * k),
                        col,
                    );
                    ui.triangle(
                        vec2(c.x - 8.0 * k, c.y),
                        vec2(c.x - 2.0 * k, c.y - 2.0 * k),
                        vec2(c.x - 2.0 * k, c.y + 2.0 * k),
                        col,
                    );
                    ui.triangle(
                        vec2(c.x + 8.0 * k, c.y),
                        vec2(c.x + 2.0 * k, c.y - 2.0 * k),
                        vec2(c.x + 2.0 * k, c.y + 2.0 * k),
                        col,
                    );
                }
                ItemKind::GlassCannon => {
                    ui.triangle(
                        vec2(c.x, c.y - 7.0 * k),
                        vec2(c.x + 6.0 * k, c.y + 5.0 * k),
                        vec2(c.x - 6.0 * k, c.y + 5.0 * k),
                        col,
                    );
                    ui.line(
                        c.x - 1.0 * k,
                        c.y - 4.0 * k,
                        c.x + 1.5 * k,
                        c.y,
                        1.0 * k,
                        Color::new(0.03, 0.02, 0.05, 1.0),
                    );
                    ui.line(
                        c.x + 1.5 * k,
                        c.y,
                        c.x - 1.5 * k,
                        c.y + 4.0 * k,
                        1.0 * k,
                        Color::new(0.03, 0.02, 0.05, 1.0),
                    );
                }
                ItemKind::Orbit => {
                    draw_circle_lines(ui.x(c.x), ui.y(c.y), ui.s(5.4 * k), ui.s(0.9), col);
                    ui.circle(c.x, c.y - 5.4 * k, 1.8 * k, col);
                    ui.circle(c.x, c.y, 1.3 * k, with_alpha(C_WHITE, a));
                }
                ItemKind::Homing => {
                    ui.circle(c.x, c.y, 6.0 * k, with_alpha(col, 0.22));
                    draw_circle_lines(ui.x(c.x), ui.y(c.y), ui.s(5.6 * k), ui.s(0.9), col);
                    ui.circle(c.x, c.y, 2.6 * k, col);
                    ui.circle(c.x, c.y, 1.1 * k, Color::new(0.03, 0.02, 0.05, a));
                }
                ItemKind::Chain => {
                    draw_circle_lines(
                        ui.x(c.x - 2.8 * k),
                        ui.y(c.y - 2.4 * k),
                        ui.s(3.0 * k),
                        ui.s(1.1),
                        col,
                    );
                    draw_circle_lines(
                        ui.x(c.x + 2.8 * k),
                        ui.y(c.y + 2.4 * k),
                        ui.s(3.0 * k),
                        ui.s(1.1),
                        col,
                    );
                }
                ItemKind::Nova => {
                    for i in 0..8 {
                        let ang = i as f32 * std::f32::consts::TAU / 8.0;
                        ui.line(
                            c.x + ang.cos() * 2.6 * k,
                            c.y + ang.sin() * 2.6 * k,
                            c.x + ang.cos() * 6.2 * k,
                            c.y + ang.sin() * 6.2 * k,
                            1.1 * k,
                            col,
                        );
                    }
                    ui.circle(c.x, c.y, 2.2 * k, with_alpha(C_WHITE, a));
                }
                ItemKind::Mine => {
                    ui.circle(c.x, c.y, 3.4 * k, col);
                    for i in 0..6 {
                        let ang = i as f32 * std::f32::consts::TAU / 6.0;
                        ui.line(
                            c.x + ang.cos() * 3.4 * k,
                            c.y + ang.sin() * 3.4 * k,
                            c.x + ang.cos() * 6.0 * k,
                            c.y + ang.sin() * 6.0 * k,
                            1.0 * k,
                            col,
                        );
                    }
                    ui.circle(c.x, c.y, 1.0 * k, with_alpha(C_WHITE, a));
                }
                ItemKind::Retaliate => {
                    ui.triangle(
                        vec2(c.x - 6.0 * k, c.y),
                        vec2(c.x - 1.0 * k, c.y - 5.0 * k),
                        vec2(c.x - 1.0 * k, c.y + 5.0 * k),
                        col,
                    );
                    ui.triangle(
                        vec2(c.x + 6.0 * k, c.y),
                        vec2(c.x + 1.0 * k, c.y - 5.0 * k),
                        vec2(c.x + 1.0 * k, c.y + 5.0 * k),
                        col,
                    );
                    ui.line(
                        c.x,
                        c.y - 4.0 * k,
                        c.x,
                        c.y + 4.0 * k,
                        0.9 * k,
                        with_alpha(C_WHITE, a),
                    );
                }
                ItemKind::DashBurst => {
                    ui.line(
                        c.x - 6.0 * k,
                        c.y + 4.0 * k,
                        c.x + 2.0 * k,
                        c.y - 4.0 * k,
                        1.8 * k,
                        col,
                    );
                    ui.line(
                        c.x + 2.0 * k,
                        c.y - 4.0 * k,
                        c.x + 1.0 * k,
                        c.y - 0.4 * k,
                        1.4 * k,
                        col,
                    );
                    ui.line(
                        c.x + 2.0 * k,
                        c.y - 4.0 * k,
                        c.x - 1.6 * k,
                        c.y - 3.0 * k,
                        1.4 * k,
                        col,
                    );
                    for i in 0..3 {
                        let o = i as f32 * 2.4 * k;
                        ui.line(
                            c.x - 6.0 * k + o,
                            c.y + 5.6 * k,
                            c.x - 3.0 * k + o,
                            c.y + 2.6 * k,
                            0.8 * k,
                            with_alpha(col, 0.7 * a),
                        );
                    }
                }
                ItemKind::Vortex => {
                    let spin = get_time() as f32 * 1.6;
                    for i in 0..3 {
                        let r = (2.2 + i as f32 * 2.0) * k;
                        for j in 0..6 {
                            let ang =
                                j as f32 * std::f32::consts::TAU / 6.0 + i as f32 * 0.55 + spin;
                            ui.circle(
                                c.x + ang.cos() * r,
                                c.y + ang.sin() * r,
                                0.8 * k,
                                with_alpha(col, (0.9 - i as f32 * 0.22) * a),
                            );
                        }
                    }
                    ui.circle(c.x, c.y, 1.2 * k, with_alpha(C_WHITE, a));
                }
                ItemKind::Bolas => {
                    for i in 0..3 {
                        let ang = i as f32 * std::f32::consts::TAU / 3.0 - 0.5;
                        let bx = c.x + ang.cos() * 4.2 * k;
                        let by = c.y + ang.sin() * 4.2 * k;
                        ui.circle(bx, by, 2.0 * k, col);
                        ui.circle(bx, by, 0.9 * k, with_alpha(C_WHITE, 0.8 * a));
                    }
                }
                ItemKind::Trail => {
                    for i in 0..4 {
                        let t = i as f32 / 3.0;
                        let x = c.x - 5.0 * k + t * 10.0 * k;
                        let y = c.y + 4.0 * k - (t * 4.0 * k).sin() * 0.6;
                        ui.circle(x, y, (1.8 - t * 0.7) * k, with_alpha(col, 0.85 * a));
                    }
                    ui.circle(
                        c.x + 4.0 * k,
                        c.y - 4.0 * k,
                        1.2 * k,
                        with_alpha(C_WHITE, a),
                    );
                }
                ItemKind::Aura => {
                    ui.circle(c.x, c.y, 6.4 * k, with_alpha(col, 0.10 * a));
                    for j in 0..10 {
                        let ang = j as f32 * std::f32::consts::TAU / 10.0;
                        ui.circle(
                            c.x + ang.cos() * 6.4 * k,
                            c.y + ang.sin() * 6.4 * k,
                            0.7 * k,
                            with_alpha(col, 0.9 * a),
                        );
                    }
                    ui.circle(c.x, c.y, 2.0 * k, with_alpha(col, 0.75 * a));
                    ui.circle(c.x, c.y, 0.9 * k, with_alpha(C_WHITE, a));
                }
                ItemKind::Mend => {
                    ui.rect(
                        c.x - 1.2 * k,
                        c.y - 5.0 * k,
                        2.4 * k,
                        10.0 * k,
                        with_alpha(col, a),
                    );
                    ui.rect(
                        c.x - 5.0 * k,
                        c.y - 1.2 * k,
                        10.0 * k,
                        2.4 * k,
                        with_alpha(col, a),
                    );
                    ui.circle(c.x, c.y, 2.0 * k, with_alpha(C_WHITE, 0.35 * a));
                }
            }
        }
    }
}

fn skin_color(i: u8) -> Color {
    match i {
        1 => Color::new(1.0, 0.42, 0.42, 1.0),
        2 => Color::new(0.46, 1.0, 0.62, 1.0),
        3 => Color::new(0.52, 0.74, 1.0, 1.0),
        4 => Color::new(0.84, 0.58, 1.0, 1.0),
        5 => Color::new(1.0, 0.85, 0.42, 1.0),
        6 => Color::new(0.72, 0.94, 1.0, 1.0),
        7 => Color::new(0.58, 0.52, 0.70, 1.0),
        _ => C_WHITE,
    }
}

fn draw_cosmetics_world(base: Vec2, scale: f32, hat: u8, mask: u8, accent: Color) {
    let s = scale;
    let hx = base.x;
    let hy = base.y - 4.8 * s;
    match hat {
        1 => {
            draw_rectangle(hx - 5.0 * s, hy - 3.0 * s, 10.0 * s, 1.6 * s, accent);
            draw_rectangle(hx - 3.4 * s, hy - 6.0 * s, 6.8 * s, 3.2 * s, accent);
        }
        2 => {
            draw_circle(hx, hy - 0.5 * s, 6.0 * s, with_alpha(accent, 0.40));
            draw_rectangle(
                hx - 5.0 * s,
                hy - 2.2 * s,
                10.0 * s,
                1.8 * s,
                with_alpha(accent, 0.85),
            );
        }
        3 => {
            draw_rectangle(
                hx - 5.2 * s,
                hy - 4.6 * s,
                10.4 * s,
                3.2 * s,
                Color::from_rgba(176, 184, 198, 255),
            );
            draw_rectangle(
                hx - 5.2 * s,
                hy - 2.2 * s,
                10.4 * s,
                1.2 * s,
                Color::from_rgba(96, 104, 118, 255),
            );
        }
        4 => {
            draw_rectangle(hx - 4.2 * s, hy - 4.8 * s, 8.4 * s, 2.0 * s, C_GOLD);
            for k in 0..3 {
                let bx = hx - 4.0 * s + k as f32 * 4.0 * s;
                draw_triangle(
                    vec2(bx, hy - 4.8 * s),
                    vec2(bx + 2.0 * s, hy - 4.8 * s),
                    vec2(bx + 1.0 * s, hy - 7.6 * s),
                    C_GOLD,
                );
            }
        }
        5 => {
            for side in [-1.0f32, 1.0] {
                let bx = hx + side * 3.6 * s;
                draw_triangle(
                    vec2(bx, hy - 3.4 * s),
                    vec2(bx + side * 2.6 * s, hy - 8.4 * s),
                    vec2(bx + side * 1.2 * s, hy - 3.4 * s),
                    Color::from_rgba(236, 230, 214, 255),
                );
            }
        }
        6 => {
            draw_ellipse(
                hx,
                hy - 7.4 * s,
                5.2 * s,
                1.8 * s,
                0.0,
                with_alpha(C_GOLD, 0.85),
            );
        }
        7 => {
            draw_triangle(
                vec2(hx - 5.0 * s, hy - 3.2 * s),
                vec2(hx + 5.0 * s, hy - 3.2 * s),
                vec2(hx, hy - 12.0 * s),
                Color::from_rgba(118, 82, 176, 255),
            );
            draw_rectangle(
                hx - 5.6 * s,
                hy - 3.6 * s,
                11.2 * s,
                1.4 * s,
                Color::from_rgba(88, 58, 136, 255),
            );
        }
        _ => {}
    }
    match mask {
        1 => {
            draw_rectangle(
                hx - 4.2 * s,
                hy - 1.2 * s,
                8.4 * s,
                2.0 * s,
                Color::from_rgba(24, 20, 32, 255),
            );
        }
        2 => {
            draw_rectangle(
                hx - 4.6 * s,
                hy + 1.4 * s,
                9.2 * s,
                2.6 * s,
                Color::from_rgba(176, 54, 64, 255),
            );
        }
        3 => {
            draw_circle(
                hx,
                hy + 0.8 * s,
                3.2 * s,
                Color::from_rgba(238, 236, 228, 255),
            );
            draw_circle(
                hx - 1.2 * s,
                hy + 0.4 * s,
                0.7 * s,
                Color::from_rgba(20, 18, 26, 255),
            );
            draw_circle(
                hx + 1.2 * s,
                hy + 0.4 * s,
                0.7 * s,
                Color::from_rgba(20, 18, 26, 255),
            );
        }
        4 => {
            draw_rectangle(
                hx - 4.6 * s,
                hy - 1.2 * s,
                9.2 * s,
                1.4 * s,
                with_alpha(C_CYAN, 0.90),
            );
        }
        5 => {
            draw_line(
                hx - 3.0 * s,
                hy - 3.0 * s,
                hx + 1.0 * s,
                hy + 2.6 * s,
                0.8 * s,
                Color::from_rgba(196, 64, 72, 255),
            );
        }
        6 => {
            draw_rectangle(
                hx - 3.8 * s,
                hy + 0.6 * s,
                2.4 * s,
                1.2 * s,
                Color::from_rgba(196, 64, 72, 255),
            );
            draw_rectangle(
                hx + 1.4 * s,
                hy + 0.6 * s,
                2.4 * s,
                1.2 * s,
                Color::from_rgba(196, 64, 72, 255),
            );
        }
        7 => {
            draw_rectangle(
                hx - 4.6 * s,
                hy - 2.4 * s,
                9.2 * s,
                6.4 * s,
                Color::new(0.72, 0.52, 0.94, 0.30),
            );
        }
        _ => {}
    }
}

fn ui_brighten(c: Color, k: f32) -> Color {
    Color::new(
        (c.r * k).min(1.0),
        (c.g * k).min(1.0),
        (c.b * k).min(1.0),
        c.a,
    )
}

/// Moneda dibujada con formas suaves para el HUD y las tiendas.
fn draw_ui_coin(ui: &Ui, pos: Vec2, r: f32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    ui.ellipse(
        pos.x,
        pos.y + r * 0.35,
        r * 0.95,
        r * 0.34,
        Color::new(0.0, 0.0, 0.0, 0.28 * a),
    );
    ui.ellipse(pos.x, pos.y, r * 0.92, r, with_alpha(C_GOLD, a));
    ui.ellipse(
        pos.x,
        pos.y,
        r * 0.70,
        r * 0.80,
        with_alpha(ui_brighten(C_GOLD, 1.16), a),
    );
    ui.ellipse(
        pos.x - r * 0.24,
        pos.y - r * 0.28,
        r * 0.18,
        r * 0.24,
        with_alpha(C_WHITE, 0.72 * a),
    );
}

/// Gema facetada del color de las almas.
fn draw_ui_gem(ui: &Ui, pos: Vec2, r: f32, tint: Color, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let top = vec2(pos.x, pos.y - r);
    let left = vec2(pos.x - r * 0.85, pos.y - r * 0.10);
    let right = vec2(pos.x + r * 0.85, pos.y - r * 0.10);
    let bottom = vec2(pos.x, pos.y + r);
    ui.triangle(top, left, bottom, with_alpha(tint, a));
    ui.triangle(top, bottom, right, with_alpha(ui_brighten(tint, 1.18), a));
    ui.triangle(top, left, right, with_alpha(ui_brighten(tint, 1.36), a));
    ui.line(
        top.x,
        top.y,
        left.x,
        left.y,
        0.5,
        with_alpha(C_WHITE, 0.55 * a),
    );
    ui.line(
        top.x,
        top.y,
        right.x,
        right.y,
        0.5,
        with_alpha(C_WHITE, 0.30 * a),
    );
    ui.circle(
        pos.x - r * 0.25,
        pos.y - r * 0.18,
        r * 0.13,
        with_alpha(C_WHITE, 0.85 * a),
    );
}

/// Llave de hierro con anilla y dientes.
fn draw_ui_key(ui: &Ui, pos: Vec2, r: f32, tint: Color, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let c = with_alpha(tint, a);
    draw_circle_lines(
        ui.x(pos.x - r * 0.35),
        ui.y(pos.y - r * 0.35),
        ui.s(r * 0.52),
        ui.s(0.9),
        c,
    );
    ui.line(
        pos.x + r * 0.05,
        pos.y - r * 0.05,
        pos.x + r * 0.85,
        pos.y + r * 0.75,
        1.1,
        c,
    );
    ui.line(
        pos.x + r * 0.44,
        pos.y + r * 0.54,
        pos.x + r * 0.70,
        pos.y + r * 0.28,
        0.9,
        c,
    );
    ui.line(
        pos.x + r * 0.68,
        pos.y + r * 0.78,
        pos.x + r * 0.94,
        pos.y + r * 0.52,
        0.9,
        c,
    );
}

/// Frasco de curación con vidrio, líquido y corcho.
fn draw_ui_vial(ui: &Ui, pos: Vec2, r: f32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let glass = with_alpha(Color::new(0.80, 0.88, 0.96, 1.0), 0.38 * a);
    ui.rect(
        pos.x - r * 0.45,
        pos.y - r * 0.35,
        r * 0.90,
        r * 1.05,
        glass,
    );
    ui.rect(
        pos.x - r * 0.34,
        pos.y + r * 0.05,
        r * 0.68,
        r * 0.55,
        with_alpha(C_RED, a),
    );
    ui.rect(
        pos.x - r * 0.34,
        pos.y + r * 0.05,
        r * 0.68,
        r * 0.16,
        with_alpha(ui_brighten(C_RED, 1.35), a),
    );
    ui.rect(
        pos.x - r * 0.22,
        pos.y - r * 0.75,
        r * 0.44,
        r * 0.42,
        glass,
    );
    ui.rect(
        pos.x - r * 0.30,
        pos.y - r * 0.95,
        r * 0.60,
        r * 0.24,
        with_alpha(Color::new(0.46, 0.30, 0.16, 1.0), a),
    );
    ui.rect_lines(
        pos.x - r * 0.45,
        pos.y - r * 0.35,
        r * 0.90,
        r * 1.05,
        0.5,
        with_alpha(C_WHITE, 0.45 * a),
    );
    ui.line(
        pos.x - r * 0.28,
        pos.y - r * 0.20,
        pos.x - r * 0.28,
        pos.y + r * 0.35,
        0.6,
        with_alpha(C_WHITE, 0.60 * a),
    );
}

/// Dibuja un héroe procedural recortado dentro de la interfaz.
///
/// Se usa una cámara con el mismo encuadre que la interfaz y un viewport
/// 16:9 del tamaño pedido. El héroe se dibuja en coordenadas de mundo, así
/// que sombreros y máscaras quedan alineados con la cabeza sin trucos.
fn draw_hero_preview(
    ui: &Ui,
    class: Class,
    center: Vec2,
    height: f32,
    tint: Color,
    hat: u8,
    mask: u8,
    time: f32,
) {
    // 46 unidades de alto: el héroe pixel art mide 26 y el sombrero puede
    // llegar a 32; queda margen arriba y abajo.
    let vp_h = ui.s(height).max(1.0);
    let vp_w = vp_h * (VW / VH);
    // Escala entera: cada píxel del sprite cae en un bloque exacto de píxeles
    // de pantalla, sin bordes de distinto grosor.
    let px = (vp_h / 46.0).round().max(1.0);
    let view_w = vp_w / px;
    let view_h = vp_h / px;
    let mut cam = Camera2D::from_display_rect(Rect::new(
        center.x - view_w * 0.5,
        center.y - view_h * 0.5,
        view_w,
        view_h,
    ));
    // La cámara directa a pantalla invierte el eje Y; lo devolvemos al
    // convenio del juego (Y crece hacia abajo).
    cam.zoom.y = -cam.zoom.y;
    cam.viewport = Some((
        (ui.x(center.x) - vp_w * 0.5).round() as i32,
        (screen_height() - (ui.y(center.y) + vp_h * 0.5)).round() as i32,
        vp_w.round().max(1.0) as i32,
        vp_h.round().max(1.0) as i32,
    ));
    set_camera(&cam);
    // Sombra de contacto en bloques de píxel, a juego con el sprite.
    let fx = center.x.round();
    let fy = (center.y + 11.0).round();
    for (dy, w, a) in [
        (1.0, 10.0, 0.34),
        (2.0, 16.0, 0.26),
        (3.0, 12.0, 0.18),
        (4.0, 6.0, 0.10),
    ] {
        draw_rectangle(fx - w * 0.5, fy + dy, w, 1.0, Color::new(0.0, 0.0, 0.0, a));
    }
    let pos = vec2(center.x, center.y + 11.0);
    figures::draw_hero(
        pos,
        class,
        vec2(1.0, 0.0),
        time * 1.6,
        false,
        0.0,
        0.0,
        tint,
        hat,
        mask,
        time,
    );
    set_default_camera();
}

#[allow(dead_code)]
fn draw_cosmetics_ui(ui: &Ui, center: Vec2, scale: f32, hat: u8, mask: u8, accent: Color) {
    let s = scale;
    let hx = center.x;
    let hy = center.y - 4.8 * s;
    match hat {
        1 => {
            ui.rect(hx - 5.0 * s, hy - 3.0 * s, 10.0 * s, 1.6 * s, accent);
            ui.rect(hx - 3.4 * s, hy - 6.0 * s, 6.8 * s, 3.2 * s, accent);
        }
        2 => {
            ui.circle(hx, hy - 0.5 * s, 6.0 * s, with_alpha(accent, 0.40));
            ui.rect(
                hx - 5.0 * s,
                hy - 2.2 * s,
                10.0 * s,
                1.8 * s,
                with_alpha(accent, 0.85),
            );
        }
        3 => {
            ui.rect(
                hx - 5.2 * s,
                hy - 4.6 * s,
                10.4 * s,
                3.2 * s,
                Color::from_rgba(176, 184, 198, 255),
            );
            ui.rect(
                hx - 5.2 * s,
                hy - 2.2 * s,
                10.4 * s,
                1.2 * s,
                Color::from_rgba(96, 104, 118, 255),
            );
        }
        4 => {
            ui.rect(hx - 4.2 * s, hy - 4.8 * s, 8.4 * s, 2.0 * s, C_GOLD);
            for k in 0..3 {
                let bx = hx - 4.0 * s + k as f32 * 4.0 * s;
                ui.triangle(
                    vec2(bx, hy - 4.8 * s),
                    vec2(bx + 2.0 * s, hy - 4.8 * s),
                    vec2(bx + 1.0 * s, hy - 7.6 * s),
                    C_GOLD,
                );
            }
        }
        5 => {
            for side in [-1.0f32, 1.0] {
                let bx = hx + side * 3.6 * s;
                ui.triangle(
                    vec2(bx, hy - 3.4 * s),
                    vec2(bx + side * 2.6 * s, hy - 8.4 * s),
                    vec2(bx + side * 1.2 * s, hy - 3.4 * s),
                    Color::from_rgba(236, 230, 214, 255),
                );
            }
        }
        6 => {
            ui.ellipse(hx, hy - 7.4 * s, 5.2 * s, 1.8 * s, with_alpha(C_GOLD, 0.85));
        }
        7 => {
            ui.triangle(
                vec2(hx - 5.0 * s, hy - 3.2 * s),
                vec2(hx + 5.0 * s, hy - 3.2 * s),
                vec2(hx, hy - 12.0 * s),
                Color::from_rgba(118, 82, 176, 255),
            );
            ui.rect(
                hx - 5.6 * s,
                hy - 3.6 * s,
                11.2 * s,
                1.4 * s,
                Color::from_rgba(88, 58, 136, 255),
            );
        }
        _ => {}
    }
    match mask {
        1 => {
            ui.rect(
                hx - 4.2 * s,
                hy - 1.2 * s,
                8.4 * s,
                2.0 * s,
                Color::from_rgba(24, 20, 32, 255),
            );
        }
        2 => {
            ui.rect(
                hx - 4.6 * s,
                hy + 1.4 * s,
                9.2 * s,
                2.6 * s,
                Color::from_rgba(176, 54, 64, 255),
            );
        }
        3 => {
            ui.circle(
                hx,
                hy + 0.8 * s,
                3.2 * s,
                Color::from_rgba(238, 236, 228, 255),
            );
            ui.circle(
                hx - 1.2 * s,
                hy + 0.4 * s,
                0.7 * s,
                Color::from_rgba(20, 18, 26, 255),
            );
            ui.circle(
                hx + 1.2 * s,
                hy + 0.4 * s,
                0.7 * s,
                Color::from_rgba(20, 18, 26, 255),
            );
        }
        4 => {
            ui.rect(
                hx - 4.6 * s,
                hy - 1.2 * s,
                9.2 * s,
                1.4 * s,
                with_alpha(C_CYAN, 0.90),
            );
        }
        5 => {
            ui.line(
                hx - 3.0 * s,
                hy - 3.0 * s,
                hx + 1.0 * s,
                hy + 2.6 * s,
                0.8 * s,
                Color::from_rgba(196, 64, 72, 255),
            );
        }
        6 => {
            ui.rect(
                hx - 3.8 * s,
                hy + 0.6 * s,
                2.4 * s,
                1.2 * s,
                Color::from_rgba(196, 64, 72, 255),
            );
            ui.rect(
                hx + 1.4 * s,
                hy + 0.6 * s,
                2.4 * s,
                1.2 * s,
                Color::from_rgba(196, 64, 72, 255),
            );
        }
        7 => {
            ui.rect(
                hx - 4.6 * s,
                hy - 2.4 * s,
                9.2 * s,
                6.4 * s,
                Color::new(0.72, 0.52, 0.94, 0.30),
            );
        }
        _ => {}
    }
}

const ITEMS: [(&str, ItemKind, f32); 24] = [
    ("Filo oxidado", ItemKind::Damage, 2.5),
    ("Sello de piedra", ItemKind::MaxHp, 12.0),
    ("Botas gastadas", ItemKind::Speed, 6.0),
    ("Daga afilada", ItemKind::Crit, 0.04),
    ("Colmillo vampírico", ItemKind::Lifesteal, 0.02),
    ("Coraza de espinas", ItemKind::Thorns, 0.12),
    ("Placas reforzadas", ItemKind::Armor, 1.5),
    ("Cetro de hueso", ItemKind::SkeletonCap, 1.0),
    ("Amuleto vital", ItemKind::Regen, 0.4),
    ("Reliquia del abismo", ItemKind::AllStats, 0.06),
    ("Núcleo de cristal", ItemKind::CrystalHeart, 18.0),
    ("Esencia violenta", ItemKind::GlassCannon, 0.15),
    ("Sigilo orbital", ItemKind::Orbit, 1.0),
    ("Ojo imán", ItemKind::Homing, 1.2),
    ("Cadena de huesos", ItemKind::Chain, 1.0),
    ("Nova de cristal", ItemKind::Nova, 1.0),
    ("Mina ritual", ItemKind::Mine, 1.0),
    ("Espejo de dolor", ItemKind::Retaliate, 1.0),
    ("Impulso astral", ItemKind::DashBurst, 1.0),
    ("Ojo del remolino", ItemKind::Vortex, 1.0),
    ("Bolas de hierro", ItemKind::Bolas, 1.0),
    ("Estela de brasas", ItemKind::Trail, 1.0),
    ("Aura de ceniza", ItemKind::Aura, 1.0),
    ("Aguja de costura", ItemKind::Mend, 1.0),
];

/// Biblioteca de ataques. Son los objetos activos que cambian el combate.
const LIBRARY_ATTACKS: [(&str, &str); 12] = [
    (
        "Sigilo orbital",
        "Un sigilo gira a tu alrededor y corta a todo enemigo que toque.",
    ),
    (
        "Ojo imán",
        "Tus proyectiles corrigen la trayectoria y persiguen al objetivo.",
    ),
    (
        "Cadena de huesos",
        "Cada impacto salta a un enemigo cercano con daño reducido.",
    ),
    (
        "Nova de cristal",
        "Cada cinco ataques estalla una onda de cristal a tu alrededor.",
    ),
    (
        "Mina ritual",
        "Al atacar dejas minas que detonan cuando el enemigo se acerca.",
    ),
    (
        "Espejo de dolor",
        "Cuando recibes daño respondes con una ráfaga de represalia.",
    ),
    (
        "Impulso astral",
        "Tu esquiva termina en una explosión que empuja y daña.",
    ),
    (
        "Ojo del remolino",
        "Invoca remolinos que arrastran a los enemigos hacia su centro.",
    ),
    (
        "Bolas de hierro",
        "Lanzas bolas pesadas que ralentizan y perforan la línea enemiga.",
    ),
    (
        "Estela de brasas",
        "Dejas un rastro de brasas que quema a quien lo pisa.",
    ),
    (
        "Aura de ceniza",
        "Un aura constante castiga a todo enemigo que se acerque.",
    ),
    (
        "Aguja de costura",
        "Pulsos de costura restauran tu vida con el tiempo.",
    ),
];

const LIBRARY_BOSSES: [(&str, &str); 10] = [
    (
        "Guardián",
        "Embiste en línea recta y barre la arena con golpes amplios.",
    ),
    (
        "Señor",
        "Invoca súbditos y castiga con ráfagas de proyectiles.",
    ),
    (
        "Custodio",
        "Se protege con runas y contraataca cuando te acercas.",
    ),
    (
        "Verdugo",
        "Salta sobre ti y deja ondas de choque al aterrizar.",
    ),
    (
        "Devorador",
        "Abre fauces que atraen y muerden en un radio amplio.",
    ),
    (
        "Heresiarca",
        "Dibuja círculos rituales que estallan en llamas.",
    ),
    (
        "Coloso",
        "Lento pero brutal: cada pisotón sacude la cámara.",
    ),
    (
        "Oráculo",
        "Predice tu posición y lanza haces de luz desde los bordes.",
    ),
    (
        "Tirano",
        "Encadena cargas cortas y golpes de área sin descanso.",
    ),
    (
        "Abismo",
        "Distorsiona el espacio con proyectiles lentos que persiguen.",
    ),
];

const LIBRARY_ENEMIES: [(&str, &str); 8] = [
    (
        "Limo",
        "Lento, blando y persistente. Se multiplica si lo dejas crecer.",
    ),
    (
        "Murciélago",
        "Rápido y errático; ataca en picado y vuelve a huir.",
    ),
    (
        "Bruto",
        "Mucha vida y golpes pesados; esquiva su embestida frontal.",
    ),
    ("Cultista", "Lanza proyectiles rúnicos desde la distancia."),
    (
        "Espectro",
        "Atraviesa obstáculos y se desvanece tras golpear.",
    ),
    (
        "Saboteador",
        "Coloca explosivos y te obliga a moverte sin parar.",
    ),
    (
        "Centinela",
        "Dispara en ráfagas y protege a los enemigos cercanos.",
    ),
    (
        "Ojo",
        "Te observa y lanza un rayo continuo si te quedas quieto.",
    ),
];

const MENU_QUOTES: [&str; 8] = [
    "La cripta no perdona la prisa.",
    "Cada descenso cobra su precio.",
    "No hay luz al final, solo más profundidad.",
    "El oro brilla; el peso te recuerda que es una cadena.",
    "Los muertos no huyen.",
    "Baja. Mata. Repite.",
    "La suerte favorece al que ya está muerto.",
    "No confíes en la puerta que se abre sola.",
];

const MENU_TIPS: [&str; 10] = [
    "El oro pesa: si llevas demasiado, esquiva más y corre menos.",
    "Las puertas se cierran al entrar en combate. Limpia la sala para abrirlas.",
    "Las llaves negras solo abren bóvedas. No las malgastes en puertas normales.",
    "Los guardianes tienen patrones. Aprende el ritmo antes de atacar.",
    "El daño en área es menor que el directo: úsalo para grupos, no para jefes.",
    "La tienda de almas premia la constancia: mejora una estadística a la vez.",
    "Cambia las opciones de mejora si ninguna encaja con tu build.",
    "Los élites sueltan más oro, pero también pegan más fuerte.",
    "En las salas de descanso puedes firmar pactos peligrosos.",
    "No todas las puertas se abren con la misma llave.",
];

fn library_len(tab: usize) -> usize {
    match tab {
        0 => LIBRARY_ATTACKS.len(),
        1 => LIBRARY_BOSSES.len(),
        2 => LIBRARY_ENEMIES.len(),
        _ => buffs::POOL.len(),
    }
}

fn library_pages(tab: usize) -> usize {
    ((library_len(tab) + 5) / 6).max(1)
}

const KEY_TABLE: &[(u16, KeyCode, &str)] = &[
    (0x41, KeyCode::A, "A"),
    (0x42, KeyCode::B, "B"),
    (0x43, KeyCode::C, "C"),
    (0x44, KeyCode::D, "D"),
    (0x45, KeyCode::E, "E"),
    (0x46, KeyCode::F, "F"),
    (0x47, KeyCode::G, "G"),
    (0x48, KeyCode::H, "H"),
    (0x49, KeyCode::I, "I"),
    (0x4a, KeyCode::J, "J"),
    (0x4b, KeyCode::K, "K"),
    (0x4c, KeyCode::L, "L"),
    (0x4d, KeyCode::M, "M"),
    (0x4e, KeyCode::N, "N"),
    (0x4f, KeyCode::O, "O"),
    (0x50, KeyCode::P, "P"),
    (0x51, KeyCode::Q, "Q"),
    (0x52, KeyCode::R, "R"),
    (0x53, KeyCode::S, "S"),
    (0x54, KeyCode::T, "T"),
    (0x55, KeyCode::U, "U"),
    (0x56, KeyCode::V, "V"),
    (0x57, KeyCode::W, "W"),
    (0x58, KeyCode::X, "X"),
    (0x59, KeyCode::Y, "Y"),
    (0x5a, KeyCode::Z, "Z"),
    (0x30, KeyCode::Key0, "0"),
    (0x31, KeyCode::Key1, "1"),
    (0x32, KeyCode::Key2, "2"),
    (0x33, KeyCode::Key3, "3"),
    (0x34, KeyCode::Key4, "4"),
    (0x35, KeyCode::Key5, "5"),
    (0x36, KeyCode::Key6, "6"),
    (0x37, KeyCode::Key7, "7"),
    (0x38, KeyCode::Key8, "8"),
    (0x39, KeyCode::Key9, "9"),
    (0x20, KeyCode::Space, "Espacio"),
    (0xff09, KeyCode::Tab, "Tab"),
    (0xff1b, KeyCode::Escape, "Esc"),
    (0xff0d, KeyCode::Enter, "Enter"),
    (0xff51, KeyCode::Left, "Izquierda"),
    (0xff52, KeyCode::Up, "Arriba"),
    (0xff53, KeyCode::Right, "Derecha"),
    (0xff54, KeyCode::Down, "Abajo"),
    (0xffe1, KeyCode::LeftShift, "Shift izq."),
    (0xffe2, KeyCode::RightShift, "Shift der."),
    (0xffe3, KeyCode::LeftControl, "Ctrl izq."),
    (0xffe4, KeyCode::RightControl, "Ctrl der."),
    (0xffe9, KeyCode::LeftAlt, "Alt izq."),
    (0xffea, KeyCode::RightAlt, "Alt der."),
    (0x2c, KeyCode::Comma, "Coma"),
    (0x2e, KeyCode::Period, "Punto"),
    (0x2d, KeyCode::Minus, "Guion"),
    (0x3d, KeyCode::Equal, "Igual"),
    (0x3b, KeyCode::Semicolon, "Punto y coma"),
    (0x2f, KeyCode::Slash, "Barra"),
];

fn key_from_code(code: u16) -> Option<KeyCode> {
    KEY_TABLE
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, k, _)| *k)
}

fn key_label(code: u16) -> &'static str {
    KEY_TABLE
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, _, l)| *l)
        .unwrap_or("?")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_binds_have_labels_and_keycodes() {
        let binds = save::KeyBinds::default();
        for (action, code) in binds.codes.iter().enumerate() {
            assert!(
                key_from_code(*code).is_some(),
                "la acción {} usa un código sin KeyCode: {:#x}",
                save::KEY_ACTION_NAMES[action],
                code
            );
            assert_ne!(
                key_label(*code),
                "?",
                "la acción {} no tiene etiqueta legible",
                save::KEY_ACTION_NAMES[action]
            );
        }
        assert_eq!(key_label(0xff09), "Tab");
    }

    fn connected(g: &Game) -> bool {
        let start = g.room_index;
        let mut seen = vec![false; g.rooms.len()];
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(i) = stack.pop() {
            let (gx, gy) = (g.rooms[i].gx, g.rooms[i].gy);
            for (dir, (nx, ny)) in [(gx, gy - 1), (gx + 1, gy), (gx, gy + 1), (gx - 1, gy)]
                .iter()
                .enumerate()
            {
                if g.rooms[i].doors[dir] {
                    if let Some(j) = g.room_at(*nx, *ny) {
                        if !seen[j] {
                            seen[j] = true;
                            stack.push(j);
                        }
                    }
                }
            }
        }
        seen.iter().all(|&s| s)
    }

    #[test]
    fn every_class_starts_a_run() {
        for class in Class::ALL {
            let mut g = Game::new(Meta::default());
            g.start_run(class);
            assert_eq!(g.floor, 1);
            assert!(matches!(g.screen, Screen::Playing));
            assert!(
                (8..=15).contains(&g.rooms.len()),
                "la mazmorra debe tener entre 8 y 15 salas"
            );
            assert_eq!(g.current_kind(), RoomKind::Start);
            assert!(g.player.hp > 0.0);
            assert!(g.enemies.is_empty(), "la entrada no tiene enemigos");
            assert_eq!(
                g.equipped_attacks.len(),
                1,
                "cada clase empieza con un ataque propio"
            );
            let starter = &g.attacks[g.equipped_attacks[0].def];
            assert_eq!(starter.class, class);
            if class == Class::Necro {
                assert_eq!(g.skeletons.len(), g.player.skeleton_cap as usize);
            } else {
                assert!(g.skeletons.is_empty());
            }
            match class {
                Class::Cleric => assert!(g.player.regen >= 0.45),
                Class::Merchant => {
                    assert!(g.player.gold_mult >= 1.25);
                    assert!(g.player.luck >= 0.10);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn forged_weapons_change_the_starting_player() {
        for class in Class::ALL {
            let mut meta = Meta::default();
            let base = Player::new(class, &meta);
            meta.unlock_weapon(class, 1);
            meta.set_weapon(class, 1);
            let armed = Player::new(class, &meta);
            let changed = armed.damage != base.damage
                || armed.max_hp != base.max_hp
                || armed.speed != base.speed
                || armed.attack_speed != base.attack_speed
                || armed.crit != base.crit
                || armed.armor != base.armor
                || armed.gold_mult != base.gold_mult
                || armed.luck != base.luck
                || armed.aoe != base.aoe
                || armed.regen != base.regen
                || armed.skeleton_cap != base.skeleton_cap
                || armed.skeleton_dmg != base.skeleton_dmg;
            assert!(
                changed,
                "el arma 1 de {:?} no cambió ninguna estadística",
                class
            );
        }
    }

    #[test]
    fn every_floor_has_the_required_rooms() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        for _ in 0..12 {
            let boss = g.rooms.iter().filter(|r| r.kind == RoomKind::Boss).count();
            let shop = g.rooms.iter().filter(|r| r.kind == RoomKind::Shop).count();
            let rest = g.rooms.iter().filter(|r| r.kind == RoomKind::Rest).count();
            let treasure = g
                .rooms
                .iter()
                .filter(|r| r.kind == RoomKind::Treasure)
                .count();
            assert_eq!(boss, 1);
            assert_eq!(shop, 1);
            assert_eq!(rest, 1);
            assert_eq!(treasure, 1);
            let chests = g.rooms.iter().filter(|r| r.has_chest).count();
            assert_eq!(chests, 3, "cada piso tiene tres cofres con llave");
            assert!(g.rooms.iter().any(|r| r.locked.iter().any(|&l| l)));
            let boss_room = g
                .rooms
                .iter()
                .find(|r| r.kind == RoomKind::Boss)
                .expect("debe haber guardián");
            assert_eq!(boss_room.size, RoomSize::Giant);
            let rest_room = g
                .rooms
                .iter()
                .find(|r| r.kind == RoomKind::Rest)
                .expect("debe haber descanso");
            assert_eq!(rest_room.size, RoomSize::Small);
            assert!(connected(&g), "todas las salas deben estar conectadas");
            g.next_floor();
        }
    }

    #[test]
    fn a_key_opens_a_locked_room() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let mut setup = None;
        for (target, room) in g.rooms.iter().enumerate() {
            // La bóveda usa llave negra; esta prueba cubre las puertas selladas
            // normales del guardián y del tesoro.
            if room.kind == RoomKind::Vault {
                continue;
            }
            for dir in 0..4 {
                if !room.locked[dir] {
                    continue;
                }
                let (nx, ny, back) = match dir {
                    0 => (room.gx, room.gy - 1, 2),
                    1 => (room.gx + 1, room.gy, 3),
                    2 => (room.gx, room.gy + 1, 0),
                    _ => (room.gx - 1, room.gy, 1),
                };
                if let Some(from) = g.room_at(nx, ny) {
                    setup = Some((target, from, back));
                    break;
                }
            }
            if setup.is_some() {
                break;
            }
        }
        let (target, from, back) = setup.expect("debe haber una puerta sellada accesible");
        g.room_index = from;
        g.rooms[from].visited = true;
        g.rooms[from].cleared = true;
        g.cleared = true;
        g.player.pos = g.door_pos(back);
        let mut sfx = Sfx::silent();

        // Sin llave no se cruza.
        g.keys = 0;
        assert!(!g.try_use_door(back, &mut sfx));
        assert_eq!(g.room_index, from);

        // Con llave se abre la sala entera y se consume una.
        g.keys = 1;
        g.player.pos = g.door_pos(back);
        assert!(g.try_use_door(back, &mut sfx));
        assert_eq!(g.room_index, target);
        assert_eq!(g.keys, 0);
        assert!(!g.rooms[target].locked.iter().any(|&l| l));
    }

    #[test]
    fn resting_heals_only_once() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let rest = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Rest)
            .expect("el piso debe tener sala de descanso");
        g.player.max_hp = 100.0;
        g.player.hp = 10.0;
        g.enter_room(rest, 4);
        assert!((g.player.hp - 55.0).abs() < 0.01, "cura el 45% de la vida");
        assert!(g.rooms[rest].used);
        g.player.hp = 10.0;
        g.enter_room(rest, 4);
        assert!((g.player.hp - 10.0).abs() < 0.01, "no cura dos veces");
    }

    #[test]
    fn shop_offer_spends_gold_and_applies() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let shop = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Shop)
            .expect("el piso debe tener tienda");
        g.player.hp = 20.0;
        g.enter_room(shop, 4);
        assert_eq!(g.shop_offers.len(), 4, "la tienda ofrece cuatro cosas");
        g.player.gold = 500;
        let cost = g.shop_offers[0].cost;
        let mut sfx = Sfx::silent();
        g.buy_shop_offer(0, &mut sfx);
        assert_eq!(g.player.gold, 500 - cost);
        assert!(g.player.hp > 20.0);
        assert!(g.shop_offers[0].sold);
        let gold = g.player.gold;
        g.buy_shop_offer(0, &mut sfx);
        assert_eq!(
            g.player.gold, gold,
            "una oferta vendida no se cobra dos veces"
        );
    }

    #[test]
    fn full_health_shop_offers_a_max_health_item() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let shop = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Shop)
            .expect("el piso debe tener tienda");
        g.player.hp = g.player.max_hp;
        g.enter_room(shop, 4);
        assert!(
            matches!(
                g.shop_offers[0].kind,
                ShopKind::Item(Item {
                    kind: ItemKind::MaxHp,
                    ..
                })
            ),
            "a vida completa la tienda ofrece subir la vida máxima"
        );

        // La poción comprada antes de curarse del todo no se puede usar a vida llena.
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let shop = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Shop)
            .unwrap();
        g.player.hp = 10.0;
        g.enter_room(shop, 4);
        let heal = g
            .shop_offers
            .iter()
            .position(|o| matches!(o.kind, ShopKind::Heal))
            .expect("debe haber poción de vida");
        g.player.hp = g.player.max_hp;
        g.player.gold = 500;
        let mut sfx = Sfx::silent();
        g.buy_shop_offer(heal, &mut sfx);
        assert_eq!(g.player.gold, 500, "no se cobra una curación a vida llena");
        assert!(!g.shop_offers[heal].sold);
    }

    #[test]
    fn chests_give_keys_and_open_once() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let chest_room = g
            .rooms
            .iter()
            .position(|r| r.has_chest)
            .expect("el piso debe tener un cofre de llave");
        g.enter_room(chest_room, 4);
        let chest = g
            .props
            .iter()
            .find(|p| p.kind == PropKind::Chest)
            .expect("la sala debe contener el cofre");
        g.player.pos = chest.pos + vec2(0.0, 10.0);
        let mut sfx = Sfx::silent();
        assert!(g.try_open_chest(&mut sfx), "el cofre se abre con E");
        assert_eq!(g.keys, 1, "el cofre de combate da una llave");
        assert!(g.rooms[chest_room].chest_opened);
        assert!(g
            .props
            .iter()
            .any(|p| p.kind == PropKind::Chest && p.opened));
        assert!(
            !g.try_open_chest(&mut sfx),
            "un cofre abierto no se puede saquear dos veces"
        );
    }

    #[test]
    fn rest_room_pact_is_taken_once() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let rest = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Rest)
            .expect("el piso debe tener descanso");
        g.enter_room(rest, 4);
        assert!(!g.rooms[rest].pact_used);
        g.player.pos = g.pact_pos(0);
        let mut sfx = Sfx::silent();
        assert!(g.try_take_pact(&mut sfx), "el altar acepta un pacto");
        assert_eq!(g.pacts_taken, 1);
        assert!(g.rooms[rest].pact_used);
        assert!(
            !g.try_take_pact(&mut sfx),
            "solo se acepta un pacto por sala de descanso"
        );
    }

    #[test]
    fn portal_descends_without_pressing_anything() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let boss = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Boss)
            .expect("el piso debe tener sala de guardián");
        g.enter_room(boss, 4);
        g.enemies.clear();
        g.wave = g.waves;
        g.cleared = true;
        g.rooms[boss].cleared = true;
        g.player.pos = g.portal;
        g.portal_t = 0.5;
        let mut sfx = Sfx::silent();
        let floor = g.floor;
        // Con la transición desactivada el descenso es inmediato.
        g.meta.floor_transitions = false;
        g.check_floor_clear(0.1, &mut sfx);
        assert_eq!(g.floor, floor + 1, "el portal desciende al acercarse");
        assert!(matches!(g.screen, Screen::Playing));
    }

    #[test]
    fn portal_starts_a_skippable_transition() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let boss = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Boss)
            .expect("el piso debe tener sala de guardián");
        g.enter_room(boss, 4);
        g.enemies.clear();
        g.wave = g.waves;
        g.cleared = true;
        g.rooms[boss].cleared = true;
        g.player.pos = g.portal;
        g.portal_t = 0.5;
        let mut sfx = Sfx::silent();
        let floor = g.floor;
        g.check_floor_clear(0.1, &mut sfx);
        assert_eq!(g.floor, floor, "la transición retrasa el cambio de piso");
        assert!(g.floor_transition_pending);
        assert!(g.floor_transition_t > 0.0);
    }

    #[test]
    fn enemy_catalog_is_large_and_varied() {
        let mut names = std::collections::HashSet::new();
        for kind in EnemyKind::NORMAL {
            for base in kind.variants() {
                for element in Element::ALL {
                    names.insert(format!("{} {}", base, element.adjective()));
                }
            }
        }
        assert_eq!(names.len(), 240, "240 enemigos normales distintos");
        let mut bosses = std::collections::HashSet::new();
        for base in EnemyKind::boss_titles() {
            for element in Element::ALL {
                bosses.insert(format!("{} {}", base, element.adjective()));
            }
        }
        assert_eq!(bosses.len(), 100, "100 jefes distintos");
    }

    #[test]
    fn items_table_is_valid() {
        assert_eq!(ITEMS.len(), 24);
        for (name, _, base) in ITEMS {
            assert!(!name.is_empty());
            assert!(base > 0.0);
        }
    }

    #[test]
    fn bosses_and_elites_scale_with_the_floor() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        g.floor = 1;
        g.spawn_enemy_at(EnemyKind::Boss, false, false, vec2(240.0, 100.0));
        let low = g.enemies.last().unwrap().max_hp;
        let low_dmg = g.enemies.last().unwrap().damage;
        g.enemies.clear();
        g.floor = 10;
        g.spawn_enemy_at(EnemyKind::Boss, false, false, vec2(240.0, 100.0));
        let high = g.enemies.last().unwrap().max_hp;
        let high_dmg = g.enemies.last().unwrap().damage;
        assert!(high > low * 4.0, "el guardián debe crecer con el piso");
        assert!(high_dmg > low_dmg * 2.5, "el guardián debe pegar más");
        g.enemies.clear();
        g.floor = 1;
        g.spawn_enemy_at(EnemyKind::Brute, false, false, vec2(240.0, 100.0));
        let normal_hp = g.enemies.last().unwrap().max_hp;
        g.enemies.clear();
        g.spawn_enemy_at(EnemyKind::Brute, true, false, vec2(240.0, 100.0));
        assert!(
            g.enemies.last().unwrap().max_hp > normal_hp * 3.5,
            "los élites deben ser mucho más duros"
        );
    }

    #[test]
    fn every_run_uses_a_new_boss_skin() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        for _ in 0..9 {
            g.next_floor();
        }
        let boss_room = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Boss)
            .expect("debe haber sala de guardián");
        g.room_index = boss_room;
        g.rooms[boss_room].cleared = false;
        g.cleared = false;
        g.spawn_wave();
        let first = g
            .enemies
            .iter()
            .find(|e| e.boss)
            .map(|e| e.boss_skin)
            .expect("el guardián debe aparecer");
        // Otra oleada de guardián en el mismo piso no repite la piel.
        g.rooms[boss_room].cleared = false;
        g.cleared = false;
        g.spawn_wave();
        let second = g
            .enemies
            .iter()
            .filter(|e| e.boss)
            .map(|e| e.boss_skin)
            .last()
            .expect("el segundo guardián debe aparecer");
        assert_ne!(first, second, "dos guardianes no comparten skin");
    }

    #[test]
    fn vault_needs_a_black_key_and_opens_once() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        let (vault, from, back) = 'find: {
            for (i, room) in g.rooms.iter().enumerate() {
                if room.kind != RoomKind::Vault {
                    continue;
                }
                for dir in 0..4 {
                    if !room.doors[dir] {
                        continue;
                    }
                    let (nx, ny, back) = match dir {
                        0 => (room.gx, room.gy - 1, 2),
                        1 => (room.gx + 1, room.gy, 3),
                        2 => (room.gx, room.gy + 1, 0),
                        _ => (room.gx - 1, room.gy, 1),
                    };
                    if let Some(from) = g.room_at(nx, ny) {
                        break 'find (i, from, back);
                    }
                }
            }
            panic!("la bóveda debe tener una puerta accesible");
        };
        g.room_index = from;
        g.rooms[from].visited = true;
        g.rooms[from].cleared = true;
        g.cleared = true;
        g.player.pos = g.door_pos(back);
        let mut sfx = Sfx::silent();

        // Sin llave negra la bóveda no se abre.
        assert!(!g.try_use_door(back, &mut sfx));
        assert!(!g.rooms[vault].vault_open);

        g.special_keys = 1;
        // La prueba comprueba la puerta, no el botín: marcamos la bóveda como
        // saqueada para que su 35% de llave extra no altere el contador.
        g.rooms[vault].used = true;
        g.player.pos = g.door_pos(back);
        assert!(g.try_use_door(back, &mut sfx));
        assert!(g.rooms[vault].vault_open);
        assert_eq!(g.special_keys, 0);
    }

    #[test]
    fn debt_and_echoes_persist_after_death() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Mage);
        g.floor = 4;
        g.debt = 90;
        g.end_run(false);
        assert_eq!(g.meta.debt, 90);
        assert_eq!(g.meta.echoes, 1);
        assert_eq!(g.meta.echo_floor, 4);
        assert_eq!(g.meta.echo_class, 2);
    }

    #[test]
    fn boss_spawns_every_ten_floors() {
        let mut g = Game::new(Meta::default());
        g.start_run(Class::Knight);
        for _ in 0..9 {
            g.next_floor();
        }
        assert_eq!(g.floor, 10);
        let boss_room = g
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Boss)
            .expect("el piso debe tener sala de guardián");
        g.enter_room(boss_room, 4);
        assert_eq!(g.waves, 1);
        assert!(g.enemies.iter().any(|e| e.boss));
        g.next_floor();
        assert_eq!(g.floor, 11);
        assert!(!g.enemies.iter().any(|e| e.boss));
    }

    #[test]
    fn classes_start_with_comparable_base_dps() {
        let meta = Meta::default();
        for class in Class::ALL {
            let p = Player::new(class, &meta);
            let dps = p.damage * p.attack_speed;
            assert!(
                (10.0..=20.0).contains(&dps),
                "{} empieza con {:.1} de DPS",
                class.name(),
                dps
            );
        }
    }

    #[test]
    fn merchant_carries_gold_with_half_the_weight() {
        let mut g = Game::new(Meta::default());
        g.player.gold = 350;
        g.player.class = Class::Knight;
        let knight = g.gold_weight();
        g.player.class = Class::Merchant;
        let merchant = g.gold_weight();
        assert!(
            (knight - 1.0).abs() < 0.001,
            "el caballero carga todo el peso"
        );
        assert!(
            (merchant - 0.5).abs() < 0.001,
            "el mercader debe cargar la mitad"
        );
    }
}
