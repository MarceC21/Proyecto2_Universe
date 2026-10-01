// Escena: dimensiones y construcción de la habitación (la mesa vive en mesa.rs).
//
// Convención de ejes (unidades de escena, Y hacia arriba):
//   +X = derecha (zona de TARS)      -X = izquierda (paneles, depósitos)
//   -Z = frente (cabina, ventanal)   +Z = atrás (compuerta de Adrian)
//   Y = 0 es la superficie del suelo.
// El origen (0, 0, 0) se mantiene bajo el centro de la mesa.
//
// Cada bloque se coloca por centro y tamaño una sola vez, al construir
// la escena. Las dimensiones se ajustan en las constantes de abajo.

use raylib::prelude::*;

use crate::cubo::Cubo;
use crate::material::{PARED_NAVE, PISO_NAVE, REFUERZO_NAVE, MONITOR_COMANDO, CONTROLES_COMANDO, INDICADOR};
use crate::mesa::{construir_mesa, obstaculos_mesa as mesa_obstaculos};

pub use crate::mesa::planetario_origen;

pub const ROOM_HALF_X: f32 = 10.0;
pub const ROOM_FRONT_Z: f32 = -15.0; // espacio delantero para la futura cabina
pub const ROOM_BACK_Z: f32 = 8.0;
pub const ROOM_HEIGHT: f32 = 7.0;
pub const WALL_THICKNESS: f32 = 0.3;
pub const FLOOR_THICKNESS: f32 = 0.3;

// Estructura desplazada 7 unidades hacia -Z respecto a la mesa.
// Se conservan los recortes y las dimensiones de los vanos existentes.
// Diagonales: (±4.2, -15) a (±10, -13.2),
// y (±10, -3.8) a (±4.2, 8).
const FRONT_INSET_X: f32 = 4.2;
const FRONT_DIAGONAL_Z: f32 = ROOM_FRONT_Z + 1.8;
const BACK_DIAGONAL_Z: f32 = ROOM_BACK_Z - 11.8;
const BACK_INSET_X: f32 = 4.2;

pub const EYE_HEIGHT: f32 = 1.65;
pub const PLAYER_RADIUS: f32 = 0.35;
pub const PLAYER_START_X: f32 = 0.0;
pub const PLAYER_START_Z: f32 = 3.2;
pub const TECHO_CERRADO: bool = false;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Pared {Frente, Atras, Izquierda, Derecha,}

// Compatibilidad con mesa.rs: el tipo pertenece al módulo de colisiones.
pub use crate::colisiones::Obstaculo;

// Contorno convexo en orden antihorario en el plano XZ.
// Paredes y colisiones usan exactamente estos mismos ocho vértices.
pub fn contorno_nave() -> [(f32, f32); 8] {
    [
        (-FRONT_INSET_X, ROOM_FRONT_Z),
        (FRONT_INSET_X, ROOM_FRONT_Z),
        (ROOM_HALF_X, FRONT_DIAGONAL_Z),
        (ROOM_HALF_X, BACK_DIAGONAL_Z),
        (BACK_INSET_X, ROOM_BACK_Z),
        (-BACK_INSET_X, ROOM_BACK_Z),
        (-ROOM_HALF_X, BACK_DIAGONAL_Z),
        (-ROOM_HALF_X, FRONT_DIAGONAL_Z),
    ]
}

// Cuatro refuerzos bajos en cantidad, con la misma huella para render y colisión.
fn pilares_laterales() -> Vec<(Vector3, Vector3)> {
    let mut pilares = Vec::with_capacity(4);
    for lado in [-1.0f32, 1.0] {
        for fraccion in [0.38f32, 0.76] {
            let z = FRONT_DIAGONAL_Z + (BACK_DIAGONAL_Z - FRONT_DIAGONAL_Z) * fraccion;
            let x = lado * (ROOM_HALF_X - WALL_THICKNESS * 0.5 - 0.07);
            pilares.push((Vector3::new(x, 2.95, z), Vector3::new(0.18, 5.90, 0.32)));
        }
    }
    pilares
}

