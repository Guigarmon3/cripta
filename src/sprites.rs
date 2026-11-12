use crate::types::Rng;
use macroquad::prelude::*;

pub const VW: f32 = 480.0;
pub const VH: f32 = 270.0;

pub struct Sprites {
    pub knight: Texture2D,
    pub archer: Texture2D,
    pub mage: Texture2D,
    pub assassin: Texture2D,
    pub necro: Texture2D,
    pub cleric: Texture2D,
    pub merchant_hero: Texture2D,
    pub skeleton: Texture2D,
    pub slime: Texture2D,
    pub bat: Texture2D,
    pub brute: Texture2D,
    pub cultist: Texture2D,
    pub wraith: Texture2D,
    pub bomber: Texture2D,
    pub sentinel: Texture2D,
    pub eye: Texture2D,
    pub merchant: Texture2D,
    pub bosses: [Texture2D; 12],
    pub barrel: Texture2D,
    pub crate_: Texture2D,
    pub chest: Texture2D,
    pub key: Texture2D,
    pub bed: Texture2D,
    pub urn: Texture2D,
    pub statue: Texture2D,
    pub portal: Texture2D,
    pub vial: Texture2D,
    pub coin: Texture2D,
    pub gem: Texture2D,
    pub bg: Texture2D,
    pub vignette: Texture2D,
    pub glow: Texture2D,
    pub glow_white: Texture2D,
}

fn palette(c: char) -> Color {
    match c {
        'k' => Color::from_rgba(13, 11, 18, 255),
        'w' => Color::from_rgba(238, 238, 244, 255),
        's' => Color::from_rgba(226, 181, 140, 255),
        'S' => Color::from_rgba(190, 140, 104, 255),
        'r' => Color::from_rgba(212, 72, 82, 255),
        'R' => Color::from_rgba(140, 40, 55, 255),
        'g' => Color::from_rgba(92, 182, 102, 255),
        'G' => Color::from_rgba(50, 112, 70, 255),
        'b' => Color::from_rgba(92, 132, 222, 255),
        'B' => Color::from_rgba(45, 70, 140, 255),
        'p' => Color::from_rgba(152, 92, 202, 255),
        'P' => Color::from_rgba(84, 50, 120, 255),
        'y' => Color::from_rgba(242, 200, 92, 255),
        'o' => Color::from_rgba(240, 140, 60, 255),
        'n' => Color::from_rgba(140, 96, 60, 255),
        'N' => Color::from_rgba(90, 60, 40, 255),
        'm' => Color::from_rgba(172, 182, 194, 255),
        'M' => Color::from_rgba(96, 106, 122, 255),
        'l' => Color::from_rgba(210, 216, 226, 255),
        'c' => Color::from_rgba(102, 232, 232, 255),
        'e' => Color::from_rgba(255, 122, 50, 255),
        _ => Color::new(0.0, 0.0, 0.0, 0.0),
    }
}

