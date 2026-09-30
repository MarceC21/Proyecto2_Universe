// Texturas: almacenamiento, muestreo UV y generadores procedurales.
//
// Una textura es una rejilla de colores (texels). Un impacto trae una
// coordenada UV en [0, 1] (u = horizontal, v = vertical, v = 0 arriba)
// y el color de la superficie se lee de esa posición:
//
//      x = u * ancho - 0.5        y = v * alto - 0.5
//
// Se interpola bilinealmente entre los 4 texels vecinos (menos
// "escalones" al acercarse que con el vecino más cercano), y las
// coordenadas se envuelven (u = 0.3 y u = 1.3 dan lo mismo), lo que
// permite repetir la textura en superficies grandes.
//
// Todas las texturas se generan UNA vez al arrancar (ver material.rs):
// durante el render solo se consulta `sample`.

use raylib::prelude::*;

pub struct Textura {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Color>,
}

impl Textura {
    pub fn new(width: usize, height: usize, fill: Color) -> Self {
        Self { width, height, pixels: vec![fill; width * height] }
    }

    fn get(&self, x: i32, y: i32) -> Color {
        let xi = x.rem_euclid(self.width as i32) as usize;
        let yi = y.rem_euclid(self.height as i32) as usize;
        self.pixels[yi * self.width + xi]
    }

    fn set(&mut self, x: i32, y: i32, c: Color) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = c;
        }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let fx = u * self.width as f32 - 0.5;
        let fy = v * self.height as f32 - 0.5;
        let x0 = fx.floor();
        let y0 = fy.floor();
        let tx = fx - x0;
        let ty = fy - y0;
        let (x0, y0) = (x0 as i32, y0 as i32);

        let c00 = self.get(x0, y0);
        let c10 = self.get(x0 + 1, y0);
        let c01 = self.get(x0, y0 + 1);
        let c11 = self.get(x0 + 1, y0 + 1);

        let lerp = |a: u8, b: u8, t: f32| a as f32 + (b as f32 - a as f32) * t;
        let channel = |f: fn(&Color) -> u8| {
            let top = lerp(f(&c00), f(&c10), tx);
            let bottom = lerp(f(&c01), f(&c11), tx);
            (top + (bottom - top) * ty) as u8
        };
        Color::new(channel(|c| c.r), channel(|c| c.g), channel(|c| c.b), 255)
    }

    // ----- Utilidades de dibujo (solo para generar texturas) -----

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    fn rect_outline(&mut self, x: i32, y: i32, w: i32, h: i32, t: i32, c: Color) {
        self.fill_rect(x, y, w, t, c);
        self.fill_rect(x, y + h - t, w, t, c);
        self.fill_rect(x, y, t, h, c);
        self.fill_rect(x + w - t, y, t, h, c);
    }

    // Anillo de radio r y grosor t; `gap_start..gap_end` (grados, 0 = derecha,
    // sentido horario en pantalla) deja un hueco para que parezca un indicador.
    fn ring(&mut self, cx: i32, cy: i32, r: f32, t: f32, c: Color, gap: Option<(f32, f32)>) {
        let m = (r + t + 1.0) as i32;
        for y in cy - m..=cy + m {
            for x in cx - m..=cx + m {
                let dx = (x - cx) as f32;
                let dy = (y - cy) as f32;
                let d = (dx * dx + dy * dy).sqrt();
                if (d - r).abs() <= t * 0.5 {
                    if let Some((g0, g1)) = gap {
                        let a = dy.atan2(dx).to_degrees().rem_euclid(360.0);
                        if a >= g0 && a <= g1 {
                            continue;
                        }
                    }
                    self.set(x, y, c);
                }
            }
        }
    }
}

// Hash entero barato -> [0, 1). Determinístico: misma entrada, mismo valor.
fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

fn shade_gray(base: (f32, f32, f32), k: f32) -> Color {
    Color::new(
        (base.0 * k).clamp(0.0, 255.0) as u8,
        (base.1 * k).clamp(0.0, 255.0) as u8,
        (base.2 * k).clamp(0.0, 255.0) as u8,
        255,
    )
}

