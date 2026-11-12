//! Figuras procedurales de CRIPTA.
//!
//! Nada de sprites de píxeles: héroes, enemigos y guardianes se dibujan con
//! formas suaves, extremidades articuladas y armas visibles. La animación de
//! caminata mueve las piernas de verdad y los ataques tienen un gesto claro.

use crate::game::{EnemyKind, PropKind};
use crate::types::{with_alpha, Class};
use macroquad::prelude::*;

const C_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const C_GOLD: Color = Color::new(0.96, 0.78, 0.34, 1.0);
const C_RED: Color = Color::new(0.86, 0.26, 0.30, 1.0);
const C_GREEN: Color = Color::new(0.42, 0.85, 0.48, 1.0);
const C_CYAN: Color = Color::new(0.40, 0.90, 0.90, 1.0);
const C_PURPLE: Color = Color::new(0.72, 0.44, 0.92, 1.0);

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn darken(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

fn lighten(c: Color, k: f32) -> Color {
    mix(c, Color::new(1.0, 0.96, 0.88, 1.0), k)
}

fn alpha(c: Color, a: f32) -> Color {
    with_alpha(c, c.a * a)
}

fn capsule(a: Vec2, b: Vec2, w: f32, color: Color) {
    draw_line(a.x, a.y, b.x, b.y, w, color);
    draw_circle(a.x, a.y, w * 0.5, color);
    draw_circle(b.x, b.y, w * 0.5, color);
}

fn glow(pos: Vec2, radius: f32, color: Color, a: f32) {
    draw_circle(pos.x, pos.y, radius, with_alpha(color, a * 0.16));
    draw_circle(pos.x, pos.y, radius * 0.62, with_alpha(color, a * 0.22));
    draw_circle(pos.x, pos.y, radius * 0.34, with_alpha(color, a * 0.40));
}

fn eye(pos: Vec2, r: f32, look: Vec2, iris: Color) {
    draw_circle(pos.x, pos.y, r, Color::new(0.98, 0.96, 0.90, 1.0));
    draw_circle(
        pos.x + look.x * r * 0.30,
        pos.y + look.y * r * 0.30,
        r * 0.55,
        iris,
    );
    draw_circle(
        pos.x + look.x * r * 0.34,
        pos.y + look.y * r * 0.34,
        r * 0.26,
        Color::new(0.04, 0.03, 0.06, 1.0),
    );
    draw_circle(
        pos.x + look.x * r * 0.22 - r * 0.18,
        pos.y + look.y * r * 0.22 - r * 0.20,
        r * 0.14,
        Color::new(1.0, 1.0, 1.0, 0.95),
    );
}

/// Héroe pixel art con contorno, sombreado y animación por pasos.
pub fn draw_hero(
    pos: Vec2,
    class: Class,
    facing: Vec2,
    walk_t: f32,
    moving: bool,
    swing_t: f32,
    hit_flash: f32,
    skin_tint: Color,
    hat: u8,
    mask: u8,
    time: f32,
) {
    crate::pixel::draw_hero(
        pos, class, facing, walk_t, moving, swing_t, hit_flash, skin_tint, hat, mask, time,
    );
}

/// Versión suave anterior, conservada como referencia y para comparar.
#[allow(dead_code)]
fn draw_hero_smooth(
    pos: Vec2,
    class: Class,
    facing: Vec2,
    walk_t: f32,
    moving: bool,
    swing_t: f32,
    hit_flash: f32,
    skin_tint: Color,
    _hat: u8,
    _mask: u8,
    time: f32,
) {
    let flash = (hit_flash / 0.18).clamp(0.0, 1.0);
    let shake = if flash > 0.0 {
        vec2(
            (time * 82.0).sin() * 2.2 * flash,
            (time * 61.0).cos() * 1.2 * flash,
        )
    } else {
        Vec2::ZERO
    };
    let base = pos + shake;
    let side = if facing.x.abs() > 0.08 {
        facing.x.signum()
    } else {
        1.0
    };
    let pace = if moving { 1.0 } else { 0.18 };
    let phase = walk_t;
    let stride = phase.sin() * 4.6 * pace;
    let lift_a = (phase.sin().max(0.0)) * 2.2 * pace;
    let lift_b = ((-phase).sin().max(0.0)) * 2.2 * pace;
    let breathe = (time * 2.4).sin() * 0.45;
    let accent = if skin_tint.r > 0.97 && skin_tint.g > 0.97 && skin_tint.b > 0.97 {
        class.color()
    } else {
        skin_tint
    };
    let cloth = mix(accent, Color::new(0.10, 0.09, 0.14, 1.0), 0.45);
    let cloth_dark = darken(cloth, 0.62);
    let leather = Color::new(0.22, 0.16, 0.13, 1.0);
    let steel = mix(Color::new(0.66, 0.70, 0.78, 1.0), C_WHITE, flash * 0.5);
    let gold = Color::new(0.88, 0.70, 0.34, 1.0);
    let skin = mix(Color::new(0.87, 0.70, 0.56, 1.0), C_WHITE, flash * 0.6);
    let hip = base + vec2(0.0, -1.0 + breathe * 0.3);
    let shoulder = base + vec2(0.0, -11.0 + breathe);
    let head = base + vec2(side * 0.6, -17.0 + breathe);

    // Piernas: dos cápsulas con rodilla y bota. El balanceo es visible incluso
    // de pie, pero mucho más marcado al caminar.
    let foot_a = base + vec2(-3.2 - stride * 0.55, 8.0 - lift_a);
    let foot_b = base + vec2(3.2 + stride * 0.55, 8.0 - lift_b);
    let knee_a = (hip + vec2(-3.0, 5.0) + foot_a) * 0.5 + vec2(-0.8, 0.0);
    let knee_b = (hip + vec2(3.0, 5.0) + foot_b) * 0.5 + vec2(0.8, 0.0);
    capsule(hip + vec2(-3.0, 0.0), knee_a, 3.4, cloth_dark);
    capsule(knee_a, foot_a, 2.9, darken(cloth, 0.48));
    capsule(hip + vec2(3.0, 0.0), knee_b, 3.4, cloth);
    capsule(knee_b, foot_b, 2.9, cloth);
    // Botas.
    capsule(
        foot_a + vec2(-0.5, 0.0),
        foot_a + vec2(2.2, 0.0),
        3.4,
        leather,
    );
    capsule(
        foot_b + vec2(-0.5, 0.0),
        foot_b + vec2(2.2, 0.0),
        3.4,
        leather,
    );

    // Torso y capa.
    draw_triangle(
        shoulder + vec2(-6.0, 1.0),
        shoulder + vec2(6.0, 1.0),
        hip + vec2(8.0, 3.0),
        cloth_dark,
    );
    draw_triangle(
        shoulder + vec2(-6.0, 1.0),
        hip + vec2(8.0, 3.0),
        hip + vec2(-8.0, 3.0),
        cloth,
    );
    draw_triangle(
        shoulder + vec2(-5.0, 2.0),
        shoulder + vec2(5.0, 2.0),
        hip + vec2(4.0, 1.0),
        lighten(cloth, 0.10 + flash * 0.4),
    );
    // Peto/coraza según clase.
    if matches!(class, Class::Knight | Class::Cleric) {
        draw_circle(shoulder.x, shoulder.y + 3.0, 5.4, steel);
        draw_rectangle(shoulder.x - 5.0, shoulder.y + 2.0, 10.0, 7.0, steel);
        draw_rectangle(shoulder.x - 5.0, shoulder.y + 2.0, 10.0, 1.6, C_WHITE);
        draw_rectangle(shoulder.x - 1.0, shoulder.y + 3.0, 2.0, 6.0, gold);
    } else if class == Class::Merchant {
        draw_circle(shoulder.x, shoulder.y + 3.0, 5.2, leather);
        draw_rectangle(shoulder.x - 5.0, shoulder.y + 2.0, 10.0, 7.0, leather);
        draw_rectangle(shoulder.x - 4.0, shoulder.y + 5.0, 8.0, 1.5, gold);
    } else {
        draw_circle(shoulder.x, shoulder.y + 3.0, 4.6, lighten(cloth, 0.14));
        draw_line(
            shoulder.x - 4.0,
            shoulder.y + 2.0,
            shoulder.x + 4.0,
            shoulder.y + 8.0,
            0.8,
            with_alpha(C_WHITE, 0.22),
        );
    }
    // Cinturón.
    draw_rectangle(base.x - 6.0, base.y - 2.0, 12.0, 2.2, leather);
    draw_rectangle(base.x - 1.2, base.y - 2.2, 2.4, 2.6, gold);

    // Cabeza y rasgos. El casco/capucha se dibuja detrás de la cara.
    let hood = match class {
        Class::Knight => steel,
        Class::Mage | Class::Necro => darken(cloth, 0.72),
        Class::Assassin => darken(cloth, 0.55),
        Class::Cleric => lighten(cloth, 0.16),
        _ => mix(cloth, Color::new(0.30, 0.20, 0.16, 1.0), 0.55),
    };
    draw_circle(head.x, head.y + 0.5, 6.4, hood);
    if class == Class::Knight {
        draw_circle(head.x, head.y, 5.0, steel);
        draw_rectangle(
            head.x - 4.6,
            head.y - 1.4,
            9.2,
            1.4,
            Color::new(0.10, 0.10, 0.14, 1.0),
        );
        draw_rectangle(head.x - 1.0, head.y - 6.0, 2.0, 5.0, gold);
    } else {
        draw_circle(head.x, head.y + 0.8, 4.6, skin);
        let look = safe_dir(facing);
        eye(
            head + vec2(-1.5 + look.x * 0.7, -0.2 + look.y * 0.6),
            1.15,
            look,
            Color::new(0.18, 0.13, 0.20, 1.0),
        );
        eye(
            head + vec2(1.5 + look.x * 0.7, -0.2 + look.y * 0.6),
            1.15,
            look,
            Color::new(0.18, 0.13, 0.20, 1.0),
        );
        if class == Class::Assassin {
            draw_rectangle(head.x - 4.8, head.y - 2.0, 9.6, 2.0, darken(cloth, 0.40));
        }
        if class == Class::Mage {
            draw_triangle(
                head + vec2(-6.0, -2.0),
                head + vec2(6.0, -2.0),
                head + vec2(0.0, -11.0),
                darken(cloth, 0.55),
            );
            draw_circle(head.x, head.y - 10.0, 1.2, C_CYAN);
        }
    }

    // Arma y brazo principal. El gesto de ataque barre hacia delante.
    let swing = (swing_t / 0.16).clamp(0.0, 1.0);
    let swing_arc = (swing * std::f32::consts::PI).sin();
    let hand_base = shoulder + vec2(side * 7.0, 4.0);
    let hand = hand_base + vec2(side * swing_arc * 4.0, -swing_arc * 2.2);
    capsule(shoulder + vec2(side * 4.5, 2.0), hand, 2.8, skin);
    let offhand = shoulder + vec2(-side * 6.0, 5.0);

    match class {
        Class::Knight => {
            // Escudo y espada.
            draw_circle(offhand.x, offhand.y, 5.2, darken(steel, 0.55));
            draw_circle(offhand.x, offhand.y, 4.4, steel);
            draw_circle(offhand.x, offhand.y, 2.4, gold);
            draw_line(
                offhand.x - 2.0,
                offhand.y,
                offhand.x + 2.0,
                offhand.y,
                0.8,
                Color::new(0.18, 0.15, 0.20, 1.0),
            );
            let dir = vec2(side * (0.45 + swing_arc * 0.55), -0.9 + swing_arc * 0.35);
            let tip = hand + dir.normalize() * (15.0 + swing_arc * 3.0);
            capsule(hand, tip, 2.2, steel);
            draw_line(hand.x, hand.y, tip.x, tip.y, 0.7, C_WHITE);
            draw_line(
                hand.x - dir.y * 3.0,
                hand.y + dir.x * 3.0,
                hand.x + dir.y * 3.0,
                hand.y - dir.x * 3.0,
                1.6,
                gold,
            );
            draw_circle(hand.x, hand.y, 2.0, gold);
        }
        Class::Archer => {
            // Arco curvo y flecha.
            let bow_c = hand + vec2(side * 3.0, 0.0);
            let up = bow_c + vec2(side * 3.2, -8.0);
            let down = bow_c + vec2(side * 3.2, 8.0);
            let mid = bow_c + vec2(side * 7.0, 0.0);
            for i in 0..6 {
                let t0 = i as f32 / 6.0;
                let t1 = (i + 1) as f32 / 6.0;
                let a = quad_bezier(up, mid, down, t0);
                let b = quad_bezier(up, mid, down, t1);
                draw_line(a.x, a.y, b.x, b.y, 1.5, Color::new(0.48, 0.30, 0.16, 1.0));
            }
            draw_line(up.x, up.y, down.x, down.y, 0.6, with_alpha(C_WHITE, 0.65));
            if swing_arc > 0.05 {
                let tip = hand + vec2(side * (10.0 + swing_arc * 8.0), 0.0);
                draw_line(hand.x, hand.y, tip.x, tip.y, 1.4, lighten(accent, 0.4));
                draw_triangle(
                    tip,
                    tip - vec2(side * 3.0, 1.5),
                    tip - vec2(side * 3.0, -1.5),
                    C_WHITE,
                );
            }
            draw_circle(offhand.x, offhand.y, 2.0, leather);
        }
        Class::Mage | Class::Necro | Class::Cleric => {
            let staff_col = if class == Class::Cleric {
                gold
            } else if class == Class::Necro {
                Color::new(0.42, 0.36, 0.30, 1.0)
            } else {
                Color::new(0.52, 0.38, 0.26, 1.0)
            };
            let top = hand + vec2(side * (4.0 + swing_arc * 3.0), -17.0 - swing_arc * 2.0);
            capsule(hand + vec2(0.0, 6.0), top, 1.8, staff_col);
            let orb_col = match class {
                Class::Mage => C_CYAN,
                Class::Necro => C_GREEN,
                _ => C_GOLD,
            };
            glow(top, 9.0, orb_col, 0.8);
            draw_circle(top.x, top.y, 3.4, orb_col);
            draw_circle(top.x - 0.8, top.y - 1.0, 1.2, C_WHITE);
            if class == Class::Necro {
                draw_circle(
                    top.x - 1.8,
                    top.y + 0.2,
                    0.8,
                    Color::new(0.06, 0.05, 0.08, 1.0),
                );
                draw_circle(
                    top.x + 1.8,
                    top.y + 0.2,
                    0.8,
                    Color::new(0.06, 0.05, 0.08, 1.0),
                );
            }
            if class == Class::Cleric {
                draw_line(top.x - 3.0, top.y, top.x + 3.0, top.y, 1.2, C_WHITE);
                draw_line(top.x, top.y - 3.0, top.x, top.y + 3.0, 1.2, C_WHITE);
            }
            draw_circle(offhand.x, offhand.y, 2.0, skin);
        }
        Class::Assassin => {
            let dir = vec2(side * (0.55 + swing_arc * 0.45), -0.35 + swing_arc * 0.2);
            let tip = hand + dir.normalize() * (9.0 + swing_arc * 4.0);
            capsule(hand, tip, 1.8, steel);
            draw_triangle(
                tip,
                tip - vec2(side * 2.4, 1.2),
                tip - vec2(side * 2.4, -1.2),
                C_WHITE,
            );
            let tip2 = offhand + vec2(-side * (7.0 + swing_arc * 3.0), 1.0);
            capsule(offhand, tip2, 1.6, steel);
            draw_circle(offhand.x, offhand.y, 1.8, darken(cloth, 0.4));
            if swing_arc > 0.1 {
                draw_arc(
                    hand.x,
                    hand.y,
                    10,
                    -0.9,
                    1.2,
                    11.0,
                    1.0,
                    with_alpha(lighten(accent, 0.4), swing_arc * 0.55),
                );
            }
        }
        Class::Merchant => {
            let tip = hand + vec2(side * (8.0 + swing_arc * 3.0), -4.0);
            capsule(hand, tip, 1.8, steel);
            draw_circle(hand.x, hand.y, 1.9, gold);
            draw_circle(offhand.x, offhand.y, 4.2, leather);
            draw_circle(offhand.x, offhand.y, 3.0, gold);
            draw_line(
                offhand.x - 2.0,
                offhand.y - 1.0,
                offhand.x + 2.0,
                offhand.y - 1.0,
                0.7,
                Color::new(0.45, 0.30, 0.10, 1.0),
            );
        }
    }

    // Estela del golpe: un arco de energía en el lado del arma.
    if swing_arc > 0.08 && matches!(class, Class::Knight | Class::Assassin | Class::Merchant) {
        let arc_color = with_alpha(lighten(accent, 0.35), swing_arc * 0.55);
        draw_arc(
            shoulder.x + side * 6.0,
            shoulder.y + 2.0,
            12,
            -1.1,
            0.9,
            15.0,
            1.4,
            arc_color,
        );
    }
    // Un destello blanco al recibir daño.
    if flash > 0.0 {
        draw_circle(
            base.x,
            base.y - 8.0,
            10.0 + flash * 3.0,
            Color::new(1.0, 0.35, 0.35, 0.18 * flash),
        );
    }
}

fn quad_bezier(a: Vec2, b: Vec2, c: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    a * (u * u) + b * (2.0 * u * t) + c * (t * t)
}

fn safe_dir(v: Vec2) -> Vec2 {
    if v.length_squared() < 0.001 {
        vec2(0.0, 1.0)
    } else {
        v.normalize()
    }
}

/// Dibuja un enemigo. Todos usan el renderizador pixel art: los normales
/// comparten lienzo pequeño y los jefes tienen siluetas propias por guardián.
pub fn draw_enemy(
    pos: Vec2,
    kind: EnemyKind,
    radius: f32,
    scale: f32,
    tint: Color,
    boss_skin: usize,
    elite: bool,
    shiny: bool,
    boss: bool,
    walk_t: f32,
    swing_t: f32,
    hit_flash: f32,
    death_alpha: f32,
    time: f32,
) {
    if boss {
        crate::pixel::draw_boss(
            pos,
            boss_skin,
            scale,
            tint,
            elite,
            shiny,
            walk_t,
            swing_t,
            hit_flash,
            death_alpha,
            time,
        );
    } else {
        crate::pixel::draw_enemy(
            pos,
            kind,
            radius,
            scale,
            tint,
            elite,
            shiny,
            walk_t,
            swing_t,
            hit_flash,
            death_alpha,
            time,
        );
    }
}

#[allow(dead_code)]
fn draw_enemy_smooth(
    pos: Vec2,
    kind: EnemyKind,
    radius: f32,
    scale: f32,
    tint: Color,
    boss_skin: usize,
    elite: bool,
    shiny: bool,
    boss: bool,
    walk_t: f32,
    swing_t: f32,
    hit_flash: f32,
    death_alpha: f32,
    time: f32,
) {
    let flash = (hit_flash / 0.18).clamp(0.0, 1.0);
    let mut body = mix(tint, Color::new(1.0, 1.0, 1.0, 1.0), flash * 0.75);
    if shiny {
        body = mix(body, C_CYAN, 0.35);
    }
    let dark = darken(body, 0.55);
    let accent = if elite { C_GOLD } else { lighten(tint, 0.25) };
    let a = death_alpha.clamp(0.0, 1.0);
    let body = alpha(body, a);
    let dark = alpha(dark, a);
    let accent = alpha(accent, a);
    let swing = (swing_t / if boss { 0.30 } else { 0.22 }).clamp(0.0, 1.0);
    let swing_arc = (swing * std::f32::consts::PI).sin();

    if boss {
        draw_boss(
            pos, radius, scale, body, dark, accent, boss_skin, walk_t, swing_arc, a, time,
        );
        return;
    }

    match kind {
        EnemyKind::Slime => {
            let squash = 1.0 + (walk_t * 2.2).sin() * 0.08;
            let bob = (walk_t * 2.2).cos() * 1.0;
            draw_ellipse(
                pos.x,
                pos.y + 4.0,
                radius * 1.15 * scale,
                radius * 0.55 * scale,
                0.0,
                with_alpha(dark, 0.55),
            );
            draw_circle(
                pos.x,
                pos.y + 1.0 + bob,
                radius * 0.92 * scale * squash,
                body,
            );
            draw_circle(
                pos.x - radius * 0.28,
                pos.y - radius * 0.15 + bob,
                radius * 0.36 * scale,
                lighten(body, 0.25),
            );
            let look = vec2(0.0, 0.7);
            eye(
                pos + vec2(-radius * 0.32 * scale, -radius * 0.2 * scale + bob),
                radius * 0.18 * scale,
                look,
                Color::new(0.15, 0.35, 0.20, 1.0),
            );
            eye(
                pos + vec2(radius * 0.32 * scale, -radius * 0.2 * scale + bob),
                radius * 0.18 * scale,
                look,
                Color::new(0.15, 0.35, 0.20, 1.0),
            );
            draw_line(
                pos.x - radius * 0.2 * scale,
                pos.y + radius * 0.28 * scale + bob,
                pos.x + radius * 0.2 * scale,
                pos.y + radius * 0.28 * scale + bob,
                0.8,
                with_alpha(dark, 0.8),
            );
        }
        EnemyKind::Bat => {
            let flap = (walk_t * 7.0).sin();
            let wing = 7.0 * scale;
            let body_c = pos + vec2(0.0, -4.0 * scale);
            draw_triangle(
                body_c,
                body_c + vec2(-wing, -wing * 0.6 * flap),
                body_c + vec2(-wing * 0.4, wing * 0.55),
                with_alpha(dark, 0.92),
            );
            draw_triangle(
                body_c,
                body_c + vec2(wing, -wing * 0.6 * flap),
                body_c + vec2(wing * 0.4, wing * 0.55),
                with_alpha(body, 0.92),
            );
            draw_circle(body_c.x, body_c.y, 4.2 * scale, body);
            draw_triangle(
                body_c + vec2(-3.2, -2.0) * scale,
                body_c + vec2(-1.2, -7.0) * scale,
                body_c + vec2(-0.2, -1.2) * scale,
                dark,
            );
            draw_triangle(
                body_c + vec2(3.2, -2.0) * scale,
                body_c + vec2(1.2, -7.0) * scale,
                body_c + vec2(0.2, -1.2) * scale,
                dark,
            );
            eye(
                body_c + vec2(-1.5, 0.2) * scale,
                1.2 * scale,
                vec2(0.0, 0.4),
                C_RED,
            );
            eye(
                body_c + vec2(1.5, 0.2) * scale,
                1.2 * scale,
                vec2(0.0, 0.4),
                C_RED,
            );
        }
        EnemyKind::Brute => {
            let stride = (walk_t * 4.2).sin() * 3.2;
            let hip = pos + vec2(0.0, -5.0 * scale);
            let foot_a = pos + vec2(-4.0 - stride * 0.5, 7.0) * scale;
            let foot_b = pos + vec2(4.0 + stride * 0.5, 7.0) * scale;
            capsule(hip + vec2(-3.0, 0.0) * scale, foot_a, 4.6 * scale, dark);
            capsule(hip + vec2(3.0, 0.0) * scale, foot_b, 4.6 * scale, body);
            draw_circle(pos.x, pos.y - 9.0 * scale, 10.5 * scale, body);
            draw_circle(
                pos.x - 3.0 * scale,
                pos.y - 12.0 * scale,
                6.5 * scale,
                lighten(body, 0.12),
            );
            draw_circle(
                pos.x + 3.0 * scale,
                pos.y - 12.0 * scale,
                6.5 * scale,
                lighten(body, 0.12),
            );
            // Cuernos.
            draw_triangle(
                pos + vec2(-7.0, -15.0) * scale,
                pos + vec2(-11.0, -23.0) * scale,
                pos + vec2(-3.0, -17.0) * scale,
                accent,
            );
            draw_triangle(
                pos + vec2(7.0, -15.0) * scale,
                pos + vec2(11.0, -23.0) * scale,
                pos + vec2(3.0, -17.0) * scale,
                accent,
            );
            eye(
                pos + vec2(-3.5, -13.0) * scale,
                1.4 * scale,
                vec2(0.0, 0.6),
                C_RED,
            );
            eye(
                pos + vec2(3.5, -13.0) * scale,
                1.4 * scale,
                vec2(0.0, 0.6),
                C_RED,
            );
            // Garrote en la mano derecha.
            let hand =
                pos + vec2(9.0, -8.0) * scale + vec2(swing_arc * 5.0, -swing_arc * 2.0) * scale;
            capsule(pos + vec2(6.0, -10.0) * scale, hand, 2.6 * scale, body);
            let club_tip = hand + vec2(5.0 + swing_arc * 5.0, -7.0) * scale;
            capsule(hand, club_tip, 3.2 * scale, Color::new(0.30, 0.22, 0.18, a));
            draw_circle(
                club_tip.x,
                club_tip.y,
                4.2 * scale,
                Color::new(0.36, 0.27, 0.21, a),
            );
        }
        EnemyKind::Cultist => {
            let stride = (walk_t * 3.4).sin() * 2.2;
            let foot_a = pos + vec2(-3.0 - stride * 0.5, 7.0) * scale;
            let foot_b = pos + vec2(3.0 + stride * 0.5, 7.0) * scale;
            capsule(pos + vec2(-2.0, -3.0) * scale, foot_a, 2.4 * scale, dark);
            capsule(pos + vec2(2.0, -3.0) * scale, foot_b, 2.4 * scale, dark);
            draw_triangle(
                pos + vec2(0.0, -22.0) * scale,
                pos + vec2(-8.0, 4.0) * scale,
                pos + vec2(8.0, 4.0) * scale,
                body,
            );
            draw_triangle(
                pos + vec2(0.0, -19.0) * scale,
                pos + vec2(-5.0, 2.0) * scale,
                pos + vec2(5.0, 2.0) * scale,
                dark,
            );
            draw_circle(pos.x, pos.y - 17.0 * scale, 5.6 * scale, dark);
            draw_circle(
                pos.x,
                pos.y - 16.0 * scale,
                4.2 * scale,
                Color::new(0.06, 0.05, 0.08, a),
            );
            eye(
                pos + vec2(-1.6, -16.0) * scale,
                1.1 * scale,
                vec2(0.0, 0.4),
                C_RED,
            );
            eye(
                pos + vec2(1.6, -16.0) * scale,
                1.1 * scale,
                vec2(0.0, 0.4),
                C_RED,
            );
            let hand = pos + vec2(8.0, -8.0) * scale;
            let top = hand + vec2(2.0 + swing_arc * 4.0, -14.0) * scale;
            capsule(hand, top, 1.4 * scale, Color::new(0.35, 0.26, 0.18, a));
            glow(top, 7.0 * scale, accent, 0.55);
            draw_circle(top.x, top.y, 2.4 * scale, accent);
        }
        EnemyKind::Wraith => {
            let hover = (time * 2.6 + walk_t).sin() * 2.0;
            let base = pos + vec2(0.0, hover);
            for k in 0..5 {
                let x = base.x - 9.0 * scale + k as f32 * 4.5 * scale;
                let sway = (time * 4.0 + k as f32).sin() * 2.2 * scale;
                draw_triangle(
                    vec2(x, base.y - 8.0 * scale),
                    vec2(x - 2.5 * scale, base.y + 8.0 * scale),
                    vec2(x + 2.5 * scale + sway, base.y + 10.0 * scale),
                    with_alpha(body, 0.65),
                );
            }
            draw_circle(
                base.x,
                base.y - 9.0 * scale,
                7.0 * scale,
                with_alpha(body, 0.92),
            );
            draw_circle(base.x, base.y - 11.0 * scale, 5.0 * scale, dark);
            eye(
                base + vec2(-2.0, -11.0) * scale,
                1.3 * scale,
                vec2(0.0, 0.5),
                C_CYAN,
            );
            eye(
                base + vec2(2.0, -11.0) * scale,
                1.3 * scale,
                vec2(0.0, 0.5),
                C_CYAN,
            );
            for k in 0..3 {
                let ang = time * 2.0 + k as f32 * 2.1;
                draw_circle(
                    base.x + ang.cos() * 10.0 * scale,
                    base.y + ang.sin() * 5.0 * scale,
                    1.1 * scale,
                    with_alpha(accent, 0.55),
                );
            }
        }
        EnemyKind::Bomber => {
            let stride = (walk_t * 5.0).sin() * 2.0;
            let foot_a = pos + vec2(-3.0 - stride * 0.5, 7.0) * scale;
            let foot_b = pos + vec2(3.0 + stride * 0.5, 7.0) * scale;
            capsule(pos + vec2(-2.0, -2.0) * scale, foot_a, 2.2 * scale, dark);
            capsule(pos + vec2(2.0, -2.0) * scale, foot_b, 2.2 * scale, dark);
            draw_circle(pos.x, pos.y - 7.0 * scale, 8.2 * scale, body);
            draw_circle(
                pos.x - 2.0 * scale,
                pos.y - 10.0 * scale,
                4.2 * scale,
                lighten(body, 0.18),
            );
            draw_circle(pos.x - 3.0 * scale, pos.y - 7.0 * scale, 1.4 * scale, C_RED);
            draw_circle(pos.x + 3.0 * scale, pos.y - 7.0 * scale, 1.4 * scale, C_RED);
            let fuse = pos + vec2(2.0, -14.0) * scale;
            draw_line(
                pos.x + 2.0 * scale,
                pos.y - 13.0 * scale,
                fuse.x,
                fuse.y,
                1.2 * scale,
                Color::new(0.42, 0.36, 0.30, a),
            );
            let spark = 0.7 + (time * 18.0).sin() * 0.3;
            glow(fuse, 5.0 * scale, C_GOLD, spark);
            draw_circle(fuse.x, fuse.y, 1.8 * scale, C_GOLD);
        }
        EnemyKind::Sentinel => {
            let stride = (walk_t * 3.6).sin() * 2.8;
            let foot_a = pos + vec2(-3.0 - stride * 0.5, 7.0) * scale;
            let foot_b = pos + vec2(3.0 + stride * 0.5, 7.0) * scale;
            capsule(pos + vec2(-2.5, -3.0) * scale, foot_a, 3.0 * scale, dark);
            capsule(pos + vec2(2.5, -3.0) * scale, foot_b, 3.0 * scale, dark);
            draw_rectangle(
                pos.x - 7.0 * scale,
                pos.y - 17.0 * scale,
                14.0 * scale,
                15.0 * scale,
                body,
            );
            draw_circle(
                pos.x,
                pos.y - 17.0 * scale,
                5.2 * scale,
                lighten(body, 0.15),
            );
            draw_rectangle(
                pos.x - 4.0 * scale,
                pos.y - 18.0 * scale,
                8.0 * scale,
                1.8 * scale,
                Color::new(0.05, 0.04, 0.07, a),
            );
            draw_rectangle(
                pos.x - 1.0 * scale,
                pos.y - 22.0 * scale,
                2.0 * scale,
                5.0 * scale,
                accent,
            );
            // Escudo y lanza.
            draw_circle(pos.x - 9.0 * scale, pos.y - 8.0 * scale, 5.4 * scale, dark);
            draw_circle(pos.x - 9.0 * scale, pos.y - 8.0 * scale, 4.2 * scale, body);
            draw_circle(
                pos.x - 9.0 * scale,
                pos.y - 8.0 * scale,
                1.8 * scale,
                accent,
            );
            let spear = pos + vec2(9.0, -24.0 + swing_arc * 3.0) * scale;
            capsule(
                pos + vec2(8.0, 2.0) * scale,
                spear,
                1.4 * scale,
                Color::new(0.55, 0.50, 0.44, a),
            );
            draw_triangle(
                spear + vec2(0.0, -4.0) * scale,
                spear + vec2(-2.2, 1.0) * scale,
                spear + vec2(2.2, 1.0) * scale,
                accent,
            );
        }
        EnemyKind::Eye => {
            let hover = (time * 3.0 + walk_t).sin() * 1.8;
            let c = pos + vec2(0.0, -7.0 + hover);
            for k in 0..6 {
                let ang = k as f32 * std::f32::consts::TAU / 6.0 + time * 0.6;
                let end = c + vec2(ang.cos() * 11.0, ang.sin() * 8.0) * scale;
                draw_line(c.x, c.y, end.x, end.y, 1.4 * scale, with_alpha(dark, 0.75));
                draw_circle(end.x, end.y, 1.2 * scale, with_alpha(accent, 0.7));
            }
            draw_circle(c.x, c.y, 8.0 * scale, body);
            draw_circle(c.x, c.y, 6.8 * scale, Color::new(0.97, 0.94, 0.86, a));
            let look = vec2((time * 1.4).sin() * 0.6, (time * 1.1).cos() * 0.4);
            draw_circle(c.x + look.x * 2.2, c.y + look.y * 2.2, 3.8 * scale, accent);
            draw_circle(
                c.x + look.x * 2.8,
                c.y + look.y * 2.8,
                2.0 * scale,
                Color::new(0.05, 0.03, 0.07, a),
            );
            draw_circle(
                c.x - 2.2 * scale,
                c.y - 2.6 * scale,
                1.2 * scale,
                with_alpha(C_WHITE, 0.85),
            );
        }
        EnemyKind::Boss => {}
    }

    // Gesto de disparo: un pulso de energía recorre el cuerpo al lanzar un
    // proyectil, para que el ataque a distancia también se lea de un vistazo.
    if swing_arc > 0.05 {
        let c = pos + vec2(0.0, -radius * 0.75 * scale);
        draw_circle_lines(
            c.x,
            c.y,
            radius * (0.95 + swing_arc * 0.85) * scale,
            1.2,
            with_alpha(accent, 0.55 * swing_arc),
        );
        draw_circle(
            c.x,
            c.y,
            radius * 0.55 * scale,
            with_alpha(accent, 0.10 * swing_arc),
        );
    }
}

fn draw_boss(
    pos: Vec2,
    radius: f32,
    scale: f32,
    body: Color,
    dark: Color,
    accent: Color,
    skin: usize,
    walk_t: f32,
    swing: f32,
    a: f32,
    time: f32,
) {
    let s = scale.max(1.2) * (radius / 20.0).clamp(0.85, 1.3);
    let stride = (walk_t * 3.0).sin() * 3.4;
    let hover = (time * 2.0).sin() * 1.6;
    let legs = |body: Color, dark: Color| {
        let foot_a = pos + vec2(-6.0 - stride * 0.6, 10.0) * s;
        let foot_b = pos + vec2(6.0 + stride * 0.6, 10.0) * s;
        capsule(pos + vec2(-5.0, -2.0) * s, foot_a, 5.0 * s, dark);
        capsule(pos + vec2(5.0, -2.0) * s, foot_b, 5.0 * s, body);
        capsule(
            foot_a + vec2(-1.0, 0.0) * s,
            foot_a + vec2(3.0, 0.0) * s,
            5.2 * s,
            darken(body, 0.55),
        );
        capsule(
            foot_b + vec2(-1.0, 0.0) * s,
            foot_b + vec2(3.0, 0.0) * s,
            5.2 * s,
            body,
        );
    };

    match skin % 12 {
        0 => {
            // Rey calavera: corona, costillas y capa.
            draw_triangle(
                pos + vec2(0.0, -24.0) * s,
                pos + vec2(-16.0, 8.0) * s,
                pos + vec2(16.0, 8.0) * s,
                with_alpha(dark, 0.75),
            );
            legs(body, dark);
            draw_circle(pos.x, pos.y - 15.0 * s, 10.0 * s, body);
            draw_circle(
                pos.x,
                pos.y - 15.0 * s,
                8.0 * s,
                Color::new(0.93, 0.90, 0.80, a),
            );
            for k in 0..4 {
                draw_line(
                    pos.x - 6.0 * s + k as f32 * 4.0 * s,
                    pos.y - 6.0 * s,
                    pos.x - 5.0 * s + k as f32 * 4.0 * s,
                    pos.y + 1.0 * s,
                    1.4 * s,
                    with_alpha(body, 0.85),
                );
            }
            eye(pos + vec2(-3.4, -16.0) * s, 2.0 * s, vec2(0.0, 0.6), C_RED);
            eye(pos + vec2(3.4, -16.0) * s, 2.0 * s, vec2(0.0, 0.6), C_RED);
            draw_rectangle(
                pos.x - 2.0 * s,
                pos.y - 11.0 * s,
                4.0 * s,
                1.4 * s,
                Color::new(0.10, 0.08, 0.10, a),
            );
            for k in 0..5 {
                let x = pos.x - 8.0 * s + k as f32 * 4.0 * s;
                draw_triangle(
                    vec2(x, pos.y - 23.0 * s),
                    vec2(x + 3.0 * s, pos.y - 23.0 * s),
                    vec2(x + 1.5 * s, pos.y - 29.0 * s),
                    accent,
                );
            }
            let hand = pos + vec2(15.0, -8.0) * s + vec2(swing * 4.0, 0.0) * s;
            capsule(pos + vec2(9.0, -10.0) * s, hand, 2.4 * s, body);
            capsule(hand, hand + vec2(9.0, -12.0) * s, 2.0 * s, C_WHITE);
        }
        1 => {
            // Bruto cornado: moles, cuernos y maza.
            legs(body, dark);
            draw_circle(pos.x, pos.y - 13.0 * s, 15.0 * s, body);
            draw_circle(
                pos.x - 6.0 * s,
                pos.y - 16.0 * s,
                8.0 * s,
                lighten(body, 0.12),
            );
            draw_circle(
                pos.x + 6.0 * s,
                pos.y - 16.0 * s,
                8.0 * s,
                lighten(body, 0.12),
            );
            draw_triangle(
                pos + vec2(-10.0, -23.0) * s,
                pos + vec2(-20.0, -33.0) * s,
                pos + vec2(-4.0, -26.0) * s,
                accent,
            );
            draw_triangle(
                pos + vec2(10.0, -23.0) * s,
                pos + vec2(20.0, -33.0) * s,
                pos + vec2(4.0, -26.0) * s,
                accent,
            );
            eye(pos + vec2(-5.0, -18.0) * s, 2.0 * s, vec2(0.0, 0.6), C_GOLD);
            eye(pos + vec2(5.0, -18.0) * s, 2.0 * s, vec2(0.0, 0.6), C_GOLD);
            draw_rectangle(
                pos.x - 5.0 * s,
                pos.y - 9.0 * s,
                10.0 * s,
                1.6 * s,
                Color::new(0.15, 0.07, 0.07, a),
            );
            let hand = pos + vec2(18.0, -9.0) * s;
            capsule(pos + vec2(12.0, -12.0) * s, hand, 3.6 * s, body);
            let club = hand + vec2(9.0 + swing * 5.0, -15.0) * s;
            capsule(hand, club, 4.4 * s, Color::new(0.32, 0.24, 0.20, a));
            draw_circle(club.x, club.y, 6.0 * s, Color::new(0.40, 0.30, 0.24, a));
        }
        2 => {
            // Espectro alado: alas amplias y mirada espectral.
            let wing = (time * 2.4).sin() * 4.0;
            draw_triangle(
                pos + vec2(-6.0, -18.0) * s,
                pos + vec2(-30.0, -30.0 - wing) * s,
                pos + vec2(-18.0, -2.0) * s,
                with_alpha(dark, 0.75),
            );
            draw_triangle(
                pos + vec2(6.0, -18.0) * s,
                pos + vec2(30.0, -30.0 - wing) * s,
                pos + vec2(18.0, -2.0) * s,
                with_alpha(body, 0.75),
            );
            for k in 0..5 {
                draw_triangle(
                    pos + vec2(-4.0 + k as f32 * 4.0, -12.0) * s,
                    pos + vec2(-6.0 + k as f32 * 4.0, 10.0 + hover) * s,
                    pos + vec2(-1.0 + k as f32 * 4.0, 12.0 + hover) * s,
                    with_alpha(body, 0.50),
                );
            }
            draw_circle(pos.x, pos.y - 17.0 * s, 9.0 * s, dark);
            draw_circle(pos.x, pos.y - 17.0 * s, 7.0 * s, body);
            eye(pos + vec2(-3.0, -18.0) * s, 2.0 * s, vec2(0.0, 0.5), C_CYAN);
            eye(pos + vec2(3.0, -18.0) * s, 2.0 * s, vec2(0.0, 0.5), C_CYAN);
            draw_triangle(
                pos + vec2(0.0, -8.0) * s,
                pos + vec2(-6.0, 4.0) * s,
                pos + vec2(6.0, 4.0) * s,
                with_alpha(dark, 0.8),
            );
        }
        3 => {
            // Orbe vidente: un gran ojo flotante con tentáculos.
            let c = pos + vec2(0.0, -15.0 + hover) * s;
            for k in 0..8 {
                let ang = k as f32 * std::f32::consts::TAU / 8.0 + time * 0.4;
                let end = c + vec2(ang.cos() * 22.0, ang.sin() * 15.0) * s;
                draw_line(c.x, c.y, end.x, end.y, 2.4 * s, with_alpha(dark, 0.75));
                draw_circle(end.x, end.y, 2.0 * s, with_alpha(accent, 0.75));
            }
            draw_circle(c.x, c.y, 15.0 * s, body);
            draw_circle(c.x, c.y, 12.5 * s, Color::new(0.97, 0.94, 0.86, a));
            let look = vec2((time * 1.3).sin() * 0.7, (time * 1.0).cos() * 0.5);
            draw_circle(c.x + look.x * 4.0, c.y + look.y * 4.0, 7.0 * s, accent);
            draw_circle(
                c.x + look.x * 5.0,
                c.y + look.y * 5.0,
                3.6 * s,
                Color::new(0.04, 0.03, 0.06, a),
            );
            draw_circle(
                c.x - 4.0 * s,
                c.y - 5.0 * s,
                2.0 * s,
                with_alpha(C_WHITE, 0.85),
            );
            for k in 0..10 {
                let ang = k as f32 * std::f32::consts::TAU / 10.0 + time * 1.5;
                draw_circle(
                    c.x + ang.cos() * 19.0 * s,
                    c.y + ang.sin() * 13.0 * s,
                    1.3 * s,
                    with_alpha(accent, 0.60),
                );
            }
        }
        4 => {
            // Serpiente: cuerpo en espiral, capucha y lengua.
            for k in 0..8 {
                let ang = k as f32 * 0.7 + time * 1.1;
                let rr = (3.0 + k as f32 * 2.4) * s;
                draw_circle(
                    pos.x + ang.cos() * rr,
                    pos.y + 6.0 * s + ang.sin() * rr * 0.32,
                    6.0 * s - k as f32 * 0.4 * s,
                    if k % 2 == 0 { body } else { dark },
                );
            }
            let head = pos + vec2(0.0, -17.0) * s;
            draw_circle(head.x, head.y, 10.0 * s, body);
            draw_triangle(
                head + vec2(-11.0, -2.0) * s,
                head + vec2(0.0, -13.0) * s,
                head + vec2(0.0, 2.0) * s,
                accent,
            );
            draw_triangle(
                head + vec2(11.0, -2.0) * s,
                head + vec2(0.0, -13.0) * s,
                head + vec2(0.0, 2.0) * s,
                accent,
            );
            eye(head + vec2(-4.0, -1.0) * s, 1.8 * s, vec2(0.0, 0.6), C_RED);
            eye(head + vec2(4.0, -1.0) * s, 1.8 * s, vec2(0.0, 0.6), C_RED);
            let tongue = head + vec2(0.0, 8.0) * s;
            draw_line(head.x, head.y + 5.0 * s, tongue.x, tongue.y, 1.2 * s, C_RED);
            draw_line(
                tongue.x,
                tongue.y,
                tongue.x - 2.0 * s,
                tongue.y + 3.0 * s,
                1.0 * s,
                C_RED,
            );
            draw_line(
                tongue.x,
                tongue.y,
                tongue.x + 2.0 * s,
                tongue.y + 3.0 * s,
                1.0 * s,
                C_RED,
            );
        }
        5 => {
            // Coloso de piedra: torso enorme y puños.
            legs(dark, dark);
            draw_circle(pos.x, pos.y - 15.0 * s, 17.0 * s, body);
            draw_circle(
                pos.x - 8.0 * s,
                pos.y - 19.0 * s,
                9.0 * s,
                lighten(body, 0.10),
            );
            draw_circle(
                pos.x + 8.0 * s,
                pos.y - 19.0 * s,
                9.0 * s,
                lighten(body, 0.10),
            );
            draw_rectangle(pos.x - 12.0 * s, pos.y - 8.0 * s, 24.0 * s, 8.0 * s, dark);
            eye(pos + vec2(-5.0, -18.0) * s, 1.8 * s, vec2(0.0, 0.5), C_GOLD);
            eye(pos + vec2(5.0, -18.0) * s, 1.8 * s, vec2(0.0, 0.5), C_GOLD);
            let fist = pos + vec2(22.0 + swing * 5.0, -6.0) * s;
            capsule(pos + vec2(13.0, -12.0) * s, fist, 6.0 * s, body);
            draw_circle(fist.x, fist.y, 7.0 * s, darken(body, 0.80));
            for k in 0..4 {
                draw_line(
                    pos.x - 10.0 * s + k as f32 * 6.0 * s,
                    pos.y - 25.0 * s,
                    pos.x - 8.0 * s + k as f32 * 6.0 * s,
                    pos.y - 15.0 * s,
                    0.8 * s,
                    with_alpha(C_WHITE, 0.16),
                );
            }
        }
        6 => {
            // Obelisco: monolito flotante con runas y ojo central.
            draw_triangle(
                pos + vec2(0.0, -34.0 + hover) * s,
                pos + vec2(-12.0, 8.0) * s,
                pos + vec2(12.0, 8.0) * s,
                body,
            );
            draw_triangle(
                pos + vec2(0.0, -30.0 + hover) * s,
                pos + vec2(-7.0, 4.0) * s,
                pos + vec2(7.0, 4.0) * s,
                dark,
            );
            for k in 0..4 {
                let y = pos.y - 24.0 * s + k as f32 * 8.0 * s + hover;
                draw_line(
                    pos.x - 5.0 * s,
                    y,
                    pos.x + 5.0 * s,
                    y,
                    1.0 * s,
                    with_alpha(accent, 0.70),
                );
            }
            eye(
                pos + vec2(0.0, -12.0 + hover) * s,
                3.4 * s,
                vec2(0.0, 0.6),
                C_CYAN,
            );
            for k in 0..6 {
                let ang = k as f32 * std::f32::consts::TAU / 6.0 + time * 0.8;
                draw_circle(
                    pos.x + ang.cos() * 18.0 * s,
                    pos.y - 12.0 * s + ang.sin() * 12.0 * s,
                    1.4 * s,
                    with_alpha(accent, 0.55),
                );
            }
        }
        7 => {
            // Reina araña: abdomen, ocho patas y corona.
            for k in 0..8 {
                let side = if k < 4 { -1.0 } else { 1.0 };
                let idx = (k % 4) as f32;
                let ang = -0.9 + idx * 0.55;
                let foot = pos + vec2(side * (12.0 + idx * 2.0), 6.0 + idx * 1.5) * s;
                let knee = pos + vec2(side * (10.0 + idx * 3.0), -10.0 + idx * 2.0) * s;
                capsule(pos + vec2(side * 5.0, -2.0) * s, knee, 2.2 * s, dark);
                capsule(knee, foot, 1.8 * s, body);
                draw_circle(foot.x, foot.y, 1.2 * s, accent);
                let _ = ang;
            }
            draw_ellipse(pos.x, pos.y + 2.0 * s, 13.0 * s, 9.0 * s, 0.0, body);
            draw_circle(pos.x, pos.y - 12.0 * s, 9.0 * s, dark);
            draw_circle(pos.x, pos.y - 13.0 * s, 7.0 * s, body);
            for k in 0..4 {
                eye(
                    pos + vec2(-5.0 + k as f32 * 3.4, -14.0) * s,
                    1.2 * s,
                    vec2(0.0, 0.6),
                    C_RED,
                );
            }
            for k in 0..5 {
                let x = pos.x - 7.0 * s + k as f32 * 3.5 * s;
                draw_triangle(
                    vec2(x, pos.y - 20.0 * s),
                    vec2(x + 2.6 * s, pos.y - 20.0 * s),
                    vec2(x + 1.3 * s, pos.y - 26.0 * s),
                    accent,
                );
            }
        }
        8 => {
            // Juez del farol: túnica alta, capucha y farol.
            let stride2 = (walk_t * 2.4).sin() * 2.0;
            capsule(
                pos + vec2(-4.0, -3.0) * s,
                pos + vec2(-5.0 - stride2, 10.0) * s,
                3.2 * s,
                dark,
            );
            capsule(
                pos + vec2(4.0, -3.0) * s,
                pos + vec2(5.0 + stride2, 10.0) * s,
                3.2 * s,
                dark,
            );
            draw_triangle(
                pos + vec2(0.0, -31.0) * s,
                pos + vec2(-14.0, 9.0) * s,
                pos + vec2(14.0, 9.0) * s,
                body,
            );
            draw_triangle(
                pos + vec2(0.0, -27.0) * s,
                pos + vec2(-8.0, 6.0) * s,
                pos + vec2(8.0, 6.0) * s,
                dark,
            );
            draw_circle(pos.x, pos.y - 26.0 * s, 7.0 * s, dark);
            draw_circle(
                pos.x,
                pos.y - 25.0 * s,
                5.4 * s,
                Color::new(0.05, 0.04, 0.07, a),
            );
            eye(pos + vec2(-2.2, -25.0) * s, 1.3 * s, vec2(0.0, 0.5), C_GOLD);
            eye(pos + vec2(2.2, -25.0) * s, 1.3 * s, vec2(0.0, 0.5), C_GOLD);
            let hand = pos + vec2(16.0, -12.0) * s;
            capsule(pos + vec2(10.0, -16.0) * s, hand, 2.0 * s, body);
            let lantern = hand + vec2(4.0, 4.0) * s;
            glow(lantern, 11.0 * s, C_GOLD, 0.75);
            draw_rectangle(
                lantern.x - 3.4 * s,
                lantern.y - 4.0 * s,
                6.8 * s,
                8.0 * s,
                Color::new(0.10, 0.08, 0.05, a),
            );
            draw_rectangle(
                lantern.x - 2.4 * s,
                lantern.y - 2.8 * s,
                4.8 * s,
                5.6 * s,
                with_alpha(C_GOLD, 0.85),
            );
            draw_line(
                lantern.x,
                lantern.y - 4.0 * s,
                lantern.x,
                lantern.y - 8.0 * s,
                1.0 * s,
                accent,
            );
        }
        9 => {
            // Ídolo doble: monolito de piedra con dos rostros tallados. La
            // piedra se tiñe con el elemento y la energía sale por las runas,
            // así no parece un rectángulo plano de color.
            let stone = mix(body, Color::new(0.10, 0.095, 0.13, 1.0), 0.76);
            let stone_mid = lighten(stone, 0.16);
            let stone_hi = lighten(stone, 0.30);
            let socket = Color::new(0.030, 0.026, 0.045, a);
            let energy = with_alpha(accent, 0.90);
            glow(pos + vec2(0.0, -18.0 + hover) * s, 30.0 * s, accent, 0.20);
            draw_ellipse(
                pos.x,
                pos.y + 9.0 * s,
                18.0 * s,
                4.6 * s,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.42 * a),
            );
            // Plinto escalonado.
            draw_rectangle(pos.x - 14.0 * s, pos.y + 1.0 * s, 28.0 * s, 8.0 * s, stone);
            draw_rectangle(
                pos.x - 14.0 * s,
                pos.y + 1.0 * s,
                28.0 * s,
                1.8 * s,
                stone_hi,
            );
            draw_rectangle(
                pos.x - 11.0 * s,
                pos.y + 6.4 * s,
                22.0 * s,
                2.4 * s,
                darken(stone, 0.72),
            );
            for k in 0..3 {
                draw_circle(
                    pos.x - 7.0 * s + k as f32 * 7.0 * s,
                    pos.y + 5.0 * s,
                    1.0 * s,
                    with_alpha(accent, 0.55),
                );
            }
            // Monolito hexagonal: cara lateral oscura, cuerpo y bisel.
            draw_poly(
                pos.x - 1.8 * s,
                pos.y - 16.0 * s + hover,
                6,
                17.5 * s,
                0.0,
                darken(stone, 0.60),
            );
            draw_poly(pos.x, pos.y - 16.0 * s + hover, 6, 17.5 * s, 0.0, stone);
            draw_poly(pos.x, pos.y - 18.0 * s + hover, 6, 13.4 * s, 0.0, stone_mid);
            // Dos rostros tallados en paneles hundidos.
            for k in 0..2 {
                let cy = pos.y - 23.0 * s + k as f32 * 15.0 * s + hover * 0.5;
                draw_rectangle(pos.x - 9.2 * s, cy - 5.0 * s, 18.4 * s, 10.0 * s, socket);
                draw_rectangle(pos.x - 8.4 * s, cy - 4.4 * s, 16.8 * s, 8.8 * s, stone_mid);
                draw_rectangle(pos.x - 8.4 * s, cy - 4.4 * s, 16.8 * s, 1.0 * s, stone_hi);
                // Cejas.
                draw_rectangle(
                    pos.x - 6.4 * s,
                    cy - 2.9 * s,
                    4.8 * s,
                    1.2 * s,
                    darken(stone, 0.45),
                );
                draw_rectangle(
                    pos.x + 1.6 * s,
                    cy - 2.9 * s,
                    4.8 * s,
                    1.2 * s,
                    darken(stone, 0.45),
                );
                // Cuencas y mirada.
                for side in [-1.0f32, 1.0] {
                    draw_circle(pos.x + side * 4.0 * s, cy - 0.7 * s, 2.1 * s, socket);
                    draw_circle(pos.x + side * 4.0 * s, cy - 0.7 * s, 1.15 * s, energy);
                    draw_circle(
                        pos.x + side * 4.0 * s - 0.35 * s,
                        cy - 1.05 * s,
                        0.38 * s,
                        with_alpha(C_WHITE, 0.95),
                    );
                }
                // Boca tallada con dientes.
                draw_rectangle(pos.x - 3.6 * s, cy + 2.5 * s, 7.2 * s, 1.8 * s, socket);
                for t in 0..3 {
                    draw_rectangle(
                        pos.x - 2.7 * s + t as f32 * 2.1 * s,
                        cy + 2.5 * s,
                        0.9 * s,
                        1.8 * s,
                        stone_hi,
                    );
                }
            }
            // Grietas y runas encendidas.
            draw_line(
                pos.x - 10.0 * s,
                pos.y - 6.0 * s,
                pos.x - 7.0 * s,
                pos.y + 0.5 * s,
                0.7 * s,
                darken(stone, 0.45),
            );
            draw_line(
                pos.x + 8.5 * s,
                pos.y - 8.0 * s,
                pos.x + 6.5 * s,
                pos.y - 2.0 * s,
                0.7 * s,
                darken(stone, 0.45),
            );
            draw_line(
                pos.x - 1.0 * s,
                pos.y - 12.0 * s,
                pos.x + 1.0 * s,
                pos.y - 8.0 * s,
                0.6 * s,
                energy,
            );
            // Esquirlas de piedra orbitando.
            for k in 0..6 {
                let ang = k as f32 * std::f32::consts::TAU / 6.0 + time * 0.5;
                let sh = pos + vec2(ang.cos() * 21.0, -13.0 + ang.sin() * 14.0) * s;
                draw_poly(sh.x, sh.y, 4, 1.9 * s, ang, with_alpha(accent, 0.65));
            }
        }
        10 => {
            // Boca del vacío: esfera oscura con anillo de dientes.
            let c = pos + vec2(0.0, -16.0 + hover) * s;
            draw_circle(c.x, c.y, 18.0 * s, Color::new(0.025, 0.015, 0.04, a));
            draw_circle_lines(c.x, c.y, 18.0 * s, 2.0 * s, with_alpha(accent, 0.55));
            for k in 0..14 {
                let ang = k as f32 * std::f32::consts::TAU / 14.0 + time * 0.3;
                let inner = c + vec2(ang.cos(), ang.sin()) * 10.0 * s;
                let outer = c + vec2(ang.cos(), ang.sin()) * 17.0 * s;
                draw_triangle(
                    inner,
                    outer + vec2(-ang.sin(), ang.cos()) * 2.0 * s,
                    outer - vec2(-ang.sin(), ang.cos()) * 2.0 * s,
                    with_alpha(Color::new(0.92, 0.88, 0.82, 1.0), a),
                );
            }
            draw_circle(c.x, c.y, 7.0 * s, Color::new(0.01, 0.005, 0.02, a));
            for k in 0..3 {
                let ang = time * 1.4 + k as f32 * 2.1;
                draw_circle(
                    c.x + ang.cos() * 12.0 * s,
                    c.y + ang.sin() * 12.0 * s,
                    1.4 * s,
                    with_alpha(accent, 0.65),
                );
            }
        }
        _ => {
            // Mano astral: palma y dedos que se cierran al atacar.
            let c = pos + vec2(0.0, -18.0 + hover) * s;
            for k in 0..5 {
                let x = -10.0 + k as f32 * 5.0;
                let spread = (k as f32 - 2.0) * 2.0;
                let tip = c + vec2(x + spread + swing * 3.0, -14.0 - (k as f32 % 2.0) * 3.0) * s;
                capsule(c + vec2(x, 2.0) * s, tip, 2.4 * s, body);
                draw_circle(tip.x, tip.y, 2.6 * s, accent);
            }
            draw_circle(c.x, c.y, 11.0 * s, body);
            draw_circle(c.x, c.y, 8.0 * s, dark);
            eye(c + vec2(0.0, 0.0) * s, 3.2 * s, vec2(0.0, 0.5), C_PURPLE);
            for k in 0..8 {
                let ang = k as f32 * std::f32::consts::TAU / 8.0 + time * 0.6;
                draw_circle(
                    c.x + ang.cos() * 20.0 * s,
                    c.y + ang.sin() * 16.0 * s,
                    1.2 * s,
                    with_alpha(accent, 0.55),
                );
            }
        }
    }
}

