// Exterior: dos alas solares, balizas de salida y tres motores tubulares.
// Toda la estructura opaca usa cubos texturizados y entra en la BVH estatica.
// No mueve el contorno, los muebles, el pasillo ni sus puertas.
use std::sync::OnceLock;
use raylib::prelude::*;
use crate::colisiones::Obstaculo;
use crate::cubo::Cubo;
use crate::material::{PARED_NAVE, REFUERZO_NAVE, PISO_NAVE, INDICADOR,
    VIDRIO_CABINA, PANEL_SOLAR, BALIZA_SALIDA};
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::escena::{contorno_nave, ROOM_HALF_X, WALL_THICKNESS};

pub struct Ventana {
    pub a: Vector3,
    pub b: Vector3,
    pub ancho: f32,
    pub abajo: f32,
    pub arriba: f32,
}

// Los antepechos mantienen su altura para no tocar las consolas.
pub fn ventanas() -> [Ventana; 3] {
    let v = contorno_nave();
    let p = |i: usize| Vector3::new(v[i].0, 0.0, v[i].1);
    [
        Ventana { a:p(0), b:p(1), ancho:7.50, abajo:0.95, arriba:5.85 },
        Ventana { a:p(0), b:p(7), ancho:5.20, abajo:0.90, arriba:5.75 },
        Ventana { a:p(2), b:p(1), ancho:5.20, abajo:0.90, arriba:5.75 },
    ]
}

fn cristales() -> &'static [Cubo; 3] {
    static CRISTALES: OnceLock<[Cubo; 3]> = OnceLock::new();
    CRISTALES.get_or_init(|| ventanas().map(|v| {
        let dx = v.b.x - v.a.x;
        let dz = v.b.z - v.a.z;
        let largo = (dx*dx + dz*dz).sqrt();
        let ancho_libre = v.ancho.min(largo - 0.36) - 0.36;
        Cubo::new(
            Vector3::new((v.a.x+v.b.x)*0.5, (v.abajo+v.arriba)*0.5,
                (v.a.z+v.b.z)*0.5),
            Vector3::new(ancho_libre+0.015, v.arriba-v.abajo+0.015, 0.06),
            Color::WHITE, 0.0, 0.0,
        ).rotado_y((-dz).atan2(dx)).con_material(VIDRIO_CABINA, None)
    }))
}

// Interseccion de las dos caras y los cantos. Normales exteriores para
// distinguir aire->vidrio y vidrio->aire, igual que en el laboratorio.
pub fn intersectar_ventanas(o: &Vector3, d: &Vector3, limite: f32) -> Intersect {
    let mut mejor = Intersect::empty();
    let mut distancia = limite;
    for cristal in cristales() {
        let mut impacto = cristal.ray_intersect(o, d);
        if !impacto.is_intersecting || impacto.distance >= distancia { continue; }
        let (centro, mitad, ejes) = cristal.marco_luz();
        let p = *o - centro;
        if p.dot(ejes[0]).abs() <= mitad.x
            && p.dot(ejes[1]).abs() <= mitad.y
            && p.dot(ejes[2]).abs() <= mitad.z {
            impacto.normal = impacto.normal * -1.0;
        }
        distancia = impacto.distance;
        mejor = impacto;
    }
    mejor
}

struct Pieza {
    centro: Vector3,
    tamano: Vector3,
    material: usize,
    bloquea: bool,
}

fn pieza(p: &mut Vec<Pieza>, centro: (f32,f32,f32), tamano: (f32,f32,f32),
    material: usize, bloquea: bool) {
    p.push(Pieza { centro:Vector3::new(centro.0,centro.1,centro.2),
        tamano:Vector3::new(tamano.0,tamano.1,tamano.2), material, bloquea });
}

const MOTOR_Y: f32 = 2.90;
const RADIO_MOTOR: f32 = 1.45;
const MOTORES_X: [f32; 3] = [-3.50, 0.0, 3.50];

const PANEL_ANCHO: f32 = 7.80;
const PANEL_FONDO: f32 = 6.00;
const PANEL_Z: f32 = 1.50;
const BORDE_PLATAFORMA: f32 = ROOM_HALF_X + WALL_THICKNESS;
const PANEL_X: f32 = BORDE_PLATAFORMA + 0.70 + PANEL_ANCHO*0.5;

