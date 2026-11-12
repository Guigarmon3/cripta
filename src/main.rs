#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod attacks;
mod audio;
mod buffs;
mod demo;
mod figures;
mod game;
mod pixel;
mod save;
mod sprites;
mod types;
mod ui;
mod weapons;

use audio::Sfx;
use buffs::BuffId;
use game::{
    Element, Game, Item, ItemKind, LevelChoice, Modifier, PropKind, RoomKind, Screen, View,
};
use macroquad::prelude::*;
use sprites::{Sprites, VH, VW};
use types::{Class, Rarity};
use ui::{Ui, UiFonts};

fn window_conf() -> Conf {
    let mut width = 1280;
    let mut height = 720;
    if std::env::var("CRIPTA_SHOT").is_ok() || std::env::var("CRIPTA_DEMO").is_ok() {
        width = 1440;
        height = 810;
    }
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        extern "system" {
            fn GetSystemMetrics(index: i32) -> i32;
        }
        let (dw, dh) = unsafe { (GetSystemMetrics(0), GetSystemMetrics(1)) };
        if dw >= 1920 && dh >= 1080 {
            width = 1440;
            height = 810;
        } else if dw < 1366 || dh < 768 {
            width = 960;
            height = 540;
        }
    }
    Conf {
        window_title: "CRIPTA".to_owned(),
        window_width: width,
        window_height: height,
        window_resizable: true,
        high_dpi: false,
        sample_count: 0,
        ..Default::default()
    }
}

/// Dibuja un fotograma completo: mundo al render target de 480x270 y después
/// la interfaz en resolución nativa, ya bliteada a la ventana.
pub(crate) fn draw_frame(game: &Game, sprites: &Sprites, fonts: &UiFonts, target: &RenderTarget) {
    let sw = screen_width();
    let sh = screen_height();
    let scale = ((sw / VW).min(sh / VH)).max(0.01);
    let ox = (sw - VW * scale) * 0.5;
    let oy = (sh - VH * scale) * 0.5;

    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VW, VH));
    cam.render_target = Some(target.clone());
    if let Some((focus, zoom)) = game.camera_focus() {
        cam.target = focus;
        cam.zoom *= zoom;
    }
    set_camera(&cam);
    clear_background(Color::from_rgba(10, 8, 14, 255));
    game.draw_world(sprites);

    set_default_camera();
    clear_background(Color::from_rgba(5, 4, 8, 255));
    let shk = game.shake_vec();
    draw_texture_ex(
        &target.texture,
        ox + shk.x * scale * 0.7,
        oy + shk.y * scale * 0.7,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VW * scale, VH * scale)),
            flip_y: true,
            ..Default::default()
        },
    );

    let ui = Ui::new(fonts, scale, ox, oy, shk);
    game.draw_world_text(&ui);
    game.draw_ui(sprites, &ui);
}

