// Texturas: almacenamiento, muestreo UV y generadores procedurales.
//
// Una textura es una rejilla de colores (texels). Un impacto trae una
// coordenada UV en [0, 1] (u = horizontal, v = vertical, v = 0 arriba)
// y el color de la superficie se lee de esa posición:
//
//      x = u * ancho - 0.5        y = v * alto - 0.5
//
// Todas las texturas se generan UNA vez al arrancar 

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

    pub(crate) fn set(&mut self, x: i32, y: i32, c: Color) {
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

    pub(crate) fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
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
// Instrumentos exclusivos de la cabina. Reutilizan las mismas utilidades
// de dibujo procedural que pantalla_tecnica() y polimero_consola().
pub fn radar_cabina() -> Textura {
    let mut t = Textura::new(640, 400, Color::new(4, 15, 24, 255));
    let grid = Color::new(12, 44, 51, 255);
    let cyan = Color::new(40, 210, 230, 255);
    let green = Color::new(75, 220, 135, 255);
    t.rect_outline(12, 12, 616, 376, 3, cyan);
    for x in (30..460).step_by(25) { t.fill_rect(x, 35, 1, 330, grid); }
    for y in (35..365).step_by(25) { t.fill_rect(30, y, 430, 1, grid); }
    for r in [35.0, 70.0, 105.0, 145.0] {
        t.ring(235, 200, r, 2.0, green, None);
    }
    t.fill_rect(85, 200, 300, 1, green);
    t.fill_rect(235, 50, 1, 300, green);
    // Sector de barrido estático y tres contactos de navegación.
    for y in 55..200 {
        for x in 235..385 {
            let dx = (x - 235) as f32;
            let dy = (200 - y) as f32;
            if dx * dx + dy * dy < 145.0 * 145.0 && dy > dx * 1.2 && dy < dx * 1.8 {
                t.set(x, y, Color::new(28, 100, 69, 255));
            }
        }
    }
    for (x, y) in [(170, 145), (315, 260), (270, 100)] {
        t.fill_rect(x - 3, y - 3, 7, 7, Color::new(200, 255, 180, 255));
        t.rect_outline(x - 8, y - 8, 17, 17, 1, green);
    }
    // Columna de estado distinta de la pantalla del planetario.
    for i in 0..6 {
        let y = 55 + i * 48;
        t.fill_rect(490, y, 105, 4, cyan);
        t.fill_rect(490, y + 12, 105, 13, grid);
        t.fill_rect(490, y + 12, 35 + (i * 13) % 70, 13, green);
    }
    t
}

pub fn botonera_cabina() -> Textura {
    let mut t = Textura::new(500, 400, Color::new(22, 27, 37, 255));
    let borde = Color::new(115, 130, 146, 255);
    let colores = [
        Color::new(55, 180, 200, 255), Color::new(210, 155, 55, 255),
        Color::new(170, 60, 62, 255), Color::new(74, 145, 110, 255),
        Color::new(122, 120, 162, 255),
    ];
    t.rect_outline(8, 8, 484, 384, 3, borde);
    // Botones con relieve pintado y etiquetas, como en polimero_consola().
    for fila in 0..4 {
        for col in 0..5 {
            let x = 28 + col * 92;
            let y = 32 + fila * 72;
            t.fill_rect(x, y, 72, 43, Color::new(9, 12, 18, 255));
            let c = colores[((col + fila * 2) % 5) as usize];
            t.fill_rect(x + 4, y + 4, 64, 32, c);
            t.fill_rect(x + 5, y + 4, 62, 3, borde);
            t.fill_rect(x + 12, y + 50, 42, 3, borde);
        }
    }
    // Dos pulsadores grandes y un grupo de testigos en la franja inferior.
    t.fill_rect(30, 330, 125, 40, colores[4]);
    t.fill_rect(173, 330, 125, 40, colores[0]);
    for i in 0..4 { t.fill_rect(326 + i * 36, 343, 22, 16, colores[i as usize]); }
    t
}

pub fn sistemas_cabina() -> Textura {
    let mut t = Textura::new(640, 400, Color::new(5, 18, 32, 255));
    let cyan = Color::new(55, 205, 235, 255);
    let tenue = Color::new(18, 65, 85, 255);
    t.rect_outline(12, 12, 616, 376, 3, cyan);
    // Gráfico de señal y cuatro barras de potencia.
    for x in (35..400).step_by(30) { t.fill_rect(x, 55, 1, 210, tenue); }
    for y in (55..270).step_by(30) { t.fill_rect(35, y, 365, 1, tenue); }
    for x in 35..400 {
        let y = 160 + ((x as f32 * 0.055).sin() * 58.0) as i32;
        t.fill_rect(x, y, 2, 4, cyan);
    }
    for i in 0..4 {
        let x = 445 + i * 42;
        let h = 70 + i * 35;
        t.fill_rect(x, 65, 24, 210, tenue);
        t.fill_rect(x, 275 - h, 24, h, Color::new(75, 210, 145, 255));
    }
    for i in 0..6 {
        t.rect_outline(35 + i * 96, 315, 76, 46, 2, cyan);
        t.fill_rect(45 + i * 96, 332, 42 + (i % 3) * 6, 8, cyan);
    }
    t
}


// Tejido gris azulado con costuras en rombos. No carga PNG ni usa
// DrawTexturePro: el ray tracer consulta esta textura mediante sample().
pub fn tapizado_cabina() -> Textura {
    let size = 256;
    let mut t = Textura::new(size, size, Color::BLACK);
    let periodo = 128;
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            let a = (x + y).rem_euclid(periodo);
            let b = (x - y).rem_euclid(periodo);
            let distancia = a.min(periodo - a).min(b.min(periodo - b));
            // Centro de cada rombo algo más claro: volumen pintado del acolchado.
            let pa = (std::f32::consts::PI * a as f32 / periodo as f32).sin();
            let pb = (std::f32::consts::PI * b as f32 / periodo as f32).sin();
            let tejido = if (x / 2 + y / 2) % 2 == 0 { 0.025 } else { -0.025 };
            let grano = (hash(x, y) - 0.5) * 0.045;
            let mut k = 0.82 + 0.22 * pa * pb + tejido + grano;
            if distancia <= 2 {
                k *= 0.48; // hendidura oscura de la costura
            } else if distancia <= 4 {
                k *= 1.12; // borde suave
            }
            let mut color = shade_gray((115.0, 133.0, 151.0), k);
            // Puntadas discontinuas sobre las dos diagonales.
            if distancia <= 1 && (x + 2 * y).rem_euclid(14) < 5 {
                color = Color::new(128, 145, 160, 255);
            }
            t.set(x, y, color);
        }
    }
    t
}


