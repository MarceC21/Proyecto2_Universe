// Mesa de cabina bajo las tres ventanas, construida una sola vez
// Reutiliza los materiales y las texturas de mesa.rs, sin modificarlos
use raylib::prelude::*;

use crate::colisiones::CajaOrientada;
use crate::cubo::Cubo;
use crate::material::{INDICADOR, METAL_CLARO, METAL_OSCURO, RADAR_CABINA, BOTONERA_CABINA, SISTEMAS_CABINA, TAPIZADO_CABINA};
use crate::mesa::{BODY_THICKNESS, FRAME_RISE, TABLE_TOP_Y};

const FONDO: f32 = 1.50;
const METAL_TILE: f32 = 1.2;
const SILLON_X: [f32; 2] = [-1.50, 1.50];
const SILLON_Z: f32 = 2.90;
const SILLON_ANCHO: f32 = 1.30;
const SILLON_FONDO: f32 = 1.40;

struct Tramo {
    x: f32,
    z: f32,
    largo: f32,
    angulo: f32,
}

impl Tramo {
    // X local recorre la pared; Z local apunta hacia el interior de la nave.
    fn punto(&self, x: f32, y: f32, z: f32) -> Vector3 {
        let (c, s) = (self.angulo.cos(), self.angulo.sin());
        Vector3::new(self.x + c * x + s * z, y, self.z - s * x + c * z)
    }

    fn bloque(
        &self, cubos: &mut Vec<Cubo>, centro: (f32, f32, f32),
        tamano: (f32, f32, f32), material: usize, tile: Option<f32>,
    ) {
        cubos.push(Cubo::new(
            self.punto(centro.0, centro.1, centro.2),
            Vector3::new(tamano.0, tamano.1, tamano.2),
            Color::WHITE, 0.0, 0.0,
        ).rotado_y(self.angulo).con_material(material, tile));
    }
}

fn tramos(contorno: &[(f32, f32); 8], pared: f32) -> Vec<Tramo> {
    // Recorrido continuo: diagonal izquierda, centro, diagonal derecha.
    [(7usize, 0usize), (0, 1), (1, 2)].iter().enumerate().map(|(i, &(a, b))| {
        let (ax, az) = contorno[a];
        let (bx, bz) = contorno[b];
        let (dx, dz) = (bx - ax, bz - az);
        let longitud = (dx * dx + dz * dz).sqrt();
        let (ux, uz) = (dx / longitud, dz / longitud);
        // Solo se recortan los dos extremos junto a los costados largos.
        // En las uniones delanteras los tableros se solapan ligeramente.
        let inicio = if i == 0 { 0.35 } else { 0.0 };
        let final_ = if i == 2 { 0.35 } else { 0.0 };
        let medio = (inicio + longitud - final_) * 0.5;
        let separacion = pared * 0.5 + 0.01;
        Tramo {
            x: ax + ux * medio - uz * separacion,
            z: az + uz * medio + ux * separacion,
            largo: longitud - inicio - final_,
            angulo: (-uz).atan2(ux),
        }
    }).collect()
}

pub fn obstaculos_cabina(contorno: &[(f32, f32); 8], pared: f32) -> Vec<CajaOrientada> {
    let tramos = tramos(contorno, pared);
    let mut cajas: Vec<CajaOrientada> = tramos.iter().map(|t| {
        let centro = t.punto(0.0, 0.0, FONDO * 0.5);
        CajaOrientada {
            centro_x: centro.x, centro_z: centro.z,
            mitad_x: t.largo * 0.5, mitad_z: FONDO * 0.5,
            angulo_y: t.angulo,
        }
    }).collect();
    // La misma referencia local coloca geometría y colisiones de los sillones
    let central = &tramos[1];
    for x in SILLON_X {
        let centro = central.punto(x, 0.0, SILLON_Z);
        cajas.push(CajaOrientada {
            centro_x: centro.x, centro_z: centro.z,
            mitad_x: SILLON_ANCHO * 0.5, mitad_z: SILLON_FONDO * 0.5,
            angulo_y: central.angulo,
        });
    }
    cajas
}

