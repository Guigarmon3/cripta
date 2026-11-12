//! Modo de grabación de la guía.
//!
//! Cuando se arranca con `CRIPTA_DEMO=1`, el juego se juega solo: recorre dos
//! pisos, abre cofres, visita la tienda y el descanso, elige mejoras y derrota
//! a los dos guardianes. Cada fotograma se envía a ffmpeg por tubería y los
//! rótulos explicativos se dibujan dentro del propio juego, de modo que el
//! vídeo queda sincronizado sin postproducción.

use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use macroquad::prelude::*;

use crate::audio::{self, Sfx};
use crate::game::{
    DemoInput, Game, LevelChoice, ObstacleKind, PropKind, RoomKind, Screen, View,
};
use crate::types::Class;
use crate::sprites::{Sprites, VH, VW};
use crate::ui::{Ui, UiFonts};

const DT: f32 = 1.0 / 30.0;

const GOLD: Color = Color::new(0.96, 0.78, 0.34, 1.0);
const PALE: Color = Color::new(0.90, 0.88, 0.82, 1.0);
const CYAN: Color = Color::new(0.55, 0.88, 0.90, 1.0);

#[derive(Clone, Copy, PartialEq, Eq)]
enum GoalKind {
    Chest,
    Shop,
    Rest,
    Treasure,
    Boss,
}

struct Goal {
    room: usize,
    kind: GoalKind,
    done: bool,
}

struct Caption {
    text: String,
    left: f32,
    total: f32,
}

struct Flags {
    intro: bool,
    tour: bool,
    knight: bool,
    floor1: bool,
    map: bool,
    controls: bool,
    first_chest: bool,
    first_level: bool,
    shop: bool,
    rest: bool,
    treasure: bool,
    boss: bool,
    floor2: bool,
    boss2: bool,
    end: bool,
    victory: bool,
}

impl Flags {
    fn new() -> Self {
        Self {
            intro: false,
            tour: false,
            knight: false,
            floor1: false,
            map: false,
            controls: false,
            first_chest: false,
            first_level: false,
            shop: false,
            rest: false,
            treasure: false,
            boss: false,
            floor2: false,
            boss2: false,
            end: false,
            victory: false,
        }
    }
}

