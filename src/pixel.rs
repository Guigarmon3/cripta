//! Pixel art procedural de CRIPTA.
//!
//! Los héroes se dibujan en un lienzo de 40x42 píxeles con contorno oscuro,
//! paleta limitada y animación por pasos. El mundo se renderiza a 480x270 y
//! se escala con filtro nearest, así que cada píxel del lienzo cae exacto en
//! pantalla. La interfaz se dibuja aparte, en resolución nativa.

use crate::game::EnemyKind;
use crate::types::Class;
use macroquad::prelude::*;

const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
const OUTLINE: Color = Color::new(0.055, 0.045, 0.075, 1.0);
const RED: Color = Color::new(0.82, 0.22, 0.26, 1.0);

const CANVAS_W: i32 = 40;
const CANVAS_H: i32 = 42;
const FEET_Y: i32 = 38;

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

fn same(a: Color, b: Color) -> bool {
    a.r == b.r && a.g == b.g && a.b == b.b && a.a == b.a
}

/// Lienzo de píxeles con transparencia. Se dibuja por filas de color continuo,
/// no píxel a píxel, para que sea barato incluso con varios héroes.
struct Canvas {
    w: i32,
    h: i32,
    px: Vec<Color>,
}

impl Canvas {
    fn new(w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            px: vec![TRANSPARENT; (w * h) as usize],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            Some((y * self.w + x) as usize)
        } else {
            None
        }
    }

    fn set(&mut self, x: i32, y: i32, color: Color) {
        if let Some(i) = self.index(x, y) {
            self.px[i] = color;
        }
    }

    fn get(&self, x: i32, y: i32) -> Color {
        self.index(x, y).map(|i| self.px[i]).unwrap_or(TRANSPARENT)
    }

    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, color);
            }
        }
    }

    fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            self.set(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Círculo relleno. Se usa para cabezas, orbes y ojos de jefe.
    fn disc(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let r2 = r * r;
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r2 {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    /// Elipse rellena para torsos y masas.
    fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        for y in -ry..=ry {
            for x in -rx..=rx {
                let nx = x as f32 / rx as f32;
                let ny = y as f32 / ry as f32;
                if nx * nx + ny * ny <= 1.0 {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    /// Triángulo relleno por scanline. Se usa para alas, capas y cuernos.
    fn tri(&mut self, a: (i32, i32), b: (i32, i32), c: (i32, i32), color: Color) {
        let min_x = a.0.min(b.0).min(c.0);
        let max_x = a.0.max(b.0).max(c.0);
        let min_y = a.1.min(b.1).min(c.1);
        let max_y = a.1.max(b.1).max(c.1);
        let edge = |px: i32, py: i32, ax: i32, ay: i32, bx: i32, by: i32| -> i32 {
            (px - ax) * (by - ay) - (py - ay) * (bx - ax)
        };
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let e0 = edge(x, y, a.0, a.1, b.0, b.1);
                let e1 = edge(x, y, b.0, b.1, c.0, c.1);
                let e2 = edge(x, y, c.0, c.1, a.0, a.1);
                let has_neg = e0 < 0 || e1 < 0 || e2 < 0;
                let has_pos = e0 > 0 || e1 > 0 || e2 > 0;
                if !(has_neg && has_pos) {
                    self.set(x, y, color);
                }
            }
        }
    }

    fn thick_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, w: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        let half = w / 2;
        loop {
            self.rect(x - half, y - half, w, w, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Añade un contorno de 1 píxel alrededor de toda la silueta. Se calcula al
    /// final, así los detalles internos no quedan llenos de líneas negras.
    fn outline(&mut self, color: Color) {
        let src = self.px.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) as usize;
                if src[i].a > 0.0 {
                    continue;
                }
                let mut edge = false;
                for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if nx >= 0 && ny >= 0 && nx < self.w && ny < self.h {
                        let ni = (ny * self.w + nx) as usize;
                        if src[ni].a > 0.0 {
                            edge = true;
                            break;
                        }
                    }
                }
                if edge {
                    self.px[i] = color;
                }
            }
        }
    }

    fn draw(&self, feet: Vec2, flip: bool, flash: f32) {
        self.draw_scaled(feet, flip, flash, 1.0, 1.0);
    }

    fn draw_scaled(&self, feet: Vec2, flip: bool, flash: f32, scale: f32, fade: f32) {
        self.draw_scaled_at(feet, flip, flash, scale, fade, FEET_Y);
    }

    fn draw_scaled_at(
        &self,
        feet: Vec2,
        flip: bool,
        flash: f32,
        scale: f32,
        fade: f32,
        feet_y: i32,
    ) {
        let s = scale.max(0.1);
        let base_x = if (s - 1.0).abs() < 0.001 {
            feet.x.round()
        } else {
            feet.x
        };
        let base_y = if (s - 1.0).abs() < 0.001 {
            feet.y.round()
        } else {
            feet.y
        };
        let ox = base_x - self.w as f32 * 0.5 * s;
        let oy = base_y - feet_y as f32 * s;
        for y in 0..self.h {
            let mut x = 0;
            while x < self.w {
                let color = self.get(x, y);
                if color.a <= 0.0 {
                    x += 1;
                    continue;
                }
                let start = x;
                let mut end = x + 1;
                while end < self.w && same(self.get(end, y), color) {
                    end += 1;
                }
                let len = end - start;
                let dx = if flip { self.w - end } else { start };
                let mut col = color;
                if flash > 0.0 {
                    col = mix(col, Color::new(1.0, 0.86, 0.86, 1.0), flash * 0.72);
                }
                if fade < 1.0 {
                    col.a *= fade.clamp(0.0, 1.0);
                }
                draw_rectangle(
                    ox + dx as f32 * s,
                    oy + y as f32 * s,
                    len as f32 * s,
                    s + 0.06,
                    col,
                );
                x = end;
            }
        }
    }
}

struct Palette {
    skin: Color,
    skin_dark: Color,
    cloth: Color,
    cloth_dark: Color,
    cloth_light: Color,
    metal: Color,
    metal_dark: Color,
    metal_light: Color,
    gold: Color,
    leather: Color,
    leather_dark: Color,
    accent: Color,
    glow: Color,
    bone: Color,
    bone_dark: Color,
}

fn palette(class: Class, tint: Color) -> Palette {
    let (cloth, cloth_dark, cloth_light, accent, glow) = match class {
        Class::Knight => (
            Color::new(0.20, 0.30, 0.56, 1.0),
            Color::new(0.10, 0.15, 0.32, 1.0),
            Color::new(0.35, 0.47, 0.74, 1.0),
            Color::new(0.96, 0.78, 0.34, 1.0),
            Color::new(1.0, 0.86, 0.42, 1.0),
        ),
        Class::Archer => (
            Color::new(0.22, 0.45, 0.28, 1.0),
            Color::new(0.10, 0.24, 0.15, 1.0),
            Color::new(0.38, 0.64, 0.42, 1.0),
            Color::new(0.56, 0.86, 0.50, 1.0),
            Color::new(0.62, 0.95, 0.52, 1.0),
        ),
        Class::Mage => (
            Color::new(0.23, 0.31, 0.70, 1.0),
            Color::new(0.10, 0.14, 0.37, 1.0),
            Color::new(0.40, 0.52, 0.90, 1.0),
            Color::new(0.42, 0.90, 0.96, 1.0),
            Color::new(0.55, 0.96, 1.0, 1.0),
        ),
        Class::Assassin => (
            Color::new(0.33, 0.19, 0.45, 1.0),
            Color::new(0.14, 0.08, 0.23, 1.0),
            Color::new(0.50, 0.31, 0.64, 1.0),
            Color::new(0.78, 0.48, 0.98, 1.0),
            Color::new(0.88, 0.58, 1.0, 1.0),
        ),
        Class::Necro => (
            Color::new(0.20, 0.30, 0.24, 1.0),
            Color::new(0.09, 0.14, 0.12, 1.0),
            Color::new(0.35, 0.47, 0.37, 1.0),
            Color::new(0.42, 0.92, 0.52, 1.0),
            Color::new(0.55, 1.0, 0.60, 1.0),
        ),
        Class::Cleric => (
            Color::new(0.82, 0.76, 0.62, 1.0),
            Color::new(0.50, 0.46, 0.38, 1.0),
            Color::new(0.97, 0.93, 0.82, 1.0),
            Color::new(0.98, 0.80, 0.36, 1.0),
            Color::new(1.0, 0.94, 0.62, 1.0),
        ),
        Class::Merchant => (
            Color::new(0.46, 0.33, 0.21, 1.0),
            Color::new(0.24, 0.16, 0.10, 1.0),
            Color::new(0.64, 0.48, 0.29, 1.0),
            Color::new(0.98, 0.80, 0.34, 1.0),
            Color::new(1.0, 0.90, 0.48, 1.0),
        ),
    };
    let base_skin = Color::new(0.91, 0.71, 0.55, 1.0);
    let white = Color::new(1.0, 1.0, 1.0, 1.0);
    let skin = if same(tint, white) {
        base_skin
    } else {
        mix(base_skin, tint, 0.45)
    };
    Palette {
        skin,
        skin_dark: darken(skin, 0.72),
        cloth,
        cloth_dark,
        cloth_light,
        metal: Color::new(0.72, 0.77, 0.86, 1.0),
        metal_dark: Color::new(0.36, 0.41, 0.52, 1.0),
        metal_light: Color::new(0.94, 0.97, 1.0, 1.0),
        gold: Color::new(0.96, 0.78, 0.34, 1.0),
        leather: Color::new(0.38, 0.25, 0.17, 1.0),
        leather_dark: Color::new(0.20, 0.13, 0.09, 1.0),
        accent,
        glow,
        bone: Color::new(0.84, 0.82, 0.70, 1.0),
        bone_dark: Color::new(0.53, 0.50, 0.42, 1.0),
    }
}

struct Anim {
    bob: i32,
    lift_l: i32,
    lift_r: i32,
    arm_l: i32,
    hand: (i32, i32),
    time: f32,
    look: i32,
}

fn draw_cape(c: &mut Canvas, p: &Palette, a: &Anim, class: Class) {
    let top = 19 + a.bob;
    let base = match class {
        Class::Knight => p.cloth,
        Class::Cleric => p.cloth_light,
        Class::Archer => p.cloth_dark,
        _ => p.cloth_dark,
    };
    // La capa sobresale a ambos lados del torso para que se lea la silueta.
    for x in 11..=29 {
        let wave = ((x as f32 * 0.7 + a.time * 2.0).sin() * 1.4).round() as i32;
        let hem = 33 + a.bob + wave.clamp(-2, 2);
        c.rect(x, top, 1, (hem - top).max(1), base);
    }
    c.rect(11, top + 2, 1, 10, p.cloth_light);
    c.rect(12, top + 10, 1, 7, darken(base, 0.82));
    c.rect(29, top + 3, 1, 9, darken(base, 0.62));
    c.rect(28, top + 11, 1, 6, darken(base, 0.74));
}

fn draw_legs(c: &mut Canvas, p: &Palette, a: &Anim, class: Class) {
    let hip = 30 + a.bob;
    let foot_l = FEET_Y - a.lift_l;
    let foot_r = FEET_Y - a.lift_r;
    let xl = 16 + if a.lift_l > 0 { -1 } else { 0 };
    let xr = 21 + if a.lift_r > 0 { 1 } else { 0 };
    let hl = (foot_l - hip - 2).max(2);
    let hr = (foot_r - hip - 2).max(2);

    c.rect(xl, hip, 3, hl, p.cloth_dark);
    c.rect(xl, hip, 1, hl, p.cloth);
    c.rect(xl - 2, foot_l - 2, 5, 2, p.leather);
    c.rect(xl - 2, foot_l - 1, 5, 1, p.leather_dark);
    c.set(xl - 2, foot_l - 2, lighten(p.leather, 0.18));

    c.rect(xr, hip, 3, hr, p.cloth_dark);
    c.rect(xr + 2, hip, 1, hr, p.cloth);
    c.rect(xr, foot_r - 2, 5, 2, p.leather);
    c.rect(xr, foot_r - 1, 5, 1, p.leather_dark);
    c.set(xr + 4, foot_r - 2, lighten(p.leather, 0.18));

    if matches!(class, Class::Knight | Class::Cleric) {
        c.rect(xl, hip + 3, 3, 1, p.metal);
        c.rect(xr, hip + 3, 3, 1, p.metal);
        c.rect(xl, hip + 4, 3, 1, p.metal_dark);
        c.rect(xr, hip + 4, 3, 1, p.metal_dark);
    }
}

fn draw_torso(c: &mut Canvas, p: &Palette, a: &Anim, class: Class) {
    let y = 20 + a.bob;
    c.rect(15, y, 10, 2, p.cloth);
    c.rect(15, y + 2, 10, 6, p.cloth);
    c.rect(14, y + 8, 12, 2, p.cloth);
    c.rect(15, y + 1, 1, 7, p.cloth_light);
    c.rect(24, y + 1, 1, 7, p.cloth_dark);

    if matches!(class, Class::Knight | Class::Cleric) {
        c.rect(17, y + 2, 6, 5, p.metal);
        c.rect(17, y + 2, 6, 1, p.metal_light);
        c.rect(17, y + 6, 6, 1, p.metal_dark);
        c.rect(20, y + 2, 1, 5, p.gold);
    } else {
        c.rect(17, y + 2, 6, 5, p.cloth_light);
        c.rect(17, y + 2, 1, 5, p.cloth);
        c.rect(22, y + 2, 1, 5, p.cloth_dark);
        if matches!(class, Class::Archer | Class::Assassin) {
            c.line(17, y + 2, 22, y + 6, p.leather_dark);
            c.line(18, y + 2, 23, y + 6, p.leather);
        }
    }

    c.rect(15, y + 8, 10, 2, p.leather);
    c.rect(15, y + 8, 10, 1, p.leather_dark);
    c.rect(19, y + 8, 2, 2, p.gold);
    c.set(20, y + 4, p.accent);
    c.set(19, y + 4, p.accent);

    if matches!(class, Class::Knight | Class::Cleric) {
        c.rect(13, y + 1, 3, 2, p.metal);
        c.rect(24, y + 1, 3, 2, p.metal);
        c.rect(13, y + 1, 3, 1, p.metal_light);
        c.rect(24, y + 1, 3, 1, p.metal_light);
    } else {
        c.rect(13, y + 1, 2, 2, p.cloth_dark);
        c.rect(25, y + 1, 2, 2, p.cloth_dark);
    }
}

fn draw_arms(c: &mut Canvas, p: &Palette, a: &Anim) {
    let shoulder_y = 21 + a.bob;
    let lx = 13;
    let ly = 28 + a.bob + a.arm_l;
    c.thick_line(lx + 1, shoulder_y, lx, ly, 2, p.cloth_dark);
    c.rect(lx - 1, ly, 2, 2, p.skin);
    c.set(lx + 1, shoulder_y + 2, p.cloth_light);

    let (hx, hy) = a.hand;
    c.thick_line(27, shoulder_y, hx, hy, 2, p.cloth_dark);
    c.rect(hx, hy, 2, 2, p.skin);
    c.set(27, shoulder_y + 2, p.cloth_light);
}

fn draw_eyes(c: &mut Canvas, p: &Palette, y: i32, look: i32) {
    c.set(18 + look, y, OUTLINE);
    c.set(22 + look, y, OUTLINE);
    c.set(18 + look, y - 1, lighten(p.skin, 0.35));
    c.set(22 + look, y - 1, lighten(p.skin, 0.35));
    c.set(20, y + 2, p.skin_dark);
    c.set(20, y + 3, OUTLINE);
}

fn draw_head(c: &mut Canvas, p: &Palette, a: &Anim, class: Class, hat: u8) {
    let y = 12 + a.bob;
    c.rect(19, y + 7, 2, 2, p.skin_dark);
    c.rect(16, y, 8, 8, p.skin);
    c.rect(16, y + 6, 8, 2, p.skin_dark);
    c.rect(17, y + 2, 6, 5, p.skin);
    let eye_y = y + 3;
    draw_eyes(c, p, eye_y, a.look);

    match class {
        Class::Knight => {
            c.rect(15, y - 1, 10, 7, p.metal);
            c.rect(15, y - 1, 10, 1, p.metal_light);
            c.rect(15, y + 5, 10, 2, p.metal_dark);
            c.rect(15, y + 3, 2, 3, p.metal);
            c.rect(23, y + 3, 2, 3, p.metal);
            c.rect(16, y + 1, 8, 3, p.metal_dark);
            c.rect(17, y + 2, 6, 1, OUTLINE);
            c.rect(20, y + 1, 1, 4, p.metal);
            c.set(18 + a.look, y + 2, p.glow);
            c.set(22 + a.look, y + 2, p.glow);
            if hat == 0 {
                c.rect(19, y - 3, 2, 2, p.gold);
                c.rect(18, y - 4, 4, 1, p.gold);
                c.rect(20, y - 5, 2, 1, RED);
                c.rect(20, y - 4, 2, 1, RED);
            }
        }
        Class::Archer => {
            c.rect(15, y - 1, 10, 8, p.cloth_dark);
            c.rect(16, y + 1, 8, 7, p.skin);
            c.rect(16, y + 1, 8, 1, p.cloth_dark);
            c.rect(17, y + 2, 6, 5, p.skin);
            c.rect(24, y - 2, 2, 3, p.cloth);
            if hat == 0 {
                c.rect(25, y - 4, 1, 2, p.cloth_light);
                c.rect(25, y - 4, 3, 1, p.gold);
            }
        }
        Class::Mage => {
            c.rect(15, y - 1, 10, 8, p.cloth_dark);
            c.rect(16, y + 1, 8, 7, p.skin);
            c.rect(17, y + 2, 6, 5, p.skin);
            c.rect(12, y - 3, 16, 2, p.cloth_dark);
            c.rect(13, y - 3, 14, 1, p.cloth_light);
            if hat == 0 {
                c.rect(15, y - 5, 10, 2, p.cloth);
                c.rect(17, y - 7, 6, 2, p.cloth);
                c.rect(19, y - 9, 2, 2, p.cloth_light);
                c.set(20, y - 4, p.gold);
                c.set(19, y - 4, p.gold);
            }
        }
        Class::Assassin => {
            c.rect(15, y - 1, 10, 8, p.cloth_dark);
            c.rect(16, y + 1, 8, 7, p.skin);
            c.rect(17, y + 2, 6, 5, p.skin);
            c.rect(16, y + 4, 8, 4, p.cloth_dark);
            c.set(18 + a.look, y + 3, Color::new(0.96, 0.93, 0.84, 1.0));
            c.set(22 + a.look, y + 3, Color::new(0.96, 0.93, 0.84, 1.0));
            c.rect(24, y + 5, 2, 4, p.cloth_dark);
            c.rect(25, y + 9, 1, 2, p.cloth);
        }
        Class::Necro => {
            c.rect(15, y - 1, 10, 8, p.cloth_dark);
            c.rect(16, y + 1, 8, 7, p.bone);
            c.rect(17, y + 2, 6, 5, p.bone);
            c.rect(18 + a.look, y + 2, 1, 2, OUTLINE);
            c.rect(22 + a.look, y + 2, 1, 2, OUTLINE);
            c.set(18 + a.look, y + 3, p.glow);
            c.set(22 + a.look, y + 3, p.glow);
            c.set(20, y + 4, p.bone_dark);
            c.rect(18, y + 6, 4, 1, p.bone_dark);
            c.rect(19, y + 6, 2, 1, OUTLINE);
        }
        Class::Cleric => {
            c.rect(15, y - 1, 10, 8, p.cloth_light);
            c.rect(16, y + 1, 8, 7, p.skin);
            c.rect(17, y + 2, 6, 5, p.skin);
            c.rect(16, y + 1, 8, 1, p.gold);
            c.rect(19, y, 2, 1, p.gold);
            if hat == 0 {
                c.rect(16, y - 4, 8, 1, p.gold);
                c.rect(16, y - 3, 1, 1, p.gold);
                c.rect(23, y - 3, 1, 1, p.gold);
            }
        }
        Class::Merchant => {
            c.rect(16, y, 8, 8, p.skin);
            c.rect(17, y + 2, 6, 5, p.skin);
            c.rect(11, y - 2, 18, 2, p.leather);
            c.rect(12, y - 2, 16, 1, p.cloth_light);
            if hat == 0 {
                c.rect(15, y - 5, 10, 3, p.leather);
                c.rect(16, y - 6, 8, 1, p.leather_dark);
                c.rect(15, y - 2, 10, 1, p.gold);
            }
            c.rect(17, y + 5, 6, 1, p.leather_dark);
            c.rect(16, y + 6, 2, 1, p.leather_dark);
            c.rect(22, y + 6, 2, 1, p.leather_dark);
        }
    }
}

fn draw_cosmetic_hat(c: &mut Canvas, p: &Palette, a: &Anim, hat: u8) {
    let y = 12 + a.bob;
    match hat {
        1 => {
            c.rect(16, y, 8, 2, p.cloth);
            c.rect(16, y, 8, 1, p.cloth_light);
            c.rect(23, y + 1, 2, 3, p.cloth_dark);
            c.set(25, y + 3, p.cloth);
        }
        2 => {
            c.rect(16, y - 4, 8, 1, p.gold);
            c.rect(16, y - 3, 1, 1, p.gold);
            c.rect(23, y - 3, 1, 1, p.gold);
            c.set(19, y - 4, p.glow);
        }
        3 => {
            c.rect(15, y - 1, 10, 6, p.metal);
            c.rect(15, y - 1, 10, 1, p.metal_light);
            c.rect(16, y + 1, 8, 2, OUTLINE);
            c.rect(17, y + 1, 6, 1, Color::new(0.17, 0.21, 0.29, 1.0));
            c.rect(19, y - 2, 2, 1, p.gold);
        }
        4 => {
            c.rect(15, y - 3, 10, 2, p.gold);
            c.rect(15, y - 4, 1, 1, p.gold);
            c.rect(19, y - 5, 2, 2, p.gold);
            c.rect(24, y - 4, 1, 1, p.gold);
            c.set(20, y - 3, p.glow);
        }
        5 => {
            c.rect(14, y - 2, 2, 4, p.bone);
            c.rect(24, y - 2, 2, 4, p.bone);
            c.rect(13, y - 3, 1, 2, p.bone_dark);
            c.rect(26, y - 3, 1, 2, p.bone_dark);
        }
        6 => {
            c.rect(17, y - 4, 6, 1, p.gold);
            c.rect(16, y - 3, 1, 1, p.gold);
            c.rect(23, y - 3, 1, 1, p.gold);
        }
        7 => {
            c.rect(12, y - 2, 16, 2, p.cloth_dark);
            c.rect(13, y - 2, 14, 1, p.cloth_light);
            c.rect(15, y - 4, 10, 2, p.cloth);
            c.rect(17, y - 6, 6, 2, p.cloth);
            c.rect(19, y - 8, 2, 2, p.cloth_light);
            c.set(20, y - 3, p.gold);
        }
        _ => {}
    }
}

fn draw_mask(c: &mut Canvas, p: &Palette, a: &Anim, mask: u8) {
    let y = 12 + a.bob;
    match mask {
        1 => {
            c.rect(16, y + 3, 8, 2, OUTLINE);
            c.rect(17, y + 3, 6, 1, Color::new(0.12, 0.10, 0.16, 1.0));
        }
        2 => {
            c.rect(17, y + 5, 6, 2, RED);
            c.rect(17, y + 5, 6, 1, lighten(RED, 0.25));
        }
        3 => {
            c.rect(16, y + 2, 8, 6, p.bone);
            c.rect(17, y + 3, 2, 2, OUTLINE);
            c.rect(21, y + 3, 2, 2, OUTLINE);
            c.rect(19, y + 5, 2, 1, p.bone_dark);
        }
        4 => {
            c.rect(16, y + 3, 8, 2, p.accent);
            c.rect(17, y + 3, 6, 1, p.glow);
        }
        5 => {
            c.line(18, y + 2, 22, y + 6, RED);
        }
        6 => {
            c.rect(17, y + 4, 2, 1, RED);
            c.rect(21, y + 4, 2, 1, RED);
        }
        7 => {
            c.rect(16, y + 2, 8, 6, p.cloth_dark);
            c.rect(17, y + 3, 6, 1, p.cloth);
        }
        _ => {}
    }
}

fn draw_shield(c: &mut Canvas, p: &Palette, a: &Anim) {
    let x = 6;
    let y = 22 + a.bob;
    c.rect(x, y, 8, 9, p.metal_dark);
    c.rect(x + 1, y + 1, 6, 7, p.metal);
    c.rect(x + 1, y + 1, 6, 1, p.metal_light);
    c.rect(x + 1, y + 7, 6, 1, p.metal_dark);
    // Cruz dorada y remaches.
    c.rect(x + 3, y + 2, 2, 5, p.gold);
    c.rect(x + 2, y + 3, 4, 2, p.gold);
    c.set(x + 3, y + 4, p.metal_dark);
    c.set(x + 1, y + 2, p.metal_light);
    c.set(x + 6, y + 2, p.metal_light);
    c.set(x + 1, y + 6, p.metal_light);
    c.set(x + 6, y + 6, p.metal_light);
    // Mano que sujeta el borde derecho del escudo.
    c.rect(x + 7, y + 3, 2, 2, p.skin);
    c.set(x + 8, y + 4, p.skin_dark);
}

fn sword(c: &mut Canvas, p: &Palette, hx: i32, hy: i32, arc: f32) {
    let angle = -1.36 + arc * 1.36;
    let len = 11.0 + arc * 2.0;
    let tipx = hx + (angle.cos() * len).round() as i32;
    let tipy = hy + (angle.sin() * len).round() as i32;
    // Hoja de dos píxeles con filo claro y lomo oscuro.
    c.thick_line(hx, hy, tipx, tipy, 2, p.metal);
    let nx = -(angle.sin()).round() as i32;
    let ny = (angle.cos()).round() as i32;
    c.line(hx + nx, hy + ny, tipx + nx, tipy + ny, p.metal_light);
    c.line(hx - nx, hy - ny, tipx - nx, tipy - ny, p.metal_dark);
    c.set(tipx + nx, tipy + ny, p.metal_light);
    // Guarda perpendicular y empuñadura.
    let gx = -(angle.sin() * 3.0).round() as i32;
    let gy = (angle.cos() * 3.0).round() as i32;
    c.line(hx - gx, hy - gy, hx + gx, hy + gy, p.gold);
    let bx = hx - (angle.cos() * 2.6).round() as i32;
    let by = hy - (angle.sin() * 2.6).round() as i32;
    c.thick_line(hx, hy, bx, by, 2, p.leather_dark);
    c.rect(bx - 1, by - 1, 3, 3, p.gold);
    c.set(bx, by, p.glow);
}

fn bow(c: &mut Canvas, p: &Palette, hx: i32, hy: i32, arc: f32) {
    let cx = hx + 3;
    let top = hy - 12;
    let bottom = hy + 4;
    c.line(cx - 1, top, cx + 2, top + 4, p.leather);
    c.line(cx + 2, top + 4, cx + 3, hy - 4, p.leather);
    c.line(cx + 3, hy - 4, cx + 3, hy + 1, p.leather);
    c.line(cx + 3, hy + 1, cx + 2, bottom - 3, p.leather);
    c.line(cx + 2, bottom - 3, cx - 1, bottom, p.leather);
    c.line(cx - 1, top, cx - 1, bottom, p.metal_light);
    c.set(cx + 3, hy - 4, p.gold);
    c.set(cx + 3, hy + 1, p.gold);
    if arc > 0.18 {
        let tip = hx + 8 + (arc * 4.0).round() as i32;
        c.line(hx, hy - 2, tip, hy - 2, p.leather);
        c.rect(tip - 1, hy - 3, 1, 3, p.metal_light);
        c.set(hx + 1, hy - 3, p.glow);
        c.set(hx + 1, hy - 1, p.glow);
    }
}

fn staff(c: &mut Canvas, p: &Palette, hx: i32, hy: i32, arc: f32, class: Class) {
    let top_x = hx + 1;
    let top_y = hy - 15;
    let shaft = if class == Class::Cleric {
        p.gold
    } else if class == Class::Necro {
        p.bone
    } else {
        p.leather
    };
    c.thick_line(hx, hy + 2, top_x, top_y, 1, shaft);
    c.set(top_x - 1, top_y + 3, lighten(shaft, 0.35));
    match class {
        Class::Necro => {
            c.rect(top_x - 2, top_y - 3, 5, 4, p.bone);
            c.rect(top_x - 1, top_y - 2, 3, 2, p.bone_dark);
            c.set(top_x - 1, top_y - 2, p.glow);
            c.set(top_x + 1, top_y - 2, p.glow);
            c.rect(top_x - 1, top_y, 3, 1, OUTLINE);
        }
        Class::Cleric => {
            c.rect(top_x - 1, top_y - 4, 3, 5, p.gold);
            c.rect(top_x - 3, top_y - 3, 7, 1, p.gold);
            c.rect(top_x, top_y - 4, 1, 5, p.glow);
        }
        _ => {
            c.rect(top_x - 1, top_y - 2, 3, 3, p.accent);
            c.set(top_x - 1, top_y - 2, p.metal_light);
            c.set(top_x, top_y - 1, p.metal_light);
        }
    }
    if arc > 0.18 {
        let r = 3 + (arc * 3.0).round() as i32;
        let oy = top_y - 1;
        c.set(top_x - r, oy, p.glow);
        c.set(top_x + r, oy, p.glow);
        c.set(top_x, oy - r, p.glow);
        c.set(top_x, oy + r, p.glow);
        c.set(top_x - r + 1, oy - r + 1, p.glow);
        c.set(top_x + r - 1, oy + r - 1, p.glow);
    }
}

fn daggers(c: &mut Canvas, p: &Palette, hx: i32, hy: i32, arc: f32) {
    let reach = 5 + (arc * 2.0).round() as i32;
    c.line(hx, hy, hx + reach, hy - 4, p.metal_light);
    c.line(hx, hy + 1, hx + reach, hy - 3, p.metal);
    c.set(hx + 1, hy, p.leather_dark);
    let lx = 12;
    let ly = 28;
    c.line(lx, ly, lx - reach, ly - 4, p.metal_light);
    c.line(lx, ly + 1, lx - reach, ly - 3, p.metal);
}

fn slash(c: &mut Canvas, p: &Palette, arc: f32) {
    if arc <= 0.12 {
        return;
    }
    for i in 0..10 {
        let angle = -1.05 + i as f32 * 0.145;
        let x = 22 + (angle.cos() * 13.0).round() as i32;
        let y = 21 + (angle.sin() * 13.0).round() as i32;
        if c.get(x, y).a <= 0.0 {
            c.set(x, y, p.metal_light);
        }
        let x2 = 22 + (angle.cos() * 11.0).round() as i32;
        let y2 = 21 + (angle.sin() * 11.0).round() as i32;
        if c.get(x2, y2).a <= 0.0 {
            c.set(x2, y2, p.accent);
        }
    }
}

/// Dibuja un héroe pixel art. `pos` es el punto de los pies.
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
    let p = palette(class, skin_tint);
    let flip = facing.x < -0.18;
    let look = if facing.x.abs() > 0.25 { 1 } else { 0 };
    let pace = if moving { 1.0 } else { 0.0 };
    let s = walk_t.sin();
    let lift_l = (s.max(0.0) * 2.6 * pace).round() as i32;
    let lift_r = ((-s).max(0.0) * 2.6 * pace).round() as i32;
    let bob = if moving {
        -(((walk_t * 2.0).sin().abs()) * pace).round() as i32
    } else {
        -((time * 2.1).sin().max(0.0)).round() as i32
    };
    let arm_l = if moving {
        (s * 1.6 * pace).round() as i32
    } else {
        0
    };
    let arm_r = -arm_l;
    let swing = (swing_t / 0.16).clamp(0.0, 1.0);
    let arc = (swing * std::f32::consts::PI).sin();
    let hand = (
        27 + (arc * 2.0).round() as i32,
        28 + bob + arm_r - (arc * 4.0).round() as i32,
    );
    let a = Anim {
        bob,
        lift_l,
        lift_r,
        arm_l,
        hand,
        time,
        look,
    };

    let mut c = Canvas::new(CANVAS_W, CANVAS_H);
    draw_cape(&mut c, &p, &a, class);
    draw_legs(&mut c, &p, &a, class);
    draw_torso(&mut c, &p, &a, class);
    draw_arms(&mut c, &p, &a);
    draw_head(&mut c, &p, &a, class, hat);
    draw_cosmetic_hat(&mut c, &p, &a, hat);
    draw_mask(&mut c, &p, &a, mask);

    match class {
        Class::Knight => {
            draw_shield(&mut c, &p, &a);
            sword(&mut c, &p, a.hand.0, a.hand.1, arc);
        }
        Class::Archer => bow(&mut c, &p, a.hand.0, a.hand.1, arc),
        Class::Mage | Class::Necro | Class::Cleric => {
            staff(&mut c, &p, a.hand.0, a.hand.1, arc, class)
        }
        Class::Assassin => {
            daggers(&mut c, &p, a.hand.0, a.hand.1, arc);
            slash(&mut c, &p, arc);
        }
        Class::Merchant => {
            sword(&mut c, &p, a.hand.0, a.hand.1, arc);
            c.rect(22, 27 + bob, 4, 4, p.leather);
            c.rect(23, 28 + bob, 2, 2, p.gold);
            c.set(23, 28 + bob, p.glow);
            slash(&mut c, &p, arc);
        }
    }

    if matches!(class, Class::Knight) {
        slash(&mut c, &p, arc);
    }

    // Destello en la mano en el pico del ataque: se lee incluso en clases
    // cuyo arma no dibuja un barrido amplio.
    if arc > 0.55 {
        let (hx, hy) = a.hand;
        let r = 2 + (arc * 3.0).round() as i32;
        for (dx, dy) in [(r, 0), (-r, 0), (0, r), (0, -r)] {
            if c.get(hx + dx, hy + dy).a <= 0.0 {
                c.set(hx + dx, hy + dy, p.glow);
            }
        }
        if c.get(hx, hy - r - 1).a <= 0.0 {
            c.set(hx, hy - r - 1, p.metal_light);
        }
    }

    c.outline(OUTLINE);
    let flash = (hit_flash / 0.18).clamp(0.0, 1.0);
    let lunge = if arc > 0.35 { 2.0 } else { 0.0 };
    let draw_pos = pos + vec2(facing.x.signum() * lunge, 0.0);
    c.draw(draw_pos, flip, flash);
}