pub fn construir_cabina(cubos: &mut Vec<Cubo>, contorno: &[(f32, f32); 8], pared: f32) {
    for (i, tramo) in tramos(contorno, pared).iter().enumerate() {
        let t = tramo;
        let top = TABLE_TOP_Y;
        let metal = Some(METAL_TILE);
        // Misma construcción por capas que la mesa central
        t.bloque(cubos, (0.0, top - BODY_THICKNESS * 0.5, FONDO * 0.5),
            (t.largo, BODY_THICKNESS, FONDO), METAL_CLARO, metal);
        t.bloque(cubos, (0.0, top - BODY_THICKNESS - 0.04, FONDO * 0.5),
            (t.largo, 0.08, FONDO - 0.10), METAL_OSCURO, metal);
        // Panel posterior pegado al antepecho y dos apoyos por tramo
        let soporte_h = top - BODY_THICKNESS - 0.08;
        t.bloque(cubos, (0.0, soporte_h * 0.5, 0.12),
            (t.largo, soporte_h, 0.24), METAL_OSCURO, metal);
        for lado in [-1.0f32, 1.0] {
            let x = lado * t.largo * 0.32;
            t.bloque(cubos, (x, soporte_h * 0.5, FONDO * 0.56),
                (0.28, soporte_h, FONDO * 0.72), METAL_OSCURO, metal);
        }
        // Bordes largos; las uniones entre tramos quedan abiertas sobre la mesa.
        for z in [0.05, FONDO - 0.05] {
            t.bloque(cubos, (0.0, top + FRAME_RISE * 0.5, z),
                (t.largo, FRAME_RISE, 0.10), METAL_CLARO, metal);
        }
        // Cierre únicamente en los extremos exteriores del conjunto.
        if i != 1 {
            let lado = if i == 0 { -1.0 } else { 1.0 };
            t.bloque(cubos, (lado * (t.largo * 0.5 - 0.05), top + FRAME_RISE * 0.5, FONDO * 0.5),
                (0.10, FRAME_RISE, FONDO), METAL_CLARO, metal);
        }
        // Indicador bajo el borde interior, igual al de la mesa central.
        t.bloque(cubos, (0.0, top - BODY_THICKNESS - 0.04, FONDO - 0.02),
            (t.largo * 0.72, 0.025, 0.025), INDICADOR, None);

        if i == 1 {
            mando_central(t, cubos);
            for x in SILLON_X { construir_sillon(t, cubos, x); }
        } else {
            mando_lateral(t, cubos, i == 0);
        }
    }
}

// Monitor vertical: su cara +Z mira al operador. UV X/Y mantiene el radar circular
fn monitor(t: &Tramo, cubos: &mut Vec<Cubo>, x: f32, ancho: f32, material: usize) {
    let alto = ancho / 1.6;
    let y = TABLE_TOP_Y + 0.23 + alto * 0.5;
    t.bloque(cubos, (x, TABLE_TOP_Y + 0.12, 0.29),
        (ancho * 0.45, 0.24, 0.30), METAL_OSCURO, Some(METAL_TILE));
    t.bloque(cubos, (x, y, 0.28),
        (ancho + 0.18, alto + 0.18, 0.18), METAL_CLARO, Some(METAL_TILE));
    t.bloque(cubos, (x, y, 0.378),
        (ancho + 0.06, alto + 0.06, 0.025), METAL_OSCURO, Some(METAL_TILE));
    t.bloque(cubos, (x, y, 0.398),
        (ancho, alto, 0.02), material, None);
}

fn panel(t: &Tramo, cubos: &mut Vec<Cubo>, x: f32, z: f32, ancho: f32, fondo: f32, material: usize) {
    t.bloque(cubos, (x, TABLE_TOP_Y + 0.035, z),
        (ancho + 0.10, 0.07, fondo + 0.10), METAL_OSCURO, Some(METAL_TILE));
    t.bloque(cubos, (x, TABLE_TOP_Y + 0.08, z),
        (ancho, 0.02, fondo), material, None);
}

fn palanca(t: &Tramo, cubos: &mut Vec<Cubo>, x: f32) {
    // Base, ranura, pivote, eje y empuñadura. Geometría estática por ahora
    let y = TABLE_TOP_Y;
    t.bloque(cubos, (x, y + 0.05, 1.08),
        (0.36, 0.10, 0.48), METAL_CLARO, Some(METAL_TILE));
    t.bloque(cubos, (x, y + 0.108, 1.08),
        (0.10, 0.018, 0.36), METAL_OSCURO, Some(METAL_TILE));
    t.bloque(cubos, (x, y + 0.16, 1.08),
        (0.18, 0.10, 0.18), METAL_OSCURO, Some(METAL_TILE));
    t.bloque(cubos, (x, y + 0.34, 1.08),
        (0.065, 0.32, 0.065), METAL_CLARO, Some(METAL_TILE));
    t.bloque(cubos, (x, y + 0.51, 1.08),
        (0.26, 0.14, 0.16), METAL_OSCURO, Some(METAL_TILE));
    t.bloque(cubos, (x, y + 0.585, 1.08),
        (0.075, 0.015, 0.07), INDICADOR, None);
}

