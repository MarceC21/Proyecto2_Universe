// TARS: cuatro columnas rectangulares independientes, como la maqueta de referencia.
// Sin dependencias nuevas. Geometría hecha solo con cubos.
//
// Marco local de TARS (mismo criterio que cabina.rs):
//   x local = a lo largo de las columnas (derecha del observador que lo mira de frente)
//   y local = hacia arriba, 0 = suelo
//   z local = hacia el frente (la cara con el nombre)
// punto() pasa de local a mundo con la misma rotación que Cubo::rotado_y.
//
// Las columnas alternan pasos durante el recorrido; las texturas se conservan.
use raylib::prelude::*;
use crate::colisiones::{Colisiones, Obstaculo};
use crate::cubo::Cubo;
use crate::material::{TARS_CASCO, TARS_NEGRO, TARS_NOMBRE, TARS_BRAILLE};
use crate::textura::Textura;

// Escala uniforme: conserva las proporciones y la animación del boceto.
const ESCALA: f32 = 2.10; // altura en reposo: 1.80 * 2.10 = 3.78

// Medidas locales originales; ESCALA se aplica al construir la geometría.
const ALTO: f32 = 1.80;
const ANCHO: f32 = 0.20;
const FONDO: f32 = 0.24;
const SEPARACION: f32 = 0.015;
const PASO: f32 = ANCHO + SEPARACION;
const TILE: f32 = 0.20; // una repetición de la textura del casco por columna
const RETRANQUEO: f32 = 0.06; // las juntas negras quedan hundidas

// Reposo: centro del espacio reservado en arco.rs, mirando hacia -X (hacia la nave).
const ORIGEN_X: f32 = 8.575;
const ORIGEN_Z: f32 = -5.55;
const ANGULO: f32 = -std::f32::consts::FRAC_PI_2;

// Parámetros para ajustar el recorrido sin tocar el render.
const VELOCIDAD: f32 = 0.85;
const VELOCIDAD_GIRO: f32 = 1.8;
const LONGITUD_PASO: f32 = 0.62;
const ESPERA_INICIAL: f32 = 4.0;
const ESPERA_BASE: f32 = 12.0;
// Envuelve todas las columnas, incluso durante un paso y al girar.
// Radio local máximo de las esquinas en un paso: 0.464; margen hasta 0.48.
const RADIO: f32 = 0.48 * ESCALA;

struct Destino {
    x: f32,
    z: f32,
    mirar: f32,
    espera: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Etapa { Caminar, Mirar, Esperar }

fn recorrido() -> Vec<Destino> {
    // Mismas visitas y pausas. Separación ajustada al cuerpo ampliado.
    // Frente del laboratorio, derivado de su huella real y su orientación.
    let cajas = crate::laboratorio::obstaculos();
    let lab = &cajas[0];
    let (s, c) = lab.angulo_y.sin_cos();
    let distancia = lab.mitad_z + RADIO + 0.26;
    let lx = lab.centro_x - s * distancia;
    let lz = lab.centro_z - c * distancia;
    let paso = |x, z| Destino { x, z, mirar: 0.0, espera: 0.0 };
    vec![
        paso(6.80, -4.65),
        paso(-6.65, -4.65),
        // Mini estación: de frente a su pantalla (-Z).
        Destino { x: -6.65, z: -4.75, mirar: std::f32::consts::PI, espera: 5.0 },
        paso(-5.10, -3.40),
        Destino { x: lx, z: lz, mirar: lab.angulo_y, espera: 6.0 },
        // Salir del laboratorio hacia el pasillo, sin cruzar la mesa.
        paso(-4.10, -3.25),
        Destino { x: 0.0, z: -3.25, mirar: 0.0, espera: 5.0 },
        paso(6.80, -4.65),
        Destino { x: ORIGEN_X, z: ORIGEN_Z, mirar: ANGULO, espera: ESPERA_BASE },
    ]
}

pub struct Tars {
    pub origen: Vector3,        // centro de la base, sobre el suelo
    pub angulo: f32,            // giro en Y (convención de Cubo::rotado_y)
    pub columnas: [Vector3; 4], // desplazamiento local de cada columna
    ruta: Vec<Destino>,
    destino: usize,
    etapa: Etapa,
    espera: f32,
    fase: f32,
    amplitud: f32,
}

impl Tars {
    pub fn reposo() -> Self {
        let ruta = recorrido();
        let destino = ruta.len() - 1;
        Self {
            ruta, destino, etapa: Etapa::Esperar, espera: ESPERA_INICIAL,
            fase: 0.0, amplitud: 0.0,
            origen: Vector3::new(ORIGEN_X, 0.0, ORIGEN_Z),
            angulo: ANGULO,
            columnas: [Vector3::zero(); 4],
        }
    }