struct EnemyPalette {
    body: Color,
    light: Color,
    dark: Color,
    eye: Color,
    accent: Color,
    metal: Color,
    metal_dark: Color,
    bone: Color,
    leather: Color,
    leather_dark: Color,
    gold: Color,
    glow: Color,
}

fn enemy_palette(tint: Color, elite: bool, shiny: bool) -> EnemyPalette {
    let mut body = tint;
    if shiny {
        body = mix(body, Color::new(0.40, 0.90, 0.90, 1.0), 0.38);
    }
    EnemyPalette {
        body,
        light: lighten(body, 0.34),
        dark: darken(body, 0.58),
        eye: Color::new(1.0, 0.94, 0.74, 1.0),
        accent: if elite {
            Color::new(0.96, 0.78, 0.34, 1.0)
        } else {
            lighten(tint, 0.28)
        },
        metal: Color::new(0.62, 0.66, 0.74, 1.0),
        metal_dark: Color::new(0.30, 0.33, 0.40, 1.0),
        bone: Color::new(0.84, 0.82, 0.70, 1.0),
        leather: Color::new(0.38, 0.26, 0.17, 1.0),
        leather_dark: Color::new(0.20, 0.13, 0.09, 1.0),
        gold: Color::new(0.96, 0.78, 0.34, 1.0),
        glow: Color::new(1.0, 0.96, 0.72, 1.0),
    }
}

