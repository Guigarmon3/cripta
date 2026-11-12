use macroquad::prelude::*;

pub struct UiFonts {
    pub regular: Font,
    pub bold: Font,
}

impl UiFonts {
    pub fn load() -> Self {
        let mut regular =
            load_ttf_font_from_bytes(include_bytes!("../assets/AtkinsonHyperlegible-Regular.ttf"))
                .expect("no se pudo cargar la fuente regular");
        regular.set_filter(FilterMode::Linear);
        let mut bold =
            load_ttf_font_from_bytes(include_bytes!("../assets/AtkinsonHyperlegible-Bold.ttf"))
                .expect("no se pudo cargar la fuente bold");
        bold.set_filter(FilterMode::Linear);
        Self { regular, bold }
    }
}

/// Capa de interfaz en resolución nativa.
///
/// Las coordenadas siguen expresadas en el espacio virtual de 480x270, pero
/// se multiplican por `scale` al dibujar. El texto se rasteriza al tamaño real
/// de pantalla, así que no se escala ni se vuelve borroso.
pub struct Ui<'a> {
    pub fonts: &'a UiFonts,
    pub scale: f32,
    pub ox: f32,
    pub oy: f32,
    pub shake: Vec2,
}

impl<'a> Ui<'a> {
    pub fn new(fonts: &'a UiFonts, scale: f32, ox: f32, oy: f32, shake: Vec2) -> Self {
        Self {
            fonts,
            scale: scale.max(0.01),
            ox,
            oy,
            shake,
        }
    }

    pub fn x(&self, x: f32) -> f32 {
        self.ox + x * self.scale
    }

    pub fn y(&self, y: f32) -> f32 {
        self.oy + y * self.scale
    }

    pub fn s(&self, v: f32) -> f32 {
        v * self.scale
    }

    pub fn world_x(&self, x: f32) -> f32 {
        self.ox + (x + self.shake.x * 0.7) * self.scale
    }

    pub fn world_y(&self, y: f32) -> f32 {
        self.oy + (y + self.shake.y * 0.7) * self.scale
    }

    fn font(&self, bold: bool) -> &Font {
        if bold {
            &self.fonts.bold
        } else {
            &self.fonts.regular
        }
    }

    fn px(&self, size: f32) -> u16 {
        (size * self.scale).round().clamp(1.0, 512.0) as u16
    }