    fn siguiente(&mut self) {
        self.destino = (self.destino + 1) % self.ruta.len();
        self.etapa = Etapa::Caminar;
    }

    fn girar(&mut self, objetivo: f32, dt: f32) -> bool {
        let pi = std::f32::consts::PI;
        let tau = std::f32::consts::TAU;
        let diferencia = (objetivo - self.angulo + pi).rem_euclid(tau) - pi;
        let paso = VELOCIDAD_GIRO * dt;
        self.angulo = (self.angulo + diferencia.clamp(-paso, paso) + pi)
            .rem_euclid(tau) - pi;
        diferencia.abs() <= paso
    }

    // Devuelve true solo si hay que refrescar los 12 cubos del personaje.
    // El jugador es opcional: en modo orbital no hay un cuerpo invisible bloqueando.
    pub fn actualizar(&mut self, dt: f32, colisiones: &Colisiones,
                      obstaculos: &[Obstaculo], jugador: Option<Vector3>) -> bool {
        if !dt.is_finite() || dt <= 0.0 { return false; }
        let anterior = (self.origen, self.angulo, self.columnas);
        // Subpasos: ni la ruta ni el jugador se atraviesan si baja el FPS.
        let tiempo = dt.min(0.25);
        let pasos = (tiempo / (1.0 / 60.0)).ceil().max(1.0) as usize;
        for _ in 0..pasos {
            self.avanzar(tiempo / pasos as f32, colisiones, obstaculos, jugador);
        }
        anterior.0 != self.origen || anterior.1 != self.angulo || anterior.2 != self.columnas
    }

    fn avanzar(&mut self, dt: f32, colisiones: &Colisiones,
               obstaculos: &[Obstaculo], jugador: Option<Vector3>) {
        let mut avance = 0.0;
        match self.etapa {
            Etapa::Esperar => {
                self.espera = (self.espera - dt).max(0.0);
                if self.espera == 0.0 { self.siguiente(); }
            }
            Etapa::Mirar => {
                let angulo = self.ruta[self.destino].mirar;
                if self.girar(angulo, dt) {
                    self.espera = self.ruta[self.destino].espera;
                    self.etapa = Etapa::Esperar;
                }
            }
            Etapa::Caminar => {
                let dx = self.ruta[self.destino].x - self.origen.x;
                let dz = self.ruta[self.destino].z - self.origen.z;
                let distancia = (dx * dx + dz * dz).sqrt();
                if distancia <= 0.0001 {
                    if self.ruta[self.destino].espera > 0.0 {
                        self.etapa = Etapa::Mirar;
                    } else { self.siguiente(); }
                } else if self.girar(dx.atan2(dz), dt) {
                    let paso = (VELOCIDAD * dt).min(distancia);
                    let x = self.origen.x + dx / distancia * paso;
                    let z = self.origen.z + dz / distancia * paso;
                    let ocupado = jugador.map_or(false, |p| {
                        let margen = crate::escena::PLAYER_RADIUS + 0.08;
                        let ex = ((p.x - x).abs() - RADIO).max(0.0);
                        let ez = ((p.z - z).abs() - RADIO).max(0.0);
                        ex * ex + ez * ez < margen * margen
                    });
                    if !ocupado && !colisiones.bloqueado(x, z, RADIO, obstaculos) {
                        self.origen.x = x;
                        self.origen.z = z;
                        avance = paso;
                    }
                    // Si hay alguien enfrente, espera y continúa cuando se aparta.
                }
            }
        }
        let objetivo = if avance > 0.0 { 1.0 } else { 0.0 };
        self.amplitud += (objetivo - self.amplitud).clamp(-dt * 5.0, dt * 5.0);
        self.fase = (self.fase + avance / LONGITUD_PASO * std::f32::consts::TAU)
            .rem_euclid(std::f32::consts::TAU);
        for (i, columna) in self.columnas.iter_mut().enumerate() {
            let fase = self.fase + if i % 2 == 0 { 0.0 } else { std::f32::consts::PI };
            *columna = Vector3::new(0.0,
                fase.sin().max(0.0) * 0.10 * self.amplitud,
                fase.cos() * 0.07 * self.amplitud);
        }
    }