/// Portal del guardián: aro de piedra con runas y un vórtice de energía.
/// Sustituye al antiguo sprite de píxeles para mantener el mismo lenguaje
/// visual suave que el resto de figuras.
pub fn draw_portal(pos: Vec2, active: bool, time: f32) {
    let a = if active { 1.0 } else { 0.55 };
    draw_ellipse(
        pos.x,
        pos.y + 10.0,
        19.0,
        6.0,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35 * a),
    );
    if active {
        for i in 0..4 {
            let t = i as f32 / 4.0;
            draw_circle(
                pos.x,
                pos.y,
                24.0 + t * 9.0,
                with_alpha(Color::new(0.55, 0.32, 1.0, 1.0), 0.07 * (1.0 - t)),
            );
        }
    }
    // Aro de piedra con remaches dorados.
    draw_circle(pos.x, pos.y, 21.0, Color::new(0.14, 0.12, 0.19, a));
    draw_circle(pos.x, pos.y, 18.0, Color::new(0.055, 0.045, 0.09, a));
    draw_circle_lines(pos.x, pos.y, 20.0, 1.0, with_alpha(C_GOLD, 0.55 * a));
    for k in 0..8 {
        let ang = k as f32 * std::f32::consts::TAU / 8.0;
        draw_circle(
            pos.x + ang.cos() * 20.0,
            pos.y + ang.sin() * 20.0,
            1.1,
            with_alpha(C_GOLD, 0.60 * a),
        );
    }
    if active {
        let spin = time * 1.7;
        for arm in 0..3 {
            let base = spin + arm as f32 * std::f32::consts::TAU / 3.0;
            let col = match arm {
                0 => C_CYAN,
                1 => Color::new(0.62, 0.36, 1.0, 1.0),
                _ => C_GOLD,
            };
            for j in 0..6 {
                let rr = 4.0 + j as f32 * 2.5;
                let ang = base + j as f32 * 0.42;
                draw_circle(
                    pos.x + ang.cos() * rr,
                    pos.y + ang.sin() * rr * 0.92,
                    (2.3 - j as f32 * 0.22).max(0.7),
                    with_alpha(col, 0.75 - j as f32 * 0.08),
                );
            }
        }
        draw_circle(
            pos.x,
            pos.y,
            6.5,
            with_alpha(Color::new(0.78, 0.68, 1.0, 1.0), 0.85),
        );
        draw_circle(pos.x, pos.y, 3.6, with_alpha(C_WHITE, 0.95));
        for k in 0..7 {
            let ang = time * 1.2 + k as f32 * std::f32::consts::TAU / 7.0;
            let rr = 22.0 + (time * 3.0 + k as f32).sin() * 2.0;
            draw_circle(
                pos.x + ang.cos() * rr,
                pos.y + ang.sin() * rr,
                0.9,
                with_alpha(C_CYAN, 0.70),
            );
        }
    } else {
        // Portal sellado: piedra agrietada, sin energía.
        draw_circle(pos.x, pos.y, 17.0, Color::new(0.045, 0.038, 0.07, a));
        for k in 0..4 {
            let ang = k as f32 * std::f32::consts::TAU / 4.0 + 0.6;
            draw_line(
                pos.x + ang.cos() * 4.0,
                pos.y + ang.sin() * 4.0,
                pos.x + ang.cos() * 15.0,
                pos.y + ang.sin() * 15.0,
                0.7,
                Color::new(0.16, 0.14, 0.22, a),
            );
        }
        draw_circle(pos.x, pos.y, 3.0, Color::new(0.10, 0.09, 0.14, a));
    }
}