fn make_sprite(art: &[&str]) -> Texture2D {
    let w = art
        .iter()
        .map(|r| r.chars().count())
        .max()
        .unwrap_or(1)
        .max(1) as u16;
    let h = art.len().max(1) as u16;
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    for (y, row) in art.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let c = palette(ch);
            if c.a > 0.0 {
                img.set_pixel(x as u32, y as u32, c);
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

// --- Guardianes únicos -------------------------------------------------
// Doce siluetas dibujadas píxel a píxel sobre una lienzo de 32x32. No
// comparten forma: corona, cuernos, alas, orbe, serpiente, coloso, obelisco,
// araña, farol, ídolo doble, boca del vacío y mano astral.

fn px(img: &mut Image, x: i32, y: i32, c: Color) {
    if x >= 0 && y >= 0 && (x as u32) < img.width as u32 && (y as u32) < img.height as u32 {
        img.set_pixel(x as u32, y as u32, c);
    }
}

fn rect(img: &mut Image, x: i32, y: i32, w: i32, h: i32, c: Color) {
    for yy in y..y + h {
        for xx in x..x + w {
            px(img, xx, yy, c);
        }
    }
}

fn circle(img: &mut Image, cx: i32, cy: i32, r: i32, c: Color) {
    for yy in -r..=r {
        for xx in -r..=r {
            if xx * xx + yy * yy <= r * r {
                px(img, cx + xx, cy + yy, c);
            }
        }
    }
}

fn line(img: &mut Image, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        px(img, x, y, c);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = err * 2;
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

fn tri(img: &mut Image, a: (i32, i32), b: (i32, i32), c: (i32, i32), col: Color) {
    let minx = a.0.min(b.0).min(c.0);
    let maxx = a.0.max(b.0).max(c.0);
    let miny = a.1.min(b.1).min(c.1);
    let maxy = a.1.max(b.1).max(c.1);
    let sign = |p: (i32, i32), q: (i32, i32), r: (i32, i32)| {
        (p.0 - r.0) * (q.1 - r.1) - (q.0 - r.0) * (p.1 - r.1)
    };
    for y in miny..=maxy {
        for x in minx..=maxx {
            let p = (x, y);
            let d1 = sign(p, a, b);
            let d2 = sign(p, b, c);
            let d3 = sign(p, c, a);
            let has_neg = d1 < 0 || d2 < 0 || d3 < 0;
            let has_pos = d1 > 0 || d2 > 0 || d3 > 0;
            if !(has_neg && has_pos) {
                px(img, x, y, col);
            }
        }
    }
}

fn boss_skin(idx: usize) -> Texture2D {
    let dark = Color::from_rgba(12, 10, 18, 255);
    let bone = Color::from_rgba(214, 204, 196, 255);
    let (main, accent) = match idx % 12 {
        0 => (
            Color::from_rgba(190, 62, 74, 255),
            Color::from_rgba(242, 196, 92, 255),
        ),
        1 => (
            Color::from_rgba(132, 48, 54, 255),
            Color::from_rgba(228, 140, 70, 255),
        ),
        2 => (
            Color::from_rgba(96, 76, 160, 255),
            Color::from_rgba(140, 220, 226, 255),
        ),
        3 => (
            Color::from_rgba(74, 130, 170, 255),
            Color::from_rgba(250, 240, 170, 255),
        ),
        4 => (
            Color::from_rgba(58, 128, 84, 255),
            Color::from_rgba(198, 230, 110, 255),
        ),
        5 => (
            Color::from_rgba(116, 102, 92, 255),
            Color::from_rgba(230, 180, 110, 255),
        ),
        6 => (
            Color::from_rgba(70, 84, 142, 255),
            Color::from_rgba(150, 200, 255, 255),
        ),
        7 => (
            Color::from_rgba(122, 62, 140, 255),
            Color::from_rgba(220, 160, 240, 255),
        ),
        8 => (
            Color::from_rgba(146, 122, 62, 255),
            Color::from_rgba(255, 230, 130, 255),
        ),
        9 => (
            Color::from_rgba(148, 76, 92, 255),
            Color::from_rgba(236, 196, 150, 255),
        ),
        10 => (
            Color::from_rgba(40, 36, 62, 255),
            Color::from_rgba(180, 110, 240, 255),
        ),
        _ => (
            Color::from_rgba(88, 120, 154, 255),
            Color::from_rgba(196, 232, 255, 255),
        ),
    };
    let mut img = Image::gen_image_color(32, 32, Color::new(0.0, 0.0, 0.0, 0.0));

    match idx % 12 {
        0 => {
            // Rey calavera: corona de tres púas y mandíbula.
            rect(&mut img, 10, 9, 12, 13, dark);
            circle(&mut img, 16, 15, 8, dark);
            circle(&mut img, 16, 15, 6, bone);
            rect(&mut img, 11, 5, 3, 5, accent);
            rect(&mut img, 15, 3, 3, 7, accent);
            rect(&mut img, 19, 5, 3, 5, accent);
            rect(&mut img, 12, 12, 3, 3, dark);
            rect(&mut img, 18, 12, 3, 3, dark);
            rect(&mut img, 14, 18, 5, 2, dark);
            rect(&mut img, 10, 21, 13, 3, main);
        }
        1 => {
            // Bruto cornudo: torso ancho y cuernos curvos.
            rect(&mut img, 8, 12, 16, 14, dark);
            rect(&mut img, 10, 13, 12, 12, main);
            circle(&mut img, 16, 11, 6, dark);
            circle(&mut img, 16, 11, 4, bone);
            line(&mut img, 10, 8, 4, 3, dark);
            line(&mut img, 10, 8, 4, 3, accent);
            line(&mut img, 22, 8, 28, 3, dark);
            line(&mut img, 22, 8, 28, 3, accent);
            rect(&mut img, 12, 10, 2, 2, accent);
            rect(&mut img, 18, 10, 2, 2, accent);
            rect(&mut img, 7, 16, 3, 8, dark);
            rect(&mut img, 22, 16, 3, 8, dark);
        }
        2 => {
            // Espectro alado: alas desplegadas y cuerpo hueco.
            tri(&mut img, (16, 10), (2, 4), (5, 20), dark);
            tri(&mut img, (16, 10), (30, 4), (27, 20), dark);
            tri(&mut img, (16, 10), (4, 6), (7, 18), main);
            tri(&mut img, (16, 10), (28, 6), (25, 18), main);
            circle(&mut img, 16, 14, 6, dark);
            circle(&mut img, 16, 14, 4, accent);
            rect(&mut img, 14, 13, 2, 2, dark);
            rect(&mut img, 18, 13, 2, 2, dark);
            for k in 0..4 {
                line(&mut img, 13 + k * 2, 20, 12 + k * 2, 28, main);
            }
        }
        3 => {
            // Orbe vidente: ojo central y tentáculos.
            circle(&mut img, 16, 14, 10, dark);
            circle(&mut img, 16, 14, 8, main);
            circle(&mut img, 16, 14, 4, accent);
            circle(&mut img, 16, 14, 2, dark);
            for k in 0..6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                line(
                    &mut img,
                    16 + (a.cos() * 8.0) as i32,
                    14 + (a.sin() * 8.0) as i32,
                    16 + (a.cos() * 13.0) as i32,
                    14 + (a.sin() * 13.0) as i32,
                    main,
                );
            }
        }
        4 => {
            // Serpiente enrollada: espiral de escamas.
            for k in 0..3 {
                circle(&mut img, 16, 17 - k * 3, 9 - k * 2, dark);
                circle(
                    &mut img,
                    16,
                    17 - k * 3,
                    7 - k * 2,
                    if k == 1 { accent } else { main },
                );
            }
            circle(&mut img, 16, 7, 4, dark);
            circle(&mut img, 16, 7, 3, main);
            rect(&mut img, 15, 6, 2, 2, accent);
            line(&mut img, 16, 10, 16, 12, accent);
        }
        5 => {
            // Coloso de piedra: puños y hombros.
            rect(&mut img, 9, 9, 14, 16, dark);
            rect(&mut img, 11, 11, 10, 13, main);
            circle(&mut img, 16, 8, 5, dark);
            circle(&mut img, 16, 8, 3, bone);
            rect(&mut img, 5, 13, 5, 10, dark);
            rect(&mut img, 22, 13, 5, 10, dark);
            rect(&mut img, 11, 24, 4, 5, dark);
            rect(&mut img, 17, 24, 4, 5, dark);
            rect(&mut img, 13, 18, 6, 2, accent);
        }
        6 => {
            // Obelisco flotante: rombo alargado con runa.
            tri(&mut img, (16, 2), (8, 12), (24, 12), dark);
            tri(&mut img, (16, 3), (10, 12), (22, 12), main);
            rect(&mut img, 10, 12, 12, 15, dark);
            rect(&mut img, 12, 12, 8, 13, main);
            tri(&mut img, (16, 27), (10, 22), (22, 22), dark);
            rect(&mut img, 15, 15, 2, 7, accent);
            rect(&mut img, 13, 17, 6, 2, accent);
        }
        7 => {
            // Reina araña: ocho patas y saco de huevos.
            circle(&mut img, 16, 18, 7, dark);
            circle(&mut img, 16, 18, 5, main);
            circle(&mut img, 16, 10, 4, dark);
            circle(&mut img, 16, 10, 2, accent);
            for k in 0..4 {
                let y = 12 + k * 4;
                line(&mut img, 11, y, 3, y - 4 + k as i32 * 2, dark);
                line(&mut img, 21, y, 29, y - 4 + k as i32 * 2, dark);
            }
            circle(&mut img, 14, 20, 1, accent);
            circle(&mut img, 18, 20, 1, accent);
        }
        8 => {
            // Juez del farol: capucha y luz colgante.
            tri(&mut img, (16, 3), (8, 26), (24, 26), dark);
            tri(&mut img, (16, 5), (10, 25), (22, 25), main);
            circle(&mut img, 16, 12, 4, dark);
            rect(&mut img, 14, 11, 2, 2, accent);
            rect(&mut img, 18, 11, 2, 2, accent);
            rect(&mut img, 22, 18, 5, 7, dark);
            rect(&mut img, 23, 19, 3, 4, accent);
            line(&mut img, 24, 15, 24, 18, dark);
        }
        9 => {
            // Ídolo de dos caras: dos cabezas unidas.
            rect(&mut img, 6, 10, 9, 9, dark);
            rect(&mut img, 17, 10, 9, 9, dark);
            circle(&mut img, 10, 13, 4, bone);
            circle(&mut img, 22, 13, 4, bone);
            rect(&mut img, 9, 12, 2, 2, dark);
            rect(&mut img, 21, 12, 2, 2, dark);
            rect(&mut img, 9, 22, 14, 5, main);
            rect(&mut img, 12, 23, 8, 2, accent);
            line(&mut img, 10, 10, 8, 5, accent);
            line(&mut img, 22, 10, 24, 5, accent);
        }
        10 => {
            // Boca del vacío: anillo de dientes y núcleo oscuro.
            circle(&mut img, 16, 16, 11, dark);
            circle(&mut img, 16, 16, 9, main);
            circle(&mut img, 16, 16, 6, dark);
            for k in 0..10 {
                let a = k as f32 * std::f32::consts::TAU / 10.0;
                let x = 16 + (a.cos() * 7.0) as i32;
                let y = 16 + (a.sin() * 7.0) as i32;
                let x2 = 16 + (a.cos() * 10.0) as i32;
                let y2 = 16 + (a.sin() * 10.0) as i32;
                line(&mut img, x, y, x2, y2, accent);
            }
            circle(&mut img, 16, 16, 3, accent);
            circle(&mut img, 16, 16, 1, dark);
        }
        _ => {
            // Mano astral: palma y dedos largos.
            rect(&mut img, 10, 14, 12, 10, dark);
            rect(&mut img, 12, 15, 8, 8, main);
            for k in 0..4 {
                let x = 11 + k * 3;
                rect(&mut img, x, 6 + (k % 2) as i32 * 2, 2, 9, dark);
                rect(&mut img, x, 7 + (k % 2) as i32 * 2, 1, 7, accent);
            }
            rect(&mut img, 6, 15, 4, 3, dark);
            rect(&mut img, 6, 15, 3, 2, main);
            circle(&mut img, 16, 18, 3, accent);
        }
    }

    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

const KNIGHT: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kmmmmmmk....",
    "....kmssssmk....",
    "....kmmmmmmk....",
    "...kkmmmmmmkk...",
    "..kmmkmmmmkmmk..",
    "..km.kmmmmk.mk..",
    "..km.kmmmmk.mk..",
    "..kk.kmmmmk.kk..",
    ".....kmmmmk.....",
    ".....kmmmmk.....",
    "....kmmkkmmk....",
    "....kmk..kmk....",
    "....kk....kk....",
    "................",
];

const ARCHER: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....knnnnnnk....",
    "....knssssnk....",
    "....knnnnnnk....",
    "...kkggggggkk...",
    "..kggkggggkggk..",
    "..kg.kggggk.gk..",
    "..kg.kggggk.gk..",
    "..kk.kggggk.kk..",
    ".....kggggk.....",
    ".....kggggk.....",
    "....kggkkggk....",
    "....kgk..kgk....",
    "....kk....kk....",
    "................",
];

const MAGE: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kbbbbbbk....",
    "....kbssssbk....",
    "....kbbbbbbk....",
    "...kkbbbbbbkk...",
    "..kbbkbbbbkbbk..",
    "..kb.kbbbbk.bk..",
    "..kb.kbbbbk.bk..",
    "..kk.kbbbbk.kk..",
    ".....kbbbbk.....",
    ".....kbbbbk.....",
    "....kbbkkbbk....",
    "....kbk..kbk....",
    "....kk....kk....",
    "................",
];