fn paneles_solares(p: &mut Vec<Pieza>) {
    // Dos alas bajas, junto a la parte sobresaliente del piso existente.
    // La cuadricula se pinta en una textura: no hay un cubo por celda.
    for lado in [-1.0f32,1.0] {
        let x = lado*PANEL_X;
        pieza(p,(x,0.60,PANEL_Z),(PANEL_ANCHO,0.14,PANEL_FONDO),REFUERZO_NAVE,true);
        pieza(p,(x,0.69,PANEL_Z),(PANEL_ANCHO-0.20,0.04,PANEL_FONDO-0.20),PANEL_SOLAR,false);
        // Marco claro y tres travesanos que dividen cada ala en cuatro paños.
        for extremo in [-1.0f32,1.0] {
            pieza(p,(x,0.71,PANEL_Z+extremo*(PANEL_FONDO*0.5-0.05)),
                (PANEL_ANCHO,0.18,0.10),PARED_NAVE,false);
            pieza(p,(x+extremo*(PANEL_ANCHO*0.5-0.05),0.71,PANEL_Z),
                (0.10,0.18,PANEL_FONDO),PARED_NAVE,false);
        }
        for offset in [-PANEL_ANCHO*0.25,0.0,PANEL_ANCHO*0.25] {
            pieza(p,(x+offset,0.735,PANEL_Z),(0.06,0.08,PANEL_FONDO-0.20),PARED_NAVE,false);
        }
        // Bases sobre la plataforma y brazos cortos hasta el marco.
        let raiz = BORDE_PLATAFORMA-0.15;
        let extremo = PANEL_X-PANEL_ANCHO*0.5+0.20;
        for z in [PANEL_Z-2.35,PANEL_Z+2.35] {
            pieza(p,(lado*raiz,0.06,z),(0.30,0.12,0.46),PARED_NAVE,true);
            pieza(p,(lado*raiz,0.32,z),(0.18,0.60,0.24),REFUERZO_NAVE,true);
            pieza(p,(lado*(raiz+extremo)*0.5,0.53,z),
                (extremo-raiz,0.16,0.20),REFUERZO_NAVE,true);
        }
    }
}

fn balizas_salida(p: &mut Vec<Pieza>) {
    let x = crate::pasillo::ANCHO_INTERIOR*0.5+WALL_THICKNESS;
    let z = crate::pasillo::PUERTA_Z;
    // Luz propia solamente: BALIZA_SALIDA no se registra como fuente en luz.rs,
    // por lo que estas señales no cambian la iluminacion existente.
    for extremo in [-1.0f32,1.0] {
        let borde = z+extremo*(crate::pasillo::PUERTA_ANCHO*0.5+0.25);
        pieza(p,(x+0.04,1.85,borde),(0.12,0.60,0.26),REFUERZO_NAVE,false);
        pieza(p,(x+0.11,1.85,borde),(0.035,0.42,0.10),BALIZA_SALIDA,false);
    }
    let y = crate::pasillo::PUERTA_ALTO+0.25;
    pieza(p,(x+0.04,y,z),(0.12,0.22,0.85),REFUERZO_NAVE,false);
    pieza(p,(x+0.11,y,z),(0.035,0.08,0.65),BALIZA_SALIDA,false);
}

// Anillo hueco aproximado con franjas de cubos: seccion redondeada sin
// nuevas primitivas ni bibliotecas. ri=0 produce un fondo cerrado.
fn anillo(p: &mut Vec<Pieza>, x: f32, z: f32, largo: f32,
    radio: f32, ri: f32, material: usize) {
    const FRANJAS: usize = 12;
    let paso = radio*2.0 / FRANJAS as f32;
    for i in 0..FRANJAS {
        let y0 = -radio+i as f32*paso;
        let y1 = y0+paso;
        let cercano = if y0<=0.0 && y1>=0.0 { 0.0 } else { y0.abs().min(y1.abs()) };
        let lejano = y0.abs().max(y1.abs());
        let exterior = (radio*radio-cercano*cercano).max(0.0).sqrt();
        let interior = (ri*ri-lejano*lejano).max(0.0).sqrt();
        let y = MOTOR_Y+(y0+y1)*0.5;
        if interior<0.001 {
            pieza(p,(x,y,z),(2.0*exterior,paso+0.002,largo),material,false);
        } else {
            let ancho = exterior-interior;
            for lado in [-1.0f32,1.0] {
                pieza(p,(x+lado*(exterior+interior)*0.5,y,z),
                    (ancho,paso+0.002,largo),material,false);
            }
        }
    }
}