fn draw_slime(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, _time: f32) {
    // Gelatina redondeada: se aplasta y estira al caminar y abre la boca al atacar.
    let bob = (walk * 2.4).sin().round() as i32;
    let wob = ((walk * 2.4).cos() * 0.6).round() as i32;
    let top = 26 + bob;
    let l = 14 - wob;
    let r = 26 + wob;
    c.rect(17, top, 6, 1, p.body);
    c.rect(15, top + 1, 10, 2, p.body);
    c.rect(l, top + 3, r - l + 1, 3, p.body);
    c.rect(l + 1, top + 6, r - l - 1, 2, p.body);
    c.rect(l + 2, top + 8, r - l - 3, 1, p.dark);
    c.rect(l + 2, top + 9, 2, 1, p.dark);
    c.rect(r - 3, top + 9, 2, 1, p.dark);
    // Volumen: luz arriba a la izquierda, sombra en el borde derecho.
    c.rect(l + 1, top + 3, 4, 2, p.light);
    c.rect(l + 1, top + 1, 3, 1, p.light);
    c.rect(r - 2, top + 3, 2, 4, p.dark);
    c.rect(l + 1, top + 7, r - l - 2, 1, p.dark);
    c.set(l + 2, top + 2, lighten(p.light, 0.45));
    // Ojos grandes y boca.
    c.rect(17, top + 3, 2, 2, p.eye);
    c.rect(21, top + 3, 2, 2, p.eye);
    c.set(18, top + 4, OUTLINE);
    c.set(22, top + 4, OUTLINE);
    if arc > 0.25 {
        c.rect(18, top + 6, 4, 2, OUTLINE);
        c.set(18, top + 7, p.eye);
        c.set(21, top + 7, p.eye);
    } else {
        c.rect(19, top + 6, 2, 1, OUTLINE);
    }
}

