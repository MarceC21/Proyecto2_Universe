use raylib::prelude::*;
use crate::colisiones::Obstaculo;
use crate::cubo::Cubo;
use crate::escena::{FLOOR_THICKNESS, ROOM_BACK_Z, ROOM_HEIGHT, WALL_THICKNESS};
use crate::material::{HOJA_COMPUERTA, PARED_NAVE, PISO_NAVE};

pub const ANCHO_INTERIOR: f32 = 5.6;
pub const FIN_Z: f32 = 24.0;
pub const PUERTA_Z: f32 = 19.0;
pub const PUERTA_ANCHO: f32 = 3.0;
pub const PUERTA_ALTO: f32 = 3.6;

const PUERTA_DESPLAZAMIENTO: f32 = PUERTA_ANCHO + 0.2;
const DURACION_APERTURA: f32 = 1.2;
const DURACION_ESPERA: f32 = 8.0;

fn longitud() -> f32 {
    FIN_Z - ROOM_BACK_Z
}

pub fn obstaculos() -> Vec<Obstaculo> {
    let mitad_exterior = ANCHO_INTERIOR * 0.5 + WALL_THICKNESS;
    let mitad_pared = WALL_THICKNESS * 0.5;
    let inicio = ROOM_BACK_Z + 0.15;
    let largo = FIN_Z - inicio;
    let x_izquierda = -(ANCHO_INTERIOR * 0.5 + mitad_pared);
    let x_derecha = ANCHO_INTERIOR * 0.5 + mitad_pared;
    let puerta_inicio = PUERTA_Z - PUERTA_ANCHO * 0.5;
    let puerta_fin = PUERTA_Z + PUERTA_ANCHO * 0.5;
    let mut lista = vec![Obstaculo {
        min_x: x_izquierda - mitad_pared,
        max_x: x_izquierda + mitad_pared,
        min_z: inicio,
        max_z: inicio + largo,
    }];
    for (min_z, max_z) in [(inicio, puerta_inicio), (puerta_fin, FIN_Z)] {
        lista.push(Obstaculo {
            min_x: x_derecha - mitad_pared,
            max_x: x_derecha + mitad_pared,
            min_z,
            max_z,
        });
    }
    lista.push(Obstaculo {
        min_x: -mitad_exterior,
        max_x: mitad_exterior,
        min_z: FIN_Z,
        max_z: FIN_Z + WALL_THICKNESS,
    });
    lista
}

pub fn construir(cubos: &mut Vec<Cubo>) {
    let largo = longitud();
    let centro_z = ROOM_BACK_Z + largo * 0.5;
    let mitad_pared = WALL_THICKNESS * 0.5;
    let mut bloque = |centro: Vector3, tamano: Vector3, material: usize, tile: Option<f32>| {
        cubos.push(Cubo::new(centro, tamano, Color::WHITE, 0.0, 0.0)
            .con_material(material, tile));
    };

    bloque(
        Vector3::new(0.0, -FLOOR_THICKNESS * 0.5, centro_z),
        Vector3::new(ANCHO_INTERIOR + WALL_THICKNESS * 2.0, FLOOR_THICKNESS, largo),
        PISO_NAVE, Some(1.2),
    );
    let x_izquierda = -(ANCHO_INTERIOR * 0.5 + mitad_pared);
    let x_derecha = ANCHO_INTERIOR * 0.5 + mitad_pared;
    bloque(
        Vector3::new(x_izquierda, ROOM_HEIGHT * 0.5, centro_z),
        Vector3::new(WALL_THICKNESS, ROOM_HEIGHT, largo),
        PARED_NAVE, Some(2.4),
    );
    let puerta_inicio = PUERTA_Z - PUERTA_ANCHO * 0.5;
    let puerta_fin = PUERTA_Z + PUERTA_ANCHO * 0.5;
    for (min_z, max_z) in [(ROOM_BACK_Z, puerta_inicio), (puerta_fin, FIN_Z)] {
        let tramo = max_z - min_z;
        bloque(
            Vector3::new(x_derecha, ROOM_HEIGHT * 0.5, (min_z + max_z) * 0.5),
            Vector3::new(WALL_THICKNESS, ROOM_HEIGHT, tramo),
            PARED_NAVE, Some(2.4),
        );
    }
    bloque(
        Vector3::new(x_derecha, (PUERTA_ALTO + ROOM_HEIGHT) * 0.5, PUERTA_Z),
        Vector3::new(WALL_THICKNESS, ROOM_HEIGHT - PUERTA_ALTO, PUERTA_ANCHO),
        PARED_NAVE, Some(2.4),
    );
    bloque(
        Vector3::new(0.0, ROOM_HEIGHT + mitad_pared, centro_z),
        Vector3::new(ANCHO_INTERIOR + WALL_THICKNESS * 2.0, WALL_THICKNESS, largo),
        PARED_NAVE, Some(2.4),
    );
    bloque(
        Vector3::new(0.0, ROOM_HEIGHT * 0.5, FIN_Z + mitad_pared),
        Vector3::new(ANCHO_INTERIOR + WALL_THICKNESS * 2.0, ROOM_HEIGHT, WALL_THICKNESS),
        PARED_NAVE, Some(2.4),
    );
}