fn mando_central(t: &Tramo, cubos: &mut Vec<Cubo>) {
    monitor(t, cubos, 0.0, 2.40, RADAR_CABINA);
    // Botoneras flanqueando la pantalla principal, como en la referencia
    for x in [-2.30f32, 2.30] {
        panel(t, cubos, x, 0.74, 1.20, 0.96, BOTONERA_CABINA);
        // Dial físico junto al panel, con indicador de posición
        let dx = x + x.signum() * 0.91;
        t.bloque(cubos, (dx, TABLE_TOP_Y + 0.10, 0.91),
            (0.24, 0.20, 0.24), METAL_CLARO, Some(METAL_TILE));
        t.bloque(cubos, (dx, TABLE_TOP_Y + 0.207, 0.84),
            (0.025, 0.015, 0.085), INDICADOR, None);
    }
    // Panel de estado entre las palancas, más cerca del operador
    panel(t, cubos, 0.0, 1.05, 0.90, 0.5625, SISTEMAS_CABINA);
    palanca(t, cubos, -0.86);
    palanca(t, cubos, 0.86);
}

fn mando_lateral(t: &Tramo, cubos: &mut Vec<Cubo>, izquierda: bool) {
    // Instrumentos auxiliares, con distribución espejada y pantalla pequeña
    let lado = if izquierda { -1.0 } else { 1.0 };
    monitor(t, cubos, lado * 0.80, 1.12, SISTEMAS_CABINA);
    panel(t, cubos, -lado * 0.78, 0.77, 1.10, 0.88, BOTONERA_CABINA);
    // Tres interruptores físicos al frente del monitor
    for i in 0..3 {
        let x = lado * 0.80 - 0.32 + i as f32 * 0.32;
        t.bloque(cubos, (x, TABLE_TOP_Y + 0.035, 1.04),
            (0.22, 0.07, 0.30), METAL_OSCURO, Some(METAL_TILE));
        t.bloque(cubos, (x, TABLE_TOP_Y + 0.12, 1.02),
            (0.06, 0.12, 0.09), METAL_CLARO, Some(METAL_TILE));
        t.bloque(cubos, (x, TABLE_TOP_Y + 0.075, 1.15),
            (0.10, 0.018, 0.035), INDICADOR, None);
    }
}


fn construir_sillon(t: &Tramo, cubos: &mut Vec<Cubo>, x: f32) {
    // Mira hacia -Z local, hacia la consola. Respaldo completamente vertical
    let metal = Some(METAL_TILE);
    let tela = Some(0.60); // tamaño del patrón en unidades de mundo, igual en cada cara
    let z = SILLON_Z;
    t.bloque(cubos, (x, 0.055, z),
        (1.10, 0.11, 1.15), METAL_OSCURO, metal);
    t.bloque(cubos, (x, 0.24, z),
        (0.34, 0.27, 0.38), METAL_CLARO, metal);
    t.bloque(cubos, (x, 0.405, z),
        (1.12, 0.13, 1.18), METAL_OSCURO, metal);
    // Cojín grueso y un panel superior más pequeño para marcar su volumen
    t.bloque(cubos, (x, 0.54, z - 0.035),
        (1.02, 0.16, 1.05), TAPIZADO_CABINA, tela);
    t.bloque(cubos, (x, 0.63, z - 0.075),
        (0.88, 0.06, 0.89), TAPIZADO_CABINA, tela);
    // Carcasa trasera; cojín en la cara que mira a las ventanas
    t.bloque(cubos, (x, 1.14, z + 0.54),
        (1.12, 1.24, 0.25), METAL_OSCURO, metal);
    t.bloque(cubos, (x, 1.17, z + 0.365),
        (0.98, 1.02, 0.18), TAPIZADO_CABINA, tela);
    t.bloque(cubos, (x, 0.85, z + 0.25),
        (0.84, 0.25, 0.12), TAPIZADO_CABINA, tela);
    // Apoyacabezas con soporte y almohadilla frontal
    t.bloque(cubos, (x, 1.78, z + 0.54),
        (0.40, 0.22, 0.13), METAL_CLARO, metal);
    t.bloque(cubos, (x, 1.89, z + 0.50),
        (0.74, 0.34, 0.22), METAL_OSCURO, metal);
    t.bloque(cubos, (x, 1.89, z + 0.355),
        (0.66, 0.27, 0.10), TAPIZADO_CABINA, tela);
    for lado in [-1.0f32, 1.0] {
        let ax = x + lado * 0.55;
        t.bloque(cubos, (ax, 0.665, z + 0.05),
            (0.10, 0.43, 0.13), METAL_CLARO, metal);
        t.bloque(cubos, (ax, 0.88, z - 0.07),
            (0.20, 0.10, 0.92), METAL_OSCURO, metal);
        t.bloque(cubos, (ax, 0.945, z - 0.07),
            (0.18, 0.05, 0.86), TAPIZADO_CABINA, tela);
    }
}