/// Cofre de madera oscura con refuerzos dorados y cerradura, como la
/// referencia: tapa abombada, dos bandas metálicas y brillo interior.
pub fn draw_chest(pos: Vec2, opened: bool, open_t: f32, time: f32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let w = 26.0;
    let h = 15.0;
    let wood = Color::new(0.22, 0.12, 0.06, a);
    let wood_mid = Color::new(0.31, 0.18, 0.09, a);
    let wood_hi = Color::new(0.46, 0.28, 0.14, a);
    let outline = Color::new(0.08, 0.045, 0.02, a);
    let gold = with_alpha(C_GOLD, a);
    let dark_gold = Color::new(0.55, 0.38, 0.12, a);
    let shine = (time * 2.2).sin() * 0.5 + 0.5;
    let lid_y = pos.y - 1.0;
    let lift = if opened { open_t.clamp(0.0, 1.0) } else { 0.0 };

    // Sombra de contacto.
    draw_ellipse(
        pos.x,
        pos.y + 8.0,
        w * 0.62,
        3.8,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.38 * a),
    );

    if opened {
        // Tapa abierta de pie DETRÁS del cofre. La silueta es abovedada, como
        // la tapa cerrada, para que se lea al instante: marco de madera, forro
        // granate acolchado, remaches y bisagras doradas. Se dibuja antes del
        // cuerpo para que este tape su base.
        let open_h = 14.5 + lift * 1.5;
        let ly = lid_y - open_h;
        let half = w * 0.5 + 1.0;
        let arch_h = 4.5;
        let cx = pos.x;

        // Hueco oscuro entre la tapa y el cuerpo.
        draw_rectangle(
            cx - half,
            lid_y - 1.6,
            half * 2.0,
            2.6,
            Color::new(0.03, 0.015, 0.008, a),
        );

        // Contorno exterior: arco superior + tablero.
        draw_ellipse(cx, ly + arch_h, half + 0.9, arch_h + 0.9, 0.0, outline);
        draw_rectangle(
            cx - half - 0.9,
            ly + arch_h,
            half * 2.0 + 1.8,
            open_h - arch_h + 0.9,
            outline,
        );
        // Madera de la tapa.
        draw_ellipse(cx, ly + arch_h, half, arch_h, 0.0, wood);
        draw_rectangle(
            cx - half,
            ly + arch_h,
            half * 2.0,
            open_h - arch_h + 0.6,
            wood,
        );
        // Luz en el canto superior.
        draw_ellipse(
            cx,
            ly + arch_h - 0.6,
            half - 1.0,
            arch_h - 1.2,
            0.0,
            wood_hi,
        );

        // Marco dorado del forro.
        let gy = ly + arch_h + 0.4;
        let gx = half - 2.2;
        let gry = arch_h - 2.0;
        draw_ellipse(cx, gy, gx, gry, 0.0, dark_gold);
        draw_rectangle(cx - gx, gy, gx * 2.0, lid_y - gy - 0.4, dark_gold);
        // Forro de terciopelo granate.
        let vy = gy + 0.3;
        let vx = gx - 0.7;
        let vry = (gry - 0.7).max(0.8);
        let velvet = Color::new(0.33, 0.07, 0.10, a);
        draw_ellipse(cx, vy, vx, vry, 0.0, velvet);
        draw_rectangle(cx - vx, vy, vx * 2.0, lid_y - vy - 0.8, velvet);
        // Botones del acolchado.
        let b1 = ly + arch_h + 3.4;
        let b2 = b1 + 3.8;
        for by in [b1, b2] {
            for bx in [cx - 6.2, cx, cx + 6.2] {
                draw_circle(bx, by, 1.0, Color::new(0.20, 0.04, 0.06, a));
                draw_circle(bx, by, 0.6, with_alpha(C_GOLD, 0.72 * a));
            }
        }
        // Resplandor cálido del tesoro.
        glow(
            pos + vec2(0.0, -2.0),
            18.0 + lift * 8.0,
            C_GOLD,
            0.20 + lift * 0.14,
        );
    }

    // Cuerpo de madera oscura con dos tonos.
    draw_rectangle(pos.x - w * 0.5, pos.y - 1.0, w, h, wood_mid);
    draw_rectangle(pos.x - w * 0.5, pos.y + h * 0.42, w, h * 0.58 - 1.0, wood);
    draw_rectangle(pos.x - w * 0.5, pos.y - 1.0, w, 1.4, wood_hi);
    draw_rectangle_lines(pos.x - w * 0.5, pos.y - 1.0, w, h, 1.0, outline);
    // Vetas y tablones.
    for k in 0..3 {
        draw_line(
            pos.x - w * 0.44,
            pos.y + 1.8 + k as f32 * 3.8,
            pos.x + w * 0.44,
            pos.y + 1.8 + k as f32 * 3.8,
            0.5,
            Color::new(0.13, 0.07, 0.035, a * 0.75),
        );
    }
    for k in 0..2 {
        let x = pos.x - w * 0.16 + k as f32 * w * 0.32;
        draw_line(
            x,
            pos.y + 1.0,
            x,
            pos.y + h - 1.0,
            0.5,
            Color::new(0.16, 0.09, 0.045, a * 0.6),
        );
    }

    if opened {
        // Abertura: hueco profundo con el tesoro dentro. El borde inferior es
        // el labio frontal del cofre y separa el interior del frente de madera.
        let ox = pos.x - w * 0.44;
        let oy = lid_y - 2.4;
        let ow = w * 0.88;
        let oh = 6.8;
        let bottom = oy + oh;
        draw_rectangle(ox, oy, ow, oh, Color::new(0.045, 0.020, 0.010, a));
        // Pared trasera y laterales, para dar profundidad.
        draw_rectangle(
            ox + 0.8,
            oy + 0.8,
            ow - 1.6,
            1.5,
            Color::new(0.24, 0.10, 0.045, a),
        );
        draw_rectangle(
            ox + 0.6,
            oy + 1.6,
            1.2,
            oh - 1.6,
            Color::new(0.14, 0.06, 0.03, a),
        );
        draw_rectangle(
            ox + ow - 1.8,
            oy + 1.6,
            1.2,
            oh - 1.6,
            Color::new(0.14, 0.06, 0.03, a),
        );
        draw_rectangle_lines(ox, oy, ow, oh, 1.0, outline);
        // Bisagras doradas en el borde trasero, a los lados de la abertura.
        for hx in [pos.x - w * 0.44 - 0.9, pos.x + w * 0.44 + 0.9] {
            draw_rectangle(hx - 1.5, lid_y - 2.8, 3.0, 3.8, outline);
            draw_rectangle(hx - 1.1, lid_y - 2.5, 2.2, 3.2, dark_gold);
            draw_rectangle(hx - 1.1, lid_y - 2.5, 2.2, 0.8, gold);
            draw_circle(hx, lid_y - 1.0, 0.55, outline);
        }
        // Labio frontal: remate superior del frente del cofre.
        draw_rectangle(pos.x - w * 0.5, bottom, w, 1.6, wood_hi);
        draw_line(
            pos.x - w * 0.5,
            bottom,
            pos.x + w * 0.5,
            bottom,
            0.8,
            outline,
        );
        draw_line(
            pos.x - w * 0.5,
            bottom + 1.6,
            pos.x + w * 0.5,
            bottom + 1.6,
            0.5,
            Color::new(0.10, 0.05, 0.02, a * 0.8),
        );

        // Montón de monedas, sin llenar todo el hueco.
        let pile = bottom - 1.0;
        draw_ellipse(
            pos.x + 0.5,
            pile,
            8.4,
            2.2,
            0.0,
            Color::new(0.42, 0.27, 0.08, a),
        );
        draw_circle(pos.x - 5.0, pile - 1.0, 2.4, dark_gold);
        draw_circle(pos.x + 0.5, pile - 1.4, 3.0, gold);
        draw_circle(pos.x + 5.6, pile - 0.8, 2.1, dark_gold);
        draw_circle(pos.x - 5.0, pile - 1.4, 1.0, with_alpha(C_WHITE, 0.35 * a));
        draw_circle(pos.x + 0.5, pile - 1.9, 1.2, with_alpha(C_WHITE, 0.50 * a));
        // Monedas sueltas.
        for (dx, dy, r) in [
            (-8.2, -2.2, 0.9),
            (-3.0, -3.2, 0.8),
            (3.0, -3.4, 0.9),
            (8.4, -1.9, 0.8),
            (6.2, -2.9, 0.7),
        ] {
            draw_circle(pos.x + dx, pile + dy, r, gold);
            draw_circle(
                pos.x + dx - 0.3,
                pile + dy - 0.3,
                r * 0.42,
                with_alpha(C_WHITE, 0.80 * a),
            );
        }
        // Gema violeta engastada en el montón.
        let gx2 = pos.x + 3.4;
        let gy2 = pile - 2.6;
        draw_triangle(
            vec2(gx2, gy2 - 2.6),
            vec2(gx2 - 2.3, gy2),
            vec2(gx2 + 2.3, gy2),
            with_alpha(C_PURPLE, a),
        );
        draw_triangle(
            vec2(gx2, gy2 - 2.6),
            vec2(gx2 + 2.3, gy2),
            vec2(gx2 + 1.2, gy2 + 0.6),
            with_alpha(C_WHITE, 0.40 * a),
        );
        draw_triangle(
            vec2(gx2, gy2 - 2.6),
            vec2(gx2 - 2.3, gy2),
            vec2(gx2 - 1.2, gy2 + 0.6),
            with_alpha(C_WHITE, 0.18 * a),
        );
        // Brillos titilantes sobre el tesoro.
        for k in 0..4 {
            let tw = (time * 3.0 + k as f32 * 1.7).sin() * 0.5 + 0.5;
            draw_circle(
                pos.x - 7.5 + k as f32 * 5.0,
                oy + 1.4 + tw * 0.7,
                0.5 + tw * 0.45,
                with_alpha(C_GOLD, (0.35 + tw * 0.45) * a),
            );
        }
        // Chispas que suben justo al abrirse.
        let burst = (1.0 - lift).max(0.0);
        if burst > 0.0 {
            for k in 0..6 {
                let ph = ((time * (1.4 + k as f32 * 0.31) + k as f32 * 2.4) % 1.0).max(0.0);
                let sx = pos.x + ((k as f32 * 1.93).sin()) * (3.0 + ph * 6.0);
                let sy = lid_y - 2.0 - ph * (9.0 + lift * 7.0);
                let sa = (1.0 - ph) * burst * 0.75;
                draw_circle(sx, sy, 0.7 + (1.0 - ph) * 0.9, with_alpha(C_GOLD, sa));
            }
        }
    } else {
        // Tapa cerrada abombada.
        draw_ellipse(pos.x, lid_y - 2.2, w * 0.52, 5.6, 0.0, wood_mid);
        draw_ellipse(pos.x, lid_y - 3.2, w * 0.47, 4.0, 0.0, wood_hi);
        draw_rectangle(pos.x - w * 0.5, lid_y - 2.2, w, 2.8, wood_mid);
        draw_rectangle_lines(pos.x - w * 0.5, lid_y - 6.6, w, 6.6, 0.9, outline);
        draw_line(
            pos.x - w * 0.46,
            lid_y - 5.2,
            pos.x + w * 0.46,
            lid_y - 5.2,
            0.7,
            with_alpha(C_WHITE, a * 0.18),
        );
    }

    // Refuerzos verticales dorados: con el cofre abierto arrancan bajo la
    // abertura para no tapar el tesoro.
    let strap_top = if opened { pos.y + 4.8 } else { pos.y - 5.6 };
    let strap_h = (pos.y + h - 1.0 - strap_top).max(2.0);
    for x in [pos.x - w * 0.36, pos.x + w * 0.36] {
        draw_rectangle(x - 1.2, strap_top, 2.4, strap_h, dark_gold);
        draw_rectangle(
            x - 1.2,
            strap_top,
            2.4,
            strap_h,
            with_alpha(C_GOLD, 0.55 * a),
        );
        draw_rectangle(
            x - 0.4,
            strap_top,
            0.8,
            strap_h,
            with_alpha(C_WHITE, 0.40 * a),
        );
        for yy in [strap_top + 2.4, pos.y + h - 3.0] {
            if yy > strap_top + 1.0 && yy < pos.y + h - 1.0 {
                draw_circle(x, yy, 0.8, with_alpha(C_WHITE, 0.55 * a));
            }
        }
    }
    draw_rectangle(pos.x - w * 0.5, pos.y + 5.0, w, 2.0, dark_gold);
    draw_rectangle(
        pos.x - w * 0.5,
        pos.y + 5.0,
        w,
        0.6,
        with_alpha(C_WHITE, 0.30 * a),
    );

    // Esquinas reforzadas (las superiores solo con la tapa cerrada).
    let mut corners = vec![
        (pos.x - w * 0.5, pos.y + h - 1.0, 1.0, -1.0),
        (pos.x + w * 0.5, pos.y + h - 1.0, -1.0, -1.0),
    ];
    if !opened {
        corners.push((pos.x - w * 0.5, pos.y - 1.0, 1.0, 1.0));
        corners.push((pos.x + w * 0.5, pos.y - 1.0, -1.0, 1.0));
    }
    for (cx2, cy2, dx, dy) in corners {
        draw_line(cx2, cy2, cx2 + dx * 4.0, cy2, 1.3, gold);
        draw_line(cx2, cy2, cx2, cy2 + dy * 4.0, 1.3, gold);
    }

    if opened {
        // Candado abierto colgando del labio frontal: cuerpo de hierro con
        // arco dorado levantado, para que no se confunda con el tesoro.
        let lock_y = pos.y + 5.4;
        let iron = Color::new(0.19, 0.18, 0.23, a);
        let iron_hi = Color::new(0.36, 0.35, 0.43, a);
        draw_rectangle(pos.x - 2.7, lock_y, 5.4, 5.8, outline);
        draw_rectangle(pos.x - 2.2, lock_y + 0.5, 4.4, 4.8, iron);
        draw_rectangle(pos.x - 2.2, lock_y + 0.5, 4.4, 0.8, iron_hi);
        // Remaches dorados en las esquinas.
        for (rx, ry) in [(-1.7, 1.1), (1.7, 1.1), (-1.7, 4.7), (1.7, 4.7)] {
            draw_circle(pos.x + rx, lock_y + ry, 0.45, gold);
        }
        // Arco del candado levantado y abierto por la derecha.
        draw_line(
            pos.x - 1.5,
            lock_y + 0.2,
            pos.x - 1.5,
            lock_y - 2.4,
            0.9,
            gold,
        );
        draw_line(
            pos.x - 1.5,
            lock_y - 2.4,
            pos.x + 1.2,
            lock_y - 2.4,
            0.9,
            gold,
        );
        draw_line(
            pos.x + 1.2,
            lock_y - 2.4,
            pos.x + 1.2,
            lock_y - 1.2,
            0.9,
            gold,
        );
        // Ojo de llave con anillo dorado.
        draw_circle(pos.x, lock_y + 2.4, 1.4, gold);
        draw_circle(pos.x, lock_y + 2.4, 0.8, Color::new(0.10, 0.09, 0.12, a));
        draw_rectangle(
            pos.x - 0.35,
            lock_y + 2.4,
            0.7,
            1.8,
            Color::new(0.10, 0.09, 0.12, a),
        );
    } else {
        // Cerradura cerrada con ojo de llave y destello.
        let lock_y = pos.y + 1.4;
        draw_rectangle(pos.x - 2.9, lock_y - 2.4, 5.8, 5.6, dark_gold);
        draw_rectangle(pos.x - 2.4, lock_y - 2.0, 4.8, 4.8, gold);
        draw_circle_lines(pos.x, lock_y - 2.8, 1.9, 0.9, gold);
        draw_circle(pos.x, lock_y, 0.9, Color::new(0.12, 0.08, 0.03, a));
        draw_rectangle(
            pos.x - 0.4,
            lock_y,
            0.8,
            2.4,
            Color::new(0.12, 0.08, 0.03, a),
        );
        draw_circle(
            pos.x + 6.0,
            pos.y - 4.6 + (time * 1.6).sin() * 0.5,
            0.9,
            with_alpha(C_WHITE, 0.60 + shine * 0.30),
        );
    }
}

