// La mesa de control: tablero metálico con marco, pantalla azul y un
// módulo de controles, inspirada en una consola futurista. Todo son
// cubos escalados (las curvas de la referencia se simplifican).
//
// Vista lateral (no a escala):
//
//        marco (sube FRAME_RISE)     pantalla (SCREEN_RISE)
//   ┌───┐                                 ┌───┐
//   │███████████████████████████████████████████│ <- tablero (BODY)
//        ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀            <- faldón oscuro
//                    ▄▄▄▄▄▄▄▄▄                       <- collar
//                      █████                         <- pedestal
//               ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄                  <- base
//
// Origen local: (MESA_CENTRO_X, 0, MESA_CENTRO_Z) en el suelo. Todas las
// piezas se definen respecto a él; cambiar MESA_CENTRO_* mueve la mesa
// completa, incluido el planetario (ver planetario_origen).

use raylib::prelude::*;

use crate::cubo::Cubo;
use crate::escena::Obstaculo;
use crate::material::{INDICADOR, METAL_CLARO, METAL_OSCURO, PANTALLA, POLIMERO};

// --- Posición ---
pub const MESA_CENTRO_X: f32 = 0.0;
pub const MESA_CENTRO_Z: f32 = 0.0;

// --- Tamaño del tablero ---
pub const TABLE_HALF_X: f32 = 2.6; // mitad del largo (X)
pub const TABLE_HALF_Z: f32 = 1.95; // mitad del fondo (Z)
pub const TABLE_TOP_Y: f32 = 0.95; // altura de la cara superior del tablero
pub const BODY_THICKNESS: f32 = 0.20; // grosor visible del tablero
pub const SKIRT_THICKNESS: f32 = 0.08; // faldón oscuro bajo el tablero
pub const SKIRT_INSET: f32 = 0.12; // cuánto más chico que el tablero

// --- Soporte ---
pub const BASE_HALF_X: f32 = 1.7;
pub const BASE_HALF_Z: f32 = 1.3;
pub const BASE_THICKNESS: f32 = 0.12;
pub const PEDESTAL_HALF_X: f32 = 0.55;
pub const PEDESTAL_HALF_Z: f32 = 0.45;
pub const COLLAR_HALF_X: f32 = 1.0;
pub const COLLAR_HALF_Z: f32 = 0.8;
pub const COLLAR_THICKNESS: f32 = 0.12;

// --- Marco y pantalla ---
pub const FRAME_WIDTH: f32 = 0.40;
pub const FRAME_RISE: f32 = 0.07; // cuánto sobresale el marco sobre el tablero
pub const SCREEN_RISE: f32 = 0.02; // cuánto sobresale la pantalla
pub const SCREEN_HALF_X: f32 = TABLE_HALF_X - FRAME_WIDTH;
pub const SCREEN_HALF_Z: f32 = TABLE_HALF_Z - FRAME_WIDTH;

// --- Módulo de controles (costado -X) ---
pub const MODULE_SIZE_X: f32 = 0.60; // cuánto sobresale del tablero
pub const MODULE_SIZE_Z: f32 = 1.10;
pub const MODULE_CENTER_Z: f32 = 0.50; // posición a lo largo del costado

// Altura del Sol del planetario sobre la cara superior del tablero.
pub const PLANETARIO_ALTURA: f32 = 0.40;

// Unidades de mundo por repetición de la textura de metal (2x2 placas).
const METAL_TILE: f32 = 1.2;
const PANTALLA_TEX_WIDTH: usize = 1024;

// Origen común del planetario: el Sol se coloca aquí y las órbitas se
// calculan respecto a este punto (Planet::update lo recibe cada frame).
pub fn planetario_origen() -> Vector3 {
    Vector3::new(MESA_CENTRO_X, TABLE_TOP_Y + PLANETARIO_ALTURA, MESA_CENTRO_Z)
}

// Tamaños de textura que deben usarse al crear los materiales: misma
// proporción que la superficie a la que se pegan, para que la
// cuadrícula tenga celdas cuadradas y los anillos no salgan ovalados.
pub fn pantalla_texture_size() -> (usize, usize) {
    let h = (PANTALLA_TEX_WIDTH as f32 * SCREEN_HALF_Z / SCREEN_HALF_X) as usize;
    (PANTALLA_TEX_WIDTH, h)
}

