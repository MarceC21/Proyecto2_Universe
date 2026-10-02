// Escena: dimensiones y construcción de la habitación

// Cada bloque se coloca por centro y tamaño una sola vez, al construir la escena. 
// Las dimensiones se ajustan en las constantes de abajo.

use raylib::prelude::*;

// Módulo privado de escena
#[path = "arco.rs"]
mod arco;
#[path = "exterior.rs"]
pub mod exterior;
pub const MOSTRAR_ARCO: bool = true;

use crate::cubo::Cubo;
use crate::material::{PARED_NAVE, PISO_NAVE, REFUERZO_NAVE, MONITOR_COMANDO, CONTROLES_COMANDO, INDICADOR, PANEL_ESCLUSA};
use crate::mesa::{construir_mesa, obstaculos_mesa as mesa_obstaculos};

pub use crate::mesa::planetario_origen;

pub const ROOM_HALF_X: f32 = 10.0;
pub const ROOM_FRONT_Z: f32 = -15.0;
pub const ROOM_BACK_Z: f32 = 8.0;
pub const ROOM_HEIGHT: f32 = 7.0;
pub const WALL_THICKNESS: f32 = 0.3;
pub const FLOOR_THICKNESS: f32 = 0.3;


const FRONT_INSET_X: f32 = 4.2;
const FRONT_DIAGONAL_Z: f32 = ROOM_FRONT_Z + 1.8;
const BACK_DIAGONAL_Z: f32 = ROOM_BACK_Z - 11.8;
const BACK_INSET_X: f32 = 4.2;

pub const EYE_HEIGHT: f32 = 1.65;
pub const PLAYER_RADIUS: f32 = 0.35;
pub const PLAYER_START_X: f32 = 0.0;
pub const PLAYER_START_Z: f32 = 3.2;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Pared {Frente, Atras, Izquierda, Derecha,}

// Compatibilidad con mesa.rs, el tipo pertenece al módulo de colisiones.
pub use crate::colisiones::Obstaculo;

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
    if MOSTRAR_ARCO { lista.extend(arco::obstaculos()); }
    for (centro, tamano) in pilares_laterales() {
        lista.push(Obstaculo {
            min_x: centro.x - tamano.x * 0.5,
            max_x: centro.x + tamano.x * 0.5,
            min_z: centro.z - tamano.z * 0.5,
            max_z: centro.z + tamano.z * 0.5,
        });
    }
    lista.extend(crate::pasillo::obstaculos());
    lista.extend(exterior::obstaculos());
    
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
    // Conductos del lado opuesto, divididos para seguir la pared diagonal.
    for i in 0..20 {
        let b = huella_comando(8.30 + (i as f32 + 0.5) * 0.20, 0.20, 0.27, 0.54);
        lista.push(Obstaculo { min_x: -b.max_x, max_x: -b.min_x,
            min_z: b.min_z, max_z: b.max_z });
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

    // Pilares de los lados del vano, el espacio entre ellos queda abierto.
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
    let t = WALL_THICKNESS;
    let floor = FLOOR_THICKNESS;
    let vertices = contorno_nave();
    let punto = |i: usize| Vector3::new(vertices[i].0, 0.0, vertices[i].1);

    // Piso plano
    cubos.push(Cubo::new(
        Vector3::new(0.0, -floor * 0.5, (ROOM_FRONT_Z + ROOM_BACK_Z) * 0.5),
        Vector3::new(2.0 * (hx + t), floor, ROOM_BACK_Z - ROOM_FRONT_Z + 2.0 * t),
        Color::WHITE, 0.0, 0.0,
    ).con_material(PISO_NAVE, Some(1.2)));

    // Las mismas medidas construyen los vanos y sus cristales. Se conservan
    // los antepechos, el contorno y las posiciones de toda la cabina.
    for ventana in exterior::ventanas() {
        pared_con_ventana(&mut cubos, ventana.a, ventana.b,
            ventana.ancho, ventana.abajo, ventana.arriba);
    }

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
    construir_puerta_trasera(&mut cubos);
    crate::pasillo::construir(&mut cubos);

    crate::techo::construir(&mut cubos);

    detalles_estructurales(&mut cubos, &vertices);
    construir_comando_derecho(&mut cubos);
    construir_servicios_traseros(&mut cubos);
    crate::laboratorio::construir(&mut cubos);
    if MOSTRAR_ARCO { arco::construir(&mut cubos); }

    // Se conserva la mesa, el planetario y sus materiales existentes.
    construir_mesa(&mut cubos);
    crate::cabina::construir_cabina(&mut cubos, &vertices, t);
    exterior::construir(&mut cubos);
    cubos
}

// Pared derecha de la MESA: diagonal entre vértices 3 y 4.
// Todas las medidas siguientes son locales: u recorre la pared y fondo se mide hacia el interior desde su cara visible.
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