fn draw_bat(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, _time: f32) {
    // Alas de membrana con tres puntos de apoyo y aleteo marcado.
    let flap = (walk * 7.0).sin();
    let up = (flap * 3.0).round() as i32;
    let bob = (walk * 3.0).cos().round() as i32;
    let cx = 20;
    let cy = 28 + bob;
    let tip_l = (cx - 10, cy - 4 + up);
    let tip_r = (cx + 10, cy - 4 + up);
    c.tri((cx - 2, cy - 2), tip_l, (cx - 7, cy + 4), p.dark);
    c.tri((cx + 2, cy - 2), tip_r, (cx + 7, cy + 4), p.dark);
    c.tri(
        (cx - 2, cy - 1),
        (cx - 8, cy - 2 + up),
        (cx - 5, cy + 3),
        p.body,
    );
    c.tri(
        (cx + 2, cy - 1),
        (cx + 8, cy - 2 + up),
        (cx + 5, cy + 3),
        p.body,
    );
    c.line(cx - 2, cy - 2, tip_l.0, tip_l.1, p.bone);
    c.line(cx - 2, cy - 2, cx - 7, cy + 4, p.bone);
    c.line(cx + 2, cy - 2, tip_r.0, tip_r.1, p.bone);
    c.line(cx + 2, cy - 2, cx + 7, cy + 4, p.bone);
    // Cuerpo, orejas y colmillos.
    c.rect(cx - 3, cy - 4, 6, 8, p.body);
    c.rect(cx - 2, cy - 4, 4, 2, p.light);
    c.rect(cx - 3, cy + 3, 6, 1, p.dark);
    c.rect(cx - 3, cy - 7, 2, 3, p.dark);
    c.rect(cx + 1, cy - 7, 2, 3, p.dark);
    c.set(cx - 2, cy - 1, p.eye);
    c.set(cx + 1, cy - 1, p.eye);
    c.set(cx - 2, cy - 2, p.accent);
    c.set(cx + 1, cy - 2, p.accent);
    if arc > 0.2 {
        c.rect(cx - 2, cy + 2, 4, 2, OUTLINE);
        c.rect(cx - 2, cy + 3, 1, 1, p.bone);
        c.rect(cx + 1, cy + 3, 1, 1, p.bone);
    } else {
        c.rect(cx - 1, cy + 3, 1, 2, p.bone);
        c.rect(cx + 1, cy + 3, 1, 2, p.bone);
    }
}

fn draw_brute(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, _time: f32) {
    // Bruto encorvado: hombros anchos, cabeza hundida y maza pesada.
    let bob = (walk * 2.0).sin().round() as i32;
    let step = (walk * 2.0).sin().round() as i32;
    let y = 19 + bob;
    c.rect(15, 31 + bob, 4, (7 - step).max(2), p.dark);
    c.rect(21, 31 + bob, 4, (7 + step).max(2), p.dark);
    c.rect(14, 37 - step, 6, 2, p.leather);
    c.rect(20, 37 + step, 6, 2, p.leather);
    c.rect(14, 36 - step, 6, 1, p.leather_dark);
    c.rect(20, 36 + step, 6, 1, p.leather_dark);
    // Torso y hombreras.
    c.rect(13, y, 15, 12, p.body);
    c.rect(11, y, 4, 5, p.dark);
    c.rect(26, y, 4, 5, p.dark);
    c.rect(14, y, 13, 2, p.light);
    c.rect(13, y + 9, 15, 3, p.dark);
    c.rect(13, y + 9, 15, 1, p.leather);
    c.rect(19, y + 9, 2, 2, p.gold);
    c.line(16, y + 3, 22, y + 7, p.dark);
    c.line(16, y + 4, 22, y + 8, p.body);
    // Cabeza con mandíbula y colmillos.
    c.rect(16, 9 + bob, 8, 10, p.body);
    c.rect(16, 9 + bob, 8, 2, p.light);
    c.rect(17, 16 + bob, 6, 3, p.dark);
    c.rect(18, 17 + bob, 4, 1, OUTLINE);
    c.rect(18, 17 + bob, 1, 2, p.bone);
    c.rect(21, 17 + bob, 1, 2, p.bone);
    c.rect(17, 12 + bob, 2, 2, p.eye);
    c.rect(21, 12 + bob, 2, 2, p.eye);
    c.set(18, 13 + bob, OUTLINE);
    c.set(22, 13 + bob, OUTLINE);
    c.tri((16, 9 + bob), (11, 3 + bob), (15, 9 + bob), p.bone);
    c.tri((24, 9 + bob), (29, 3 + bob), (25, 9 + bob), p.bone);
    c.set(12, 4 + bob, p.light);
    c.set(28, 4 + bob, p.light);
    // Brazos y maza: sube al descansar y cae en el ataque.
    c.thick_line(13, y + 3, 10, y + 10, 3, p.body);
    c.rect(9, y + 10, 3, 2, p.dark);
    c.thick_line(27, y + 3, 28, y + 8, 3, p.body);
    let hx = 28 + (arc * 2.0).round() as i32;
    let hy = 27 + bob - (arc * 3.0).round() as i32;
    c.rect(hx - 1, hy - 1, 3, 3, p.dark);
    let tx = hx + 4;
    let ty = hy - 10 + (arc * 12.0).round() as i32;
    c.thick_line(hx, hy, tx, ty, 2, p.leather);
    c.rect(tx - 2, ty - 3, 5, 5, p.leather);
    c.set(tx - 1, ty - 2, p.metal);
    c.set(tx + 1, ty, p.metal);
    c.set(tx, ty - 4, p.metal_dark);
}

fn draw_cultist(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, _time: f32) {
    // Túnica con capucha picuda, rostro en sombra y báculo con orbe.
    let bob = (walk * 2.0).sin().round() as i32;
    let sway = (walk * 1.4).sin().round() as i32;
    c.tri((20, 18 + bob), (13, 36 + bob), (27, 36 + bob), p.dark);
    c.tri((20, 20 + bob), (15, 36 + bob), (25, 36 + bob), p.body);
    c.rect(14, 20 + bob, 12, 3, p.body);
    c.rect(15, 20 + bob, 10, 1, p.light);
    // Harapos del bajo de la túnica.
    for x in (14..=26).step_by(2) {
        c.rect(x, 34 + bob, 1, 3, p.dark);
    }
    c.rect(15, 29 + bob, 10, 1, p.leather);
    c.rect(19, 29 + bob, 2, 2, p.gold);
    // Capucha y rostro.
    c.tri((20, 8 + bob), (12, 24 + bob), (28, 24 + bob), p.dark);
    c.tri((20, 10 + bob), (14, 24 + bob), (26, 24 + bob), p.body);
    c.rect(17, 13 + bob, 6, 7, OUTLINE);
    c.rect(18, 14 + bob, 4, 5, Color::new(0.05, 0.04, 0.07, 1.0));
    c.set(18, 15 + bob, p.eye);
    c.set(21, 15 + bob, p.eye);
    c.set(18, 16 + bob, p.accent);
    c.set(21, 16 + bob, p.accent);
    // Mangas y manos.
    c.thick_line(15, 22 + bob, 12 + sway, 28 + bob, 3, p.body);
    c.rect(11 + sway, 28 + bob, 2, 2, p.bone);
    c.thick_line(25, 22 + bob, 27, 26 + bob, 3, p.body);
    // Báculo y orbe; al atacar se eleva y brilla más.
    let lift = (arc * 3.0).round() as i32;
    let top = 8 + bob - lift;
    c.thick_line(28, 37, 28, top + 3, 1, p.leather);
    c.set(27, top + 6, lighten(p.leather, 0.35));
    let orb = 2 + (arc * 2.0).round() as i32;
    c.rect(28 - orb, top - orb + 1, orb * 2 + 1, orb * 2 + 1, p.accent);
    c.rect(
        28 - orb + 1,
        top - orb + 2,
        orb * 2 - 1,
        orb * 2 - 1,
        p.light,
    );
    c.set(28, top + 1, p.eye);
}

fn draw_wraith(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, time: f32) {
    // Espectro flotante: capucha, jirones y garras que se alargan al atacar.
    let hover = ((time * 2.2 + walk).sin() * 2.0).round() as i32;
    let sway = (time * 2.0).sin().round() as i32;
    let ghost = Color::new(p.body.r, p.body.g, p.body.b, 0.86);
    let ghost_dark = Color::new(p.dark.r, p.dark.g, p.dark.b, 0.86);
    // Jirones inferiores.
    for x in 14..=26 {
        let hem = 33 + hover + ((x as f32 * 0.8 + time * 2.4).sin() * 2.0).round() as i32;
        let col = if x % 3 == 0 { ghost_dark } else { ghost };
        c.rect(x, 21 + hover, 1, (hem - 21 - hover).max(1), col);
    }
    c.rect(15, 20 + hover, 10, 3, p.dark);
    // Capucha.
    c.tri((20, 8 + hover), (13, 24 + hover), (27, 24 + hover), p.dark);
    c.tri((20, 10 + hover), (15, 23 + hover), (25, 23 + hover), p.body);
    c.rect(17, 13 + hover, 6, 7, OUTLINE);
    c.rect(18, 14 + hover, 4, 5, Color::new(0.04, 0.035, 0.06, 1.0));
    c.set(18, 15 + hover, p.eye);
    c.set(21, 15 + hover, p.eye);
    c.set(18, 16 + hover, p.glow);
    c.set(21, 16 + hover, p.glow);
    // Brazos y garras.
    let reach = 2 + (arc * 4.0).round() as i32;
    c.thick_line(15, 22 + hover, 12 - sway - reach / 2, 27 + hover, 2, p.body);
    c.thick_line(25, 22 + hover, 28 + sway + reach / 2, 27 + hover, 2, p.body);
    for k in 0..3 {
        c.set(11 - sway - reach / 2 + k, 28 + hover + k % 2, p.bone);
        c.set(29 + sway + reach / 2 - k, 28 + hover + k % 2, p.bone);
    }
}

fn draw_bomber(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, _time: f32) {
    // Bombardero con gafas, mochila explosiva y bomba en la mano.
    let bob = (walk * 2.2).sin().round() as i32;
    let step = (walk * 2.2).sin().round() as i32;
    c.rect(16, 32 + bob, 3, (6 - step).max(2), p.dark);
    c.rect(21, 32 + bob, 3, (6 + step).max(2), p.dark);
    c.rect(15, 37 - step, 4, 2, p.leather);
    c.rect(21, 37 + step, 4, 2, p.leather);
    // Cuerpo redondeado y correa.
    c.rect(15, 22 + bob, 10, 2, p.body);
    c.rect(14, 24 + bob, 12, 7, p.body);
    c.rect(15, 31 + bob, 10, 2, p.dark);
    c.rect(16, 23 + bob, 8, 1, p.light);
    c.rect(15, 29 + bob, 10, 1, p.leather);
    // Cabeza con gafas.
    c.rect(16, 14 + bob, 8, 8, p.body);
    c.rect(16, 14 + bob, 8, 2, p.light);
    c.rect(13, 16 + bob, 2, 3, p.body);
    c.rect(25, 16 + bob, 2, 3, p.body);
    c.rect(17, 17 + bob, 2, 2, p.metal);
    c.rect(21, 17 + bob, 2, 2, p.metal);
    c.set(18, 18 + bob, p.glow);
    c.set(22, 18 + bob, p.glow);
    c.rect(19, 18 + bob, 2, 1, p.metal_dark);
    c.rect(18, 20 + bob, 4, 1, OUTLINE);
    // Mochila con bomba y mecha.
    c.rect(25, 24 + bob, 5, 6, p.dark);
    c.rect(26, 23 + bob, 3, 1, p.metal_dark);
    c.set(27, 22 + bob, p.accent);
    c.set(27, 21 + bob, p.eye);
    // Brazo y bomba.
    let hy = 27 + bob - (arc * 5.0).round() as i32;
    c.thick_line(15, 25 + bob, 12, hy, 2, p.body);
    c.rect(10, hy - 1, 3, 3, p.dark);
    c.set(11, hy - 2, p.accent);
    if arc > 0.3 {
        c.set(9, hy - 4, p.light);
        c.set(12, hy - 4, p.accent);
        c.set(11, hy - 5, p.eye);
    }
}