fn motores(p: &mut Vec<Pieza>) {
    let base = crate::pasillo::FIN_Z + WALL_THICKNESS;
    // Soporte unido a la cara EXTERIOR de la pared terminal del pasillo.
    pieza(p,(0.0,MOTOR_Y,base+0.30),(10.30,3.50,0.64),PARED_NAVE,true);
    for y in [MOTOR_Y-1.65,MOTOR_Y+1.65] {
        pieza(p,(0.0,y,base+0.64),(10.30,0.18,0.16),REFUERZO_NAVE,false);
    }
    for x in MOTORES_X {
        // Cuerpo tubular, labio de salida y fondo hundido de la tobera.
        anillo(p,x,base+2.30,3.60,RADIO_MOTOR,1.14,PARED_NAVE);
        anillo(p,x,base+4.10,0.45,RADIO_MOTOR+0.10,1.14,REFUERZO_NAVE);
        anillo(p,x,base+1.00,0.16,1.18,0.0,REFUERZO_NAVE);
        pieza(p,(x,MOTOR_Y,base+1.10),(1.10,1.10,0.025),INDICADOR,false);
        // Aletas interiores: visibles mirando la boca desde atras.
        for lado in [-1.0f32,1.0] {
            pieza(p,(x+lado*0.84,MOTOR_Y,base+2.42),(0.12,1.15,2.70),PISO_NAVE,false);
        }
    }
}

fn piezas() -> Vec<Pieza> {
    let mut p = Vec::with_capacity(256);
    paneles_solares(&mut p);
    balizas_salida(&mut p);
    motores(&mut p);
    p
}

pub fn construir(cubos: &mut Vec<Cubo>) {
    let _ = cristales();
    for p in piezas() {
        cubos.push(Cubo::new(p.centro,p.tamano,Color::WHITE,0.0,0.0)
            .con_material(p.material,Some(1.2)));
    }
}

pub fn obstaculos() -> Vec<Obstaculo> {
    let mut resultado: Vec<Obstaculo> = piezas().into_iter().filter(|p|p.bloquea)
        .map(|p| Obstaculo { min_x:p.centro.x-p.tamano.x*0.5,
            max_x:p.centro.x+p.tamano.x*0.5,
            min_z:p.centro.z-p.tamano.z*0.5,
            max_z:p.centro.z+p.tamano.z*0.5 }).collect();
    // Huellas conservadoras de los motores; dejan libre todo el acceso
    // lateral situado en PUERTA_Z, bastante antes de este conjunto.
    let base = crate::pasillo::FIN_Z + WALL_THICKNESS;
    for x in MOTORES_X {
        resultado.push(Obstaculo { min_x:x-RADIO_MOTOR-0.10,
            max_x:x+RADIO_MOTOR+0.10, min_z:base+0.50, max_z:base+4.325 });
    }
    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cristales_se_atraviesan_en_ambos_sentidos_y_respetan_oclusores() {
        for cristal in cristales() {
            let (centro, _, ejes) = cristal.marco_luz();
            for lado in [-1.0f32, 1.0] {
                let normal = ejes[2]*lado;
                let origen = centro+normal*0.6;
                let d = normal * -1.0;
                // Un objeto a 0.1 unidades tapa la ventana a 0.57 unidades.
                assert!(!intersectar_ventanas(&origen,&d,0.1).is_intersecting);
                let entrada = intersectar_ventanas(&origen,&d,f32::INFINITY);
                assert!(entrada.is_intersecting);
                assert_eq!(entrada.material,Some(VIDRIO_CABINA));
                assert!(entrada.normal.dot(d) < -0.99);
                let interior = crate::refractar(d,entrada.normal,1.0/1.5).unwrap();
                let salida = intersectar_ventanas(
                    &(entrada.point-entrada.normal*crate::REFLECTION_BIAS),
                    &interior,f32::INFINITY);
                assert!(salida.is_intersecting);
                assert!(salida.distance>0.05 && salida.distance<0.07);
                assert!(salida.normal.dot(interior)>0.99);
                let final_ = crate::refractar(interior,salida.normal * -1.0,1.5).unwrap();
                assert!(final_.dot(d)>0.99999);
            }
        }
    }

    #[test]
    fn revestimiento_no_invade_el_interior_ni_la_puerta_lateral() {
        let mut paneles = Vec::new();
        paneles_solares(&mut paneles);
        for p in paneles {
            assert!(p.centro.x.abs()-p.tamano.x*0.5 >= ROOM_HALF_X-0.01);
            assert!(p.centro.y+p.tamano.y*0.5<0.90);
            assert!(p.centro.z+p.tamano.z*0.5<crate::pasillo::PUERTA_Z-2.0);
        }
        let mut atras = Vec::new();
        motores(&mut atras);
        for p in atras {
            assert!(p.centro.z-p.tamano.z*0.5>crate::pasillo::FIN_Z);
            assert!(p.tamano.x>0.0 && p.tamano.y>0.0 && p.tamano.z>0.0);
        }
        for o in obstaculos() {
            assert!(o.max_z<crate::pasillo::PUERTA_Z-2.0 || o.min_z>crate::pasillo::FIN_Z);
        }
    }
}
