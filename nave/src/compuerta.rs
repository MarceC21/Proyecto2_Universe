// Este es para la compuerta que se abre y cierra 

use raylib::prelude::*;
use crate::colisiones::Obstaculo;
use crate::cubo::Cubo;
use crate::escena::{control_esclusa, ROOM_BACK_Z};
use crate::material::HOJA_COMPUERTA;

const ANCHO_HOJA: f32 = 1.895;
const ALTO_HOJA: f32 = 4.50;
const DURACION_APERTURA: f32 = 1.2;
const DURACION_ESPERA: f32 = 8.0;

pub struct Compuerta {
    apertura: f32,
    espera: f32,
}

impl Compuerta {
    pub fn new() -> Self {
        Self { apertura: 0.0, espera: 0.0 }
    }

    pub fn abierta(&self) -> bool {
        self.apertura >= 0.98
    }

    pub fn actualizar(&mut self, dt: f32, jugador: Option<Vector3>) -> bool {
        let anterior = self.apertura;
        if !dt.is_finite() || dt <= 0.0 { return false; }
        let activar = jugador.map_or(false, |p| {
            let control = control_esclusa();
            let cerca_del_control = (p.x - control.x).powi(2) + (p.z - control.z).powi(2) < 1.35f32.powi(2);
            let cerca_del_umbral = p.x.abs() < 2.5 && p.z > ROOM_BACK_Z + 0.25
                && p.z < ROOM_BACK_Z + 3.0;
            cerca_del_control || cerca_del_umbral
        });
        if activar {
            self.espera = DURACION_ESPERA;
        } else {
            self.espera = (self.espera - dt).max(0.0);
        }
        let mantener_abierta = jugador.map_or(false, |p| {
            p.x.abs() < 2.0 && p.z > ROOM_BACK_Z - 0.8 && p.z < ROOM_BACK_Z + 0.7
        });
        let cambio = dt / DURACION_APERTURA;
        self.apertura = if self.espera > 0.0 || mantener_abierta {
            (self.apertura + cambio).min(1.0)
        } else {
            (self.apertura - cambio).max(0.0)
        };
        (self.apertura - anterior).abs() > 0.0001
    }

    pub fn obstaculos(&self) -> [Obstaculo; 2] {
        let desplazamiento = self.apertura * 1.92;
        [-1.0f32, 1.0].map(|lado| {
            let centro_x = lado * (0.9525 + desplazamiento);
            Obstaculo {
                min_x: centro_x - ANCHO_HOJA * 0.5,
                max_x: centro_x + ANCHO_HOJA * 0.5,
                min_z: ROOM_BACK_Z - 0.19,
                max_z: ROOM_BACK_Z + 0.07,
            }
        })
    }

    pub fn construir(&self, cubos: &mut Vec<Cubo>) {
        let desplazamiento = self.apertura * 1.92;
        for lado in [-1.0f32, 1.0] {
            let centro_x = lado * (0.9525 + desplazamiento);
            cubos.push(Cubo::new(
                Vector3::new(centro_x, ALTO_HOJA * 0.5, ROOM_BACK_Z - 0.06),
                Vector3::new(ANCHO_HOJA, ALTO_HOJA, 0.26),
                Color::WHITE, 0.0, 0.0,
            ).con_material(HOJA_COMPUERTA, None));
        }
    }
}