fn draw_sentinel(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, _time: f32) {
    // Centinela acorazado con lanza y escudo; la lanza avanza al atacar.
    let bob = (walk * 1.6).sin().round() as i32;
    let step = (walk * 2.0).sin().round() as i32;
    c.rect(15, 31 + bob, 4, (7 - step).max(2), p.metal_dark);
    c.rect(21, 31 + bob, 4, (7 + step).max(2), p.metal_dark);
    c.rect(14, 37 - step, 6, 2, p.metal);
    c.rect(20, 37 + step, 6, 2, p.metal);
    // Torso, hombreras y cinturón.
    c.rect(14, 19 + bob, 12, 12, p.metal);
    c.rect(15, 19 + bob, 10, 2, p.light);
    c.rect(14, 28 + bob, 12, 3, p.metal_dark);
    c.rect(14, 28 + bob, 12, 1, p.accent);
    c.rect(11, 19 + bob, 4, 4, p.metal_dark);
    c.rect(25, 19 + bob, 4, 4, p.metal_dark);
    c.rect(11, 19 + bob, 4, 1, p.metal);
    c.rect(25, 19 + bob, 4, 1, p.metal);
    // Yelmo con visor encendido y cresta.
    c.rect(16, 11 + bob, 8, 8, p.metal);
    c.rect(16, 11 + bob, 8, 1, p.light);
    c.rect(17, 14 + bob, 6, 2, OUTLINE);
    c.set(18, 14 + bob, p.glow);
    c.set(22, 14 + bob, p.glow);
    c.rect(19, 15 + bob, 2, 1, p.metal_dark);
    c.rect(19, 9 + bob, 2, 2, p.accent);
    c.set(20, 8 + bob, p.light);
    // Escudo.
    c.rect(8, 22 + bob, 6, 8, p.metal_dark);
    c.rect(9, 23 + bob, 4, 6, p.metal);
    c.rect(9, 23 + bob, 4, 1, p.light);
    c.rect(10, 25 + bob, 2, 2, p.accent);
    // Lanza: de vertical a horizontal en el ataque.
    let tip_x = 30 + (arc * 5.0).round() as i32;
    let tip_y = 10 + (arc * 8.0).round() as i32;
    c.thick_line(27, 31 + bob, tip_x, tip_y, 1, p.leather);
    c.rect(tip_x - 1, tip_y - 3, 2, 4, p.metal);
    c.set(tip_x, tip_y - 4, p.light);
    c.rect(26, 30 + bob, 3, 2, p.dark);
}

fn draw_eye(c: &mut Canvas, p: &EnemyPalette, walk: f32, arc: f32, time: f32) {
    // Ojo flotante con esclerótica, iris que sigue al héroe y tentáculos.
    let bob = ((time * 2.0 + walk).sin() * 2.0).round() as i32;
    let look = (time * 1.3).sin();
    let lx = (look * 2.0).round() as i32;
    let ly = (time * 1.1).cos().round() as i32;
    for k in 0..4 {
        let x = 16 + k * 3;
        let sway = ((time * 3.0 + k as f32).sin() * 1.6).round() as i32;
        let col = if k % 2 == 0 { p.dark } else { p.body };
        c.thick_line(x, 29 + bob, x + sway, 36 + bob, 1, col);
        c.set(x + sway, 37 + bob, p.accent);
    }
    // Globo ocular.
    c.rect(17, 18 + bob, 6, 1, p.body);
    c.rect(15, 19 + bob, 10, 2, p.body);
    c.rect(14, 21 + bob, 12, 5, p.body);
    c.rect(15, 26 + bob, 10, 2, p.dark);
    c.rect(17, 28 + bob, 6, 1, p.dark);
    c.rect(16, 20 + bob, 8, 1, p.light);
    c.rect(15, 21 + bob, 2, 4, p.dark);
    // Esclerótica y venas.
    c.rect(16, 21 + bob, 8, 5, Color::new(0.96, 0.93, 0.82, 1.0));
    c.rect(17, 22 + bob, 6, 3, Color::new(0.99, 0.97, 0.90, 1.0));
    c.set(16, 23 + bob, p.dark);
    c.set(23, 22 + bob, p.dark);
    c.set(17, 25 + bob, p.dark);
    // Iris y pupila.
    let iris = if arc > 0.2 { 2 } else { 3 };
    c.rect(
        20 + lx - iris,
        23 + bob + ly - iris,
        iris * 2,
        iris * 2,
        p.accent,
    );
    c.rect(20 + lx - 1, 23 + bob + ly - 1, 2, 2, OUTLINE);
    c.set(19 + lx, 22 + bob + ly, p.eye);
    if arc > 0.2 {
        // Destello de carga alrededor del ojo.
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.0 + time * 1.5;
            let x = 20 + (a.cos() * 9.0).round() as i32;
            let y = 23 + bob + (a.sin() * 7.0).round() as i32;
            c.set(x, y, p.glow);
        }
    }
}

const BOSS_W: i32 = 56;
const BOSS_H: i32 = 58;
const BOSS_FEET: i32 = 53;

struct BossPalette {
    body: Color,
    light: Color,
    dark: Color,
    deep: Color,
    accent: Color,
    glow: Color,
    bone: Color,
    bone_dark: Color,
    metal: Color,
    metal_dark: Color,
    stone: Color,
    stone_light: Color,
}

fn boss_palette(tint: Color, elite: bool) -> BossPalette {
    let accent = if elite {
        Color::new(0.98, 0.80, 0.32, 1.0)
    } else {
        lighten(tint, 0.58)
    };
    BossPalette {
        body: tint,
        light: lighten(tint, 0.32),
        dark: darken(tint, 0.55),
        deep: darken(tint, 0.30),
        accent,
        glow: lighten(accent, 0.55),
        bone: Color::new(0.86, 0.83, 0.72, 1.0),
        bone_dark: Color::new(0.55, 0.51, 0.43, 1.0),
        metal: Color::new(0.62, 0.66, 0.74, 1.0),
        metal_dark: Color::new(0.28, 0.31, 0.38, 1.0),
        stone: mix(tint, Color::new(0.11, 0.10, 0.14, 1.0), 0.72),
        stone_light: mix(tint, Color::new(0.27, 0.25, 0.31, 1.0), 0.52),
    }
}

fn boss_step(walk: f32) -> (i32, i32) {
    let s = (walk * 2.1).sin();
    (
        (s.max(0.0) * 2.0).round() as i32,
        ((-s).max(0.0) * 2.0).round() as i32,
    )
}

fn draw_boss_king(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let (ll, lr) = boss_step(walk);
    // Capa amplia con borde ondulado y forro interior.
    for x in 6..=50 {
        let edge = (x as i32 - 28).abs() as f32;
        let top = 16 + (edge * 0.48).round() as i32;
        let wave = ((x as f32 * 0.55 + time * 2.2).sin() * 1.8).round() as i32;
        let bottom = 50 + wave;
        let col = if x % 6 == 0 { p.dark } else { p.deep };
        c.rect(x, top, 1, (bottom - top).max(1), col);
    }
    for x in 8..=20 {
        let wave = ((x as f32 * 0.7 + time * 2.2).sin() * 1.4).round() as i32;
        c.rect(x, 20, 1, (30 - wave - 20).max(1), p.light);
    }
    // Piernas y botas.
    c.rect(19, 39, 6, (49 - ll - 39).max(2), p.metal_dark);
    c.rect(31, 39, 6, (49 - lr - 39).max(2), p.metal_dark);
    c.rect(18, 49 - ll, 8, 4, p.metal_dark);
    c.rect(30, 49 - lr, 8, 4, p.metal_dark);
    c.rect(18, 49 - ll, 8, 1, p.metal);
    c.rect(30, 49 - lr, 8, 1, p.metal);
    // Túnica y costillas.
    c.rect(20, 25, 16, 17, p.dark);
    c.rect(21, 25, 14, 2, p.deep);
    c.rect(27, 26, 2, 15, p.bone_dark);
    for y in [28, 32, 36] {
        c.rect(22, y, 12, 2, p.bone);
        c.rect(22, y + 2, 12, 1, p.bone_dark);
    }
    c.rect(22, 40, 12, 2, p.accent);
    // Brazos.
    c.thick_line(21, 27, 14, 37, 4, p.dark);
    c.rect(12, 37, 4, 4, p.bone);
    c.thick_line(35, 27, 42, 35, 4, p.dark);
    c.rect(41, 34, 4, 4, p.bone);
    // Calavera.
    c.disc(28, 16, 8, p.bone);
    c.rect(23, 21, 10, 5, p.bone);
    c.rect(23, 21, 10, 1, p.bone_dark);
    for x in (24..=31).step_by(2) {
        c.rect(x, 23, 1, 3, p.bone_dark);
    }
    c.rect(21, 14, 4, 4, OUTLINE);
    c.rect(31, 14, 4, 4, OUTLINE);
    c.rect(22, 15, 2, 2, RED);
    c.rect(32, 15, 2, 2, RED);
    c.set(22, 15, p.glow);
    c.set(32, 15, p.glow);
    c.rect(27, 18, 2, 2, OUTLINE);
    c.rect(25, 20, 6, 1, OUTLINE);
    // Corona de cinco púas.
    c.rect(18, 6, 20, 4, p.accent);
    c.rect(18, 6, 20, 1, p.glow);
    for x in [18, 23, 28, 33, 38] {
        c.tri((x, 7), (x + 4, 7), (x + 2, 1), p.accent);
    }
    c.rect(27, 7, 2, 2, RED);
    c.set(19, 8, p.glow);
    c.set(37, 8, p.glow);
    // Espada que baja al atacar.
    let hx = 43;
    let hy = 36;
    let tip_x = 48 + (arc * 4.0).round() as i32;
    let tip_y = 14 + (arc * 17.0).round() as i32;
    c.thick_line(hx, hy, tip_x, tip_y, 2, p.metal);
    c.line(hx - 1, hy, tip_x - 1, tip_y, p.metal_dark);
    c.line(hx + 1, hy, tip_x + 1, tip_y, p.light);
    c.line(hx - 4, hy + 1, hx + 3, hy - 3, p.accent);
    c.rect(hx - 1, hy, 3, 4, p.bone_dark);
}

fn draw_boss_brute(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let bob = (walk * 2.0).sin().round() as i32;
    let (ll, lr) = boss_step(walk);
    // Piernas de tronco.
    c.rect(16, 39, 9, (49 - ll - 39).max(2), p.dark);
    c.rect(31, 39, 9, (49 - lr - 39).max(2), p.dark);
    c.rect(14, 49 - ll, 12, 4, p.metal_dark);
    c.rect(30, 49 - lr, 12, 4, p.metal_dark);
    c.rect(14, 49 - ll, 12, 1, p.metal);
    c.rect(30, 49 - lr, 12, 1, p.metal);
    // Torso enorme y panza.
    c.ellipse(28, 31 + bob, 16, 13, p.body);
    c.ellipse(27, 32 + bob, 10, 8, p.light);
    c.rect(13, 39 + bob, 30, 4, p.bone_dark);
    c.rect(13, 39 + bob, 30, 1, p.bone);
    c.rect(26, 39 + bob, 4, 4, p.accent);
    // Hombreras.
    c.disc(14, 23 + bob, 7, p.dark);
    c.disc(42, 23 + bob, 7, p.dark);
    c.disc(14, 22 + bob, 5, p.body);
    c.disc(42, 22 + bob, 5, p.body);
    c.rect(10, 20 + bob, 3, 2, p.light);
    c.rect(38, 20 + bob, 3, 2, p.light);
    // Brazos.
    c.thick_line(14, 27 + bob, 9, 38 + bob, 6, p.body);
    c.disc(9, 40 + bob, 3, p.dark);
    c.thick_line(42, 27 + bob, 45, 34 + bob, 6, p.body);
    c.disc(45, 35 + bob, 3, p.dark);
    // Cabeza con mandíbula y colmillos.
    c.disc(28, 14 + bob, 7, p.body);
    c.rect(22, 18 + bob, 12, 5, p.dark);
    c.rect(23, 19 + bob, 10, 1, OUTLINE);
    c.rect(23, 19 + bob, 1, 3, p.bone);
    c.rect(31, 19 + bob, 1, 3, p.bone);
    c.rect(24, 12 + bob, 3, 2, p.glow);
    c.rect(29, 12 + bob, 3, 2, p.glow);
    c.set(25, 12 + bob, OUTLINE);
    c.set(30, 12 + bob, OUTLINE);
    c.rect(26, 16 + bob, 4, 1, OUTLINE);
    // Cuernos gigantes.
    c.tri((23, 11 + bob), (6, 1 + bob), (19, 15 + bob), p.bone);
    c.tri((33, 11 + bob), (50, 1 + bob), (37, 15 + bob), p.bone);
    c.tri((22, 10 + bob), (10, 4 + bob), (20, 13 + bob), p.bone_dark);
    c.tri((34, 10 + bob), (46, 4 + bob), (36, 13 + bob), p.bone_dark);
    // Maza que sube y cae.
    let hx = 45;
    let hy = 34 + bob;
    let tx = 49 + (arc * 2.0).round() as i32;
    let ty = 15 + (arc * 13.0).round() as i32;
    c.thick_line(hx, hy, tx, ty, 3, p.bone_dark);
    c.disc(tx, ty, 5, p.stone_light);
    c.disc(tx, ty, 3, p.metal_dark);
    for (dx, dy) in [(5, 0), (-5, 0), (0, 5), (0, -5)] {
        c.set(tx + dx, ty + dy, p.metal);
    }
    let _ = time;
}