pub fn control_esclusa() -> Vector3 {
    let columnas = columnas_comando();
    punto_comando((columnas[2] + columnas[3]) * 0.5, 2.18, 0.31)
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
    let mut huellas = Vec::with_capacity(7);
    for u in columnas { huellas.push((u, 0.34, 0.31, 0.62)); }
    for i in 0..2 {
        huellas.push(((columnas[i] + columnas[i + 1]) * 0.5,
            columnas[i + 1] - columnas[i] - 0.48, 0.84, 1.56));
    }
    huellas.push(((columnas[2] + columnas[3]) * 0.5, 1.65, 0.30, 0.60));
    huellas
}

// Reserva de TARS del lado de la mesa, delante del nuevo tabique derecho.
// No se registra como obstáculo hasta agregar el personaje.
#[allow(dead_code)]
pub fn espacio_tars() -> Obstaculo {
    arco::espacio_tars()
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

// El marco queda estático
fn construir_puerta_trasera(cubos: &mut Vec<Cubo>) {
    let z = ROOM_BACK_Z;
    let ancho = 3.80;
    let alto = 4.50;
    let lado = BACK_INSET_X - ancho * 0.5;
    let mut bloque = |x: f32, y: f32, z: f32, w: f32, h: f32, d: f32, m: usize| {
        cubos.push(Cubo::new(Vector3::new(x, y, z), Vector3::new(w,h,d),
            Color::WHITE, 0.0, 0.0).con_material(m,
                if m == PARED_NAVE { Some(2.4) } else { None }));
    };
    for signo in [-1.0f32, 1.0] {
        bloque(signo * (ancho * 0.5 + lado * 0.5), ROOM_HEIGHT * 0.5,
            z, lado, ROOM_HEIGHT, WALL_THICKNESS, PARED_NAVE);
        // Jambas y carriles gruesos con tira de estado interior.
        bloque(signo * 2.15, 2.35, z - 0.15, 0.50, 4.70, 0.66, REFUERZO_NAVE);
        bloque(signo * 1.96, 2.35, z - 0.49, 0.045, 3.95, 0.025, INDICADOR);
    }
    bloque(0.0, (alto + ROOM_HEIGHT) * 0.5, z,
        ancho, ROOM_HEIGHT - alto, WALL_THICKNESS, PARED_NAVE);
    bloque(0.0, 4.78, z - 0.15, 4.80, 0.56, 0.66, REFUERZO_NAVE);
    bloque(0.0, 4.79, z - 0.49, 2.80, 0.08, 0.025, INDICADOR);
    bloque(0.0, 0.035, z - 0.20, 4.10, 0.07, 0.54, REFUERZO_NAVE);
}

fn construir_servicios_traseros(cubos: &mut Vec<Cubo>) {
    let (_, _, tx, tz, _) = marco_comando();
    let columnas = columnas_comando();
    let u = (columnas[2] + columnas[3]) * 0.5;
    let mut bloque = |u: f32, y: f32, fondo: f32, w: f32, h: f32, d: f32,
                      material: usize, espejo: bool, color: Color| {
        let mut p = punto_comando(u, y, fondo);
        let angulo = if espejo { p.x = -p.x; (-tz).atan2(-tx) }
                     else { (-tz).atan2(tx) };
        cubos.push(Cubo::new(p, Vector3::new(w,h,d), color, 0.0, 0.0)
            .rotado_y(angulo).con_material(material, None));
    };
    bloque(u, 2.18, 0.16, 1.65, 2.50, 0.26, REFUERZO_NAVE, false, Color::WHITE);
    bloque(u, 2.18, 0.31, 1.45, 2.28, 0.025, PANEL_ESCLUSA, false, Color::WHITE);
    // Botón físico y palanca vertical, sin interacción en esta etapa.
    bloque(u - 0.39, 1.36, 0.39, 0.27, 0.27, 0.13, INDICADOR, false, Color::WHITE);
    bloque(u + 0.33, 1.44, 0.42, 0.07, 0.55, 0.14, REFUERZO_NAVE, false, Color::WHITE);
    bloque(u + 0.33, 1.72, 0.49, 0.38, 0.12, 0.22, REFUERZO_NAVE, false, Color::WHITE);
    // Tres conductos de sección cuadrada en la pared opuesta al mando.
    // Pocas piezas: abrazaderas y codos rectos, sin cilindros subdivididos.
    for (i, y) in [1.10f32, 1.60, 2.10].iter().copied().enumerate() {
        bloque(10.30, y, 0.24, 4.0, 0.18, 0.22, REFUERZO_NAVE, true, Color::WHITE);
        let extremo = 8.40 + i as f32 * 0.32;
        bloque(extremo, (y + 4.65) * 0.5, 0.40, 0.18, 4.65 - y, 0.22,
            REFUERZO_NAVE, true, Color::WHITE);
    }
    for u in [9.70f32, 11.50] {
        bloque(u, 1.60, 0.37, 0.13, 1.40, 0.20, REFUERZO_NAVE, true, Color::WHITE);
    }
    bloque(10.95, 3.40, 0.18, 1.85, 1.20, 0.26, REFUERZO_NAVE, true, Color::WHITE);
    bloque(10.95, 3.40, 0.325, 1.64, 0.98, 0.025, CONTROLES_COMANDO, true, Color::WHITE);
}