const ASSASSIN: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kppppppk....",
    "....kpsssspk....",
    "....kppppppk....",
    "...kkppppppkk...",
    "..kppkppppkppk..",
    "..kp.kppppk.pk..",
    "..kp.kppppk.pk..",
    "..kk.kppppk.kk..",
    ".....kppppk.....",
    ".....kppppk.....",
    "....kppkkppk....",
    "....kpk..kpk....",
    "....kk....kk....",
    "................",
];

const NECRO: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kPPPPPPk....",
    "....kPwwwwPk....",
    "....kPwwwwPk....",
    "...kkPPPPPPkk...",
    "..kPPkPPPPkPPk..",
    "..kP.kPPPPk.Pk..",
    "..kP.kPPPPk.Pk..",
    "..kk.kPPPPk.kk..",
    ".....kPPPPk.....",
    ".....kPPPPk.....",
    "....kPPkkPPk....",
    "....kPk..kPk....",
    "....kk....kk....",
    "................",
];

/// Clérigo de túnica clara, halo dorado y báculo.
const CLERIC: &[&str] = &[
    "......yyyy......",
    ".....kkkkkk.....",
    "....kwwwwwwk....",
    "....kwsssswk....",
    "....kwwwwwwk....",
    "...kkwllllwkk...",
    "..kwwkwwwwkwwk..",
    "..kw.kwwwwk.wk..",
    "..kw.kwwwwk.wk..",
    "..kk.kwwwwk.kk..",
    ".....kwwwwk..y..",
    ".....kwwwwk..y..",
    "....kwwkkwwk.y..",
    "....kwk..kwk.y..",
    "....kk....kk.y..",
    "................",
];