fn draw_boss_wraith(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let hover = ((time * 2.1).sin() * 2.0).round() as i32;
    let flap = (time * 2.8).sin();
    let up = (flap * 4.0).round() as i32;
    let top = 18 + hover;
    // Alas exteriores y membrana.
    c.tri((22, top), (1, 4 + up + hover), (9, 42 + hover), p.dark);
    c.tri((34, top), (55, 4 + up + hover), (47, 42 + hover), p.dark);
    c.tri(
        (22, top + 1),
        (7, 10 + up + hover),
        (15, 40 + hover),
        p.body,
    );
    c.tri(
        (34, top + 1),
        (49, 10 + up + hover),
        (41, 40 + hover),
        p.body,
    );
    c.line(22, top, 1, 4 + up + hover, p.bone_dark);
    c.line(22, top, 9, 42 + hover, p.bone_dark);
    c.line(34, top, 55, 4 + up + hover, p.bone_dark);
    c.line(34, top, 47, 42 + hover, p.bone_dark);
    c.line(22, top + 1, 15, 40 + hover, p.bone);
    c.line(34, top + 1, 41, 40 + hover, p.bone);
    // Túnica con jirones.
    for x in 20..=36 {
        let hem = 47 + hover + ((x as f32 * 0.8 + time * 2.6).sin() * 2.0).round() as i32;
        let col = if x % 4 == 0 { p.dark } else { p.body };
        c.rect(x, top, 1, (hem - top).max(1), col);
    }
    for x in (20..=36).step_by(4) {
        c.rect(x, 42 + hover, 1, 6, p.deep);
    }
    // Capucha y rostro.
    c.tri((28, 5 + hover), (18, 27 + hover), (38, 27 + hover), p.dark);
    c.tri((28, 9 + hover), (21, 26 + hover), (35, 26 + hover), p.body);
    c.rect(23, 13 + hover, 10, 9, OUTLINE);
    c.rect(24, 14 + hover, 8, 7, Color::new(0.035, 0.03, 0.055, 1.0));
    c.rect(25, 16 + hover, 2, 2, p.glow);
    c.rect(30, 16 + hover, 2, 2, p.glow);
    c.set(25, 17 + hover, p.accent);
    c.set(30, 17 + hover, p.accent);
    c.rect(27, 20 + hover, 2, 1, p.bone_dark);
    // Brazos y garras que se alargan al atacar.
    let reach = (arc * 5.0).round() as i32;
    let sway = (time * 2.2).sin().round() as i32;
    c.thick_line(
        21,
        25 + hover,
        13 - sway - reach / 2,
        33 + hover + reach / 2,
        3,
        p.body,
    );
    c.thick_line(
        35,
        25 + hover,
        43 + sway + reach / 2,
        33 + hover + reach / 2,
        3,
        p.body,
    );
    for k in 0..3 {
        c.set(
            11 - sway - reach / 2 + k,
            35 + hover + reach / 2 + k % 2,
            p.bone,
        );
        c.set(
            45 + sway + reach / 2 - k,
            35 + hover + reach / 2 + k % 2,
            p.bone,
        );
    }
    c.rect(26, 30 + hover, 4, 2, p.accent);
    let _ = walk;
}

fn draw_boss_orb(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let hover = ((time * 1.8).sin() * 2.0).round() as i32;
    let cx = 28;
    let cy = 27 + hover;
    // Tentáculos radiales.
    let len = 21 + (arc * 3.0).round() as i32;
    for k in 0..8 {
        let ang = k as f32 * std::f32::consts::TAU / 8.0 + time * 0.45;
        let tx = cx + (ang.cos() * len as f32).round() as i32;
        let ty = cy + (ang.sin() * (len as f32 * 0.72)).round() as i32;
        c.thick_line(cx, cy, tx, ty, 2, p.dark);
        c.disc(tx, ty, 2, p.accent);
        c.set(tx, ty, p.glow);
    }
    // Globo ocular.
    c.disc(cx, cy, 14, p.body);
    c.disc(cx, cy, 12, p.deep);
    c.disc(cx, cy, 10, p.bone);
    c.disc(cx - 1, cy - 1, 8, Color::new(0.97, 0.95, 0.88, 1.0));
    // Párpado superior.
    c.rect(cx - 10, cy - 10, 20, 3, p.body);
    c.rect(cx - 8, cy - 8, 16, 1, p.light);
    // Iris que sigue al héroe.
    let lx = (time * 1.4).sin() * 3.0;
    let ly = (time * 1.1).cos() * 2.0;
    let ix = cx + lx.round() as i32;
    let iy = cy + 1 + ly.round() as i32;
    let iris = if arc > 0.25 { 3 } else { 5 };
    c.disc(ix, iy, iris, p.accent);
    c.disc(ix, iy, 2, OUTLINE);
    c.set(ix - 1, iy - 2, p.glow);
    c.set(ix + 2, iy + 1, p.light);
    // Venas.
    c.set(cx - 7, cy + 4, p.dark);
    c.set(cx - 6, cy + 5, p.dark);
    c.set(cx + 6, cy - 4, p.dark);
    // Runas en órbita.
    let ring = if arc > 0.25 { 20 } else { 18 };
    for k in 0..10 {
        let ang = k as f32 * std::f32::consts::TAU / 10.0 - time * 1.3;
        let x = cx + (ang.cos() * ring as f32).round() as i32;
        let y = cy + (ang.sin() * (ring as f32 * 0.66)).round() as i32;
        c.set(x, y, p.glow);
    }
    let _ = walk;
}

fn draw_boss_serpent(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let sway = (walk * 1.6 + time * 0.8).sin().round() as i32;
    let head_y = 15 + ((time * 1.5).sin() * 1.0).round() as i32;
    // Anillos enrollados en la base.
    c.ellipse(28, 48, 18, 5, p.dark);
    c.ellipse(28, 46, 16, 4, p.body);
    c.ellipse(27, 45, 12, 3, p.light);
    c.ellipse(28, 43, 14, 4, p.dark);
    c.ellipse(28, 41, 12, 3, p.body);
    c.ellipse(28, 39, 9, 3, p.dark);
    // Cuerpo en S.
    let bx = 30 + sway;
    c.thick_line(bx, 40, 20 + sway, 32, 7, p.body);
    c.thick_line(20 + sway, 32, 35 + sway, 26, 6, p.body);
    c.thick_line(35 + sway, 26, 28 + sway, head_y + 3, 6, p.body);
    c.thick_line(bx - 2, 40, 18 + sway, 32, 2, p.light);
    c.thick_line(18 + sway, 32, 33 + sway, 26, 2, p.light);
    c.thick_line(33 + sway, 26, 26 + sway, head_y + 4, 2, p.light);
    // Capucha desplegada.
    let hood = 11 + (arc * 4.0).round() as i32;
    c.ellipse(28 + sway, head_y + 2, hood, 9, p.dark);
    c.ellipse(28 + sway, head_y + 2, hood - 3, 7, p.body);
    c.ellipse(28 + sway, head_y + 3, hood - 6, 5, p.deep);
    // Cabeza, ojos y colmillos.
    let hx = 28 + sway;
    c.disc(hx, head_y, 8, p.body);
    c.rect(hx - 4, head_y + 4, 8, 4, p.light);
    c.rect(hx - 4, head_y + 2, 3, 3, RED);
    c.rect(hx + 1, head_y + 2, 3, 3, RED);
    c.rect(hx - 3, head_y + 3, 1, 2, OUTLINE);
    c.rect(hx + 2, head_y + 3, 1, 2, OUTLINE);
    c.rect(hx - 3, head_y + 7, 1, 3, p.bone);
    c.rect(hx + 2, head_y + 7, 1, 3, p.bone);
    // Lengua bífida.
    let tongue = if arc > 0.2 { 5 } else { 3 };
    c.rect(hx, head_y + 9, 1, tongue, RED);
    c.set(hx - 1, head_y + 9 + tongue, RED);
    c.set(hx + 1, head_y + 9 + tongue, RED);
    // Marcas de la capucha.
    c.rect(hx - 6, head_y - 2, 2, 2, p.accent);
    c.rect(hx + 4, head_y - 2, 2, 2, p.accent);
}

fn draw_boss_colossus(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let bob = (walk * 1.6).sin().round() as i32;
    let (ll, lr) = boss_step(walk);
    // Piernas de piedra.
    c.rect(17, 41, 10, (49 - ll - 41).max(2), p.stone);
    c.rect(30, 41, 10, (49 - lr - 41).max(2), p.stone);
    c.rect(15, 49 - ll, 13, 4, p.stone_light);
    c.rect(29, 49 - lr, 13, 4, p.stone_light);
    c.rect(15, 49 - ll, 13, 1, p.light);
    c.rect(29, 49 - lr, 13, 1, p.light);
    // Torso colosal.
    c.rect(12, 20 + bob, 32, 22, p.stone);
    c.rect(15, 20 + bob, 26, 3, p.stone_light);
    c.rect(12, 38 + bob, 32, 4, p.dark);
    c.rect(17, 25 + bob, 22, 8, p.stone_light);
    c.rect(17, 25 + bob, 22, 1, p.light);
    c.line(19, 26 + bob, 23, 33 + bob, p.dark);
    c.line(36, 25 + bob, 33, 34 + bob, p.dark);
    c.line(24, 20 + bob, 27, 28 + bob, p.dark);
    c.rect(28, 28 + bob, 2, 6, p.accent);
    c.rect(20, 35 + bob, 4, 1, p.accent);
    c.rect(33, 35 + bob, 4, 1, p.accent);
    // Hombreras.
    c.disc(12, 23 + bob, 7, p.stone_light);
    c.disc(44, 23 + bob, 7, p.stone_light);
    c.disc(12, 23 + bob, 5, p.stone);
    c.disc(44, 23 + bob, 5, p.stone);
    c.set(9, 20 + bob, p.light);
    c.set(41, 20 + bob, p.light);
    // Cabeza hundida.
    c.disc(28, 14 + bob, 7, p.stone_light);
    c.rect(23, 12 + bob, 10, 2, p.stone);
    c.rect(24, 14 + bob, 2, 2, p.glow);
    c.rect(30, 14 + bob, 2, 2, p.glow);
    c.rect(26, 18 + bob, 4, 1, p.dark);
    // Brazo izquierdo y puño.
    c.thick_line(12, 27 + bob, 7, 39 + bob, 7, p.stone);
    c.disc(7, 41 + bob, 4, p.stone_light);
    // Puño derecho: se alza al atacar.
    let fx = 47 + (arc * 2.0).round() as i32;
    let fy = 39 + bob - (arc * 9.0).round() as i32;
    c.thick_line(44, 27 + bob, fx, fy, 7, p.stone);
    c.disc(fx, fy, 5, p.stone_light);
    c.disc(fx, fy, 3, p.stone);
    c.set(fx - 2, fy - 2, p.light);
    if arc > 0.4 {
        c.set(fx + 7, fy - 4, p.accent);
        c.set(fx + 8, fy, p.accent);
        c.set(fx + 7, fy + 4, p.accent);
    }
    let _ = time;
}

fn draw_boss_obelisk(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let hover = ((time * 1.6).sin() * 2.0).round() as i32;
    // Monolito flotante.
    c.tri((28, 3 + hover), (18, 47 + hover), (38, 47 + hover), p.stone);
    c.tri((28, 7 + hover), (22, 45 + hover), (34, 45 + hover), p.dark);
    c.tri((28, 7 + hover), (24, 45 + hover), (34, 45 + hover), p.deep);
    c.line(22, 12 + hover, 19, 44 + hover, p.stone_light);
    c.line(19, 44 + hover, 22, 44 + hover, p.stone_light);
    c.tri((18, 47 + hover), (38, 47 + hover), (28, 55 + hover), p.dark);
    // Runas horizontales.
    for (y, w) in [(18, 5), (24, 7), (36, 7), (42, 4)] {
        c.rect(28 - w, y + hover, w * 2, 1, p.accent);
        c.set(28 - w, y + hover, p.glow);
        c.set(28 + w - 1, y + hover, p.glow);
    }
    // Ojo central.
    c.disc(28, 30 + hover, 5, p.bone);
    c.disc(28, 30 + hover, 3, p.accent);
    c.disc(28, 30 + hover, 1, OUTLINE);
    c.set(27, 29 + hover, p.glow);
    if arc > 0.25 {
        let r = 8 + (arc * 3.0).round() as i32;
        for k in 0..8 {
            let ang = k as f32 * std::f32::consts::TAU / 8.0 + time * 2.0;
            c.set(
                28 + (ang.cos() * r as f32).round() as i32,
                30 + hover + (ang.sin() * r as f32).round() as i32,
                p.glow,
            );
        }
    }
    // Esquirlas en órbita.
    let ring = 21 + (arc * 4.0).round() as i32;
    for k in 0..6 {
        let ang = k as f32 * std::f32::consts::TAU / 6.0 + time * 0.7;
        let x = 28 + (ang.cos() * ring as f32).round() as i32;
        let y = 26 + hover + (ang.sin() * (ring as f32 * 0.70)).round() as i32;
        c.rect(x - 1, y, 3, 1, p.stone_light);
        c.rect(x, y - 1, 1, 3, p.stone_light);
        c.set(x, y, p.accent);
    }
    let _ = walk;
}