pub fn obstaculos() -> Vec<Obstaculo> {
    let mut lista = mesa_obstaculos();
    for (centro, tamano) in pilares_laterales() {
        lista.push(Obstaculo {
            min_x: centro.x - tamano.x * 0.5,
            max_x: centro.x + tamano.x * 0.5,
            min_z: centro.z - tamano.z * 0.5,
            max_z: centro.z + tamano.z * 0.5,
        });
    }
    // Huella de la consola derecha; el espacio futuro de TARS queda transitable.
    for (u, ancho, fondo, profundidad) in huellas_comando() {
        // Divisiones cortas para aproximar la huella diagonal sin bloquear
        // con una sola AABB el pasillo que hay frente a los paneles.
        let pasos = (ancho / 0.20).ceil() as usize;
        let paso = ancho / pasos as f32;
        for i in 0..pasos {
            lista.push(huella_comando(u - ancho * 0.5 + paso * (i as f32 + 0.5),
                paso, fondo, profundidad));
        }
    }
    lista
}

fn detalles_estructurales(cubos: &mut Vec<Cubo>, vertices: &[(f32, f32); 8]) {
    for (centro, tamano) in pilares_laterales() {
        cubos.push(Cubo::new(centro, tamano, Color::WHITE, 0.0, 0.0)
            .con_material(REFUERZO_NAVE, None));
    }
    // Dos conductos rectangulares por pared cerrada. Quedan por encima
    // del jugador y no cruzan las tres ventanas delanteras.
    for i in 2..7 {
        let (ax, az) = vertices[i];
        let (bx, bz) = vertices[i + 1];
        let (dx, dz) = (bx - ax, bz - az);
        let largo = (dx * dx + dz * dz).sqrt();
        let (nx, nz) = (-dz / largo, dx / largo);
        for (y, alto) in [(5.80f32, 0.20f32), (6.15, 0.10)] {
            let separacion = WALL_THICKNESS * 0.5 + 0.03;
            let centro = Vector3::new(
                (ax + bx) * 0.5 + nx * separacion, y,
                (az + bz) * 0.5 + nz * separacion,
            );
            cubos.push(Cubo::new(centro, Vector3::new(largo, alto, 0.10), Color::WHITE, 0.0, 0.0)
                .rotado_y((-dz).atan2(dx)).con_material(REFUERZO_NAVE, None));
        }
    }
}

// Crea un bloque cuyo eje X local sigue el segmento a-b.
fn bloque_en_pared(
    cubos: &mut Vec<Cubo>,
    a: Vector3,
    b: Vector3,
    along: f32,
    width: f32,
    y: f32,
    height: f32,
    material: usize,
) {
    let dx = b.x - a.x;
    let dz = b.z - a.z;
    let length = (dx * dx + dz * dz).sqrt();
    let ux = dx / length;
    let uz = dz / length;
    let center = Vector3::new(
        a.x + ux * along,
        y,
        a.z + uz * along,
    );
    let angle = (-dz).atan2(dx);

    cubos.push(
        Cubo::new(
            center,
            Vector3::new(width, height, WALL_THICKNESS),
            Color::WHITE,
            0.0,
            0.0,
        )
        .rotado_y(angle)
        .con_material(material, Some(2.4)),
    );
}

// Pared con hueco real: antepecho, dintel y pilares laterales.
// El ancho libre de ventana queda reducido por los dos pilares.
fn pared_con_ventana(
    cubos: &mut Vec<Cubo>,
    a: Vector3,
    b: Vector3,
    opening_width: f32,
    sill_height: f32,
    window_top: f32,
) {
    let length = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt();
    let h = ROOM_HEIGHT;
    let post = 0.18;
    let opening_width = opening_width.min(length - 0.36);
    let left_width = (length - opening_width) * 0.5;
    let center = length * 0.5;

    if left_width > 0.0 {
        bloque_en_pared(cubos, a, b, left_width * 0.5, left_width + 0.08,
                         h * 0.5, h, PARED_NAVE);
        bloque_en_pared(cubos, a, b, length - left_width * 0.5,
                         left_width + 0.08, h * 0.5, h, PARED_NAVE);
    }

    // Antepecho y viga superior.
    bloque_en_pared(cubos, a, b, center, opening_width,
                     sill_height * 0.5, sill_height, PARED_NAVE);
    bloque_en_pared(cubos, a, b, center, opening_width,
                     (window_top + h) * 0.5, h - window_top, PARED_NAVE);

    // Pilares de los lados del vano; el espacio entre ellos queda abierto.
    let post_center = opening_width * 0.5 - post * 0.5;
    for along in [center - post_center, center + post_center] {
        bloque_en_pared(cubos, a, b, along, post,
                         (sill_height + window_top) * 0.5,
                         window_top - sill_height, PARED_NAVE);
    }
}