/// Mercader con sombrero ancho y monedero dorado.
const MERCHANT_HERO: &[&str] = &[
    "................",
    "....kkkkkkkk....",
    "...knnnnnnnnk...",
    "..knnnnnnnnnnk..",
    "....knssssnk....",
    "....knnnnnnk....",
    "...kknnnnnnkk...",
    "..knnknnnnknnk..",
    "..kn.knnnnk.nk..",
    "..kn.kyyyyk.nk..",
    "..kk.knnnnk.kk..",
    ".....knnnnk.....",
    "....knnkknnk....",
    "....knk..knk....",
    "....kk....kk....",
    "................",
];

const SKELETON: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kwwwwwwk....",
    "....kwkwwkwk....",
    "....kwwwwwwk....",
    ".....kkkkkk.....",
    "....kwwwwwwk....",
    "...kwkwwwwkwk...",
    "...kk.kwwk.kk...",
    "......kwwk......",
    ".....kwwwwk.....",
    ".....kwkkwk.....",
    "....kwk..kwk....",
    "....kk....kk....",
    "................",
    "................",
];

const SLIME: &[&str] = &[
    "................",
    "................",
    "................",
    "......kkkk......",
    "....kkggggkk....",
    "...kggggggggk...",
    "..kggwggggwggk..",
    "..kggwggggwggk..",
    "..kggggggggggk..",
    "..kggggggggggk..",
    "..kggggggggggk..",
    "...kggggggggk...",
    "....kkggggkk....",
    "......kkkk......",
    "................",
    "................",
];