/// Props del escenario dibujados con formas suaves: barriles, cajas, urnas y
/// estatuas. `pos` es la base del objeto.
pub fn draw_prop(kind: PropKind, pos: Vec2, hit_flash: f32, time: f32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let flash = (hit_flash / 0.18).clamp(0.0, 1.0);
    let wood = mix(Color::new(0.31, 0.19, 0.11, 1.0), C_WHITE, flash * 0.55);
    let wood_hi = mix(Color::new(0.46, 0.30, 0.17, 1.0), C_WHITE, flash * 0.55);
    let wood_dark = darken(wood, 0.55);
    let metal = Color::new(0.58, 0.52, 0.42, 1.0);
    let stone = mix(Color::new(0.35, 0.34, 0.44, 1.0), C_WHITE, flash * 0.5);
    let stone_dark = darken(stone, 0.58);
    match kind {
        PropKind::Barrel => {
            let w = 13.0;
            let h = 18.0;
            let body = with_alpha(wood, a);
            draw_rectangle(pos.x - w * 0.5, pos.y - h, w, h, body);
            draw_circle(pos.x, pos.y - h, w * 0.5, body);
            draw_circle(pos.x, pos.y, w * 0.5, with_alpha(wood_dark, a));
            // Duelas.
            for k in 0..3 {
                let x = pos.x - w * 0.5 + 2.0 + k as f32 * 4.4;
                draw_line(
                    x,
                    pos.y - h + 1.0,
                    x,
                    pos.y - 1.0,
                    0.6,
                    with_alpha(wood_dark, a * 0.8),
                );
            }
            // Aros metálicos.
            for y in [pos.y - h + 3.0, pos.y - 5.0] {
                draw_rectangle(pos.x - w * 0.5 - 0.4, y, w + 0.8, 1.7, with_alpha(metal, a));
                draw_rectangle(
                    pos.x - w * 0.5 - 0.4,
                    y,
                    w + 0.8,
                    0.5,
                    with_alpha(C_WHITE, a * 0.35),
                );
            }
            draw_ellipse(pos.x, pos.y - h, w * 0.5, 2.2, 0.0, with_alpha(wood_hi, a));
            draw_circle(
                pos.x - 2.4,
                pos.y - h + 0.6,
                0.8,
                with_alpha(C_WHITE, a * 0.5),
            );
        }
        PropKind::Crate => {
            let w = 14.0;
            let h = 13.0;
            let body = with_alpha(wood, a);
            draw_rectangle(pos.x - w * 0.5, pos.y - h, w, h, body);
            draw_rectangle(pos.x - w * 0.5, pos.y - h, w, 1.8, with_alpha(wood_hi, a));
            // Tablones y refuerzos en aspa.
            for k in 0..2 {
                let y = pos.y - h + 3.2 + k as f32 * 5.0;
                draw_line(
                    pos.x - w * 0.5 + 1.0,
                    y,
                    pos.x + w * 0.5 - 1.0,
                    y,
                    0.6,
                    with_alpha(wood_dark, a * 0.75),
                );
            }
            draw_line(
                pos.x - w * 0.5 + 1.0,
                pos.y - h + 1.5,
                pos.x + w * 0.5 - 1.0,
                pos.y - 1.5,
                1.0,
                with_alpha(metal, a * 0.75),
            );
            draw_line(
                pos.x + w * 0.5 - 1.0,
                pos.y - h + 1.5,
                pos.x - w * 0.5 + 1.0,
                pos.y - 1.5,
                1.0,
                with_alpha(metal, a * 0.75),
            );
            draw_rectangle_lines(
                pos.x - w * 0.5,
                pos.y - h,
                w,
                h,
                0.9,
                with_alpha(wood_dark, a),
            );
            // Esquinas metálicas.
            for (cx2, cy2) in [
                (pos.x - w * 0.5 + 0.6, pos.y - h + 0.6),
                (pos.x + w * 0.5 - 0.6, pos.y - h + 0.6),
                (pos.x - w * 0.5 + 0.6, pos.y - 0.6),
                (pos.x + w * 0.5 - 0.6, pos.y - 0.6),
            ] {
                draw_circle(cx2, cy2, 1.0, with_alpha(metal, a * 0.9));
            }
        }
        PropKind::Urn => {
            let body = with_alpha(wood, a);
            draw_ellipse(pos.x, pos.y - 5.0, 5.4, 6.2, 0.0, body);
            draw_ellipse(
                pos.x,
                pos.y - 5.0,
                4.2,
                5.2,
                0.0,
                with_alpha(wood_hi, a * 0.65),
            );
            draw_rectangle(pos.x - 3.4, pos.y - 13.0, 6.8, 4.0, body);
            draw_ellipse(pos.x, pos.y - 13.0, 3.4, 1.6, 0.0, with_alpha(wood_dark, a));
            // Asas.
            for side in [-1.0f32, 1.0] {
                draw_arc(
                    pos.x + side * 5.0,
                    pos.y - 7.0,
                    8,
                    3.0,
                    if side < 0.0 { 90.0 } else { -90.0 },
                    0.8,
                    180.0,
                    with_alpha(metal, a * 0.75),
                );
            }
            draw_rectangle(
                pos.x - 4.6,
                pos.y - 2.4,
                9.2,
                1.6,
                with_alpha(metal, a * 0.8),
            );
        }
        PropKind::Statue => {
            let body = with_alpha(stone, a);
            // Pedestal.
            draw_rectangle(
                pos.x - 6.0,
                pos.y - 4.0,
                12.0,
                4.0,
                with_alpha(stone_dark, a),
            );
            draw_rectangle(pos.x - 5.0, pos.y - 6.0, 10.0, 2.2, body);
            // Torso y cabeza de ídolo.
            draw_triangle(
                vec2(pos.x, pos.y - 22.0),
                vec2(pos.x - 5.0, pos.y - 6.0),
                vec2(pos.x + 5.0, pos.y - 6.0),
                body,
            );
            draw_circle(pos.x, pos.y - 22.0, 3.6, body);
            draw_circle(
                pos.x - 1.3,
                pos.y - 22.6,
                0.7,
                with_alpha(C_CYAN, a * (0.55 + 0.25 * (time * 2.0).sin())),
            );
            draw_circle(
                pos.x + 1.3,
                pos.y - 22.6,
                0.7,
                with_alpha(C_CYAN, a * (0.55 + 0.25 * (time * 2.0).sin())),
            );
            draw_line(
                pos.x - 3.0,
                pos.y - 17.0,
                pos.x + 3.0,
                pos.y - 17.0,
                0.8,
                with_alpha(stone_dark, a),
            );
            draw_line(
                pos.x,
                pos.y - 20.0,
                pos.x,
                pos.y - 8.0,
                0.7,
                with_alpha(stone_dark, a * 0.8),
            );
            draw_circle_lines(pos.x, pos.y - 14.0, 6.0, 0.5, with_alpha(C_CYAN, a * 0.18));
        }
        PropKind::Chest => {}
    }
}

