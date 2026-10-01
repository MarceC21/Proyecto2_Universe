// División de cabina hasta el techo con entrada abierta. Solo geometría estática, sin dependencias nuevas.
// escena.rs decide si construye este conjunto y registra sus colisiones.
use raylib::prelude::*;
use crate::colisiones::Obstaculo;
use crate::cubo::Cubo;
use crate::material::{CONTROLES_COMANDO, INDICADOR, PISO_NAVE, PARED_NAVE, REFUERZO_NAVE, MONITOR_COMANDO};

// Cambiar Z mueve juntos estructura, gabinetes y colisiones.
pub const Z: f32 = -7.60;
const POSTE_X: f32 = 3.70;
// Reserva sobre la pared lateral derecha, del lado de la mesa.
// El tabique ocupa la reserva anterior: se conserva un hueco libre delante.
pub fn espacio_tars() -> Obstaculo {
    Obstaculo { min_x:7.75, max_x:9.40, min_z:Z+1.10, max_z:Z+3.00 }
}

struct Pieza {
    centro: Vector3,
    tamano: Vector3,
    material: usize,
    bloquea: bool,
}
fn piezas() -> Vec<Pieza> {
    let mut piezas = Vec::with_capacity(64);
    let mut poner = |x:f32,y:f32,z:f32,w:f32,h:f32,d:f32,m:usize,bloquea:bool| {
        piezas.push(Pieza { centro:Vector3::new(x,y,Z+z), tamano:Vector3::new(w,h,d),
            material:m, bloquea });
    };
    let techo=crate::escena::ROOM_HEIGHT;
    let extremo=crate::escena::ROOM_HALF_X;
    for lado in [-1.0f32,1.0] {
        // Tabiques desde el borde exterior del arco hasta las paredes laterales.
        poner(lado*(3.97+extremo)*0.5,techo*0.5,0.0,
            extremo-3.97,techo,0.60,PARED_NAVE,true);
        // Relleno sobre los hombros: cierra toda la división hasta el techo.
        poner(lado*3.755,(4.38+techo)*0.5,0.0,0.43,techo-4.38,0.60,PARED_NAVE,false);
        poner(lado*3.34,(4.78+techo)*0.5,0.0,0.40,techo-4.78,0.60,PARED_NAVE,false);
        // Postes, zapatas y caras metálicas: hueco central ancho y sin umbral.
        poner(lado*POSTE_X,0.12,0.0,0.86,0.24,1.10,REFUERZO_NAVE,true);
        poner(lado*POSTE_X,2.06,0.0,0.54,3.88,0.72,REFUERZO_NAVE,true);
        poner(lado*POSTE_X,2.10,0.375,0.39,3.54,0.035,PISO_NAVE,false);
        // Hombros escalonados: no requieren nuevas primitivas ni rotaciones.
        poner(lado*3.44,4.15,0.0,1.06,0.46,0.72,REFUERZO_NAVE,false);
        poner(lado*3.11,4.57,0.0,0.86,0.42,0.72,REFUERZO_NAVE,false);
        poner(lado*(POSTE_X-0.15),2.82,0.406,0.065,1.10,0.022,INDICADOR,false);

    }
    poner(0.0,(5.16+techo)*0.5,0.0,6.28,techo-5.16,0.60,PARED_NAVE,false);
    // Viga superior por encima de la vista frontal hacia las ventanas.
    poner(0.0,4.96,0.0,6.28,0.40,0.72,REFUERZO_NAVE,false);
    poner(0.0,4.96,0.375,5.90,0.23,0.035,PISO_NAVE,false);
    // Estación de trabajo IZQUIERDA, frente al tabique y mirando a la mesa.
    // Ancho 4.90: ocupa la mayor parte de ese lado, dejando libre el arco.
    let puesto=-6.65;
    poner(puesto,1.16,1.01,4.90,0.16,1.28,PISO_NAVE,true);
    for x in [puesto-1.84,puesto+1.84] {
        poner(x,0.54,0.96,0.80,1.08,1.10,REFUERZO_NAVE,true);
        poner(x,0.62,1.527,0.64,0.68,0.025,PARED_NAVE,false);
        poner(x,0.77,1.55,0.22,0.045,0.045,REFUERZO_NAVE,false);
    }
    // Pantalla mural ancha y mando al alcance desde el pasillo.
    poner(puesto,2.27,0.37,3.66,1.58,0.12,REFUERZO_NAVE,false);
    poner(puesto,2.27,0.447,3.42,1.34,0.025,MONITOR_COMANDO,false);
    poner(puesto,1.265,1.06,3.56,0.045,0.80,CONTROLES_COMANDO,false);
    for x in [puesto-1.12,puesto+1.12] {
        poner(x,1.34,1.22,0.16,0.11,0.16,REFUERZO_NAVE,false);
    }
    // Equipo auxiliar, sin otro monitor ni más fuentes de luz.
    poner(puesto-1.95,1.43,0.86,0.56,0.38,0.48,PARED_NAVE,false);

    // Gabinetes DERECHOS. Terminan en X=7.20 para reservar el extremo a TARS.
    for x in [5.10f32,6.55] {
        poner(x,0.09,0.93,1.30,0.18,1.18,REFUERZO_NAVE,true);
        poner(x,0.93,0.93,1.24,1.50,1.12,REFUERZO_NAVE,true);
        poner(x,1.72,0.93,1.30,0.08,1.18,PISO_NAVE,true);
        for y in [0.43f32,0.92,1.41] {
            poner(x,y,1.508,1.08,0.40,0.025,PARED_NAVE,false);
            poner(x,y+0.06,1.54,0.30,0.05,0.04,REFUERZO_NAVE,false);
        }
    }
    poner(5.10,1.91,0.90,0.90,0.30,0.58,PISO_NAVE,false);
    poner(5.10,2.08,0.90,0.94,0.04,0.62,REFUERZO_NAVE,false);
    poner(5.10,2.14,0.90,0.30,0.08,0.11,REFUERZO_NAVE,false);
    piezas
}

pub fn construir(cubos:&mut Vec<Cubo>) {
    for pieza in piezas() {
        let tile=match pieza.material { PISO_NAVE=>Some(1.2), PARED_NAVE=>Some(2.4), _=>None };
        cubos.push(Cubo::new(pieza.centro,pieza.tamano,Color::WHITE,0.0,0.0)
            .con_material(pieza.material,tile));
    }
}

pub fn obstaculos() -> Vec<Obstaculo> {
    // Misma descripción que el dibujo: no hay barrera invisible bajo la viga.
    piezas().into_iter().filter(|p|p.bloquea).map(|p|Obstaculo {
        min_x:p.centro.x-p.tamano.x*0.5, max_x:p.centro.x+p.tamano.x*0.5,
        min_z:p.centro.z-p.tamano.z*0.5, max_z:p.centro.z+p.tamano.z*0.5,
    }).collect()
}