pub struct Demo {
    pub t: f32,
    pub finished: bool,
    menu_t: f32,
    level_t: f32,
    screen: Screen,
    route_floor: u32,
    goals: Vec<Goal>,
    goal_i: usize,
    caption: Option<Caption>,
    queue: VecDeque<(String, f32)>,
    log: Vec<(f32, f32, String)>,
    flags: Flags,
    strafe: f32,
    dash_wait: f32,
    loot_t: f32,
    chest_t: f32,
    shop_t: f32,
    rest_t: f32,
    end_t: f32,
    map_t: f32,
    last_pos: Vec2,
    stuck_t: f32,
    unstuck_t: f32,
    combat_t: f32,
    stray_t: f32,
    last_room: usize,
    last_floor: u32,
    unstuck_dir: Vec2,
    rng: u32,
    events: Vec<String>,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            t: 0.0,
            finished: false,
            menu_t: 0.0,
            level_t: 0.0,
            screen: Screen::Menu,
            route_floor: 0,
            goals: Vec::new(),
            goal_i: 0,
            caption: None,
            queue: VecDeque::new(),
            log: Vec::new(),
            flags: Flags::new(),
            strafe: 1.0,
            dash_wait: 0.0,
            loot_t: 0.0,
            chest_t: 0.0,
            shop_t: 0.0,
            rest_t: 0.0,
            end_t: 0.0,
            map_t: 0.0,
            last_pos: Vec2::ZERO,
            stuck_t: 0.0,
            unstuck_t: 0.0,
            combat_t: 0.0,
            stray_t: 0.0,
            last_room: usize::MAX,
            last_floor: 0,
            unstuck_dir: Vec2::ZERO,
            rng: 0x51A7_C0DE,
            events: Vec::new(),
        }
    }

    fn rand(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    fn cue(&mut self, text: impl Into<String>, dur: f32) {
        let text = text.into();
        if let Some(c) = &self.caption {
            if c.text == text {
                return;
            }
        }
        // Los rótulos de la guía sustituyen al anterior: cada uno debe contar
        // lo que está pasando ahora, no llegar con retraso.
        self.queue.clear();
        self.caption = Some(Caption {
            text,
            left: dur,
            total: dur,
        });
    }

    fn tick_caption(&mut self, dt: f32) {
        if let Some(c) = &mut self.caption {
            c.left -= dt;
            if c.left <= 0.0 {
                self.caption = None;
            }
        }
        if self.caption.is_none() {
            if let Some((text, total)) = self.queue.pop_front() {
                self.caption = Some(Caption {
                    text,
                    left: total,
                    total,
                });
            }
        }
    }

    fn show(&mut self, text: &str, dur: f32) {
        self.cue(text, dur);
    }

    pub fn update(&mut self, dt: f32, game: &mut Game, sfx: &mut Sfx) {
        self.t += dt;
        if self.screen != game.screen {
            self.screen = game.screen;
            self.level_t = 0.0;
        }
        self.tick_caption(dt);

        match game.screen {
            Screen::Menu => self.update_menu(dt, game),
            Screen::Playing => self.update_playing(dt, game, sfx),
            Screen::LevelUp => self.update_levelup(dt, game, sfx),
            _ => {}
        }
    }

    fn update_menu(&mut self, dt: f32, game: &mut Game) {
        self.menu_t += dt;
        if !self.flags.intro && self.menu_t > 1.2 {
            self.flags.intro = true;
            self.show(
                "Bienvenido a CRIPTA. Guía rápida: elige héroe, saquea la necrópolis y sobrevive a dos pisos.",
                6.5,
            );
        }
        // Recorre algunas clases para que se vean sus fichas y estadísticas.
        if self.menu_t < 6.0 {
            game.menu_index = 0;
        } else if self.menu_t < 7.6 {
            game.menu_index = 2;
            if !self.flags.tour {
                self.flags.tour = true;
                self.show(
                    "Siete clases, cada una con su arma, sus ataques y su estilo: tanque, tirador, arcano, sombra, nigromante, apoyo y oro.",
                    7.0,
                );
            }        } else if self.menu_t < 9.2 {
            game.menu_index = 5;
        } else if self.menu_t < 10.8 {
            game.menu_index = 6;
        } else {
            game.menu_index = 0;
            if !self.flags.knight {
                self.flags.knight = true;
                self.show(
                    "Para esta partida: Caballero. 130 de vida, armadura y bloqueo: perdona los errores mientras aprendes.",
                    5.5,
                );
            }
        }
        if self.menu_t > 14.0 {
            self.events.push(format!("{:.2} empieza la partida (Caballero)", self.t));
            game.start_run(Class::Knight);
            self.menu_t = 0.0;
        }
    }

    fn update_levelup(&mut self, dt: f32, game: &mut Game, sfx: &mut Sfx) {
        self.level_t += dt;
        if !self.flags.first_level {
            self.flags.first_level = true;
            self.show(
                "Subes de nivel: elige 1 de 3. Aquí se combinan ataques nuevos, subidas de ataque y mejoras de estadísticas.",
                6.0,
            );
        }
        if game.pending_replace.is_some() {
            if self.level_t > 2.2 {
                game.demo_resolve_replace(0, sfx);
            }
            return;
        }
        if self.level_t > 3.0 {
            // Prioriza mejoras de estadísticas y subidas de ataque; los ataques
            // nuevos son más vistosos pero llenan la mochila de seis huecos.
            let pick = (0..3)
                .find(|&i| matches!(game.level_choices[i], LevelChoice::Buff(_)))
                .or_else(|| {
                    (0..3).find(|&i| matches!(game.level_choices[i], LevelChoice::UpgradeAttack(_)))
                })
                .unwrap_or(0);
            game.choose_level_choice(pick, sfx);
            self.level_t = 0.0;
        }
    }

    fn plan_floor(&mut self, game: &Game) {
        self.route_floor = game.floor;
        self.goals.clear();
        self.goal_i = 0;
        self.loot_t = 0.0;
        self.shop_t = 0.0;
        self.rest_t = 0.0;

        let start = game.room_index;
        let boss = game
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Boss)
            .unwrap_or(start);
        let mut chests: Vec<usize> = game
            .rooms
            .iter()
            .enumerate()
            .filter(|(_, r)| r.has_chest && !r.chest_opened)
            .map(|(i, _)| i)
            .collect();
        chests.sort_by_key(|&i| {
            bfs(game, start, i, boss, true, true)
                .map(|p| p.len())
                .unwrap_or(999)
        });
        for &room in &chests {
            self.goals.push(Goal {
                room,
                kind: GoalKind::Chest,
                done: false,
            });
        }
        if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Shop) {
            self.goals.push(Goal {
                room: i,
                kind: GoalKind::Shop,
                done: false,
            });
        }
        if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Rest) {
            self.goals.push(Goal {
                room: i,
                kind: GoalKind::Rest,
                done: false,
            });
        }
        if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Treasure) {
            self.goals.push(Goal {
                room: i,
                kind: GoalKind::Treasure,
                done: false,
            });
        }
        self.goals.push(Goal {
            room: boss,
            kind: GoalKind::Boss,
            done: false,
        });
        self.events.push(format!(
            "{:.2} piso {}: ruta con {} objetivos",
            self.t,
            game.floor,
            self.goals.len()
        ));
    }

    fn update_playing(&mut self, dt: f32, game: &mut Game, sfx: &mut Sfx) {
        game.demo_input = DemoInput {
            active: true,
            ..Default::default()
        };
        if game.floor != self.route_floor {
            self.plan_floor(game);
        }
        if game.room_index != self.last_room || game.floor != self.last_floor {
            self.last_room = game.room_index;
            self.last_floor = game.floor;
            self.combat_t = 0.0;
            self.stray_t = 0.0;
        }
        if game.floor == 1 && !self.flags.floor1 {
            self.flags.floor1 = true;
            self.show(
                "Piso 1. WASD para moverte; atacas automático. Espacio esquiva y te hace invulnerable.",
                7.0,
            );
        }
        if game.floor >= 2 && !self.flags.floor2 {
            self.flags.floor2 = true;
            self.show(
                "Piso 2. Cada piso sube la dificultad, añade oleadas y cambia el elemento de las criaturas. El cristal agrietado te da más daño.",
                7.5,
            );
        }

        // La Carta de la cripta se abre una vez, al empezar, para explicar el
        // mapa. Se espera a que el rótulo del piso se haya leído.
        if game.floor == 1
            && !self.flags.map
            && game.rooms[game.room_index].kind == RoomKind::Start
        {
            if game.time < 5.2 {
                game.demo_input.dir = Vec2::ZERO;
                return;
            }
            self.flags.map = true;
            self.map_t = 3.2;
            game.map_open = true;
            self.show(
                "La Carta de la cripta: las líneas se trazan al explorar. La llave lavanda marca la bóveda sellada.",
                3.4,
            );
            return;
        }
        if game.map_open {
            self.map_t -= dt;
            if self.map_t <= 0.0 {
                game.map_open = false;
            }
            return;
        }

        if game.boss_intro_t > 0.0 {
            game.demo_input.dir = Vec2::ZERO;
            if game.floor >= 2 && !self.flags.boss2 {
                self.flags.boss2 = true;
                self.show(
                    "El segundo guardián: más vida, más daño y firmas más frecuentes. Observa el anillo de runas y esquiva fuera de él.",
                    7.0,
                );
            } else if !self.flags.boss {
                self.flags.boss = true;
                self.show(
                    "El guardián se presenta. Antes de desatar su firma se hincha y dibuja un anillo de runas: esa es la señal para esquivar.",
                    7.0,
                );
            }
            return;
        }
        if game.floor_transition_t > 0.0 || game.floor_arrival_t > 0.0 {
            game.demo_input.dir = Vec2::ZERO;
            return;
        }

        // Fin de la guía: segundo guardián derrotado.
        if game.floor >= 2
            && game.rooms[game.room_index].kind == RoomKind::Boss
            && game.cleared
        {
            game.demo_input.dir = Vec2::ZERO;
            if !self.flags.victory {
                self.flags.victory = true;
                self.show(
                    "Guardián derrotado. Dos pisos completos: cofres, llaves, tienda, pactos, mejoras y dos jefes. Fin de la guía.",
                    7.0,
                );
                self.events
                    .push(format!("{:.2} victoria sobre el segundo guardián", self.t));
            }
            self.end_t += dt;
            if self.end_t > 7.0 {
                self.finished = true;
            }
            return;
        }

        // Combate: la sala aún no está limpia.
        if !game.cleared && matches!(game.rooms[game.room_index].kind, RoomKind::Combat | RoomKind::Boss)
        {
            self.combat_t += dt;
            let boss_room = game.rooms[game.room_index].kind == RoomKind::Boss;
            let cap = if boss_room { 100.0 } else { 35.0 };
            if self.combat_t > cap {
                // Enemigo atrapado fuera del arena o inalcanzable: la sala se
                // cierra igual para no bloquear la guía.
                game.demo_force_clear();
                self.combat_t = 0.0;
                self.events
                    .push(format!("{:.2} sala forzada a limpiar", self.t));
            }
            self.fight(dt, game, sfx);
            return;
        }

        if self.loot_t > 0.0 {
            self.loot_t -= dt;
            if self.walk_to_loot(game) {
                return;
            }
        }

        // Objetivo que corresponde a la sala actual.
        if let Some(gi) = self
            .goals
            .iter()
            .position(|g| !g.done && g.room == game.room_index)
        {
            let kind = self.goals[gi].kind;
            match kind {
                GoalKind::Chest | GoalKind::Treasure => {
                    if self.open_chest(game) {
                        self.goals[gi].done = true;
                        self.loot_t = 2.4;
                        if kind == GoalKind::Chest && !self.flags.first_chest {
                            self.flags.first_chest = true;
                            self.chest_t = self.t;
                            self.show(
                                "Cofre abierto: una llave oxidada y oro. Las llaves abren la cámara del tesoro y la puerta del guardián.",
                                6.5,
                            );
                        }
                        if kind == GoalKind::Treasure && !self.flags.treasure {
                            self.flags.treasure = true;
                            self.show(
                                "Tesoro: una llave por objeto, oro y curación.",
                                6.5,
                            );
                        }
                    }
                }
                GoalKind::Shop => {
                    self.shop_t += dt;
                    if self.walk_shop(game) {
                        return;
                    }
                    if !self.flags.shop && self.t >= self.chest_t + 4.0 {
                        self.flags.shop = true;
                        self.show(
                            "Tienda: curación, llave, objeto y mejora. Se compra con 1-4 o clic; con F puedes comprar a crédito y pagar luego.",
                            6.5,
                        );
                    }
                    if self.shop_t > 6.0 {
                        self.goals[gi].done = true;
                        self.shop_t = 0.0;
                    }
                    return;
                }
                GoalKind::Rest => {
                    self.rest_t += dt;
                    self.walk_rest(game);
                    if !self.flags.rest {
                        self.flags.rest = true;
                        self.show(
                            "Sala de descanso: dos altares, un pacto por sala. Una ventaja permanente a cambio de una desventaja permanente.",
                            6.5,
                        );
                    }
                    if self.rest_t > 4.2 {
                        self.goals[gi].done = true;
                        self.rest_t = 0.0;
                    }
                    return;
                }
                GoalKind::Boss => {
                    // Si la sala ya está limpia pero el objetivo sigue vivo,
                    // se camina hacia el portal.
                    if game.rooms[game.room_index].cleared {
                        self.goals[gi].done = true;
                    }
                }
            }
        }

        // Descenso tras el primer guardián.
        if game.rooms[game.room_index].kind == RoomKind::Boss && game.cleared {
            let d = game.player.pos.distance(game.portal);
            if d > 12.0 {
                game.demo_input.dir = (game.portal - game.player.pos).normalize_or_zero();
            }
            return;
        }

        // Elige el siguiente objetivo y sigue la ruta.
        self.follow_route(game);
    }

    fn needed_keys(&self) -> u32 {
        let treasure_done = self
            .goals
            .iter()
            .find(|g| g.kind == GoalKind::Treasure)
            .map(|g| g.done)
            .unwrap_or(true);
        if treasure_done {
            1
        } else {
            2
        }
    }

    fn next_goal(&mut self, game: &mut Game) -> Option<usize> {
        // Si ya tenemos llaves suficientes, los cofres restantes sobran.
        let need = self.needed_keys();
        while self.goal_i < self.goals.len() {
            if self.goals[self.goal_i].done {
                self.goal_i += 1;
                continue;
            }
            if self.goals[self.goal_i].kind == GoalKind::Chest
                && game.keys >= need
                && self.goals.iter().any(|g| g.kind == GoalKind::Boss && !g.done)
            {
                self.goals[self.goal_i].done = true;
                self.goal_i += 1;
                continue;
            }
            return Some(self.goal_i);
        }
        None
    }

    fn follow_route(&mut self, game: &mut Game) {
        let boss = game
            .rooms
            .iter()
            .position(|r| r.kind == RoomKind::Boss)
            .unwrap_or(usize::MAX);
        let (goal_room, is_boss_goal, goal_idx) = match self.next_goal(game) {
            Some(gi) => {
                let kind = self.goals[gi].kind;
                (self.goals[gi].room, kind == GoalKind::Boss, Some(gi))
            }
            None => {
                // Sin objetivos pendientes: el único destino válido es el
                // guardián. Nunca se deja al héroe parado en una sala.
                if boss == usize::MAX
                    || game.room_index == boss
                    || game.rooms.get(boss).map(|r| r.cleared).unwrap_or(true)
                {
                    return;
                }
                (boss, true, None)
            }
        };
        if goal_room == game.room_index {
            return;
        }
        let path = match bfs(game, game.room_index, goal_room, boss, !is_boss_goal, false) {
            Some(p) => p,
            None => {
                // Si el guardián corta el paso, se permite cruzar su sala para
                // llegar al objetivo; es mejor que darse por vencido.
                match bfs(game, game.room_index, goal_room, boss, false, false) {
                    Some(p) => p,
                    None => {
                        if let Some(gi) = goal_idx {
                            self.goals[gi].done = true;
                            self.goal_i = gi + 1;
                        }
                        return;
                    }
                }
            }
        };
        if path.len() < 2 {
            return;
        }
        let next = path[1];
        let (gx, gy) = (game.rooms[game.room_index].gx, game.rooms[game.room_index].gy);
        let (nx, ny) = (game.rooms[next].gx, game.rooms[next].gy);
        let dir = if nx > gx {
            1
        } else if nx < gx {
            3
        } else if ny > gy {
            2
        } else {
            0
        };
        let (l, r, t, b) = game.rooms[game.room_index].size.bounds();
        let target = match dir {
            0 => vec2((l + r) * 0.5, t),
            1 => vec2(r, (t + b) * 0.5),
            2 => vec2((l + r) * 0.5, b),
            _ => vec2(l, (t + b) * 0.5),
        };
        let d = target - game.player.pos;
        if d.length() > 8.0 {
            game.demo_input.dir = d.normalize_or_zero();
        }
    }

    fn fight(&mut self, dt: f32, game: &mut Game, _sfx: &mut Sfx) {
        let p = game.player.pos;
        let (l, r, t, b) = game.rooms[game.room_index].size.bounds();
        // Los enemigos que quedan fuera del arena no se pueden golpear: se
        // ignoran para no empujar al héroe contra la puerta sellada.
        let margin = 14.0;
        let mut target: Option<(Vec2, f32)> = None;
        let mut stray = false;
        for e in &game.enemies {
            if e.dead {
                continue;
            }
            if e.pos.x < l - margin
                || e.pos.x > r + margin
                || e.pos.y < t - margin
                || e.pos.y > b + margin
            {
                stray = true;
                continue;
            }
            let d = e.pos.distance(p);
            if target.map(|(_, bd)| d < bd).unwrap_or(true) {
                target = Some((e.pos, d));
            }
        }
        if target.is_none() && stray {
            self.stray_t += dt;
            game.demo_input.dir = Vec2::ZERO;
            if self.stray_t > 2.0 {
                game.demo_force_clear();
                self.stray_t = 0.0;
                self.events
                    .push(format!("{:.2} enemigo extraviado eliminado", self.t));
            }
            return;
        }
        self.stray_t = 0.0;
        let Some((epos, dist)) = target else {
            game.demo_input.dir = Vec2::ZERO;
            return;
        };
        let to = epos - p;
        let n = to.normalize_or_zero();
        let desired = game.player.range * 0.62;
        let mut dir = if dist > desired + 4.0 {
            n
        } else if dist < desired - 5.0 {
            -n
        } else {
            vec2(-n.y, n.x) * self.strafe
        };

        // No quedarse contra las paredes ni sobre los pinchos.
        if p.x < l + 22.0 {
            dir.x += 0.9;
        }
        if p.x > r - 22.0 {
            dir.x -= 0.9;
        }
        if p.y < t + 22.0 {
            dir.y += 0.9;
        }
        if p.y > b - 22.0 {
            dir.y -= 0.9;
        }
        for o in &game.obstacles {
            if o.kind == ObstacleKind::Spikes && p.distance(o.pos) < o.radius + 18.0 {
                dir += (p - o.pos).normalize_or_zero() * 1.4;
            }
        }
        game.demo_input.dir = dir.normalize_or_zero();

        // Esquiva periódica y, sobre todo, cuando el guardián carga su firma.
        self.dash_wait -= dt;
        let boss_windup = game
            .enemies
            .iter()
            .any(|e| e.boss && !e.dead && e.signature_t < 1.15);
        if game.player.dash_cd <= 0.0 && (boss_windup || (self.dash_wait <= 0.0 && dist < 92.0)) {
            game.demo_input.dash = true;
            self.dash_wait = 2.3 + self.rand() * 1.1;
            self.strafe = -self.strafe;
        }

        // Pequeño baile para no quedarse quieto mientras se recarga el ataque.
        if self.rand() < 0.004 {
            self.strafe = -self.strafe;
        }
    }

    fn walk_shop(&mut self, game: &mut Game) -> bool {
        let (l, r, t, b) = game.rooms[game.room_index].size.bounds();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        let target = vec2(cx, cy - 18.0);
        let d = target - game.player.pos;
        if d.length() > 4.0 {
            game.demo_input.dir = d.normalize_or_zero();
            return true;
        }
        game.demo_input.dir = Vec2::ZERO;
        false
    }

    fn walk_rest(&mut self, game: &mut Game) {
        let (l, r, t, b) = game.rooms[game.room_index].size.bounds();
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        let target = vec2(cx - 76.0, cy + 26.0);
        let d = target - game.player.pos;
        if d.length() > 6.0 {
            game.demo_input.dir = d.normalize_or_zero();
        } else {
            game.demo_input.dir = Vec2::ZERO;
            game.demo_input.interact = true;
        }
    }

    fn open_chest(&mut self, game: &mut Game) -> bool {
        let chest = game
            .props
            .iter()
            .find(|p| p.kind == PropKind::Chest && !p.opened)
            .map(|p| p.pos);
        let Some(pos) = chest else {
            return true;
        };
        let target = pos + vec2(0.0, 12.0);
        let d = target - game.player.pos;
        if d.length() > 5.0 {
            game.demo_input.dir = d.normalize_or_zero();
            false
        } else {
            game.demo_input.dir = Vec2::ZERO;
            game.demo_input.interact = true;
            false
        }
    }

    fn walk_to_loot(&mut self, game: &mut Game) -> bool {
        let p = game.player.pos;
        let mut best: Option<(Vec2, f32)> = None;
        for pk in &game.pickups {
            let d = pk.pos.distance(p);
            if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                best = Some((pk.pos, d));
            }
        }
        if let Some((pos, _)) = best {
            game.demo_input.dir = (pos - p).normalize_or_zero();
            true
        } else {
            false
        }
    }

    /// Dibuja los rótulos de la guía sobre la interfaz del juego.
    pub fn draw(&self, game: &Game, s: &Sprites, ui: &Ui) {
        let _ = s;
        if let Some(c) = &self.caption {
            let menu = game.screen == Screen::Menu;
            let (y, h, label_y, text_y, size, line_h) = if menu {
                (242.0, 28.0, 8.0, 8.0, 5.5, 8.0)
            } else {
                (0.0, 40.0, 9.0, 9.0, 6.1, 9.0)
            };
            let alpha = (c.left.min(0.35) / 0.35).min(1.0);
            let fade = (c.total - c.left).min(0.25) / 0.25;
            let a = alpha.min(fade.max(0.0));
            ui.rect(0.0, y, VW, h, Color::new(0.015, 0.012, 0.022, 0.97 * a));
            ui.rect(0.0, y + h - 1.0, VW, 1.0, Color::new(GOLD.r, GOLD.g, GOLD.b, 0.85 * a));
            ui.text_bold(
                "GUÍA",
                8.0,
                y + label_y,
                4.6,
                Color::new(GOLD.r, GOLD.g, GOLD.b, a),
                true,
            );
            ui.wrap_left(
                &c.text,
                42.0,
                y + text_y,
                VW - 54.0,
                size,
                line_h,
                Color::new(PALE.r, PALE.g, PALE.b, a),
            );
        }

        if game.screen == Screen::Playing && !self.flags.victory {
            // Recordatorio discreto de controles mientras dura la partida.
            let label = "WASD mover   Espacio esquiva   E interactuar   Tab mapa   Esc pausa";
            ui.text_center_shadow(
                label,
                263.0,
                4.6,
                Color::new(0.62, 0.66, 0.70, 0.55),
            );
        }

        // Cierre de la guía.
        if self.flags.victory && self.end_t > 3.5 {
            let a = ((self.end_t - 3.5) / 1.2).min(1.0);
            ui.rect(0.0, 0.0, VW, VH, Color::new(0.01, 0.008, 0.016, 0.86 * a));
            ui.text_center_shadow("CRIPTA", 118.0, 30.0, Color::new(GOLD.r, GOLD.g, GOLD.b, a));
            ui.text_center(
                "Dos pisos. Dos guardianes.",
                146.0,
                7.5,
                Color::new(PALE.r, PALE.g, PALE.b, a),
            );
            ui.text_center(
                "Gracias por ver la guía.",
                160.0,
                6.5,
                Color::new(CYAN.r, CYAN.g, CYAN.b, a),
            );
            ui.text_center(
                "WASD · Espacio · E · Tab · Esc",
                184.0,
                5.6,
                Color::new(0.65, 0.68, 0.72, a),
            );
        }
    }

    pub fn write_log(&self) {
        let dir = Path::new("qa");
        let _ = std::fs::create_dir_all(dir);
        let mut text = String::new();
        for (start, dur, line) in &self.log {
            text.push_str(&format!("{start:.2}\t{dur:.2}\t{line}\n"));
        }
        let _ = std::fs::write(dir.join("demo_captions.txt"), text);
        let mut ev = String::new();
        for line in &self.events {
            ev.push_str(line);
            ev.push('\n');
        }
        let _ = std::fs::write(dir.join("demo_timeline.txt"), ev);
    }

    /// Guarda en el log cada rótulo mostrado, con su tiempo de inicio.
    pub fn log_caption(&mut self, caption: &Caption) {
        self.log
            .push((self.t, caption.total, caption.text.clone()));
    }
}