/// Prepara una partida de muestra para revisar el diseño de cada pantalla.
/// Solo se usa con CRIPTA_SHOT, nunca en el juego normal.
fn prepare_shot(game: &mut Game, shot: &str) {
    // CRIPTA_SHOT_RAW conserva la partida real del jugador. Sirve para
    // comprobar el guardado portable sin que la muestra de QA lo pise.
    if std::env::var("CRIPTA_SHOT_RAW").is_err() {
        game.meta.souls = 340;
        game.meta.best_floor = 7;
        game.meta.runs = 12;
        game.meta.hp_lv = 2;
        game.meta.dmg_lv = 1;
        game.meta.luck_lv = 1;
        game.meta.total_kills = 187;
        game.meta.name = "Lyra".to_owned();
        game.meta.skin = 3;
        game.meta.hat = 4;
        game.meta.mask = 1;
        game.meta.owned_skins = 0b0000_0111;
        game.meta.owned_hats = 0b0001_0011;
        game.meta.owned_masks = 0b0010_0011;
    }
    game.name_buffer = game.meta.name.clone();

    match shot {
        "menu" => {
            game.screen = Screen::Menu;
            game.menu_index = 0;
        }
        "shop" => {
            game.screen = Screen::Shop;
            game.shop_tab = 0;
        }
        "library" => {
            game.screen = Screen::Library;
            game.library_tab = 0;
            game.library_page = 0;
        }
        "settings" => {
            game.screen = Screen::Settings;
            game.settings_row = 0;
            game.rebinding = false;
        }
        "audio" => {
            game.start_run(Class::Mage);
            game.screen = Screen::Audio;
            game.audio_return = Screen::Menu;
            game.audio_row = 1;
            game.meta.music_vol = 45;
            game.meta.sfx_vol = 80;
            game.meta.enemy_vol = 65;
        }
        "locker" => {
            game.screen = Screen::Locker;
            game.locker_class = 2;
        }
        "forge" => {
            game.screen = Screen::Forge;
            game.forge_class = 5;
            game.meta.souls = 420;
            game.meta.best_floor = 7;
            game.meta.unlock_weapon(Class::Cleric, 1);
            game.meta.set_weapon(Class::Cleric, 1);
        }
        "tickets" => {
            game.screen = Screen::Tickets;
            game.meta.ticket = 2;
        }
        "inventory" => {
            game.start_run(Class::Cleric);
            game.screen = Screen::Inventory;
            game.floor = 5;
            game.player.level = 7;
            game.player.gold = 126;
            game.equipped_attacks.clear();
            let base = attacks::starter(Class::Cleric);
            for step in [0usize, 10, 20, 30, 40, 50] {
                game.equipped_attacks
                    .push(attacks::EquippedAttack::new(base + step));
            }
            for (name, kind, rarity) in [
                ("Filo ardiente", ItemKind::Damage, Rarity::Rare),
                ("Sello de piedra", ItemKind::MaxHp, Rarity::Common),
                ("Anillo crítico", ItemKind::Crit, Rarity::Rare),
            ] {
                game.items.push(Item {
                    name,
                    kind,
                    rarity,
                    value: 12.0,
                });
            }
        }
        "play_cleric" => {
            game.start_run(Class::Cleric);
            game.floor = 3;
            game.player.level = 4;
            game.player.gold = 88;
            game.player.hp = 74.0;
            game.player.max_hp = 96.0;
        }
        "play_merchant" => {
            game.start_run(Class::Merchant);
            game.floor = 3;
            game.player.level = 4;
            game.player.gold = 154;
            game.player.hp = 62.0;
            game.player.max_hp = 84.0;
        }
        "stats" => {
            game.start_run(Class::Mage);
            game.screen = Screen::Stats;
            game.stats_return = Screen::Menu;
            game.floor = 6;
            game.player.level = 9;
            game.player.gold = 148;
            game.kills = 63;
            game.keys = 2;
            game.buff_stacks[BuffId::Fury as usize] = 2;
            game.buff_stacks[BuffId::Armor as usize] = 1;
            game.buff_stacks[BuffId::Wisdom as usize] = 1;
            game.items.push(Item {
                name: "Filo ardiente",
                kind: ItemKind::Damage,
                rarity: Rarity::Rare,
                value: 14.0,
            });
        }
        "map" => {
            game.start_run(Class::Knight);
            game.floor = 4;
            game.keys = 2;
            game.map_open = true;
            game.map_t = 1.0;
            for (i, room) in game.rooms.iter_mut().enumerate() {
                room.visited = i == 0 || i == 1 || i == 3 || i == 4 || i == 5;
                room.map_anim = 1.0;
            }
            if game.rooms.len() > 4 {
                game.room_index = 4;
            }
        }
        "play" => {
            game.start_run(Class::Knight);
            game.floor = 4;
            game.wave = 2;
            game.waves = 2;
            game.player.level = 5;
            game.player.xp = 48.0;
            game.player.xp_next = 90.0;
            game.player.gold = 137;
            game.player.hp = 104.0;
            game.player.max_hp = 130.0;
            game.kills = 42;
            game.keys = 2;
            game.modifier = Modifier::Fog;
            for (i, room) in game.rooms.iter_mut().enumerate() {
                room.visited = i < 4;
                room.map_anim = 1.0;
            }
            if let Some(room) = game.rooms.get_mut(game.room_index) {
                room.visited = true;
            }
            game.buff_stacks[BuffId::Fury as usize] = 2;
            game.buff_stacks[BuffId::Armor as usize] = 1;
            game.items.push(Item {
                name: "Filo ardiente",
                kind: ItemKind::Damage,
                rarity: Rarity::Rare,
                value: 14.0,
            });
            game.items.push(Item {
                name: "Sello de piedra",
                kind: ItemKind::MaxHp,
                rarity: Rarity::Common,
                value: 12.0,
            });
        }
        "shoproom" => {
            game.start_run(Class::Mage);
            for _ in 0..4 {
                game.next_floor();
            }
            game.player.level = 7;
            game.player.gold = 180;
            game.player.hp = 68.0;
            game.player.max_hp = 100.0;
            game.kills = 28;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Shop) {
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 140.0);
            }
        }
        "rest" => {
            game.start_run(Class::Knight);
            for _ in 0..2 {
                game.next_floor();
            }
            game.player.hp = 42.0;
            game.player.max_hp = 100.0;
            game.player.gold = 65;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Rest) {
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 196.0);
            }
        }
        "treasure" => {
            game.start_run(Class::Archer);
            game.next_floor();
            game.player.gold = 120;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Treasure) {
                for d in 0..4 {
                    game.rooms[i].locked[d] = false;
                }
                game.enter_room(i, 4);
                if let Some(chest) = game.props.iter().find(|p| p.kind == PropKind::Chest) {
                    game.player.pos = chest.pos + vec2(0.0, 34.0);
                }
            }
        }
        "chest_open" | "chest_mid" | "chest_closed" => {
            game.start_run(Class::Archer);
            game.next_floor();
            game.player.gold = 120;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Treasure) {
                for d in 0..4 {
                    game.rooms[i].locked[d] = false;
                }
                game.enter_room(i, 4);
                if let Some(chest) = game.props.iter_mut().find(|p| p.kind == PropKind::Chest) {
                    match shot {
                        "chest_open" => {
                            chest.opened = true;
                            chest.open_t = 1.0;
                        }
                        "chest_mid" => {
                            chest.opened = true;
                            chest.open_t = 0.45;
                        }
                        _ => {}
                    }
                    game.player.pos = chest.pos + vec2(0.0, 38.0);
                }
            }
        }
        "vault" => {
            game.start_run(Class::Necro);
            game.next_floor();
            game.player.gold = 90;
            game.special_keys = 1;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Vault) {
                for d in 0..4 {
                    game.rooms[i].locked[d] = false;
                }
                game.rooms[i].vault_open = true;
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 196.0);
            }
        }
        "bridge" | "doors" => {
            game.start_run(Class::Knight);
            game.floor = 3;
            for _ in 0..5 {
                game.next_floor();
            }
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Combat) {
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 180.0);
                game.door_seal = 1.0;
                game.door_slam_t = 0.0;
            }
        }
        "entry" => {
            game.start_run(Class::Mage);
            game.floor = 5;
            game.debt = 80;
            game.meta.debt = 80;
            game.meta.echoes = 2;
            game.meta.echo_floor = 7;
            game.meta.echo_class = 2;
            game.player.gold = 54;
            game.player.hp = 72.0;
            game.player.max_hp = 100.0;
            game.player.pos = vec2(240.0, 190.0);
        }
        "combat" | "combat_anim" | "combat_anim2" => {
            game.start_run(Class::Knight);
            for _ in 0..6 {
                game.next_floor();
            }
            game.player.level = 8;
            game.player.hp = 110.0;
            game.player.max_hp = 130.0;
            game.player.gold = 90;
            game.floor_element = Element::Frost;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Combat) {
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 210.0);
            }
            if shot != "combat" {
                // Pose fija para revisar el ciclo de caminata y el gesto de ataque.
                game.player.moving = true;
                game.player.walk_t = if shot == "combat_anim" { 1.3 } else { 2.6 };
                game.player.swing_t = if shot == "combat_anim" { 0.08 } else { 0.04 };
                for (i, e) in game.enemies.iter_mut().enumerate() {
                    e.walk_t = 1.0 + i as f32 * 0.4;
                    e.swing_t = if i == 0 { 0.11 } else { 0.0 };
                }
            }
        }
        "aura" => {
            game.start_run(Class::Knight);
            for _ in 0..6 {
                game.next_floor();
            }
            game.player.level = 8;
            game.player.hp = 110.0;
            game.player.max_hp = 130.0;
            game.player.gold = 90;
            game.floor_element = Element::Frost;
            game.equipped_attacks.clear();
            let base = attacks::starter(Class::Knight);
            game.equipped_attacks
                .push(attacks::EquippedAttack::new(base + 20));
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Combat) {
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 210.0);
            }
        }
        "boss" | "bossintro" | "boss_idol" | "boss_clear" => {
            game.start_run(Class::Knight);
            for _ in 0..9 {
                game.next_floor();
            }
            game.player.level = 10;
            game.player.gold = 95;
            game.player.hp = 118.0;
            game.player.max_hp = 150.0;
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Boss) {
                for d in 0..4 {
                    game.rooms[i].locked[d] = false;
                }
                game.rooms[i].cleared = false;
                game.cleared = false;
                game.wave = 1;
                game.waves = 1;
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 222.0);
                if shot == "boss_idol" {
                    game.floor_element = Element::Spirit;
                    game.boss_intro_skin = 9;
                    if let Some(b) = game.enemies.iter_mut().find(|e| e.boss) {
                        b.boss_skin = 9;
                    }
                }
                if shot == "boss_clear" {
                    game.enemies.clear();
                    game.cleared = true;
                    game.rooms[i].cleared = true;
                    game.player.pos = vec2(240.0, 180.0);
                }
                if shot == "bossintro" {
                    game.boss_intro_t = 2.55;
                } else {
                    // La captura de combate del jefe debe mostrar la pelea,
                    // no la cinemática de presentación.
                    game.boss_intro_t = 0.0;
                }
            }
        }
        "levelup" => {
            game.start_run(Class::Necro);
            game.screen = Screen::LevelUp;
            game.floor = 6;
            game.player.level = 9;
            game.player.hp = 96.0;
            game.player.max_hp = 150.0;
            game.player.gold = 88;
            game.level_choices = [
                LevelChoice::NewAttack(attacks::starter(Class::Necro) + 30),
                LevelChoice::UpgradeAttack(0),
                LevelChoice::Buff(buffs::POOL[20]),
            ];
            game.buff_stacks[0] = 2;
            game.buff_stacks[20] = 1;
        }
        "replace" => {
            game.start_run(Class::Necro);
            game.screen = Screen::LevelUp;
            game.floor = 6;
            game.player.level = 9;
            game.player.gold = 88;
            game.equipped_attacks.clear();
            let base = attacks::starter(Class::Necro);
            for step in [0usize, 10, 20, 30, 40, 50] {
                game.equipped_attacks
                    .push(attacks::EquippedAttack::new(base + step));
            }
            game.level_choices = [
                LevelChoice::NewAttack(base + 60),
                LevelChoice::UpgradeAttack(0),
                LevelChoice::Buff(buffs::POOL[20]),
            ];
            game.pending_replace = Some(base + 60);
            game.levelup_t = 1.0;
        }
        "pause" => {
            game.start_run(Class::Mage);
            game.screen = Screen::Paused;
            game.floor = 7;
            game.player.level = 8;
            game.player.gold = 212;
            game.kills = 76;
            game.run_gold = 212;
            game.buff_stacks[0] = 2;
            game.buff_stacks[8] = 1;
            game.buff_stacks[24] = 1;
            for (name, kind, rarity) in [
                ("Filo ardiente", ItemKind::Damage, Rarity::Rare),
                ("Botas de niebla", ItemKind::Speed, Rarity::Uncommon),
                ("Núcleo de cristal", ItemKind::CrystalHeart, Rarity::Epic),
                ("Anillo crítico", ItemKind::Crit, Rarity::Rare),
            ] {
                game.items.push(Item {
                    name,
                    kind,
                    rarity,
                    value: 12.0,
                });
            }
        }
        "dead" => {
            game.start_run(Class::Assassin);
            game.screen = Screen::Dead;
            game.floor = 7;
            game.kills = 89;
            game.run_gold = 214;
            game.souls_earned = 57;
            game.player.level = 8;
        }
        "victory" => {
            game.start_run(Class::Archer);
            game.screen = Screen::Victory;
            game.floor = 10;
            game.kills = 143;
            game.run_gold = 402;
            game.souls_earned = 120;
            game.player.level = 12;
        }
        _ if shot.starts_with("boss_skin_") => {
            // Revisión de las doce pieles de guardián con el mismo encuadre.
            let skin = shot
                .trim_start_matches("boss_skin_")
                .parse::<usize>()
                .unwrap_or(0)
                .min(11);
            game.start_run(Class::Knight);
            for _ in 0..9 {
                game.next_floor();
            }
            game.player.level = 10;
            game.player.gold = 95;
            game.player.hp = 118.0;
            game.player.max_hp = 150.0;
            game.floor_element = Element::ALL[skin % Element::ALL.len()];
            if let Some(i) = game.rooms.iter().position(|r| r.kind == RoomKind::Boss) {
                for d in 0..4 {
                    game.rooms[i].locked[d] = false;
                }
                game.rooms[i].cleared = false;
                game.cleared = false;
                game.wave = 1;
                game.waves = 1;
                game.enter_room(i, 4);
                game.player.pos = vec2(240.0, 236.0);
                game.boss_intro_skin = skin;
                if let Some(b) = game.enemies.iter_mut().find(|e| e.boss) {
                    b.boss_skin = skin;
                    b.tint = Element::ALL[skin % Element::ALL.len()].color();
                    b.walk_t = 1.1;
                    b.swing_t = 0.15;
                }
                game.boss_intro_t = 0.0;
            }
        }
        _ => {}
    }
    game.room_fade = 0.0;
    game.shake = 0.0;
    game.hitstop = 0.0;
    // La captura no ejecuta la simulación: los enemigos se quedarían en su
    // animación de aparición (suspendidos sobre la sala). Los asentamos para
    // fotografiar la sala ya en juego.
    for e in &mut game.enemies {
        e.spawn_t = 0.0;
    }
    if shot != "menu" {
        game.toasts.clear();
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // El mundo vuelve a su resolución lógica de 480x270 y se escala con filtro
    // nearest: los sprites de píxeles quedan nítidos y sin mezclas. La interfaz
    // se dibuja aparte, en resolución nativa, así que el texto sigue limpio.
    let target = render_target(VW as u32, VH as u32);
    target.texture.set_filter(FilterMode::Nearest);

    let fonts = UiFonts::load();

    let sprites = Sprites::load();
    let mut sfx = Sfx::load().await;
    let demo_mode = std::env::var("CRIPTA_DEMO").is_ok();
    let meta = if demo_mode {
        let mut m = save::Meta::default();
        m.name = "Lyra".to_owned();
        m.ticket = 0;
        m.music = false;
        m
    } else {
        save::load()
    };
    let mut game = Game::new(meta);
    sfx.set_volumes(game.meta.music_vol, game.meta.sfx_vol, game.meta.enemy_vol);
    sfx.set_music_enabled(game.music_on);

    if demo_mode {
        // La guía se graba sin audio del juego: la música se mezcla después
        // desde las capas exportadas y la narración se añade en postproducción.
        sfx.set_volumes(0, 0, 0);
        demo::run(&mut game, &sprites, &fonts, &target, &mut sfx).await;
        return;
    }

    if let Ok(path) = std::env::var("CRIPTA_SHOT") {
        let log_path = format!("{path}.log");
        let _ = std::fs::write(&log_path, "shot: inicio\n");
        let hook_path = log_path.clone();
        std::panic::set_hook(Box::new(move |info| {
            let _ = std::fs::write(&hook_path, format!("shot: panic\n{info}\n"));
        }));

        let screen = std::env::var("CRIPTA_SHOT_SCREEN").unwrap_or_else(|_| "play".to_owned());
        prepare_shot(&mut game, &screen);

        // Deja que se rastericen fuentes y texturas antes de capturar.
        for _ in 0..8 {
            next_frame().await;
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
        if screen == "replace" {
            // Coloca el cursor sobre la primera carta para ver el detalle de
            // sustitución en la captura.
            game.view.mx = ox + 324.0 * scale;
            game.view.my = oy + 99.0 * scale;
        }

        // Primer dibujo descartado: fuerza a rasterizar todos los glifos y
        // tamaños de fuente antes de la captura definitiva. Sin esto, la
        // primera línea de texto del HUD puede perderse al crecer el atlas.
        draw_frame(&game, &sprites, &fonts, &target);
        next_frame().await;

        let _ = std::fs::write(&log_path, format!("shot: draw {sw}x{sh} escala {scale}\n"));
        draw_frame(&game, &sprites, &fonts, &target);
        let image = get_screen_data();
        let _ = std::fs::write(&log_path, "shot: pantalla capturada\n");
        image.export_png(&path);
        let _ = std::fs::write(&log_path, "shot: png exportado\n");
        std::process::exit(0);
    }

    let mut last = get_time();
    let mut fps_acc = 0.0f32;
    let mut fps_frames = 0u32;

    let mut fullscreen = false;
    loop {
        if is_key_pressed(KeyCode::F11) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }

        let now = get_time();
        let frame = ((now - last) as f32).clamp(0.0, 0.05);
        last = now;

        let sw = screen_width();
        let sh = screen_height();
        let scale = ((sw / VW).min(sh / VH)).max(0.01);
        let ox = (sw - VW * scale) * 0.5;
        let oy = (sh - VH * scale) * 0.5;
        let (mx, my) = mouse_position();
        game.view = View {
            scale,
            ox,
            oy,
            mx,
            my,
        };

        // Una sola actualización por fotograma: las pulsaciones se leen una
        // vez y los menús responden al instante sin repetir acciones.
        game.update(frame, &mut sfx);

        fps_acc += frame;
        fps_frames += 1;
        if fps_acc >= 0.5 {
            game.fps = fps_frames as f32 / fps_acc;
            game.frame_ms = fps_acc / fps_frames as f32 * 1000.0;
            fps_acc = 0.0;
            fps_frames = 0;
        }

        draw_frame(&game, &sprites, &fonts, &target);

        next_frame().await;
    }
}