// ---------------------------------------------------------------------
// Metal de placas: 2x2 placas por textura, juntas oscuras entre ellas,
// remaches en las esquinas de cada placa y un cepillado sutil.
// La textura se repite al mapearla por unidades de mundo (ver Cubo).
// ---------------------------------------------------------------------
pub fn metal_placas(size: usize, base: (f32, f32, f32)) -> Textura {
    let mut t = Textura::new(size, size, Color::BLACK);
    let plate = (size / 2) as i32;
    let seam = (size as f32 * 0.02).max(2.0) as i32; // ancho de la junta
    let rivet_r = (size as f32 * 0.028).max(2.0);
    let rivet_off = (size as f32 * 0.07) as i32;

    for y in 0..size as i32 {
        for x in 0..size as i32 {
            // Cepillado: ruido por fila (rayas horizontales) + grano fino.
            let streak = (hash(0, y) - 0.5) * 0.10;
            let grain = (hash(x, y) - 0.5) * 0.06;
            let mut k = 1.0 + streak + grain;

            // Juntas: cerca del borde de cada placa.
            let px = x % plate;
            let py = y % plate;
            if px < seam || py < seam {
                k *= 0.45;
            } else if px < seam + 2 || py < seam + 2 {
                k *= 1.12; // labio de luz junto a la junta
            }
            t.set(x, y, shade_gray(base, k));
        }
    }

    // Remaches: disco con brillo arriba a la izquierda.
    for py in 0..2 {
        for px in 0..2 {
            for (ox, oy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let cx = px * plate + if ox > 0 { rivet_off } else { plate - rivet_off };
                let cy = py * plate + if oy > 0 { rivet_off } else { plate - rivet_off };
                let m = rivet_r as i32 + 1;
                for y in cy - m..=cy + m {
                    for x in cx - m..=cx + m {
                        let dx = (x - cx) as f32;
                        let dy = (y - cy) as f32;
                        if dx * dx + dy * dy <= rivet_r * rivet_r {
                            let light = if dx + dy < -rivet_r * 0.3 { 1.35 } else if dx + dy > rivet_r * 0.5 { 0.6 } else { 1.0 };
                            t.set(x, y, shade_gray(base, light));
                        }
                    }
                }
            }
        }
    }
    t
}

// Color sólido (para indicadores emisivos).
pub fn solida(color: Color) -> Textura {
    Textura::new(4, 4, color)
}

// ---------------------------------------------------------------------
// Pantalla técnica. Se mapea UNA vez sobre toda la pantalla (UV local),
// vista desde arriba con el lado +Z (jugador) abajo de la imagen.
//
// Distribución (u, v en [0, 1]):
//   - centro (u 0.15..0.85): solo cuadrícula tenue, para que el
//     planetario se lea limpio;
//   - izquierda: barras de indicadores;
//   - derecha: dos anillos tipo medidor;
//   - abajo a la derecha: teclado; abajo: tres botones;
//   - marco cian fino con esquinas marcadas.
// ---------------------------------------------------------------------
pub fn pantalla_tecnica(w: usize, h: usize) -> Textura {
    let mut t = Textura::new(w, h, Color::BLACK);
    let (wi, hi) = (w as i32, h as i32);

    // Fondo azul oscuro con viñeta suave (más oscuro hacia los bordes).
    for y in 0..hi {
        for x in 0..wi {
            let nx = x as f32 / w as f32 - 0.5;
            let ny = y as f32 / h as f32 - 0.5;
            let r = (nx * nx * 1.2 + ny * ny * 1.6).sqrt().min(1.0);
            let k = 1.0 - 0.55 * r;
            t.set(x, y, Color::new((6.0 * k) as u8, (20.0 * k + 4.0) as u8, (48.0 * k + 8.0) as u8, 255));
        }
    }

    // Cuadrícula fina: líneas cada `minor` px, marcadas cada 5.
    let minor = (h / 31).max(8) as i32;
    let fine = Color::new(16, 58, 96, 255);
    let strong = Color::new(24, 98, 142, 255);
    for y in 0..hi {
        for x in 0..wi {
            let on_x = x % minor == 0;
            let on_y = y % minor == 0;
            if on_x || on_y {
                let major = (on_x && x % (minor * 5) == 0) || (on_y && y % (minor * 5) == 0);
                t.set(x, y, if major { strong } else { fine });
            }
        }
    }

    let cyan = Color::new(40, 220, 255, 255);
    let cyan_dim = Color::new(28, 150, 190, 255);
    let m = (h / 30).max(6) as i32; // margen del marco

    // Marco fino y esquinas marcadas.
    t.rect_outline(m, m, wi - 2 * m, hi - 2 * m, 2, cyan_dim);
    let c = (h / 12) as i32;
    for (x, y, sx, sy) in [(m, m, 1, 1), (wi - m, m, -1, 1), (m, hi - m, 1, -1), (wi - m, hi - m, -1, -1)] {
        let (bx, by) = (if sx > 0 { x } else { x - c }, if sy > 0 { y } else { y - c });
        t.fill_rect(bx, if sy > 0 { y } else { y - 4 }, c, 4, cyan);
        t.fill_rect(if sx > 0 { x } else { x - 4 }, by, 4, c, cyan);
    }

    // Izquierda: barras horizontales de longitud variable.
    let left_x = (w as f32 * 0.035) as i32;
    let bar_w = (w as f32 * 0.09) as i32;
    let bar_h = (h / 42).max(3) as i32;
    for i in 0..9 {
        let y = (h as f32 * 0.16) as i32 + i * (h as f32 * 0.075) as i32;
        let len = (bar_w as f32 * (0.35 + 0.65 * hash(i, 7))) as i32;
        t.fill_rect(left_x, y, bar_w, bar_h, Color::new(12, 44, 74, 255));
        t.fill_rect(left_x, y, len, bar_h, if i % 3 == 0 { cyan } else { cyan_dim });
    }

    // Derecha: dos anillos (medidores) con un hueco.
    let rx = (w as f32 * 0.925) as i32;
    let r1 = h as f32 * 0.075;
    t.ring(rx, (h as f32 * 0.25) as i32, r1, 4.0, cyan, Some((300.0, 350.0)));
    t.ring(rx, (h as f32 * 0.25) as i32, r1 * 0.55, 3.0, cyan_dim, None);
    t.ring(rx, (h as f32 * 0.50) as i32, r1 * 1.15, 4.0, cyan, Some((100.0, 170.0)));
    t.ring(rx, (h as f32 * 0.50) as i32, r1 * 0.6, 3.0, cyan_dim, None);

    // Abajo a la derecha: teclado (rejilla de teclas).
    let kx = (w as f32 * 0.84) as i32;
    let ky = (h as f32 * 0.72) as i32;
    let key = (h as f32 * 0.032) as i32;
    for j in 0..4 {
        for i in 0..5 {
            t.rect_outline(kx + i * (key + 3), ky + j * (key + 3), key, key, 1, cyan_dim);
        }
    }

    // Abajo: tres botones rectangulares (uno relleno = activo).
    let bw = (w as f32 * 0.075) as i32;
    let bh = (h as f32 * 0.045) as i32;
    let by = hi - m - bh - (h as f32 * 0.03) as i32;
    for i in 0..3 {
        let bx = (w as f32 * 0.04) as i32 + i * (bw + (w as f32 * 0.015) as i32);
        t.rect_outline(bx, by, bw, bh, 2, cyan);
        if i == 0 {
            t.fill_rect(bx + 4, by + 4, bw - 8, bh - 8, cyan_dim);
        }
    }

    // Arriba: marcas de escala a lo largo del borde superior.
    for i in 0..40 {
        let x = (w as f32 * 0.2) as i32 + i * (w as f32 * 0.015) as i32;
        let len = if i % 5 == 0 { 10 } else { 5 };
        t.fill_rect(x, m + 6, 2, len, cyan_dim);
    }

    t
}