/// Cama de la sala de descanso: dosel de madera, colchón, almohada y manta.
/// `pos` es el centro de la cama.
pub fn draw_bed(pos: Vec2, time: f32) {
    let wood = Color::new(0.30, 0.18, 0.10, 1.0);
    let wood_hi = Color::new(0.44, 0.28, 0.16, 1.0);
    let cloth = Color::new(0.22, 0.30, 0.46, 1.0);
    let cloth_hi = Color::new(0.31, 0.42, 0.62, 1.0);
    let sheet = Color::new(0.82, 0.80, 0.76, 1.0);
    let w = 46.0;
    // Postes del dosel.
    for side in [-1.0f32, 1.0] {
        let x = pos.x + side * (w * 0.5 - 2.0);
        draw_rectangle(x - 1.6, pos.y - 34.0, 3.2, 34.0, wood);
        draw_rectangle(x - 1.6, pos.y - 34.0, 1.0, 34.0, wood_hi);
        draw_circle(x, pos.y - 34.0, 2.0, with_alpha(C_GOLD, 0.80));
    }
    // Barra superior y cortina.
    draw_rectangle(pos.x - w * 0.5, pos.y - 34.0, w, 2.4, wood_hi);
    draw_rectangle(
        pos.x - w * 0.5 + 2.0,
        pos.y - 31.6,
        w - 4.0,
        8.0 + (time * 1.2).sin() * 0.6,
        Color::new(0.20, 0.12, 0.26, 0.85),
    );
    for k in 0..5 {
        let x = pos.x - w * 0.5 + 6.0 + k as f32 * 8.5;
        draw_line(
            x,
            pos.y - 31.0,
            x,
            pos.y - 23.0,
            0.7,
            Color::new(0.30, 0.18, 0.36, 0.9),
        );
    }
    // Base y patas.
    draw_rectangle(pos.x - w * 0.5, pos.y - 8.0, w, 4.0, wood);
    draw_rectangle(pos.x - w * 0.5, pos.y - 8.0, w, 1.2, wood_hi);
    for side in [-1.0f32, 1.0] {
        draw_rectangle(
            pos.x + side * (w * 0.5 - 3.0) - 1.2,
            pos.y - 4.0,
            2.4,
            5.0,
            wood,
        );
    }
    // Colchón.
    draw_rectangle(pos.x - w * 0.5 + 2.0, pos.y - 12.0, w - 4.0, 5.0, sheet);
    draw_rectangle(
        pos.x - w * 0.5 + 2.0,
        pos.y - 12.0,
        w - 4.0,
        1.4,
        Color::new(0.94, 0.93, 0.90, 1.0),
    );
    // Manta.
    draw_rectangle(pos.x - 4.0, pos.y - 12.4, w * 0.5 - 1.0, 6.2, cloth);
    draw_rectangle(pos.x - 4.0, pos.y - 12.4, w * 0.5 - 1.0, 1.6, cloth_hi);
    for k in 0..3 {
        draw_line(
            pos.x - 1.0 + k as f32 * 6.0,
            pos.y - 11.0,
            pos.x - 2.0 + k as f32 * 6.0,
            pos.y - 7.0,
            0.5,
            with_alpha(Color::new(0.10, 0.14, 0.24, 1.0), 0.6),
        );
    }
    // Almohada.
    draw_ellipse(
        pos.x - w * 0.5 + 9.0,
        pos.y - 13.0,
        6.0,
        3.2,
        -0.08,
        Color::new(0.94, 0.93, 0.89, 1.0),
    );
    draw_ellipse(
        pos.x - w * 0.5 + 9.0,
        pos.y - 13.6,
        4.2,
        1.6,
        -0.08,
        Color::new(1.0, 0.99, 0.95, 1.0),
    );
    // Halo tenue bajo el dosel.
    draw_circle(
        pos.x,
        pos.y - 26.0,
        16.0,
        Color::new(0.42, 0.72, 0.95, 0.035),
    );
}