fn draw_boss_spider(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let bob = (walk * 2.4).sin().round() as i32;
    // Ocho patas articuladas.
    for k in 0..4 {
        let lift = if k < 2 { (arc * 9.0).round() as i32 } else { 0 };
        let swing = ((walk * 2.0 + k as f32 * 1.4).sin() * 2.0).round() as i32;
        let hip_l = (24, 32 + bob);
        let knee_l = (12 - k * 2, 21 + k * 6 + bob + swing);
        let foot_l = (5 - k, 43 + k * 2 - lift);
        c.thick_line(hip_l.0, hip_l.1, knee_l.0, knee_l.1, 3, p.dark);
        c.thick_line(knee_l.0, knee_l.1, foot_l.0, foot_l.1, 2, p.body);
        c.set(foot_l.0, foot_l.1, p.accent);
        let hip_r = (32, 32 + bob);
        let knee_r = (44 + k * 2, 21 + k * 6 + bob - swing);
        let foot_r = (51 + k, 43 + k * 2 - lift);
        c.thick_line(hip_r.0, hip_r.1, knee_r.0, knee_r.1, 3, p.dark);
        c.thick_line(knee_r.0, knee_r.1, foot_r.0, foot_r.1, 2, p.body);
        c.set(foot_r.0, foot_r.1, p.accent);
    }
    // Abdomen con dibujo.
    c.ellipse(28, 40 + bob, 13, 10, p.body);
    c.ellipse(27, 38 + bob, 9, 7, p.light);
    c.rect(24, 36 + bob, 2, 2, p.dark);
    c.rect(30, 36 + bob, 2, 2, p.dark);
    c.rect(27, 42 + bob, 2, 2, p.dark);
    c.rect(23, 44 + bob, 2, 1, p.dark);
    c.rect(31, 44 + bob, 2, 1, p.dark);
    // Cefalotórax y cabeza.
    c.ellipse(28, 29 + bob, 9, 8, p.dark);
    c.ellipse(28, 26 + bob, 7, 6, p.body);
    c.rect(24, 23 + bob, 2, 2, RED);
    c.rect(27, 22 + bob, 2, 2, RED);
    c.rect(30, 22 + bob, 2, 2, RED);
    c.rect(33, 23 + bob, 2, 2, RED);
    // Mandíbulas.
    let open = (arc * 3.0).round() as i32;
    c.rect(25, 29 + bob, 2, 3 + open, p.bone);
    c.rect(30, 29 + bob, 2, 3 + open, p.bone);
    // Corona de púas.
    for k in 0..5 {
        let x = 22 + k * 3;
        c.tri(
            (x, 20 + bob),
            (x + 2, 20 + bob),
            (x + 1, 14 + bob),
            p.accent,
        );
    }
    let _ = time;
}

fn draw_boss_judge(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let sway = (walk * 1.5).sin().round() as i32;
    let (ll, lr) = boss_step(walk);
    // Túnica larga.
    c.tri((28, 9), (11, 52), (45, 52), p.body);
    c.tri((28, 14), (17, 50), (39, 50), p.dark);
    for x in 13..=43 {
        let wave = ((x as f32 * 0.6 + time * 2.0).sin() * 1.5).round() as i32;
        if x % 3 == 0 {
            c.rect(x, 48 + wave, 1, 5, p.deep);
        }
    }
    c.rect(18, 49 - ll, 7, 4, p.metal_dark);
    c.rect(32, 49 - lr, 7, 4, p.metal_dark);
    // Fajín y medallón.
    c.line(20, 24, 37, 33, p.accent);
    c.line(20, 25, 37, 34, p.bone_dark);
    c.disc(28, 29, 3, p.accent);
    c.set(28, 29, p.glow);
    // Capucha y rostro.
    c.tri((28, 2), (18, 27), (38, 27), p.dark);
    c.tri((28, 6), (21, 26), (35, 26), p.body);
    c.rect(23, 13, 10, 9, OUTLINE);
    c.rect(24, 14, 8, 7, Color::new(0.04, 0.035, 0.06, 1.0));
    c.rect(25, 16, 2, 2, p.glow);
    c.rect(30, 16, 2, 2, p.glow);
    c.set(25, 17, p.accent);
    c.set(30, 17, p.accent);
    // Brazos y farol.
    c.thick_line(21, 25, 15 + sway, 35, 3, p.body);
    c.rect(13 + sway, 35, 3, 3, p.bone);
    c.thick_line(35, 25, 41, 31, 3, p.body);
    let lift = (arc * 5.0).round() as i32;
    let lx = 44;
    let ly = 36 - lift;
    c.line(41, 31, lx, ly - 4, p.accent);
    c.rect(lx - 4, ly - 3, 9, 10, p.metal_dark);
    c.rect(lx - 3, ly - 2, 7, 8, p.accent);
    c.rect(lx - 2, ly - 1, 5, 6, p.glow);
    c.rect(lx - 3, ly - 4, 7, 1, p.metal);
    c.rect(lx - 1, ly + 7, 3, 1, p.metal);
    for k in 0..4 {
        let ang = time * 2.4 + k as f32 * std::f32::consts::TAU / 4.0;
        c.set(
            lx + (ang.cos() * 8.0).round() as i32,
            ly + 2 + (ang.sin() * 7.0).round() as i32,
            p.glow,
        );
    }
}

fn draw_boss_idol(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let hover = ((time * 1.3).sin() * 1.0).round() as i32;
    // Plinto escalonado.
    c.rect(13, 48, 30, 5, p.stone);
    c.rect(13, 48, 30, 1, p.stone_light);
    c.rect(16, 46, 24, 3, p.stone_light);
    c.rect(13, 52, 30, 1, p.dark);
    // Monolito hexagonal.
    let h = hover;
    c.rect(19, 11 + h, 18, 36, p.stone);
    c.tri((19, 11 + h), (37, 11 + h), (28, 4 + h), p.stone_light);
    c.tri((19, 47 + h), (37, 47 + h), (28, 52 + h), p.stone);
    c.rect(34, 13 + h, 3, 33, p.dark);
    c.line(19, 13 + h, 19, 45 + h, p.stone_light);
    c.rect(22, 10 + h, 12, 1, p.light);
    // Dos rostros tallados.
    for (i, cy) in [16, 32].iter().enumerate() {
        let cy = *cy + h;
        c.rect(21, cy, 14, 11, OUTLINE);
        c.rect(22, cy + 1, 12, 9, p.stone_light);
        c.rect(22, cy + 1, 12, 1, p.light);
        c.rect(23, cy + 3, 3, 3, OUTLINE);
        c.rect(30, cy + 3, 3, 3, OUTLINE);
        let eye = if arc > 0.25 || i == 0 {
            p.accent
        } else {
            p.glow
        };
        c.set(24, cy + 4, eye);
        c.set(31, cy + 4, eye);
        c.rect(23, cy + 2, 3, 1, p.dark);
        c.rect(30, cy + 2, 3, 1, p.dark);
        let mouth_h = if i == 1 && arc > 0.3 { 3 } else { 1 };
        c.rect(25, cy + 7, 6, mouth_h, OUTLINE);
        for t in 0..3 {
            c.set(26 + t * 2, cy + 7, p.bone);
        }
    }
    // Runas laterales.
    for y in [26, 39] {
        c.rect(20, y + h, 1, 3, p.accent);
        c.rect(35, y + h, 1, 3, p.accent);
    }
    // Esquirlas de piedra en órbita.
    let ring = 21 + (arc * 3.0).round() as i32;
    for k in 0..6 {
        let ang = k as f32 * std::f32::consts::TAU / 6.0 + time * 0.55;
        let x = 28 + (ang.cos() * ring as f32).round() as i32;
        let y = 26 + h + (ang.sin() * (ring as f32 * 0.68)).round() as i32;
        c.rect(x - 1, y, 3, 1, p.stone_light);
        c.rect(x, y - 1, 1, 3, p.stone_light);
        c.set(x, y, p.accent);
    }
    let _ = walk;
}

fn draw_boss_maw(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let hover = ((time * 1.9).sin() * 2.0).round() as i32;
    let cx = 28;
    let cy = 27 + hover;
    let void = mix(p.body, Color::new(0.018, 0.012, 0.032, 1.0), 0.80);
    // Esfera del vacío con borde de energía.
    c.disc(cx, cy, 18, p.dark);
    c.disc(cx, cy, 17, void);
    for k in 0..12 {
        let ang = k as f32 * std::f32::consts::TAU / 12.0 - time * 0.25;
        let x = cx + (ang.cos() * 16.5).round() as i32;
        let y = cy + (ang.sin() * 16.5).round() as i32;
        c.set(x, y, p.accent);
    }
    // Anillo de dientes que se abre al atacar.
    let inner = if arc > 0.3 { 7 } else { 9 };
    let outer = 16 + (arc * 2.0).round() as i32;
    for k in 0..14 {
        let ang = k as f32 * std::f32::consts::TAU / 14.0 + time * 0.2;
        let c1 = ang.cos();
        let s1 = ang.sin();
        let px = cx + (c1 * inner as f32).round() as i32;
        let py = cy + (s1 * inner as f32).round() as i32;
        let ox = cx + (c1 * outer as f32).round() as i32;
        let oy = cy + (s1 * outer as f32).round() as i32;
        let tx = (-s1 * 2.6).round() as i32;
        let ty = (c1 * 2.6).round() as i32;
        c.tri((px, py), (ox + tx, oy + ty), (ox - tx, oy - ty), p.bone);
        c.set(ox, oy, p.bone_dark);
    }
    // Fauces y núcleo.
    c.disc(cx, cy, inner - 1, Color::new(0.008, 0.004, 0.016, 1.0));
    let core = 2 + (arc * 2.0).round() as i32;
    c.disc(cx, cy, core, p.accent);
    c.disc(cx, cy, core - 1, p.glow);
    // Motas de energía.
    for k in 0..3 {
        let ang = time * 1.5 + k as f32 * 2.1;
        c.set(
            cx + (ang.cos() * 12.0).round() as i32,
            cy + (ang.sin() * 12.0).round() as i32,
            p.glow,
        );
    }
    let _ = walk;
}

fn draw_boss_hand(c: &mut Canvas, p: &BossPalette, walk: f32, arc: f32, time: f32) {
    let hover = ((time * 1.7).sin() * 2.0).round() as i32;
    let cx = 28;
    let cy = 33 + hover;
    // Muñeca y puño.
    c.thick_line(28, cy + 8, 28, 52, 8, p.body);
    c.rect(22, 45, 12, 4, p.dark);
    c.rect(22, 45, 12, 1, p.light);
    c.rect(20, 49, 16, 4, p.deep);
    // Dedos que se cierran al atacar.
    let curl = (arc * 10.0).round() as i32;
    for k in 0..5 {
        let bx = cx + (k - 2) * 5;
        let spread = (k - 2) * 2;
        let base_y = cy - 4;
        let tip_x = bx + spread;
        let tip_y = 10 + (k % 2) * 3 + curl;
        c.thick_line(bx, base_y, tip_x, tip_y, 3, p.body);
        c.disc(tip_x, tip_y, 2, p.light);
        c.set(tip_x, tip_y - 1, p.accent);
    }
    // Pulgar.
    let thumb_y = 30 - (arc * 3.0).round() as i32;
    c.thick_line(cx + 8, cy - 1, 47, thumb_y, 4, p.body);
    c.disc(47, thumb_y, 2, p.light);
    c.set(47, thumb_y - 1, p.accent);
    // Palma y ojo astral.
    c.ellipse(cx, cy, 12, 11, p.body);
    c.ellipse(cx, cy + 1, 8, 8, p.dark);
    c.disc(cx, cy, 5, p.bone);
    c.disc(cx, cy, 3, p.accent);
    c.disc(cx, cy, 1, OUTLINE);
    c.set(cx - 1, cy - 2, p.glow);
    // Runas en órbita.
    for k in 0..8 {
        let ang = k as f32 * std::f32::consts::TAU / 8.0 + time * 0.8;
        c.set(
            cx + (ang.cos() * 21.0).round() as i32,
            cy - 2 + (ang.sin() * 17.0).round() as i32,
            p.accent,
        );
    }
    let _ = walk;
}

