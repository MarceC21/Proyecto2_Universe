// Cabina de experimentos independiente: geometría, visor y futura interacción.
// Sin dependencias nuevas. Geometría estática construida una sola vez.
use std::sync::OnceLock;
use raylib::prelude::*;
use crate::colisiones::CajaOrientada;
use crate::cubo::Cubo;
use crate::material::{CONTROLES_COMANDO, INDICADOR, PARED_NAVE, REFUERZO_NAVE, LUZ_LABORATORIO, BOTON_ROJO, VIDRIO_LABORATORIO};
use crate::ray_intersect::{Intersect, RayIntersect};

const CENTRO_U: f32 = 4.60;
const ANCHO: f32 = 5.40;
const FONDO_VISOR: f32 = 1.72;
const VISOR_ABAJO: f32 = 1.36;
const VISOR_ARRIBA: f32 = 3.36;
const SEMIANCHO_VISOR: f32 = 2.55;
const ESPESOR_VISOR: f32 = 0.16;

// Tangente e interior de la diagonal IZQUIERDA junto a la mesa central.
// Se calcula al construir el módulo, nunca se recalcula la raíz por píxel.
struct Marco { x: f32, z: f32, tx: f32, tz: f32, nx: f32, nz: f32, angulo: f32 }
static MARCO: OnceLock<Marco> = OnceLock::new();
fn marco() -> &'static Marco {
    MARCO.get_or_init(|| {
        let vertices = crate::escena::contorno_nave();
        let (ax, az) = vertices[6];
        let (bx, bz) = vertices[5];
        let longitud = ((bx-ax).powi(2)+(bz-az).powi(2)).sqrt();
        let (tx, tz) = ((bx-ax)/longitud, (bz-az)/longitud);
        let (nx, nz) = (tz, -tx);
        let d = crate::escena::WALL_THICKNESS * 0.5;
        Marco { x: ax + tx*CENTRO_U + nx*d, z: az + tz*CENTRO_U + nz*d,
            tx, tz, nx, nz, angulo: (-tz).atan2(tx) }
    })
}
fn punto(u: f32, y: f32, fondo: f32) -> Vector3 {
    let m = marco();
    Vector3::new(m.x + m.tx*u + m.nx*fondo, y, m.z + m.tz*u + m.nz*fondo)
}
fn bloque(cubos: &mut Vec<Cubo>, u: f32, y: f32, fondo: f32,
          ancho: f32, alto: f32, profundidad: f32, material: usize) {
    cubos.push(Cubo::new(punto(u,y,fondo), Vector3::new(ancho,alto,profundidad),
        Color::WHITE, 0.0, 0.0).rotado_y(marco().angulo).con_material(material,None));
}
fn muestra(cubos: &mut Vec<Cubo>, u: f32, y: f32, fondo: f32,
           ancho: f32, alto: f32, profundidad: f32, color: Color) {
    // Recipientes estilizados opacos: el color sugiere el contenido.
    cubos.push(Cubo::new(punto(u,y,fondo), Vector3::new(ancho,alto,profundidad),
        color, 0.55, 0.50).rotado_y(marco().angulo));
}

pub fn construir(cubos: &mut Vec<Cubo>) {
    // Gabinete, cubierta y respaldo. El hueco frontal permite ver los objetos.
    bloque(cubos,0.0,0.53,0.80,ANCHO,1.06,1.50,REFUERZO_NAVE);
    bloque(cubos,0.0,1.14,0.86,ANCHO,0.16,1.66,PARED_NAVE);
    bloque(cubos,0.0,2.29,0.09,ANCHO,2.18,0.14,PARED_NAVE);
    bloque(cubos,0.0,3.64,0.86,ANCHO,0.56,1.66,PARED_NAVE);
    bloque(cubos,0.0,3.345,0.78,4.85,0.025,0.80,LUZ_LABORATORIO);
    for u in [-2.62f32,2.62] {
        bloque(cubos,u,2.28,0.86,0.16,2.16,1.66,REFUERZO_NAVE);
        bloque(cubos,u,2.35,1.73,0.06,1.80,0.04,INDICADOR);
    }
    // Carriles y travesanos del visor. El bloque de vidrio se intersecta
    // aparte de la BVH opaca, para transmitir luz y refractar los rayos.
    bloque(cubos,0.0,1.30,FONDO_VISOR,5.24,0.12,0.10,REFUERZO_NAVE);
    bloque(cubos,0.0,3.39,FONDO_VISOR,5.24,0.06,0.10,REFUERZO_NAVE);
    bloque(cubos,0.0,1.40,FONDO_VISOR+0.05,0.65,0.07,0.10,REFUERZO_NAVE);
    // Ventilación pintada con una textura existente, sin una caja por ranura.
    bloque(cubos,0.0,1.235,1.50,4.92,0.025,0.22,CONTROLES_COMANDO);
    bloque(cubos,0.0,3.65,1.70,4.55,0.26,0.025,CONTROLES_COMANDO);
    // Bandeja de muestras y tres frascos cuadrados de distintos tamaños.
    bloque(cubos,-0.60,1.255,0.91,2.72,0.07,0.78,REFUERZO_NAVE);
    for (u,h,color) in [(-1.45f32,0.55,Color::new(43,184,213,255)),
                        (-0.63,0.74,Color::new(119,201,74,255)),
                        (0.20,0.44,Color::new(219,158,59,255))] {
        muestra(cubos,u,1.29+h*0.5,0.92,0.38,h,0.38,color);
        muestra(cubos,u,1.29+h+0.055,0.92,0.16,0.11,0.16,Color::new(194,216,220,255));
        bloque(cubos,u,1.29+h+0.13,0.92,0.23,0.055,0.23,REFUERZO_NAVE);
        muestra(cubos,u,1.42,1.116,0.23,0.12,0.025,Color::new(223,229,227,255));
    }
    // Instrumento de medición simple a la derecha, con una pantalla pequeña.
    bloque(cubos,1.43,1.27,0.77,0.90,0.10,0.76,REFUERZO_NAVE);
    bloque(cubos,1.70,1.70,0.52,0.16,0.80,0.18,PARED_NAVE);
    bloque(cubos,1.43,2.09,0.66,0.70,0.17,0.42,REFUERZO_NAVE);
    bloque(cubos,1.20,1.92,0.78,0.17,0.25,0.19,PARED_NAVE);
    bloque(cubos,1.43,1.43,0.91,0.54,0.04,0.39,CONTROLES_COMANDO);
    // Panel y botón físico de apertura/cierre: aún sin entrada ni animación.
    bloque(cubos,2.30,2.14,1.75,0.38,0.74,0.08,REFUERZO_NAVE);
    bloque(cubos,2.30,2.33,1.799,0.28,0.19,0.025,CONTROLES_COMANDO);
    bloque(cubos,2.30,1.99,1.82,0.23,0.23,0.08,BOTON_ROJO);
}

