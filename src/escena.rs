// Escena: dimensiones y construcción de la habitación (la mesa vive en mesa.rs).
//
// Convención de ejes (unidades de escena, Y hacia arriba):
//   +X = derecha (zona de TARS)      -X = izquierda (paneles, depósitos)
//   -Z = frente (cabina, ventanal)   +Z = atrás (compuerta de Adrian)
//   Y = 0 es la superficie del suelo.
// Todo se define respecto al origen (0, 0, 0) = centro del suelo.
//
// Cada bloque se coloca por centro y tamaño una sola vez, al construir
// la escena. Las dimensiones se ajustan en las constantes de abajo.

use raylib::prelude::*;

use crate::cubo::Cubo;
use crate::material::{METAL_CLARO, METAL_OSCURO};
use crate::mesa::{construir_mesa, obstaculos_mesa as mesa_obstaculos};

pub use crate::mesa::planetario_origen;

pub const ROOM_HALF_X: f32 = 10.0;
pub const ROOM_HALF_Z: f32 = 20.0; // extensión frontal para la cabina
pub const ROOM_FRONT_Z: f32 = -8.0;
pub const ROOM_BACK_Z: f32 = 15.0;
pub const ROOM_HEIGHT: f32 = 7.0;
pub const WALL_THICKNESS: f32 = 0.3;
pub const FLOOR_THICKNESS: f32 = 0.3;

// Esquinas recortadas: el segmento diagonal conecta (±4.2, -8)
// con (±6, -6.2), y (±6, 3.2) con (±4.2, 5).
const FRONT_INSET_X: f32 = 4.2;
const FRONT_DIAGONAL_Z: f32 = -6.2;
const BACK_DIAGONAL_Z: f32 = 3.2;
const BACK_INSET_X: f32 = 4.2;

pub const EYE_HEIGHT: f32 = 1.65;
pub const PLAYER_RADIUS: f32 = 0.35;
pub const PLAYER_START_X: f32 = 0.0;
pub const PLAYER_START_Z: f32 = 3.2;
pub const TECHO_CERRADO: bool = false;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Pared {Frente, Atras, Izquierda, Derecha,}

// Obsataculo, para detectar colisiones
pub struct Obstaculo {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

pub fn obstaculos() -> Vec<Obstaculo> {
    mesa_obstaculos()
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
        .con_material(material, Some(1.2)),
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
                         h * 0.5, h, METAL_CLARO);
        bloque_en_pared(cubos, a, b, length - left_width * 0.5,
                         left_width + 0.08, h * 0.5, h, METAL_CLARO);
    }

    // Antepecho y viga superior.
    bloque_en_pared(cubos, a, b, center, opening_width,
                     sill_height * 0.5, sill_height, METAL_CLARO);
    bloque_en_pared(cubos, a, b, center, opening_width,
                     (window_top + h) * 0.5, h - window_top, METAL_CLARO);

    // Pilares de los lados del vano; el espacio entre ellos queda abierto.
    let post_center = opening_width * 0.5 - post * 0.5;
    for along in [center - post_center, center + post_center] {
        bloque_en_pared(cubos, a, b, along, post,
                         (sill_height + window_top) * 0.5,
                         window_top - sill_height, METAL_CLARO);
    }
}

fn pared_completa(cubos: &mut Vec<Cubo>, a: Vector3, b: Vector3) {
    let length = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt();
    // Solape pequeño en los extremos para evitar juntas abiertas.
    bloque_en_pared(
        cubos, a, b, length * 0.5, length + WALL_THICKNESS * 0.5,
        ROOM_HEIGHT * 0.5, ROOM_HEIGHT, METAL_CLARO,
    );
}

pub fn crear_habitacion() -> Vec<Cubo> {
    let mut cubos = Vec::new();
    let hx = ROOM_HALF_X;
    let h = ROOM_HEIGHT;
    let t = WALL_THICKNESS;
    let floor = FLOOR_THICKNESS;

    // Piso plano y techo plano.
    cubos.push(Cubo::new(
        Vector3::new(0.0, -floor * 0.5, (ROOM_FRONT_Z + ROOM_BACK_Z) * 0.5),
        Vector3::new(2.0 * (hx + t), floor, ROOM_BACK_Z - ROOM_FRONT_Z + 2.0 * t),
        Color::new(190, 185, 170, 255), 0.35, 0.9,
    ));

    // Tres paredes delanteras con vanos para la ventana central y las dos laterales.
    pared_con_ventana(
        &mut cubos,
        Vector3::new(-FRONT_INSET_X, 0.0, ROOM_FRONT_Z),
        Vector3::new(FRONT_INSET_X, 0.0, ROOM_FRONT_Z),
        4.6, 0.95, 4.25,
    );
    pared_con_ventana(
        &mut cubos,
        Vector3::new(-FRONT_INSET_X, 0.0, ROOM_FRONT_Z),
        Vector3::new(-hx, 0.0, FRONT_DIAGONAL_Z),
        1.55, 0.90, 4.15,
    );
    pared_con_ventana(
        &mut cubos,
        Vector3::new(hx, 0.0, FRONT_DIAGONAL_Z),
        Vector3::new(FRONT_INSET_X, 0.0, ROOM_FRONT_Z),
        1.55, 0.90, 4.15,
    );

    // Costados largos, diagonales traseras y pared posterior.
    pared_completa(&mut cubos,
        Vector3::new(-hx, 0.0, FRONT_DIAGONAL_Z),
        Vector3::new(-hx, 0.0, BACK_DIAGONAL_Z));
    pared_completa(&mut cubos,
        Vector3::new(hx, 0.0, FRONT_DIAGONAL_Z),
        Vector3::new(hx, 0.0, BACK_DIAGONAL_Z));
    pared_completa(&mut cubos,
        Vector3::new(-hx, 0.0, BACK_DIAGONAL_Z),
        Vector3::new(-BACK_INSET_X, 0.0, ROOM_BACK_Z));
    pared_completa(&mut cubos,
        Vector3::new(hx, 0.0, BACK_DIAGONAL_Z),
        Vector3::new(BACK_INSET_X, 0.0, ROOM_BACK_Z));
    pared_completa(&mut cubos,
        Vector3::new(-BACK_INSET_X, 0.0, ROOM_BACK_Z),
        Vector3::new(BACK_INSET_X, 0.0, ROOM_BACK_Z));

    if TECHO_CERRADO {
        cubos.push(
            Cubo::new(
                Vector3::new(0.0, h + t * 0.5, (ROOM_FRONT_Z + ROOM_BACK_Z) * 0.5),
                Vector3::new(2.0 * (hx + t), t, ROOM_BACK_Z - ROOM_FRONT_Z + 2.0 * t),
                Color::WHITE, 0.0, 0.0,
            )
            .con_material(METAL_OSCURO, Some(1.2)),
        );
    }

    // Se conserva la mesa, el planetario y sus materiales existentes.
    construir_mesa(&mut cubos);
    cubos
}