/// Guardianes en pixel art. Cada piel tiene una silueta propia y anima
/// caminata, ataque y flotación sin salirse del lienzo.
pub fn draw_boss(
    pos: Vec2,
    skin: usize,
    scale: f32,
    tint: Color,
    elite: bool,
    shiny: bool,
    walk_t: f32,
    swing_t: f32,
    hit_flash: f32,
    death_alpha: f32,
    time: f32,
) {
    let p = boss_palette(tint, elite);
    let mut c = Canvas::new(BOSS_W, BOSS_H);
    // El lienzo es más alto que el de un enemigo normal; se compensa la
    // escala para que la cabeza quede por debajo de la barra del guardián.
    let scale = scale * 0.68;
    let swing = (swing_t / 0.30).clamp(0.0, 1.0);
    let arc = (swing * std::f32::consts::PI).sin();
    match skin % 12 {
        0 => draw_boss_king(&mut c, &p, walk_t, arc, time),
        1 => draw_boss_brute(&mut c, &p, walk_t, arc, time),
        2 => draw_boss_wraith(&mut c, &p, walk_t, arc, time),
        3 => draw_boss_orb(&mut c, &p, walk_t, arc, time),
        4 => draw_boss_serpent(&mut c, &p, walk_t, arc, time),
        5 => draw_boss_colossus(&mut c, &p, walk_t, arc, time),
        6 => draw_boss_obelisk(&mut c, &p, walk_t, arc, time),
        7 => draw_boss_spider(&mut c, &p, walk_t, arc, time),
        8 => draw_boss_judge(&mut c, &p, walk_t, arc, time),
        9 => draw_boss_idol(&mut c, &p, walk_t, arc, time),
        10 => draw_boss_maw(&mut c, &p, walk_t, arc, time),
        _ => draw_boss_hand(&mut c, &p, walk_t, arc, time),
    }
    let outline = if shiny {
        Color::new(0.42, 0.94, 0.94, 1.0)
    } else if elite {
        Color::new(0.96, 0.78, 0.34, 1.0)
    } else {
        OUTLINE
    };
    c.outline(outline);
    let flash = (hit_flash / 0.18).clamp(0.0, 1.0);
    c.draw_scaled_at(pos, false, flash, scale, death_alpha, BOSS_FEET);
}

/// Iconos pixel art de armas para la forja. 13x14 píxeles, anclados al centro.
pub fn draw_weapon_icon(
    class: Class,
    index: usize,
    x: f32,
    y: f32,
    color: Color,
    alpha: f32,
    s: f32,
) {
    let mut c = Canvas::new(13, 14);
    let i = index.clamp(1, 3) - 1;
    let w = color;
    let w_dk = darken(color, 0.55);
    let glow = lighten(color, 0.62);
    let steel = Color::new(0.74, 0.78, 0.84, 1.0);
    let steel_dk = Color::new(0.38, 0.42, 0.50, 1.0);
    let wood = Color::new(0.46, 0.30, 0.16, 1.0);
    let wood_dk = Color::new(0.26, 0.16, 0.09, 1.0);
    let gold = Color::new(0.96, 0.78, 0.34, 1.0);
    let bone = Color::new(0.85, 0.82, 0.72, 1.0);
    match (class, i) {
        (Class::Knight, 0) => {
            // Espada: hoja vertical, guarda dorada, puño de madera.
            c.rect(5, 0, 3, 8, steel);
            c.rect(7, 0, 1, 8, steel_dk);
            c.set(6, 0, steel);
            c.rect(4, 8, 5, 2, gold);
            c.rect(5, 10, 2, 2, wood);
            c.rect(5, 12, 2, 1, gold);
        }
        (Class::Knight, 1) => {
            // Espada y escudo redondo con umbo dorado.
            c.rect(2, 1, 2, 7, steel);
            c.rect(1, 8, 4, 1, gold);
            c.rect(2, 9, 2, 3, wood);
            c.disc(9, 7, 4, steel_dk);
            c.disc(9, 7, 3, w);
            c.set(9, 7, gold);
        }
        (Class::Knight, 2) => {
            // Mandoble: hoja ancha con canal central y guarda larga.
            c.rect(4, 0, 5, 9, steel);
            c.rect(7, 0, 2, 9, steel_dk);
            c.rect(6, 1, 1, 8, w);
            c.rect(2, 9, 9, 1, gold);
            c.rect(5, 10, 3, 3, wood_dk);
            c.rect(5, 13, 3, 1, gold);
        }
        (Class::Archer, 0) => {
            // Arco: palas curvas de madera y cuerda tensa.
            c.line(5, 0, 8, 2, wood);
            c.line(8, 2, 9, 5, wood);
            c.line(9, 5, 8, 9, wood);
            c.line(8, 9, 5, 11, wood);
            c.line(5, 0, 5, 11, steel);
            c.rect(4, 5, 2, 2, wood_dk);
        }
        (Class::Archer, 1) => {
            // Arco con flecha montada y plumas doradas.
            c.line(5, 0, 8, 2, wood);
            c.line(8, 2, 9, 5, wood);
            c.line(9, 5, 8, 9, wood);
            c.line(8, 9, 5, 11, wood);
            c.line(5, 0, 5, 11, steel);
            c.rect(5, 5, 6, 1, wood_dk);
            c.set(11, 5, steel);
            c.set(5, 4, gold);
            c.set(5, 6, gold);
        }
        (Class::Archer, 2) => {
            // Ballesta: cuerpo de madera, proyectiles de acero y virote.
            c.rect(2, 6, 8, 2, wood);
            c.rect(10, 1, 1, 10, steel);
            c.set(10, 0, gold);
            c.set(10, 11, gold);
            c.rect(4, 5, 6, 1, gold);
            c.set(9, 5, steel);
            c.set(3, 8, steel_dk);
        }
        (Class::Mage, 0) => {
            // Báculo: asta curvada con orbe brillante y chispas.
            c.line(4, 13, 8, 4, wood);
            c.line(5, 13, 9, 4, wood_dk);
            c.disc(9, 3, 2, w);
            c.set(8, 2, glow);
            c.set(11, 1, glow);
            c.set(7, 0, glow);
        }
        (Class::Mage, 1) => {
            // Orbe arcano sobre peana de madera.
            c.rect(4, 11, 5, 2, wood_dk);
            c.disc(6, 6, 3, steel_dk);
            c.disc(6, 5, 4, w);
            c.disc(5, 4, 2, glow);
            c.set(9, 2, glow);
            c.set(2, 8, glow);
        }
        (Class::Mage, 2) => {
            // Grimorio: tapa del arma, páginas de acero y broche dorado.
            c.rect(2, 4, 9, 7, w_dk);
            c.rect(3, 5, 7, 5, steel);
            c.rect(6, 5, 1, 5, w_dk);
            c.set(4, 7, glow);
            c.set(8, 7, glow);
            c.rect(10, 6, 1, 3, gold);
        }
        (Class::Assassin, 0) => {
            // Daga corta con guarda fina y pomo dorado.
            c.rect(6, 3, 2, 6, steel);
            c.rect(7, 3, 1, 6, steel_dk);
            c.rect(5, 9, 3, 1, steel_dk);
            c.rect(6, 10, 1, 3, wood_dk);
            c.set(6, 13, gold);
        }
        (Class::Assassin, 1) => {
            // Dagas gemelas cruzadas en X.
            c.line(2, 10, 8, 2, steel);
            c.line(10, 10, 4, 2, steel);
            c.set(3, 9, gold);
            c.set(9, 9, gold);
            c.line(2, 10, 3, 11, wood_dk);
            c.line(10, 10, 9, 11, wood_dk);
        }
        (Class::Assassin, 2) => {
            // Estrella de lanzamiento: cuatro puntas y núcleo dorado.
            c.rect(5, 6, 3, 3, w);
            c.set(6, 2, steel);
            c.set(6, 12, steel);
            c.set(1, 7, steel);
            c.set(11, 7, steel);
            c.set(2, 3, w);
            c.set(10, 3, w);
            c.set(2, 11, w);
            c.set(10, 11, w);
            c.set(6, 7, gold);
        }
        (Class::Necro, 0) => {
            // Guadaña: mango largo y hoja curva de acero frío.
            c.line(4, 13, 9, 1, wood);
            c.line(9, 1, 11, 2, steel);
            c.line(11, 2, 12, 5, steel);
            c.line(11, 3, 12, 6, steel_dk);
            c.set(9, 1, glow);
        }
        (Class::Necro, 1) => {
            // Cráneo con cuencas encendidas en el color del arma.
            c.disc(6, 6, 4, bone);
            c.rect(4, 9, 5, 3, bone);
            c.rect(4, 5, 2, 2, OUTLINE);
            c.rect(7, 5, 2, 2, OUTLINE);
            c.set(4, 6, glow);
            c.set(8, 6, glow);
            c.set(6, 8, OUTLINE);
            c.rect(4, 11, 5, 1, steel_dk);
        }
        (Class::Necro, 2) => {
            // Frasco de esencias: tapón, cuello y licor brillante.
            c.rect(5, 3, 3, 1, wood_dk);
            c.rect(5, 4, 3, 2, steel);
            c.disc(6, 9, 3, w);
            c.rect(5, 9, 1, 2, glow);
            c.set(4, 11, w_dk);
        }
        (Class::Cleric, 0) => {
            // Báculo del alba: sol dorado con rayos sobre asta.
            c.rect(6, 5, 1, 8, wood);
            c.disc(6, 3, 2, gold);
            c.set(3, 1, gold);
            c.set(9, 1, gold);
            c.set(6, 0, gold);
            c.set(2, 4, gold);
            c.set(10, 4, gold);
        }
        (Class::Cleric, 1) => {
            // Cáliz: copa del arma, tallo y peana dorados.
            c.rect(3, 2, 7, 3, w);
            c.rect(4, 5, 5, 1, w_dk);
            c.rect(4, 3, 1, 1, glow);
            c.rect(6, 6, 1, 3, gold);
            c.rect(4, 9, 5, 1, gold);
            c.rect(5, 10, 3, 1, w_dk);
        }
        (Class::Cleric, 2) => {
            // Campana del santuario: cúpula, ala dorada y badajo.
            c.set(6, 1, steel);
            c.disc(6, 5, 3, w);
            c.rect(3, 7, 7, 2, w_dk);
            c.rect(3, 9, 7, 1, gold);
            c.set(6, 10, gold);
            c.set(4, 4, glow);
        }
        (Class::Merchant, 0) => {
            // Moneda del mercado: disco del arma con brillo y canto.
            c.disc(6, 7, 4, w);
            c.disc(6, 7, 3, glow);
            c.rect(5, 6, 3, 1, w_dk);
            c.rect(5, 8, 3, 1, w_dk);
            c.set(4, 5, Color::new(1.0, 1.0, 1.0, 0.85));
        }
        (Class::Merchant, 1) => {
            // Bolsa de oro atada con monedas asomando.
            c.disc(6, 9, 4, wood);
            c.disc(4, 10, 2, wood);
            c.rect(4, 5, 5, 2, wood_dk);
            c.set(6, 4, gold);
            c.set(8, 4, gold);
            c.set(5, 8, wood_dk);
        }
        _ => {
            // Balanza: fiel al centavo, platos dorados.
            c.rect(2, 5, 9, 1, steel);
            c.rect(6, 6, 1, 5, wood);
            c.rect(4, 11, 5, 1, gold);
            c.line(2, 6, 1, 8, steel);
            c.line(2, 6, 4, 8, steel);
            c.line(10, 6, 9, 8, steel);
            c.line(10, 6, 11, 8, steel);
            c.rect(1, 8, 4, 1, gold);
            c.rect(8, 8, 4, 1, gold);
        }
    }
    c.outline(OUTLINE);
    c.draw_scaled_at(vec2(x, y), false, 0.0, s.max(0.5), alpha, 7);
}

/// Enemigos normales en pixel art. Los jefes usan `draw_boss`.
pub fn draw_enemy(
    pos: Vec2,
    kind: EnemyKind,
    _radius: f32,
    scale: f32,
    tint: Color,
    elite: bool,
    shiny: bool,
    walk_t: f32,
    swing_t: f32,
    hit_flash: f32,
    death_alpha: f32,
    time: f32,
) {
    let p = enemy_palette(tint, elite, shiny);
    let mut c = Canvas::new(CANVAS_W, CANVAS_H);
    let swing = (swing_t / 0.22).clamp(0.0, 1.0);
    let arc = (swing * std::f32::consts::PI).sin();
    match kind {
        EnemyKind::Slime => draw_slime(&mut c, &p, walk_t, arc, time),
        EnemyKind::Bat => draw_bat(&mut c, &p, walk_t, arc, time),
        EnemyKind::Brute => draw_brute(&mut c, &p, walk_t, arc, time),
        EnemyKind::Cultist => draw_cultist(&mut c, &p, walk_t, arc, time),
        EnemyKind::Wraith => draw_wraith(&mut c, &p, walk_t, arc, time),
        EnemyKind::Bomber => draw_bomber(&mut c, &p, walk_t, arc, time),
        EnemyKind::Sentinel => draw_sentinel(&mut c, &p, walk_t, arc, time),
        EnemyKind::Eye => draw_eye(&mut c, &p, walk_t, arc, time),
        EnemyKind::Boss => draw_brute(&mut c, &p, walk_t, arc, time),
    }
    let outline = if shiny {
        Color::new(0.42, 0.94, 0.94, 1.0)
    } else if elite {
        Color::new(0.96, 0.78, 0.34, 1.0)
    } else {
        OUTLINE
    };
    c.outline(outline);
    let flash = (hit_flash / 0.18).clamp(0.0, 1.0);
    c.draw_scaled(pos, false, flash, scale, death_alpha);
}