pub fn obstaculos() -> Vec<CajaOrientada> {
    let p = punto(0.0,0.0,0.94);
    vec![CajaOrientada { centro_x:p.x, centro_z:p.z,
        mitad_x:ANCHO*0.5, mitad_z:0.94, angulo_y:marco().angulo }]
}

// Punto para conectar después la interacción por proximidad o ray picking.
#[allow(dead_code)]
pub fn posicion_boton() -> Vector3 { punto(2.30,1.99,1.87) }

// Volumen cerrado: frente, dorso y cantos del vidrio. La cara frontal
// conserva la posicion anterior; el grosor crece hacia el laboratorio.
fn vidrio() -> &'static Cubo {
    static VIDRIO: OnceLock<Cubo> = OnceLock::new();
    VIDRIO.get_or_init(|| {
        Cubo::new(
            punto(0.0, (VISOR_ABAJO + VISOR_ARRIBA) * 0.5,
                FONDO_VISOR - ESPESOR_VISOR * 0.5),
            Vector3::new(SEMIANCHO_VISOR * 2.0, VISOR_ARRIBA - VISOR_ABAJO,
                ESPESOR_VISOR), Color::WHITE, 0.0, 0.0,
        ).rotado_y(marco().angulo).con_material(VIDRIO_LABORATORIO, None)
    })
}

// Cubo orienta sus normales contra el rayo para sombrear solidos opacos.
// Para Snell necesitamos la normal EXTERIOR, incluso al salir del vidrio.
pub fn intersectar_visor(origen: &Vector3, direccion: &Vector3) -> Intersect {
    let mut impacto = vidrio().ray_intersect(origen, direccion);
    if !impacto.is_intersecting { return impacto; }
    let m = marco();
    let ox = origen.x-m.x;
    let oz = origen.z-m.z;
    let u = ox*m.tx + oz*m.tz;
    let fondo = ox*m.nx + oz*m.nz;
    let dentro = u.abs() <= SEMIANCHO_VISOR
        && (VISOR_ABAJO..=VISOR_ARRIBA).contains(&origen.y)
        && (FONDO_VISOR - ESPESOR_VISOR..=FONDO_VISOR).contains(&fondo);
    if dentro { impacto.normal = impacto.normal * -1.0; }
    impacto
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vidrio_desvia_dentro_y_sale_paralelo_con_desplazamiento() {
        let m = marco();
        let n = Vector3::new(m.nx, 0.0, m.nz);
        let tangente = Vector3::new(m.tx, 0.0, m.tz);
        let direccion = (n * -1.0 + tangente * 0.60).normalize();
        let entrada = intersectar_visor(&punto(-0.5, 2.3, 2.7), &direccion);
        assert!(entrada.is_intersecting);
        assert!(direccion.dot(entrada.normal) < -0.5);
        let interior = crate::refractar(direccion, entrada.normal, 1.0 / 1.52).unwrap();
        assert!(interior.dot(tangente).abs() < direccion.dot(tangente).abs());
        let salida = intersectar_visor(
            &(entrada.point - entrada.normal * crate::REFLECTION_BIAS), &interior);
        assert!(salida.is_intersecting);
        assert!(interior.dot(salida.normal) > 0.0);
        let exterior = crate::refractar(interior, salida.normal * -1.0, 1.52).unwrap();
        assert!(exterior.dot(direccion) > 0.99999);
        let distancia_recta = ESPESOR_VISOR / (-direccion.dot(n));
        let sin_vidrio = entrada.point + direccion * distancia_recta;
        assert!((salida.point - sin_vidrio).dot(tangente).abs() > 0.02);
        assert!(!intersectar_visor(
            &(salida.point + salida.normal * crate::REFLECTION_BIAS), &exterior
        ).is_intersecting);
    }

    #[test]
    fn fuera_del_marco_no_hay_vidrio() {
        let m = marco();
        let direccion = Vector3::new(-m.nx, 0.0, -m.nz);
        for (u, y) in [(SEMIANCHO_VISOR + 0.1, 2.3), (0.0, VISOR_ARRIBA + 0.1)] {
            assert!(!intersectar_visor(&punto(u, y, 2.7), &direccion).is_intersecting);
        }
    }
}