const BAT: &[&str] = &[
    "................",
    "................",
    "..kk........kk..",
    ".kppk......kppk.",
    ".kpppk....kpppk.",
    ".kppppk..kppppk.",
    "..kppppkkppppk..",
    "...kppppppppk...",
    "....kppppppk....",
    ".....kppppk.....",
    "....kpk..kpk....",
    "...kk......kk...",
    "................",
    "................",
    "................",
    "................",
];

const BRUTE: &[&str] = &[
    "................",
    "....kkkkkkkk....",
    "...kmmmmmmmmk...",
    "..kmmrrrrrrmmk..",
    "..kmrwwwwwwrmk..",
    "..kmrwwwwwwrmk..",
    "..kmmrrrrrrmmk..",
    "...kmmmmmmmmk...",
    "..kkmmmmmmmmkk..",
    ".kmmkmmmmmmkmmk.",
    ".km.kmmmmmmk.mk.",
    ".kk.kmmmmmmk.kk.",
    "....kmmmmmmk....",
    "...kmmk..kmmk...",
    "...kk......kk...",
    "................",
];

const CULTIST: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kRRRRRRk....",
    "....kRssssRk....",
    "....kRRRRRRk....",
    "...kkRRRRRRkk...",
    "..kRRkRRRRkRRk..",
    "..kR.kRRRRk.Rk..",
    "..kR.kRRRRk.Rk..",
    "..kk.kRRRRk.kk..",
    ".....kRRRRk.....",
    ".....kRRRRk.....",
    "....kRRkkRRk....",
    "....kRk..kRk....",
    "....kk....kk....",
    "................",
];

const WRAITH: &[&str] = &[
    "................",
    "......kkkk......",
    "....kkcccckk....",
    "...kcccccccck...",
    "..kccckkkkccck..",
    "..kcckkyykkcck..",
    "..kccckkkkccck..",
    "..kcccccccccck..",
    "...kcccccccck...",
    "...kcckcckcck...",
    "..kcccccccccck..",
    "..kcckcccckcck..",
    "...kkcccccckk...",
    "....kck..kck....",
    "....kk....kk....",
    "................",
];

const BOMBER: &[&str] = &[
    "................",
    "....k......k....",
    "...kek....kek...",
    "...kkkkkkkkkk...",
    "..krrrrrrrrrrk..",
    "..krwwrrrrwwrk..",
    "..krrrrkkrrrrk..",
    "..krrrrrrrrrrk..",
    "...krrrrrrrrk...",
    "..krrkrrrrkrrk..",
    ".krrrkrrrrkrrrk.",
    ".krrrkkkkkkrrrk.",
    "..kkk......kkk..",
    "...k........k...",
    "................",
    "................",
];