/// Moneda de oro que gira sobre sí misma. `pos` es el centro del objeto.
pub fn draw_coin(pos: Vec2, bob: f32, time: f32) {
    let y = pos.y + bob;
    draw_ellipse(
        pos.x,
        pos.y + 4.0,
        4.2,
        1.5,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.30),
    );
    let spin = (time * 4.0).sin();
    let sx = (0.22 + spin.abs() * 0.78).max(0.16);
    draw_ellipse(pos.x, y, 4.6 * sx, 4.6, 0.0, C_GOLD);
    draw_ellipse(pos.x, y, 3.4 * sx, 3.4, 0.0, lighten(C_GOLD, 0.28));
    draw_ellipse(
        pos.x - 1.0 * sx,
        y - 1.3,
        1.1 * sx,
        1.5,
        0.0,
        with_alpha(C_WHITE, 0.78),
    );
    if spin.abs() < 0.30 {
        draw_line(pos.x, y - 4.2, pos.x, y + 4.2, 0.8, darken(C_GOLD, 0.52));
    }
}

/// Gema facetada con el color de su rareza.
pub fn draw_gem(pos: Vec2, bob: f32, tint: Color) {
    let y = pos.y + bob;
    draw_ellipse(
        pos.x,
        pos.y + 4.0,
        5.0,
        1.8,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.28),
    );
    draw_circle(pos.x, y, 8.0, with_alpha(tint, 0.10));
    let top = vec2(pos.x, y - 6.0);
    let left = vec2(pos.x - 5.0, y - 1.0);
    let right = vec2(pos.x + 5.0, y - 1.0);
    let bottom = vec2(pos.x, y + 6.0);
    draw_triangle(top, left, bottom, tint);
    draw_triangle(top, bottom, right, lighten(tint, 0.16));
    draw_triangle(top, left, right, lighten(tint, 0.34));
    draw_line(
        pos.x,
        y - 6.0,
        pos.x - 5.0,
        y - 1.0,
        0.5,
        with_alpha(C_WHITE, 0.55),
    );
    draw_line(
        pos.x,
        y - 6.0,
        pos.x + 5.0,
        y - 1.0,
        0.5,
        with_alpha(C_WHITE, 0.25),
    );
    draw_circle(pos.x - 1.6, y - 2.2, 0.8, with_alpha(C_WHITE, 0.85));
}