pub struct CompuertaEspacio {
    apertura: f32,
    espera: f32,
}

impl CompuertaEspacio {
    pub fn new() -> Self {
        Self { apertura: 0.0, espera: 0.0 }
    }

    pub fn abierta(&self) -> bool {
        self.apertura >= 0.98
    }

    pub fn actualizar(&mut self, dt: f32, jugador: Option<Vector3>) -> bool {
        let anterior = self.apertura;
        if !dt.is_finite() || dt <= 0.0 { return false; }
        let x_puerta = ANCHO_INTERIOR * 0.5 + WALL_THICKNESS * 0.5;
        let activar = jugador.map_or(false, |p| {
            (p.x - x_puerta).abs() < 1.6
                && (p.z - PUERTA_Z).abs() < PUERTA_ANCHO * 0.5 + 0.8
        });
        if activar {
            self.espera = DURACION_ESPERA;
        } else {
            self.espera = (self.espera - dt).max(0.0);
        }
        let mantener_abierta = jugador.map_or(false, |p| {
            (p.x - x_puerta).abs() < 1.0
                && (p.z - PUERTA_Z).abs() < PUERTA_ANCHO * 0.5 + 0.4
        });
        let cambio = dt / DURACION_APERTURA;
        self.apertura = if self.espera > 0.0 || mantener_abierta {
            (self.apertura + cambio).min(1.0)
        } else {
            (self.apertura - cambio).max(0.0)
        };
        (self.apertura - anterior).abs() > 0.0001
    }

    pub fn obstaculo(&self) -> Obstaculo {
        let x = ANCHO_INTERIOR * 0.5 + WALL_THICKNESS * 0.5;
        let z = PUERTA_Z - self.apertura * PUERTA_DESPLAZAMIENTO;
        Obstaculo {
            min_x: x - WALL_THICKNESS * 0.5,
            max_x: x + WALL_THICKNESS * 0.5,
            min_z: z - PUERTA_ANCHO * 0.5,
            max_z: z + PUERTA_ANCHO * 0.5,
        }
    }

    pub fn construir(&self, cubos: &mut Vec<Cubo>) {
        let x = ANCHO_INTERIOR * 0.5 + WALL_THICKNESS * 0.5;
        let z = PUERTA_Z - self.apertura * PUERTA_DESPLAZAMIENTO;
        cubos.push(Cubo::new(
            Vector3::new(x, PUERTA_ALTO * 0.5, z),
            Vector3::new(WALL_THICKNESS, PUERTA_ALTO, PUERTA_ANCHO),
            Color::WHITE, 0.0, 0.0,
        ).con_material(HOJA_COMPUERTA, None));
    }
}