// ---------------------------------------------------------------------
// Polímero de consola: panel oscuro con grano, borde fino y una
// rejilla de botones (3 columnas) con pequeñas etiquetas debajo.
// Se mapea UNA vez sobre la placa del módulo de controles.
// ---------------------------------------------------------------------
pub fn polimero_consola(w: usize, h: usize) -> Textura {
    let base = (40.0, 44.0, 56.0);
    let mut t = Textura::new(w, h, Color::BLACK);
    let (wi, hi) = (w as i32, h as i32);

    for y in 0..hi {
        for x in 0..wi {
            let k = 1.0 + (hash(x, y) - 0.5) * 0.10;
            t.set(x, y, shade_gray(base, k));
        }
    }

    t.rect_outline(4, 4, wi - 8, hi - 8, 3, Color::new(84, 94, 116, 255));

    let cols = 3;
    let rows = 6;
    let margin_x = wi / 9;
    let margin_y = hi / 14;
    let cell_w = (wi - 2 * margin_x) / cols;
    let cell_h = (hi - 2 * margin_y) / rows;
    let btn_w = cell_w * 6 / 10;
    let btn_h = cell_h * 4 / 10;

    for j in 0..rows {
        for i in 0..cols {
            let x = margin_x + i * cell_w + (cell_w - btn_w) / 2;
            let y = margin_y + j * cell_h + (cell_h - btn_h) / 4;
            t.fill_rect(x, y, btn_w, btn_h, Color::new(22, 25, 34, 255));
            t.rect_outline(x, y, btn_w, btn_h, 2, Color::new(110, 124, 150, 255));
            // Etiqueta: barrita bajo el botón.
            let lw = btn_w * (4 + (hash(i, j) * 6.0) as i32) / 10;
            t.fill_rect(x, y + btn_h + 4, lw, 3, Color::new(120, 134, 158, 255));
            // Un botón "activo": interior cian tenue.
            if (i + j * cols) % 5 == 2 {
                t.fill_rect(x + 4, y + 4, btn_w - 8, btn_h - 8, Color::new(28, 150, 190, 255));
            }
        }
    }
    t
}