fn bfs(
    game: &Game,
    from: usize,
    to: usize,
    boss: usize,
    block_boss: bool,
    block_treasure: bool,
) -> Option<Vec<usize>> {
    if from == to {
        return Some(vec![from]);
    }
    let n = game.rooms.len();
    let mut prev = vec![usize::MAX; n];
    let mut q = VecDeque::new();
    prev[from] = from;
    q.push_back(from);
    while let Some(u) = q.pop_front() {
        let r = &game.rooms[u];
        for dir in 0..4 {
            if !r.doors[dir] {
                continue;
            }
            let (nx, ny) = match dir {
                0 => (r.gx, r.gy - 1),
                1 => (r.gx + 1, r.gy),
                2 => (r.gx, r.gy + 1),
                _ => (r.gx - 1, r.gy),
            };
            let Some(v) = game.rooms.iter().position(|x| x.gx == nx && x.gy == ny) else {
                continue;
            };
            if v == boss && block_boss {
                continue;
            }
            if game.rooms[v].kind == RoomKind::Vault {
                continue;
            }
            if block_treasure && v != to && game.rooms[v].kind == RoomKind::Treasure {
                continue;
            }
            if prev[v] == usize::MAX {
                prev[v] = u;
                if v == to {
                    let mut path = vec![v];
                    let mut c = u;
                    while c != from {
                        path.push(c);
                        c = prev[c];
                    }
                    path.push(from);
                    path.reverse();
                    return Some(path);
                }
                q.push_back(v);
            }
        }
    }
    None
}