const SENTINEL: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kmmmmmmk....",
    "....kmmmmmmk....",
    "....kmssssmk....",
    "....kmmmmmmk....",
    "...kkmmmmmmkk...",
    "..kmmkkkkkkmmk..",
    ".kmmmkMMMMkmmmk.",
    ".kmmmkMllMkmmmk.",
    ".kmmmkMllMkmmmk.",
    ".kmmmkMMMMkmmmk.",
    ".kmmmkmmmmkmmmk.",
    "..kkkkmmmmkkkk..",
    ".....kkkkkk.....",
    "................",
];

const EYE: &[&str] = &[
    "................",
    "......kkkk......",
    "....kkppppkk....",
    "...kppppppppk...",
    "..kppppppppppk..",
    "..kpppkkkkpppk..",
    ".kpppkkwwkkpppk.",
    ".kpppkwkkwkpppk.",
    ".kpppkwkkwkpppk.",
    ".kpppkkwwkkpppk.",
    "..kpppkkkkpppk..",
    "..kppppppppppk..",
    "...kppppppppk...",
    "....kkppppkk....",
    "......kkkk......",
    "................",
];

const MERCHANT: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kppppppk....",
    "...kppppppppk...",
    "...kppkkkkppk...",
    "...kppkyykppk...",
    "...kppkkkkppk...",
    "..kpppppppppk...",
    "..kpppppppppk...",
    ".kpppppppppppk..",
    ".kppPppppppPppk.",
    "kppppppppppppppk",
    "kppppppppppppppk",
    ".kkkkkkkkkkkkkk.",
    "................",
    "................",
];

const BARREL: &[&str] = &[
    "................",
    "....kkkkkkkk....",
    "...knnnnnnnnk...",
    "...knNnnnnNnk...",
    "...knnnnnnnnk...",
    "...knNnnnnNnk...",
    "...knnnnnnnnk...",
    "...knNnnnnNnk...",
    "...knnnnnnnnk...",
    "...knnnnnnnnk...",
    "...knNnnnnNnk...",
    "...knnnnnnnnk...",
    "...knnnnnnnnk...",
    "...kkkkkkkkkk...",
    "................",
    "................",
];

/// Cofre de madera con tapa, bisagras y cerradura dorada.
const CHEST: &[&str] = &[
    "................",
    "..kkkkkkkkkkkk..",
    ".knnnnnnnnnnnnk.",
    ".knNNNNNNNNNNnk.",
    ".knyyyyyyyyyynk.",
    ".knyooooooooynk.",
    ".kkkkkkkkkkkkkk.",
    ".knnnnnnnnnnnnk.",
    ".knyyyyyyyyyynk.",
    ".knynnnnnnnnynk.",
    ".knynkkkkkknynk.",
    ".knynkyyyyknynk.",
    ".knynkkkkkknynk.",
    ".knnnnnnnnnnnnk.",
    ".kkkkkkkkkkkkkk.",
    "................",
];

/// Caja de madera con tablones y refuerzo en cruz.
const CRATE: &[&str] = &[
    "................",
    ".kkkkkkkkkkkkkk.",
    ".knnnnnnnnnnnnk.",
    ".knNnnnnnnnnNnk.",
    ".knnNnnnnnnNnnk.",
    ".knnnNnnnnNnnnk.",
    ".knnnnNnnNnnnnk.",
    ".knnnnnNNnnnnnk.",
    ".knnnnnNNnnnnnk.",
    ".knnnnNnnNnnnnk.",
    ".knnnNnnnnNnnnk.",
    ".knnNnnnnnnNnnk.",
    ".knNnnnnnnnnNnk.",
    ".knnnnnnnnnnnnk.",
    ".kkkkkkkkkkkkkk.",
    "................",
];

const KEY: &[&str] = &[
    "........", ".kkkk...", "kyyyyk..", "kykkyk..", "kyyyyk..", ".kyyk...", "..kyk...", "..kk....",
];

const BED: &[&str] = &[
    "....................",
    "..kkkkkkkkkkkkkkkk..",
    ".kmmmmmmmmmmmmmmmmk.",
    ".kmllllllllllllllmk.",
    ".kmllllllllllllllmk.",
    ".kmccccccccccccccmk.",
    ".kmccccccccccccccmk.",
    ".kmccccccccccccccmk.",
    ".kmccccccccccccccmk.",
    ".kmccccccccccccccmk.",
    ".kkkkkkkkkkkkkkkkkk.",
    ".knk............knk.",
    ".knk............knk.",
    ".kkk............kkk.",
    "....................",
    "....................",
];

const URN: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kppppppk....",
    "...kppppppppk...",
    "...kppppppppk...",
    "...kppppppppk...",
    "....kppppppk....",
    ".....kppppk.....",
    ".....kppppk.....",
    ".....kppppk.....",
    "....kppppppk....",
    "....kppppppk....",
    "....kkkkkkkk....",
    "................",
    "................",
    "................",
];