/// Frasco de curación: vidrio, líquido rojo y corcho.
pub fn draw_vial(pos: Vec2, bob: f32, time: f32) {
    let y = pos.y + bob;
    draw_ellipse(
        pos.x,
        pos.y + 4.0,
        3.5,
        1.4,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.28),
    );
    let glass = Color::new(0.82, 0.88, 0.94, 0.40);
    draw_rectangle(pos.x - 2.6, y - 4.0, 5.2, 8.0, glass);
    draw_rectangle(pos.x - 2.0, y - 2.2, 4.0, 5.8, C_RED);
    draw_rectangle(pos.x - 2.0, y - 2.2, 4.0, 1.2, lighten(C_RED, 0.35));
    draw_rectangle(pos.x - 1.2, y - 6.0, 2.4, 2.2, glass);
    draw_rectangle(
        pos.x - 1.6,
        y - 7.4,
        3.2,
        1.6,
        Color::new(0.46, 0.30, 0.16, 1.0),
    );
    draw_rectangle_lines(
        pos.x - 2.6,
        y - 4.0,
        5.2,
        8.0,
        0.6,
        with_alpha(C_WHITE, 0.48),
    );
    draw_line(
        pos.x - 1.7,
        y - 3.2,
        pos.x - 1.7,
        y + 1.0,
        0.6,
        with_alpha(C_WHITE, 0.65),
    );
    let bubble = (time * 2.4).sin() * 1.2;
    draw_circle(
        pos.x + 0.6,
        y + 1.2 + bubble,
        0.6,
        with_alpha(C_WHITE, 0.45),
    );
}