    pub fn measure(&self, text: &str, size: f32, bold: bool) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        measure_text(text, Some(self.font(bold)), self.px(size), 1.0).width / self.scale
    }

    pub fn text(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        self.text_bold(text, x, y, size, color, false);
    }

    pub fn text_bold(&self, text: &str, x: f32, y: f32, size: f32, color: Color, bold: bool) {
        if text.is_empty() {
            return;
        }
        draw_text_ex(
            text,
            self.x(x),
            self.y(y),
            TextParams {
                font: Some(self.font(bold)),
                font_size: self.px(size),
                color,
                ..Default::default()
            },
        );
    }

    pub fn text_shadow(&self, text: &str, x: f32, y: f32, size: f32, color: Color, bold: bool) {
        if text.is_empty() {
            return;
        }
        let px = self.px(size);
        let font = self.font(bold);
        let shadow = Color::new(0.0, 0.0, 0.0, 0.80);
        draw_text_ex(
            text,
            self.x(x) + 2.0,
            self.y(y) + 2.0,
            TextParams {
                font: Some(font),
                font_size: px,
                color: shadow,
                ..Default::default()
            },
        );
        draw_text_ex(
            text,
            self.x(x),
            self.y(y),
            TextParams {
                font: Some(font),
                font_size: px,
                color,
                ..Default::default()
            },
        );
    }

    pub fn text_shadow_world(
        &self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        bold: bool,
    ) {
        if text.is_empty() {
            return;
        }
        let px = self.px(size);
        let font = self.font(bold);
        let shadow = Color::new(0.0, 0.0, 0.0, 0.85);
        draw_text_ex(
            text,
            self.world_x(x) + 2.0,
            self.world_y(y) + 2.0,
            TextParams {
                font: Some(font),
                font_size: px,
                color: shadow,
                ..Default::default()
            },
        );
        draw_text_ex(
            text,
            self.world_x(x),
            self.world_y(y),
            TextParams {
                font: Some(font),
                font_size: px,
                color,
                ..Default::default()
            },
        );
    }

    pub fn text_center(&self, text: &str, y: f32, size: f32, color: Color) {
        let w = self.measure(text, size, false);
        self.text(text, 240.0 - w * 0.5, y, size, color);
    }

    pub fn text_center_shadow(&self, text: &str, y: f32, size: f32, color: Color) {
        let w = self.measure(text, size, true);
        self.text_shadow(text, 240.0 - w * 0.5, y, size, color, true);
    }

    pub fn text_right(&self, text: &str, right: f32, y: f32, size: f32, color: Color, bold: bool) {
        let w = self.measure(text, size, bold);
        self.text_bold(text, right - w, y, size, color, bold);
    }

    pub fn text_right_shadow(
        &self,
        text: &str,
        right: f32,
        y: f32,
        size: f32,
        color: Color,
        bold: bool,
    ) {
        let w = self.measure(text, size, bold);
        self.text_shadow(text, right - w, y, size, color, bold);
    }

    /// Texto con saltos de línea alineado a la izquierda. Devuelve la Y final.
    pub fn wrap_left(
        &self,
        text: &str,
        x: f32,
        y: f32,
        max_w: f32,
        size: f32,
        line_h: f32,
        color: Color,
    ) -> f32 {
        let mut cy = y;
        for paragraph in text.split('\n') {
            let mut line = String::with_capacity(48);
            for word in paragraph.split_whitespace() {
                let mut candidate = String::with_capacity(line.len() + word.len() + 1);
                candidate.push_str(&line);
                if !line.is_empty() {
                    candidate.push(' ');
                }
                candidate.push_str(word);
                if !line.is_empty() && self.measure(&candidate, size, false) > max_w {
                    self.text(&line, x, cy, size, color);
                    cy += line_h;
                    line.clear();
                    line.push_str(word);
                } else {
                    line = candidate;
                }
            }
            if !line.is_empty() {
                self.text(&line, x, cy, size, color);
                cy += line_h;
            }
        }
        cy
    }

    pub fn wrap_center(
        &self,
        text: &str,
        cx: f32,
        y: f32,
        max_w: f32,
        size: f32,
        line_h: f32,
        color: Color,
    ) -> f32 {
        let mut cy = y;
        for paragraph in text.split('\n') {
            let mut line = String::with_capacity(48);
            for word in paragraph.split_whitespace() {
                let mut candidate = String::with_capacity(line.len() + word.len() + 1);
                candidate.push_str(&line);
                if !line.is_empty() {
                    candidate.push(' ');
                }
                candidate.push_str(word);
                if !line.is_empty() && self.measure(&candidate, size, false) > max_w {
                    let w = self.measure(&line, size, false);
                    self.text(&line, cx - w * 0.5, cy, size, color);
                    cy += line_h;
                    line.clear();
                    line.push_str(word);
                } else {
                    line = candidate;
                }
            }
            if !line.is_empty() {
                let w = self.measure(&line, size, false);
                self.text(&line, cx - w * 0.5, cy, size, color);
                cy += line_h;
            }
        }
        cy
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        draw_rectangle(self.x(x), self.y(y), self.s(w), self.s(h), color);
    }

    pub fn rect_lines(&self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        draw_rectangle_lines(
            self.x(x),
            self.y(y),
            self.s(w),
            self.s(h),
            self.s(thickness),
            color,
        );
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        draw_line(
            self.x(x1),
            self.y(y1),
            self.x(x2),
            self.y(y2),
            self.s(thickness),
            color,
        );
    }

    pub fn circle(&self, x: f32, y: f32, r: f32, color: Color) {
        draw_circle(self.x(x), self.y(y), self.s(r), color);
    }

    pub fn ellipse(&self, x: f32, y: f32, rx: f32, ry: f32, color: Color) {
        draw_ellipse(self.x(x), self.y(y), self.s(rx), self.s(ry), 0.0, color);
    }

    pub fn triangle(&self, a: Vec2, b: Vec2, c: Vec2, color: Color) {
        draw_triangle(
            vec2(self.x(a.x), self.y(a.y)),
            vec2(self.x(b.x), self.y(b.y)),
            vec2(self.x(c.x), self.y(c.y)),
            color,
        );
    }

    pub fn panel(&self, x: f32, y: f32, w: f32, h: f32, border: Color) {
        self.rect(x, y, w, h, Color::new(0.045, 0.035, 0.065, 0.96));
        self.rect_lines(x, y, w, h, 1.0, border);
    }

    pub fn texture_centered(&self, tex: &Texture2D, pos: Vec2, scale: f32, tint: Color) {
        let w = tex.width() * scale * self.scale;
        let h = tex.height() * scale * self.scale;
        draw_texture_ex(
            tex,
            self.x(pos.x) - w * 0.5,
            self.y(pos.y) - h * 0.5,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
    }
}