    // Huella conservadora móvil: también reserva el espacio necesario para girar.
    pub fn obstaculo(&self) -> Obstaculo {
        Obstaculo { min_x: self.origen.x - RADIO, max_x: self.origen.x + RADIO,
                    min_z: self.origen.z - RADIO, max_z: self.origen.z + RADIO }
    }

    fn punto(&self, x: f32, y: f32, z: f32) -> Vector3 {
        let (s, c) = self.angulo.sin_cos();
        Vector3::new(self.origen.x + (c * x + s * z) * ESCALA,
                     self.origen.y + y * ESCALA,
                     self.origen.z + (-s * x + c * z) * ESCALA)
    }

    pub fn construir(&self, cubos: &mut Vec<Cubo>) {
        for (i, d) in self.columnas.iter().enumerate() {
            let x = (i as f32 - 1.5) * PASO + d.x;
            let mut pieza = |y0: f32, y1: f32, fondo: f32, material: usize, tile: Option<f32>| {
                cubos.push(Cubo::new(
                    self.punto(x, d.y + (y0 + y1) * 0.5, d.z),
                    Vector3::new(ANCHO, y1 - y0, fondo) * ESCALA, Color::WHITE, 0.0, 0.0,
                ).rotado_y(self.angulo).con_material(material, tile.map(|t| t * ESCALA)));
            };
            if i == 1 || i == 2 {
                // Columnas centrales: dos juntas negras y una placa con la etiqueta.
                let placa = if i == 1 { TARS_NOMBRE } else { TARS_BRAILLE };
                pieza(0.00, 0.40, FONDO, TARS_CASCO, Some(TILE));
                pieza(0.40, 0.70, FONDO - RETRANQUEO, TARS_NEGRO, None);
                pieza(0.70, 1.30, FONDO, placa, None);
                pieza(1.30, 1.58, FONDO - RETRANQUEO, TARS_NEGRO, None);
                pieza(1.58, ALTO, FONDO, TARS_CASCO, Some(TILE));
            } else {
                // Columnas exteriores: una sola pieza de casco segmentado.
                pieza(0.0, ALTO, FONDO, TARS_CASCO, Some(TILE));
            }
        }
    }
}

// ---------------------------------------------------------------------
// Texturas (se generan una vez al arrancar, desde material.rs)
// ---------------------------------------------------------------------
const GRIS: (f32, f32, f32) = (172.0, 176.0, 182.0);
const AMARILLO: Color = Color { r: 232, g: 196, b: 48, a: 255 };

fn ruido(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

fn tono(base: (f32, f32, f32), k: f32) -> Color {
    Color::new((base.0 * k).clamp(0.0, 255.0) as u8, (base.1 * k).clamp(0.0, 255.0) as u8,
               (base.2 * k).clamp(0.0, 255.0) as u8, 255)
}

// Gris metálico. Con rejilla: paneles pequeños "de ladrillo" (64 px = una columna).
// Sin rejilla: placa lisa con borde fino, para llevar texto encima.
fn placa(w: usize, h: usize, rejilla: bool) -> Textura {
    let mut t = Textura::new(w, h, Color::BLACK);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut k = 1.0 + (ruido(x, y) - 0.5) * 0.05;
            if rejilla {
                let desfase = ((y / 16) % 2) * 16;
                let v = (x - desfase).rem_euclid(32);
                if v == 0 || y % 16 == 0 { k *= 0.55; }
                else if v == 1 || y % 16 == 1 { k *= 1.10; }
            } else if x == 0 || y == 0 || x == w as i32 - 1 || y == h as i32 - 1 {
                k *= 0.6;
            }
            t.set(x, y, tono(GRIS, k));
        }
    }
    t
}

pub fn textura_casco() -> Textura { placa(64, 64, true) }

pub fn textura_negro() -> Textura {
    let mut t = Textura::new(32, 32, Color::BLACK);
    for y in 0..32 {
        for x in 0..32 {
            t.set(x, y, tono((38.0, 40.0, 46.0), 1.0 + (ruido(x, y) - 0.5) * 0.16));
        }
    }
    t
}

