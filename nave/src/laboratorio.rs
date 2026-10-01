// Cabina de experimentos independiente: geometría, visor y futura interacción.
// Sin dependencias nuevas. Geometría estática construida una sola vez.
use std::sync::OnceLock;
use raylib::prelude::*;
use crate::colisiones::CajaOrientada;
use crate::cubo::Cubo;
use crate::material::{CONTROLES_COMANDO, INDICADOR, PARED_NAVE, REFUERZO_NAVE, LUZ_LABORATORIO, BOTON_ROJO};

const CENTRO_U: f32 = 4.60;
const ANCHO: f32 = 5.40;
const FONDO_VISOR: f32 = 1.72;
const VISOR_ABAJO: f32 = 1.36;
const VISOR_ARRIBA: f32 = 3.36;
const SEMIANCHO_VISOR: f32 = 2.55;

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
    // Carriles y travesaños del visor. El cristal NO es un cubo opaco:
    // filtrar_visor() lo compone después del trazado usando su profundidad.
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

// Vidrio plano de transparencia sencilla, sin refracción ni rayos adicionales.
// limite es la distancia al objeto opaco/anillo ya trazado: así no se pinta
// vidrio delante de una columna, una mesa o un objeto más cercano.
// Al animarlo después, desplazar también estos límites junto al marco móvil.
pub fn filtrar_visor(color: Color, origen: &Vector3, direccion: &Vector3, limite: f32) -> Color {
    let m = marco();
    let denominador = direccion.x*m.nx + direccion.z*m.nz;
    if denominador.abs() < 0.000001 { return color; }
    let ox = origen.x-m.x;
    let oz = origen.z-m.z;
    let t = (FONDO_VISOR - ox*m.nx - oz*m.nz) / denominador;
    if t <= 0.001 || t >= limite { return color; }
    let u = (ox + direccion.x*t)*m.tx + (oz + direccion.z*t)*m.tz;
    let y = origen.y + direccion.y*t;
    if u.abs() > SEMIANCHO_VISOR || !(VISOR_ABAJO..=VISOR_ARRIBA).contains(&y) { return color; }
    let borde = SEMIANCHO_VISOR-u.abs() < 0.035 || y-VISOR_ABAJO < 0.025 || VISOR_ARRIBA-y < 0.025;
    let alpha = if borde { 0.30 } else { 0.065 };
    let mezclar = |a: u8, b: u8| (a as f32*(1.0-alpha)+b as f32*alpha) as u8;
    Color::new(mezclar(color.r,102),mezclar(color.g,199),mezclar(color.b,214),255)
}