const STATUE: &[&str] = &[
    "................",
    ".....kkkkkk.....",
    "....kllllllk....",
    "....klsssslk....",
    "....kllllllk....",
    ".....kllllk.....",
    "....kllllllk....",
    "...kllllllllk...",
    "...kllllllllk...",
    "...kllllllllk...",
    "...kllllllllk...",
    "...kllllllllk...",
    "..kkkkkkkkkkkk..",
    "................",
    "................",
    "................",
];

const PORTAL: &[&str] = &[
    "................",
    ".......kk.......",
    "......kppk......",
    ".....kpccpk.....",
    "....kpccccpk....",
    "...kpcc..ccpk...",
    "..kpcc....ccpk..",
    "..kpc......cpk..",
    "..kpc......cpk..",
    "..kpc......cpk..",
    "..kpc......cpk..",
    "..kpc......cpk..",
    "..kpcc....ccpk..",
    "...kpccccccpk...",
    "....kkkkkkkk....",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const VIAL: &[&str] = &[
    "..kkkk..", ".knnnnk.", ".krrrrk.", ".krwwrk.", ".krrrrk.", ".krrrrk.", "..krrk..", "...kk...",
];

const COIN: &[&str] = &[
    "........", "..kkkk..", ".kyyyyk.", "kyyyyoyk", "kyyoyyyk", "kyyyyyyk", ".kyyyyk.", "..kkkk..",
];

const GEM: &[&str] = &[
    "........", "..kkkk..", ".kcccck.", "kccwwcck", "kccwwcck", ".kcccck.", "..kkkk..", "........",
];

fn make_background() -> Texture2D {
    let mut img = Image::gen_image_color(480, 270, Color::from_rgba(7, 7, 18, 255));
    let mut rng = Rng::new(0xC0FFEE_1234);

    // Cielo del abismo: degradado frío y dos nebulosas tenues.
    for y in 0..270 {
        let t = y as f32 / 269.0;
        let base = Color::new(
            0.018 + (1.0 - t) * 0.030,
            0.018 + (1.0 - t) * 0.024,
            0.052 + (1.0 - t) * 0.060,
            1.0,
        );
        for x in 0..480 {
            img.set_pixel(x, y, base);
        }
    }
    for y in 0..270 {
        for x in 0..480 {
            let dx = (x as f32 - 240.0) / 210.0;
            let dy = (y as f32 - 120.0) / 150.0;
            let d = (dx * dx + dy * dy).sqrt();
            let violet = ((1.0 - d).max(0.0)).powf(2.2) * 0.34;
            let dx2 = (x as f32 - 88.0) / 160.0;
            let dy2 = (y as f32 - 210.0) / 110.0;
            let d2 = (dx2 * dx2 + dy2 * dy2).sqrt();
            let cyan = ((1.0 - d2).max(0.0)).powf(2.4) * 0.18;
            if violet > 0.0 || cyan > 0.0 {
                let c = img.get_pixel(x, y);
                img.set_pixel(
                    x,
                    y,
                    Color::new(
                        (c.r + violet * 0.30 + cyan * 0.10).min(1.0),
                        (c.g + violet * 0.16 + cyan * 0.24).min(1.0),
                        (c.b + violet * 0.50 + cyan * 0.42).min(1.0),
                        1.0,
                    ),
                );
            }
        }
    }

    // Polvo de estrellas.
    for _ in 0..190 {
        let x = rng.i32(480);
        let y = rng.i32(270);
        let b = rng.range(0.30, 1.0);
        let c = match rng.usize(9) {
            0 => Color::new(0.62, 0.82, 1.0, b),
            1 => Color::new(1.0, 0.84, 0.54, b),
            _ => Color::new(0.82, 0.86, 1.0, b),
        };
        img.set_pixel(x as u32, y as u32, c);
        if rng.chance(0.10) {
            let faint = Color::new(c.r, c.g, c.b, b * 0.45);
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let px = x + dx;
                let py = y + dy;
                if px >= 0 && px < 480 && py >= 0 && py < 270 {
                    img.set_pixel(px as u32, py as u32, faint);
                }
            }
        }
    }

    // Constelación de la cripta.
    let nodes = [
        (54.0f32, 38.0f32),
        (108.0, 70.0),
        (164.0, 44.0),
        (220.0, 84.0),
        (278.0, 50.0),
        (336.0, 88.0),
        (396.0, 54.0),
        (438.0, 92.0),
    ];
    let thread = Color::from_rgba(48, 60, 96, 255);
    for pair in nodes.windows(2) {
        bg_line(
            &mut img,
            pair[0].0 as i32,
            pair[0].1 as i32,
            pair[1].0 as i32,
            pair[1].1 as i32,
            thread,
        );
    }
    for (nx, ny) in nodes {
        let x = nx as i32;
        let y = ny as i32;
        for (dx, dy) in [(0i32, 0i32), (-1, 0), (1, 0), (0, -1), (0, 1)] {
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && px < 480 && py >= 0 && py < 270 {
                img.set_pixel(px as u32, py as u32, Color::from_rgba(150, 180, 230, 255));
            }
        }
    }

    // Aro ritual lejano.
    for k in 0..720 {
        let a = k as f32 * std::f32::consts::TAU / 720.0;
        let x = 240.0 + a.cos() * 118.0;
        let y = 136.0 + a.sin() * 118.0;
        if x >= 0.0 && x < 480.0 && y >= 0.0 && y < 270.0 {
            img.set_pixel(x as u32, y as u32, Color::from_rgba(38, 44, 76, 255));
        }
    }

    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn bg_line(img: &mut Image, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        if x >= 0 && x < 480 && y >= 0 && y < 270 {
            img.set_pixel(x as u32, y as u32, c);
        }
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

fn make_vignette() -> Texture2D {
    let mut img = Image::gen_image_color(480, 270, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..270 {
        for x in 0..480 {
            let dx = (x as f32 - 240.0) / 240.0;
            let dy = (y as f32 - 135.0) / 135.0;
            let d = (dx * dx + dy * dy).sqrt();
            let a = ((d - 0.52) * 1.15).clamp(0.0, 0.68);
            img.set_pixel(x, y, Color::new(0.02, 0.01, 0.04, a));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn make_glow_white() -> Texture2D {
    let mut img = Image::gen_image_color(64, 64, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..64 {
        for x in 0..64 {
            let dx = (x as f32 - 32.0) / 32.0;
            let dy = (y as f32 - 32.0) / 32.0;
            let d = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
            let a = (1.0 - d).powf(2.2) * 0.55;
            img.set_pixel(x, y, Color::new(1.0, 1.0, 1.0, a));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn make_glow() -> Texture2D {
    let mut img = Image::gen_image_color(64, 64, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..64 {
        for x in 0..64 {
            let dx = (x as f32 - 32.0) / 32.0;
            let dy = (y as f32 - 32.0) / 32.0;
            let d = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
            let a = (1.0 - d).powf(2.2) * 0.55;
            img.set_pixel(x, y, Color::new(1.0, 0.62, 0.24, a));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

impl Sprites {
    pub fn load() -> Self {
        Self {
            knight: make_sprite(KNIGHT),
            archer: make_sprite(ARCHER),
            mage: make_sprite(MAGE),
            assassin: make_sprite(ASSASSIN),
            necro: make_sprite(NECRO),
            cleric: make_sprite(CLERIC),
            merchant_hero: make_sprite(MERCHANT_HERO),
            skeleton: make_sprite(SKELETON),
            slime: make_sprite(SLIME),
            bat: make_sprite(BAT),
            brute: make_sprite(BRUTE),
            cultist: make_sprite(CULTIST),
            wraith: make_sprite(WRAITH),
            bomber: make_sprite(BOMBER),
            sentinel: make_sprite(SENTINEL),
            eye: make_sprite(EYE),
            merchant: make_sprite(MERCHANT),
            bosses: std::array::from_fn(boss_skin),
            barrel: make_sprite(BARREL),
            crate_: make_sprite(CRATE),
            chest: make_sprite(CHEST),
            key: make_sprite(KEY),
            bed: make_sprite(BED),
            urn: make_sprite(URN),
            statue: make_sprite(STATUE),
            portal: make_sprite(PORTAL),
            vial: make_sprite(VIAL),
            coin: make_sprite(COIN),
            gem: make_sprite(GEM),
            bg: make_background(),
            vignette: make_vignette(),
            glow: make_glow(),
            glow_white: make_glow_white(),
        }
    }
}

#[allow(dead_code)]
pub fn draw_centered(tex: &Texture2D, pos: Vec2, scale: f32, tint: Color) {
    let w = tex.width() * scale;
    let h = tex.height() * scale;
    draw_texture_ex(
        tex,
        pos.x - w * 0.5,
        pos.y - h * 0.5,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        },
    );
}

pub fn draw_shadow(pos: Vec2, w: f32, h: f32) {
    draw_ellipse(
        pos.x,
        pos.y + h * 0.34,
        w * 0.42,
        h * 0.16,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.30),
    );
}