// Letras 5x7. El texto se lee de abajo hacia arriba (girado 90°), como en la maqueta.
const LETRAS: [[&str; 7]; 4] = [
    ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."], // T
    [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"], // A
    ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"], // R
    [".####", "#....", "#....", ".###.", "....#", "....#", "####."], // S
];
// Braille de T, A, R, S: puntos 1-3 columna izquierda, 4-6 columna derecha.
const BRAILLE: [&[u8]; 4] = [&[2, 3, 4, 5], &[1], &[1, 2, 3, 5], &[2, 3, 4]];
const BASE_TEXTO: i32 = 90; // borde inferior del texto en la textura de 64 x 192

pub fn textura_nombre() -> Textura {
    let mut t = placa(64, 192, false);
    let s = 3;
    for (li, letra) in LETRAS.iter().enumerate() {
        for (r, fila) in letra.iter().enumerate() {
            for (c, ch) in fila.bytes().enumerate() {
                if ch == b'#' {
                    // Fila del glifo -> x; columna del glifo -> y hacia arriba.
                    let x = 21 + r as i32 * s;
                    let y = BASE_TEXTO - (li as i32 * 6 + c as i32 + 1) * s;
                    t.fill_rect(x, y, s, s, AMARILLO);
                }
            }
        }
    }
    t
}

pub fn textura_braille() -> Textura {
    let mut t = placa(64, 192, false);
    for (k, puntos) in BRAILLE.iter().enumerate() {
        for &p in puntos.iter() {
            let (col, fila) = ((p - 1) / 3, (p - 1) % 3);
            let x = 24 + fila as i32 * 6;
            let y = BASE_TEXTO - (k as i32 * 16 + col as i32 * 6 + 3);
            t.fill_rect(x, y, 3, 3, AMARILLO);
        }
    }
    t
}
#[cfg(test)]
mod tests {
    use super::*;

    fn entorno() -> (Colisiones, Vec<Obstaculo>) {
        let contorno = crate::escena::contorno_nave();
        let colisiones = Colisiones::new(&contorno, crate::escena::WALL_THICKNESS)
            .con_cajas(crate::cabina::obstaculos_cabina(&contorno, crate::escena::WALL_THICKNESS))
            .con_cajas(crate::laboratorio::obstaculos());
        (colisiones, crate::escena::obstaculos())
    }

    #[test]
    fn completa_dos_bucles_sin_cruzar_muebles() {
        let (colisiones, obstaculos) = entorno();
        for dt in [1.0 / 60.0, 0.25] {
            let mut t = Tars::reposo();
            let mut visitas = Vec::new();
            let mut tiempo = 0.0;
            while tiempo < 220.0 {
                let estaba_esperando = t.etapa == Etapa::Esperar;
                t.actualizar(dt, &colisiones, &obstaculos, None);
                assert!(!colisiones.bloqueado(t.origen.x, t.origen.z, RADIO, &obstaculos),
                    "Obstaculo en ({}, {})", t.origen.x, t.origen.z);
                assert!(t.columnas.iter().all(|d| d.y >= 0.0));
                if !estaba_esperando && t.etapa == Etapa::Esperar { visitas.push(t.destino); }
                tiempo += dt;
            }
            assert!(visitas.len() >= 8, "TARS no termino dos vueltas: {:?}", visitas);
            assert_eq!(&visitas[..8], &[2, 4, 6, 8, 2, 4, 6, 8]);
        }
    }

    #[test]
    fn espera_al_jugador_y_reanuda() {
        let (colisiones, obstaculos) = entorno();
        let mut t = Tars::reposo();
        // Tramo horizontal libre que pasa frente al arco.
        t.origen = Vector3::new(2.0, 0.0, -4.65);
        t.angulo = ANGULO;
        t.destino = 1;
        t.etapa = Etapa::Caminar;
        let jugador = Vector3::new(0.0, 1.65, -4.65);
        for _ in 0..1200 { t.actualizar(1.0 / 60.0, &colisiones, &obstaculos, Some(jugador)); }
        assert!(t.origen.x >= RADIO + crate::escena::PLAYER_RADIUS);
        let detenido = t.origen.x;
        for _ in 0..120 { t.actualizar(1.0 / 60.0, &colisiones, &obstaculos, Some(jugador)); }
        assert!((t.origen.x - detenido).abs() < 0.0001);
        for _ in 0..180 { t.actualizar(1.0 / 60.0, &colisiones, &obstaculos, None); }
        assert!(t.origen.x < 0.0);
    }
}