/// Aliado invocado: esqueleto, ojo flotante, murciélago o espectro.
pub fn draw_ally(kind: usize, pos: Vec2, tint: Color, bob: f32, swing: f32) {
    let p = pos + vec2(0.0, bob);
    let body = tint;
    let dark = darken(tint, 0.55);
    match kind {
        1 => {
            // Esqueleto con espada.
            draw_circle(p.x, p.y - 9.0, 4.0, C_WHITE);
            draw_circle(p.x - 1.3, p.y - 9.6, 0.8, C_RED);
            draw_circle(p.x + 1.3, p.y - 9.6, 0.8, C_RED);
            draw_rectangle(p.x - 2.6, p.y - 5.0, 5.2, 7.0, body);
            for k in 0..3 {
                draw_line(
                    p.x - 2.6,
                    p.y - 4.0 + k as f32 * 2.0,
                    p.x + 2.6,
                    p.y - 4.0 + k as f32 * 2.0,
                    0.6,
                    dark,
                );
            }
            draw_line(p.x - 2.0, p.y + 2.0, p.x - 3.2, p.y + 6.0, 1.1, body);
            draw_line(p.x + 2.0, p.y + 2.0, p.x + 3.2, p.y + 6.0, 1.1, body);
            let hand = vec2(p.x + 4.0, p.y - 3.0);
            draw_line(p.x + 2.6, p.y - 3.5, hand.x, hand.y, 1.2, body);
            draw_line(
                hand.x,
                hand.y,
                hand.x + 4.0 + swing * 2.0,
                hand.y - 7.0,
                1.0,
                C_WHITE,
            );
        }
        2 => {
            // Ojo flotante con tentáculos.
            draw_circle(p.x, p.y - 4.0, 5.2, body);
            draw_circle(p.x, p.y - 4.0, 3.0, C_WHITE);
            draw_circle(p.x + 0.8, p.y - 4.0, 1.7, dark);
            draw_circle(p.x + 0.8, p.y - 4.0, 0.8, C_WHITE);
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::TAU / 4.0 + 0.4;
                draw_line(
                    p.x + a.cos() * 4.0,
                    p.y - 2.0 + a.sin() * 3.0,
                    p.x + a.cos() * 7.0,
                    p.y + 2.0 + a.sin() * 4.0,
                    0.8,
                    with_alpha(dark, 0.75),
                );
            }
        }
        3 => {
            // Murciélago que aletea.
            let flap = (swing * std::f32::consts::PI).sin();
            draw_triangle(
                vec2(p.x, p.y - 4.0),
                vec2(p.x - 8.0, p.y - 7.0 - flap * 3.0),
                vec2(p.x - 2.5, p.y + 2.0),
                dark,
            );
            draw_triangle(
                vec2(p.x, p.y - 4.0),
                vec2(p.x + 8.0, p.y - 7.0 - flap * 3.0),
                vec2(p.x + 2.5, p.y + 2.0),
                body,
            );
            draw_circle(p.x, p.y - 3.0, 2.8, body);
            draw_circle(p.x - 1.0, p.y - 3.5, 0.7, C_RED);
            draw_circle(p.x + 1.0, p.y - 3.5, 0.7, C_RED);
        }
        _ => {
            // Espectro encapuchado.
            draw_triangle(
                vec2(p.x, p.y - 13.0),
                vec2(p.x - 7.0, p.y + 6.0),
                vec2(p.x + 7.0, p.y + 6.0),
                with_alpha(body, 0.55),
            );
            draw_circle(p.x, p.y - 8.0, 4.2, dark);
            draw_circle(p.x - 1.4, p.y - 8.0, 0.8, C_CYAN);
            draw_circle(p.x + 1.4, p.y - 8.0, 0.8, C_CYAN);
            for k in 0..3 {
                draw_line(
                    p.x - 3.0 + k as f32 * 3.0,
                    p.y + 4.0,
                    p.x - 4.0 + k as f32 * 4.0,
                    p.y + 7.0 + (swing * 2.0).sin(),
                    0.7,
                    with_alpha(body, 0.45),
                );
            }
        }
    }
}

/// Sombra roja en los bordes cuando el héroe está herido o recibe un golpe.
pub fn draw_damage_vignette(hp_frac: f32, hit_flash: f32, time: f32) {
    let low = (1.0 - (hp_frac / 0.34).clamp(0.0, 1.0)).max(0.0);
    let pulse = 0.55 + 0.45 * (time * 3.4).sin();
    let hit = (hit_flash / 0.18).clamp(0.0, 1.0);
    let a = low * (0.10 + pulse * 0.09) + hit * 0.18;
    if a <= 0.001 {
        return;
    }
    let red = Color::new(0.62, 0.05, 0.08, 1.0);
    let bands = 7;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let band_a = a * (1.0 - t) * 0.55;
        let inset = 46.0 * t;
        draw_rectangle(0.0, inset, 480.0, 2.2, with_alpha(red, band_a));
        draw_rectangle(
            0.0,
            270.0 - inset - 2.2,
            480.0,
            2.2,
            with_alpha(red, band_a),
        );
        draw_rectangle(inset, 0.0, 2.2, 270.0, with_alpha(red, band_a));
        draw_rectangle(
            480.0 - inset - 2.2,
            0.0,
            2.2,
            270.0,
            with_alpha(red, band_a),
        );
    }
    draw_rectangle(
        0.0,
        0.0,
        480.0,
        270.0,
        Color::new(0.55, 0.03, 0.06, a * 0.10),
    );
}