// Acabados de la nave: 128 x 128, calculados una vez al iniciar.
// El relieve es pintado; no añade geometría ni rayos secundarios.
pub fn pared_nave() -> Textura {
    let size = 128;
    let mut t = Textura::new(size, size, Color::BLACK);
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            // Un módulo cuadrado con panel central de esquinas recortadas.
            let dx = (x - 64).abs();
            let dy = (y - 64).abs();
            let dentro = dx <= 46 && dy <= 45 && dx + dy <= 80;
            let borde = dx >= 44 || dy >= 43 || dx + dy >= 77;
            let gris = if dentro {
                if borde { 100.0 } else { 174.0 + (hash(x, y) - 0.5) * 2.0 }
            } else { 143.0 };
            let v = gris as u8;
            t.set(x, y, Color::new(v, v, v, 255));
        }
    }
    let oscuro = Color::new(73, 73, 73, 255);
    let claro = Color::new(190, 190, 190, 255);
    // Canales laterales: rejillas pintadas, sin cubos por cada ranura.
    for x in [4, 114] {
        t.fill_rect(x, 18, 10, 92, oscuro);
        for y in (22..106).step_by(7) {
            t.fill_rect(x + 1, y, 8, 2, claro);
        }
    }
    // Tapa de acceso con marco doble y cierre discreto.
    t.rect_outline(33, 43, 62, 39, 2, Color::new(118, 118, 118, 255));
    t.rect_outline(36, 46, 56, 33, 1, claro);
    t.fill_rect(80, 59, 6, 12, oscuro);
    t.fill_rect(81, 60, 3, 9, claro);
    // Canal horizontal superior e inferior y pequeñas fijaciones.
    for y in [6, 118] {
        t.fill_rect(19, y, 90, 3, oscuro);
        t.fill_rect(19, y + 3, 90, 1, claro);
    }
    for (x, y) in [(27, 30), (99, 30), (27, 96), (99, 96)] {
        t.fill_rect(x, y, 3, 3, oscuro);
    }
    t
}