fn pared_completa(cubos: &mut Vec<Cubo>, a: Vector3, b: Vector3) {
    let length = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt();
    // Solape pequeño en los extremos para evitar juntas abiertas.
    bloque_en_pared(
        cubos, a, b, length * 0.5, length + WALL_THICKNESS * 0.5,
        ROOM_HEIGHT * 0.5, ROOM_HEIGHT, PARED_NAVE,
    );
}

pub fn crear_habitacion() -> Vec<Cubo> {
    let mut cubos = Vec::new();
    let hx = ROOM_HALF_X;
    let h = ROOM_HEIGHT;
    let t = WALL_THICKNESS;
    let floor = FLOOR_THICKNESS;
    let vertices = contorno_nave();
    let punto = |i: usize| Vector3::new(vertices[i].0, 0.0, vertices[i].1);

    // Piso plano y techo plano.
    cubos.push(Cubo::new(
        Vector3::new(0.0, -floor * 0.5, (ROOM_FRONT_Z + ROOM_BACK_Z) * 0.5),
        Vector3::new(2.0 * (hx + t), floor, ROOM_BACK_Z - ROOM_FRONT_Z + 2.0 * t),
        Color::WHITE, 0.0, 0.0,
    ).con_material(PISO_NAVE, Some(1.2)));

    // Tres paredes delanteras con vanos para la ventana central y las dos laterales.
    pared_con_ventana(
        &mut cubos,
        punto(0),
        punto(1),
        4.6, 0.95, 4.25,
    );
    pared_con_ventana(
        &mut cubos,
        punto(0),
        punto(7),
        2.25, 0.90, 4.15,
    );
    pared_con_ventana(
        &mut cubos,
        punto(2),
        punto(1),
        2.25, 0.90, 4.15,
    );

    // Costados largos, diagonales traseras y pared posterior.
    pared_completa(&mut cubos,
        punto(7),
        punto(6));
    pared_completa(&mut cubos,
        punto(2),
        punto(3));
    pared_completa(&mut cubos,
        punto(6),
        punto(5));
    pared_completa(&mut cubos,
        punto(3),
        punto(4));
    pared_completa(&mut cubos,
        punto(5),
        punto(4));

    if TECHO_CERRADO {
        cubos.push(
            Cubo::new(
                Vector3::new(0.0, h + t * 0.5, (ROOM_FRONT_Z + ROOM_BACK_Z) * 0.5),
                Vector3::new(2.0 * (hx + t), t, ROOM_BACK_Z - ROOM_FRONT_Z + 2.0 * t),
                Color::WHITE, 0.0, 0.0,
            )
            .con_material(PARED_NAVE, Some(2.4)),
        );
    }

    detalles_estructurales(&mut cubos, &vertices);
    construir_comando_derecho(&mut cubos);

    // Se conserva la mesa, el planetario y sus materiales existentes.
    construir_mesa(&mut cubos);
    crate::cabina::construir_cabina(&mut cubos, &vertices, t);
    cubos
}

// Pared derecha de la MESA: diagonal entre vértices 3 y 4.
// Todas las medidas siguientes son locales: u recorre la pared y fondo
// se mide hacia el interior desde su cara visible.
fn marco_comando() -> (f32, f32, f32, f32, f32) {
    let vertices = contorno_nave();
    let (ax, az) = vertices[3];
    let (bx, bz) = vertices[4];
    let largo = ((bx - ax).powi(2) + (bz - az).powi(2)).sqrt();
    (ax, az, (bx - ax) / largo, (bz - az) / largo, largo)
}

fn punto_comando(u: f32, y: f32, fondo: f32) -> Vector3 {
    let (ax, az, tx, tz, _) = marco_comando();
    let d = WALL_THICKNESS * 0.5 + fondo;
    Vector3::new(ax + tx * u - tz * d, y, az + tz * u + tx * d)
}

fn huella_comando(u: f32, ancho: f32, fondo: f32, profundidad: f32) -> Obstaculo {
    let (_, _, tx, tz, _) = marco_comando();
    let p = punto_comando(u, 0.0, fondo);
    let hx = (tx.abs() * ancho + tz.abs() * profundidad) * 0.5;
    let hz = (tz.abs() * ancho + tx.abs() * profundidad) * 0.5;
    Obstaculo { min_x: p.x - hx, max_x: p.x + hx, min_z: p.z - hz, max_z: p.z + hz }
}