pub fn consola_texture_size() -> (usize, usize) {
    // La placa mide 0.5 (X) x 0.9 (Z): 256 x 460 texels.
    (256, 460)
}

// Zona del suelo que la mesa (con su módulo) impide pisar.
pub fn obstaculos_mesa() -> Vec<Obstaculo> {
    vec![Obstaculo {
        min_x: MESA_CENTRO_X - TABLE_HALF_X - MODULE_SIZE_X,
        max_x: MESA_CENTRO_X + TABLE_HALF_X,
        min_z: MESA_CENTRO_Z - TABLE_HALF_Z,
        max_z: MESA_CENTRO_Z + TABLE_HALF_Z,
    }]
}

pub fn construir_mesa(cubos: &mut Vec<Cubo>) {
    // Ayuda: coloca un bloque por centro y tamaño LOCALES (respecto al
    // origen de la mesa) y le asigna material.
    let mut poner = |c: (f32, f32, f32), s: (f32, f32, f32), material: usize, tile: Option<f32>| {
        cubos.push(
            Cubo::new(
                Vector3::new(MESA_CENTRO_X + c.0, c.1, MESA_CENTRO_Z + c.2),
                Vector3::new(s.0, s.1, s.2),
                Color::WHITE,
                0.0,
                0.0,
            )
            .con_material(material, tile),
        );
    };

    let top = TABLE_TOP_Y;
    let (hx, hz) = (TABLE_HALF_X, TABLE_HALF_Z);
    let metal = Some(METAL_TILE);

    // ---------------- Soporte ----------------
    // Base ancha y baja (estabilidad).
    poner((0.0, BASE_THICKNESS * 0.5, 0.0),
          (2.0 * BASE_HALF_X, BASE_THICKNESS, 2.0 * BASE_HALF_Z), METAL_OSCURO, metal);

    // Pedestal: llena el hueco entre la base y el collar.
    let collar_top = top - BODY_THICKNESS - SKIRT_THICKNESS;
    let collar_bottom = collar_top - COLLAR_THICKNESS;
    let pedestal_h = collar_bottom - BASE_THICKNESS;
    poner((0.0, BASE_THICKNESS + pedestal_h * 0.5, 0.0),
          (2.0 * PEDESTAL_HALF_X, pedestal_h, 2.0 * PEDESTAL_HALF_Z), METAL_OSCURO, metal);

    // Collar bajo el tablero (repartir el peso).
    poner((0.0, collar_bottom + COLLAR_THICKNESS * 0.5, 0.0),
          (2.0 * COLLAR_HALF_X, COLLAR_THICKNESS, 2.0 * COLLAR_HALF_Z), METAL_OSCURO, metal);

    // Faldón oscuro: da profundidad al borde del tablero.
    poner((0.0, top - BODY_THICKNESS - SKIRT_THICKNESS * 0.5, 0.0),
          (2.0 * (hx - SKIRT_INSET), SKIRT_THICKNESS, 2.0 * (hz - SKIRT_INSET)), METAL_OSCURO, metal);

    // ---------------- Tablero ----------------
    poner((0.0, top - BODY_THICKNESS * 0.5, 0.0),
          (2.0 * hx, BODY_THICKNESS, 2.0 * hz), METAL_CLARO, metal);

    // ---------------- Marco elevado ----------------
    let fw = FRAME_WIDTH;
    let fy = top + FRAME_RISE * 0.5;
    // Frente y fondo (todo el largo, incluidas las esquinas).
    poner((0.0, fy, hz - fw * 0.5), (2.0 * hx, FRAME_RISE, fw), METAL_CLARO, metal);
    poner((0.0, fy, -(hz - fw * 0.5)), (2.0 * hx, FRAME_RISE, fw), METAL_CLARO, metal);
    // Laterales (entre frente y fondo).
    poner((-(hx - fw * 0.5), fy, 0.0), (fw, FRAME_RISE, 2.0 * (hz - fw)), METAL_CLARO, metal);
    poner((hx - fw * 0.5, fy, 0.0), (fw, FRAME_RISE, 2.0 * (hz - fw)), METAL_CLARO, metal);

    // Ranuras oscuras sobre el marco (sobresalen 0.004 para no coincidir
    // con la cara del marco).
    let sy = top + FRAME_RISE;
    poner((0.0, sy, hz - fw * 0.5), (2.0 * hx * 0.7, 0.008, 0.07), METAL_OSCURO, metal);
    poner((0.0, sy, -(hz - fw * 0.5)), (2.0 * hx * 0.7, 0.008, 0.07), METAL_OSCURO, metal);
    poner((-(hx - fw * 0.5), sy, 0.0), (0.07, 0.008, 2.0 * (hz - fw) * 0.6), METAL_OSCURO, metal);
    poner((hx - fw * 0.5, sy, 0.0), (0.07, 0.008, 2.0 * (hz - fw) * 0.6), METAL_OSCURO, metal);

    // ---------------- Pantalla ----------------
    // Ligeramente hundida en el tablero (0.02) y sobresaliendo SCREEN_RISE;
    // su textura cubre la cara superior completa una sola vez.
    let screen_h = SCREEN_RISE + 0.02;
    poner((0.0, top + (SCREEN_RISE - 0.02) * 0.5, 0.0),
          (2.0 * SCREEN_HALF_X, screen_h, 2.0 * SCREEN_HALF_Z), PANTALLA, None);

    // ---------------- Detalles del costado ----------------
    let panel_h = BODY_THICKNESS * 0.55;
    let strip_h = BODY_THICKNESS * 0.85;
    let by = top - BODY_THICKNESS * 0.5;

    // Frente (+Z): dos paneles y tres juntas verticales.
    for &x in &[-0.65f32, 0.65] {
        poner((x, by, hz + 0.02), (1.2, panel_h, 0.05), METAL_OSCURO, metal);
    }
    for &x in &[-1.3f32, 0.0, 1.3] {
        poner((x, by, hz + 0.005), (0.05, strip_h, 0.02), METAL_OSCURO, metal);
    }
    // Costados (±X): un panel largo y dos juntas.
    for &sgn in &[-1.0f32, 1.0] {
        poner((sgn * (hx + 0.02), by, 0.0), (0.05, panel_h, 1.6), METAL_OSCURO, metal);
        for &z in &[-0.9f32, 0.9] {
            poner((sgn * (hx + 0.005), by, z), (0.02, strip_h, 0.05), METAL_OSCURO, metal);
        }
    }

    // Línea de luz cian bajo el borde frontal (emisión moderada).
    poner((0.0, top - BODY_THICKNESS - SKIRT_THICKNESS * 0.5, hz - SKIRT_INSET + 0.005),
          (3.0, 0.02, 0.02), INDICADOR, None);

    // ---------------- Módulo de controles (-X) ----------------
    // Cuerpo metálico oscuro pegado al costado (0.05 dentro del tablero).
    let mx = -(hx + MODULE_SIZE_X * 0.5 - 0.05);
    let mtop = top + 0.06;
    let mh = 0.30;
    poner((mx, mtop - mh * 0.5, MODULE_CENTER_Z), (MODULE_SIZE_X, mh, MODULE_SIZE_Z), METAL_OSCURO, metal);

    // Placa de polímero con botones: textura completa, una sola vez.
    let plate_x = mx - 0.02;
    poner((plate_x, mtop + 0.01, MODULE_CENTER_Z), (0.5, 0.02, 0.9), POLIMERO, None);

    // Dial (metal claro) y tres indicadores emisivos sobre la placa.
    poner((plate_x, mtop + 0.02 + 0.03, MODULE_CENTER_Z - 0.28), (0.20, 0.06, 0.20), METAL_CLARO, metal);
    for i in 0..3 {
        let z = MODULE_CENTER_Z + 0.18 + i as f32 * 0.1;
        poner((plate_x + 0.12, mtop + 0.02 + 0.01, z), (0.06, 0.02, 0.06), INDICADOR, None);
    }
}