pub fn piso_nave() -> Textura {
    let size = 128;
    let mut t = Textura::new(size, size, Color::BLACK);
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            // Una baldosa de 1.2 x 1.2, más oscura que las paredes.
            let mut gris = 105.0 + (hash(x, y) - 0.5) * 3.0;
            if x < 2 || y < 2 {
                gris = 53.0;
            } else if x == 2 || y == 2 {
                gris = 126.0;
            } else if x >= 126 || y >= 126 {
                gris = 83.0;
            } else if x > 7 && x < 120 && y > 7 && y < 120 {
                // Relieves cortos alternados: patrón antideslizante de bajo contraste.
                let cx = x / 16;
                let cy = y / 16;
                let px = x % 16;
                let py = y % 16;
                let (a, b) = if (cx + cy) % 2 == 0 { (px, py) } else { (py, px) };
                if (4..=11).contains(&a) {
                    if b == 6 || b == 7 { gris += 9.0; }
                    if b == 8 { gris -= 7.0; }
                }
            }
            let v = gris as u8;
            t.set(x, y, Color::new(v, v, v, 255));
        }
    }
    t
}


pub fn monitor_comando() -> Textura {
    let mut t = Textura::new(512, 256, Color::new(4, 18, 31, 255));
    let cyan = Color::new(52, 200, 225, 255);
    let dim = Color::new(15, 62, 83, 255);
    let verde = Color::new(80, 205, 152, 255);
    t.rect_outline(6, 6, 500, 244, 2, cyan);
    for x in (16..496).step_by(24) { t.fill_rect(x, 24, 1, 208, dim); }
    for y in (24..232).step_by(24) { t.fill_rect(16, y, 480, 1, dim); }
    // Vista esquemática de la nave en el centro, con nodos de subsistemas.
    for y in 48..201 {
        let mitad = if y < 95 { (y - 48) / 2 + 8 } else { 31 - (y - 95) / 8 };
        t.fill_rect(256 - mitad, y, mitad * 2, 1, dim);
        t.set(256 - mitad, y, cyan);
        t.set(256 + mitad - 1, y, cyan);
    }
    t.fill_rect(255, 53, 2, 140, cyan);
    for (x,y) in [(205, 118), (304, 118), (224, 181), (285, 181)] {
        t.ring(x,y,12.0,2.0,cyan,None);
        t.fill_rect(x-3,y-3,6,6,verde);
    }
    // Dos instrumentos de navegación a la izquierda.
    for y in [83, 177] {
        t.ring(79,y,33.0,2.0,cyan,Some((305.0,345.0)));
        t.ring(79,y,23.0,1.5,verde,None);
        t.fill_rect(75,y-3,20,5,cyan);
        t.fill_rect(125,y-20,54,4,cyan);
        t.fill_rect(125,y-8,37,3,verde);
        t.fill_rect(125,y+4,46,3,cyan);
    }
    // Estado de energía y señal a la derecha.
    for i in 0..5 {
        let y=44+i*27;
        t.fill_rect(351,y,130,12,dim);
        t.fill_rect(351,y,45+(i*17)%83,12,if i==3 { Color::new(225,164,70,255) } else { verde });
    }
    for x in 351..482 {
        let y=207+((x as f32*0.13).sin()*12.0) as i32;
        t.fill_rect(x,y,1,3,cyan);
    }
    t
}