fn columnas_comando() -> [f32; 4] {
    let (_, _, _, _, largo) = marco_comando();
    [0.30, largo / 3.0, largo * 2.0 / 3.0, largo - 0.30]
}

fn huellas_comando() -> Vec<(f32, f32, f32, f32)> {
    let columnas = columnas_comando();
    let mut huellas = Vec::with_capacity(6);
    for u in columnas { huellas.push((u, 0.34, 0.31, 0.62)); }
    for i in 0..2 {
        huellas.push(((columnas[i] + columnas[i + 1]) * 0.5,
            columnas[i + 1] - columnas[i] - 0.48, 0.84, 1.56));
    }
    huellas
}

// Reserva de TARS en la pared lateral derecha original, junto a la cabina.
// Entre sus columnas existentes; queda transitable hasta agregar el personaje.
#[allow(dead_code)]
pub fn espacio_tars() -> Obstaculo {
    Obstaculo { min_x: 7.90, max_x: 9.65, min_z: -8.90, max_z: -6.90 }
}

fn construir_comando_derecho(cubos: &mut Vec<Cubo>) {
    let (_, _, tx, tz, _) = marco_comando();
    let angulo = (-tz).atan2(tx);
    let columnas = columnas_comando();
    let mut bloque = |u: f32, y: f32, fondo: f32, ancho: f32,
                      alto: f32, profundidad: f32, material: usize| {
        cubos.push(Cubo::new(punto_comando(u, y, fondo),
            Vector3::new(ancho, alto, profundidad), Color::WHITE, 0.0, 0.0)
            .rotado_y(angulo).con_material(material, None));
    };
    // La columna entre las dos mesas termina debajo de la pantalla:
    // ninguna columna atraviesa la imagen panorámica.
    for (i, u) in columnas.iter().copied().enumerate() {
        let alto = if i == 1 { 1.65 } else { 5.90 };
        bloque(u, alto * 0.5, 0.31, 0.34, alto, 0.62, REFUERZO_NAVE);
        bloque(u, alto * 0.5, 0.635, 0.12, alto - 0.50, 0.025, INDICADOR);
    }
    // Una sola superficie de pantalla, continua sobre las dos mesas.
    let centro_pantalla = (columnas[0] + columnas[2]) * 0.5;
    let ancho_pantalla = columnas[2] - columnas[0] - 0.48;
    bloque(centro_pantalla, 3.70, 0.13, ancho_pantalla, 3.50, 0.22, REFUERZO_NAVE);
    bloque(centro_pantalla, 3.70, 0.255, ancho_pantalla - 0.20, 3.26, 0.025, MONITOR_COMANDO);
    bloque(centro_pantalla, 5.68, 0.15, ancho_pantalla, 0.30, 0.30, REFUERZO_NAVE);
    // El antiguo módulo reservado ya no tiene monitor; TARS está en otra pared.
    bloque((columnas[2] + columnas[3]) * 0.5, 5.68, 0.15,
        columnas[3] - columnas[2] - 0.48, 0.30, 0.30, REFUERZO_NAVE);
    for i in 0..2 {
        let u = (columnas[i] + columnas[i + 1]) * 0.5;
        let ancho = columnas[i + 1] - columnas[i] - 0.48;
        bloque(u, 0.55, 0.50, ancho - 0.12, 1.10, 0.90, REFUERZO_NAVE);
        bloque(u, 1.16, 0.84, ancho, 0.20, 1.56, REFUERZO_NAVE);
        bloque(u, 1.28, 0.88, ancho - 0.20, 0.04, 1.24, CONTROLES_COMANDO);
        for offset in [-0.95f32, 0.95] {
            bloque(u + offset, 1.39, 1.02, 0.24, 0.18, 0.24, REFUERZO_NAVE);
            bloque(u + offset, 1.485, 1.02, 0.035, 0.015, 0.11, INDICADOR);
        }
        bloque(u, 1.35, 1.00, 0.09, 0.10, 0.42, REFUERZO_NAVE);
        bloque(u, 1.46, 1.00, 0.28, 0.12, 0.13, REFUERZO_NAVE);
        bloque(u, 1.73, 0.25, ancho - 0.24, 0.035, 0.035, INDICADOR);
    }
}