/// Arranca la grabación completa. `game` debe venir creado con `Meta::default`.
pub async fn run(
    game: &mut Game,
    sprites: &Sprites,
    fonts: &UiFonts,
    target: &RenderTarget,
    sfx: &mut Sfx,
) {
    let _ = std::fs::create_dir_all("qa");
    audio::export_music_layers(Path::new("qa/demo_audio"));

    // Primeros fotogramas sin grabar: fuerza a rasterizar fuentes y texturas
    // para que el vídeo no empiece con un atlas a medio construir.
    for _ in 0..8 {
        let sw = screen_width();
        let sh = screen_height();
        let scale = ((sw / VW).min(sh / VH)).max(0.01);
        let ox = (sw - VW * scale) * 0.5;
        let oy = (sh - VH * scale) * 0.5;
        game.view = View {
            scale,
            ox,
            oy,
            mx: -1000.0,
            my: -1000.0,
        };
        crate::draw_frame(game, sprites, fonts, target);
        next_frame().await;
    }

    let out = std::env::var("CRIPTA_DEMO_OUT")
        .unwrap_or_else(|_| "qa/cripta_guia_raw.mp4".to_owned());
    let record = std::env::var_os("CRIPTA_DEMO_NORECORD").is_none();
    let mut child: Option<Child> = None;
    let mut stdin = None;
    if record {
        let ffmpeg = std::env::var("FFMPEG").unwrap_or_else(|_| "ffmpeg".to_owned());
        let mut cmd = Command::new(ffmpeg);
        cmd.args([
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            "1440x810",
            "-r",
            "30",
            "-i",
            "-",
            "-vf",
            "vflip",
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "16",
            "-pix_fmt",
            "yuv420p",
            &out,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
        match cmd.spawn() {
            Ok(mut c) => {
                stdin = c.stdin.take();
                child = Some(c);
            }
            Err(e) => {
                let _ = std::fs::write("qa/demo_error.txt", format!("ffmpeg: {e}"));
            }
        }
    }
    let _ = std::fs::write("qa/demo_progress.txt", "inicio\n");

    let limit = std::env::var("CRIPTA_DEMO_LIMIT")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(f32::INFINITY);
    let mut demo = Demo::new();
    let mut last_caption: Option<String> = None;
    let mut frame = 0u64;
    loop {
        let dt = DT;
        demo.update(dt, game, sfx);
        game.update(dt, sfx);
        // Los rótulos se registran con el tiempo real en que aparecen.
        if let Some(c) = &demo.caption {
            if last_caption.as_deref() != Some(c.text.as_str()) {
                last_caption = Some(c.text.clone());
                let cc = Caption {
                    text: c.text.clone(),
                    left: c.left,
                    total: c.total,
                };
                demo.log_caption(&cc);
            }
        }

        let sw = screen_width();
        let sh = screen_height();
        let scale = ((sw / VW).min(sh / VH)).max(0.01);
        let ox = (sw - VW * scale) * 0.5;
        let oy = (sh - VH * scale) * 0.5;
        game.view = View {
            scale,
            ox,
            oy,
            mx: -1000.0,
            my: -1000.0,
        };

        crate::draw_frame(game, sprites, fonts, target);
        let ui = Ui::new(fonts, scale, ox, oy, Vec2::ZERO);
        demo.draw(game, sprites, &ui);

        if let Some(pipe) = &mut stdin {
            let image = get_screen_data();
            if image.width as usize != sw as usize || image.height as usize != sh as usize {
                eprintln!(
                    "captura inesperada {}x{} (ventana {}x{})",
                    image.width, image.height, sw, sh
                );
            }
            if pipe.write_all(&image.bytes).is_err() {
                break;
            }
        }

        next_frame().await;
        frame += 1;
        if frame % 300 == 0 {
            let _ = std::fs::write(
                "qa/demo_progress.txt",
                format!(
                    "t={:.1}s piso={} sala={} objetivos={}/{} capt={:?}\n",
                    demo.t,
                    game.floor,
                    game.room_index,
                    demo.goal_i,
                    demo.goals.len(),
                    demo.caption.as_ref().map(|c| c.text.as_str()).unwrap_or("")
                ),
            );
        }
        if demo.finished && demo.end_t > 8.0 {
            break;
        }
        if demo.t > limit {
            break;
        }
    }

    drop(stdin);
    if let Some(mut c) = child {
        let _ = c.wait();
    }
    demo.write_log();
    let _ = std::fs::write(
        "qa/demo_progress.txt",
        format!(
            "fin t={:.1}s piso={} victoria={} frames={}\n",
            demo.t, game.floor, demo.flags.victory, frame
        ),
    );
}