pub fn controles_comando() -> Textura {
    let mut t = Textura::new(512, 128, Color::new(139, 146, 150, 255));
    let negro=Color::new(23,29,34,255);
    let claro=Color::new(208,213,215,255);
    let colores=[Color::new(183,54,49,255),Color::new(85,178,91,255),Color::new(226,181,57,255),Color::new(51,177,204,255)];
    t.rect_outline(3,3,506,122,3,negro);
    // Teclado a la izquierda.
    t.fill_rect(12,12,125,104,negro);
    for fila in 0..4 {
        for col in 0..4 {
            let x=18+col*29; let y=18+fila*24;
            t.fill_rect(x,y,22,17,if fila==0 { colores[col as usize] } else { claro });
            t.fill_rect(x+7,y+6,8,3,negro);
        }
    }
    // Correderas y grupos de interruptores en módulos separados.
    for i in 0..5 {
        let x=151+i*30;
        t.fill_rect(x,17,3,87,negro);
        t.fill_rect(x-5,30+(i*13)%53,14,12,negro);
        t.fill_rect(x-4,31+(i*13)%53,12,2,claro);
    }
    t.rect_outline(309,12,89,104,2,negro);
    for i in 0..4 {
        t.fill_rect(320,23+i*22,27,13,colores[i as usize]);
        t.fill_rect(359,23+i*22,27,13,negro);
    }
    // Rejilla y testigos de comunicación.
    for y in (20..91).step_by(8) { t.fill_rect(412,y,50,4,negro); }
    for i in 0..4 { t.fill_rect(478,18+i*24,13,15,colores[i as usize]); }
    t.fill_rect(415,103,45,5,colores[3]);
    t
}


// Detalle de las hojas pintado una vez; no añade cubos ni reflejos.
pub fn hoja_compuerta() -> Textura {
    let mut t = Textura::new(256, 512, Color::new(135, 141, 144, 255));
    let oscuro = Color::new(43, 49, 53, 255);
    let claro = Color::new(186, 192, 194, 255);
    t.rect_outline(5, 5, 246, 502, 5, oscuro);
    // Placas con esquinas recortadas y doble borde.
    for (inicio, fin) in [(28, 215), (242, 430)] {
        for y in inicio..fin {
            let margen = 20 + (22 - (y - inicio).min(fin - 1 - y)).max(0);
            t.fill_rect(margen, y, 256 - 2 * margen, 1, oscuro);
            t.fill_rect(margen + 3, y, 250 - 2 * margen, 1, claro);
            t.fill_rect(margen + 5, y, 246 - 2 * margen, 1, Color::new(116,124,129,255));
        }
    }
    for y in (275..331).step_by(8) { t.fill_rect(46,y,164,3,oscuro); }
    t.ring(128, 151, 33.0, 3.0, oscuro, None);
    t.fill_rect(110, 146, 36, 10, oscuro);
    // Banda de advertencia amarilla y negra en la base.
    for y in 457..490 {
        for x in 13..243 {
            t.set(x,y,if ((x+y)/18)%2==0 { Color::new(201,159,53,255) } else { oscuro });
        }
    }
    for (x,y) in [(14,15),(239,15),(14,441),(239,441)] { t.fill_rect(x,y,4,4,claro); }
    t
}

pub fn panel_esclusa() -> Textura {
    let mut t = Textura::new(256, 384, Color::new(159, 166, 169, 255));
    let oscuro = Color::new(22,31,38,255);
    let cyan = Color::new(61,204,223,255);
    let naranja = Color::new(222,156,63,255);
    t.rect_outline(5,5,246,374,4,oscuro);
    t.fill_rect(18,25,220,180,oscuro);
    t.rect_outline(26,33,204,164,2,cyan);
    // Diagrama de dos hojas cerradas y dos barras de estado.
    t.rect_outline(79,52,47,93,3,cyan);
    t.rect_outline(129,52,47,93,3,cyan);
    t.fill_rect(44,163,168,7,naranja);
    t.fill_rect(44,178,112,4,cyan);
    t.fill_rect(22,232,93,117,oscuro);
    t.ring(68,284,29.0,3.0,naranja,None);
    t.fill_rect(154,232,54,118,oscuro);
    for y in (240..343).step_by(13) { t.fill_rect(216,y,12,3,oscuro); }
    t
}
