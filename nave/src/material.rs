// Materiales: textura + parámetros de respuesta a la luz.
//
// Modelo de color de una superficie (Blinn-Phong + reflexión + emisión):
//
//   base   = textura(u, v) * albedo                (color propio)
//   local  = AMBIENTE*ka*base                      (ambiente tenue)
//          + kd*base*luz*max(N.L, 0)               (difusa de Lambert)
//          + ks*luz*max(N.H, 0)^shininess          (especular; H = normalizado(L+V))
//   color  = local*(1 - reflectividad) + reflejo*reflectividad + base*emision
//
// `reflejo` es el color que devuelve OTRO rayo lanzado en la dirección
// reflejada R = D - 2(D.N)N (esto es reflexión trazada, distinta del
// brillo especular, que solo es un punto de luz).
//
// El visor del laboratorio usa transparencia e ior para trazar rayos
// refractados en sus dos superficies, ademas de la reflexion de Fresnel.
//
// Las texturas se generan una sola vez, aquí, al arrancar.

use raylib::prelude::*;

use crate::textura::{self, Textura};

// Índices del catálogo (posición en el Vec devuelto por crear_materiales).
pub const METAL_CLARO: usize = 0;
pub const METAL_OSCURO: usize = 1;
pub const PANTALLA: usize = 2;
pub const POLIMERO: usize = 3;
pub const INDICADOR: usize = 4;
// Materiales exclusivos de cabina; los cinco anteriores se conservan.
pub const RADAR_CABINA: usize = 5;
pub const BOTONERA_CABINA: usize = 6;
pub const SISTEMAS_CABINA: usize = 7;
pub const TAPIZADO_CABINA: usize = 8;
pub const PARED_NAVE: usize = 9;
pub const PISO_NAVE: usize = 10;
pub const REFUERZO_NAVE: usize = 11;
pub const MONITOR_COMANDO: usize = 12;
pub const CONTROLES_COMANDO: usize = 13;
pub const HOJA_COMPUERTA: usize = 14;
pub const PANEL_ESCLUSA: usize = 15;
pub const LUZ_LABORATORIO: usize = 16;
pub const BOTON_ROJO: usize = 17;
pub const TARS_CASCO: usize = 18;
pub const TARS_NEGRO: usize = 19;
pub const TARS_NOMBRE: usize = 20;
pub const TARS_BRAILLE: usize = 21;
pub const VIDRIO_LABORATORIO: usize = 22;

#[allow(dead_code)]
pub struct Material {
    pub nombre: &'static str,
    pub textura: Textura,
    pub albedo: Color,       // tinte que multiplica a la textura
    pub ka: f32,             // coeficiente ambiental
    pub kd: f32,             // coeficiente difuso
    pub ks: f32,             // coeficiente especular
    pub shininess: f32,      // exponente especular (más alto = brillo más pequeño)
    pub reflectividad: f32,  // 0..1: peso del rayo reflejado
    pub transparencia: f32,  // 0..1: transmision del visor refractivo
    pub ior: f32,            // indice de refraccion (aire = 1.0)
    pub emision: f32,        // 0..1+: cuánto color propio se suma sin depender de la luz
}

// `pantalla_size` = tamaño en texels de la textura de la pantalla; debe
// tener la misma proporción que la pantalla real (lo calcula mesa.rs).
pub fn crear_materiales(pantalla_size: (usize, usize), consola_size: (usize, usize)) -> Vec<Material> {
    vec![
        // Marco y tablero: metal gris claro, brillo controlado y algo de
        // reflexión (no es un espejo).
        Material {
            nombre: "Metal claro",
            textura: textura::metal_placas(256, (225.0, 225.0, 230.0)),
            albedo: Color::new(200, 206, 216, 255),
            ka: 0.30, kd: 0.65, ks: 0.50, shininess: 64.0,
            reflectividad: 0.25, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        // Soporte: metal oscuro, más mate y menos reflectante.
        Material {
            nombre: "Metal oscuro",
            textura: textura::metal_placas(256, (150.0, 152.0, 160.0)),
            albedo: Color::new(95, 100, 116, 255),
            ka: 0.30, kd: 0.60, ks: 0.35, shininess: 40.0,
            reflectividad: 0.10, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        // Pantalla: azul oscuro con cuadrícula; emisiva moderada para que
        // las líneas cian se lean aunque la luz sea poca.
        Material {
            nombre: "Pantalla",
            textura: textura::pantalla_tecnica(pantalla_size.0, pantalla_size.1),
            albedo: Color::new(255, 255, 255, 255),
            ka: 0.25, kd: 0.30, ks: 0.20, shininess: 90.0,
            reflectividad: 0.05, transparencia: 0.0, ior: 1.0, emision: 0.80,
        },
        // Módulo de controles.
        Material {
            nombre: "Polimero de consola",
            textura: textura::polimero_consola(consola_size.0, consola_size.1),
            albedo: Color::new(255, 255, 255, 255),
            ka: 0.30, kd: 0.60, ks: 0.30, shininess: 40.0,
            reflectividad: 0.05, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        // Luces indicadoras: emisión visible y fuente local registrada en luz.rs.
        Material {
            nombre: "Indicador cian",
            textura: textura::solida(Color::new(70, 230, 255, 255)),
            albedo: Color::new(255, 255, 255, 255),
            ka: 0.0, kd: 0.0, ks: 0.0, shininess: 1.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 1.0,
        },
        Material {
            nombre: "Radar de navegacion",
            textura: textura::radar_cabina(),
            albedo: Color::WHITE,
            ka: 0.25, kd: 0.15, ks: 0.20, shininess: 80.0,
            reflectividad: 0.02, transparencia: 0.0, ior: 1.0, emision: 0.85,
        },
        Material {
            nombre: "Botonera de mando",
            textura: textura::botonera_cabina(),
            albedo: Color::WHITE,
            ka: 0.35, kd: 0.50, ks: 0.25, shininess: 40.0,
            reflectividad: 0.03, transparencia: 0.0, ior: 1.0, emision: 0.30,
        },
        Material {
            nombre: "Telemetria de cabina",
            textura: textura::sistemas_cabina(),
            albedo: Color::WHITE,
            ka: 0.25, kd: 0.15, ks: 0.20, shininess: 80.0,
            reflectividad: 0.02, transparencia: 0.0, ior: 1.0, emision: 0.80,
        },
        Material {
            nombre: "Tapizado acolchado de cabina",
            textura: textura::tapizado_cabina(),
            albedo: Color::WHITE,
            ka: 0.42, kd: 0.75, ks: 0.08, shininess: 12.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        // Superficies extensas: brillo local suave, sin rayos de reflexión.
        Material {
            nombre: "Panel gris de pared",
            textura: textura::pared_nave(),
            albedo: Color::WHITE,
            ka: 0.44, kd: 0.52, ks: 0.08, shininess: 24.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Piso gris antideslizante",
            textura: textura::piso_nave(),
            albedo: Color::WHITE,
            ka: 0.40, kd: 0.58, ks: 0.04, shininess: 12.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Refuerzos y conductos grises",
            textura: textura::solida(Color::new(105, 108, 112, 255)),
            albedo: Color::WHITE,
            ka: 0.44, kd: 0.55, ks: 0.12, shininess: 24.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Monitor de comando derecho",
            textura: textura::monitor_comando(), albedo: Color::WHITE,
            ka: 0.25, kd: 0.15, ks: 0.12, shininess: 40.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.75,
        },
        Material {
            nombre: "Instrumentos de pared",
            textura: textura::controles_comando(), albedo: Color::WHITE,
            ka: 0.40, kd: 0.50, ks: 0.08, shininess: 20.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.12,
        },
        Material {
            nombre: "Compuerta industrial",
            textura: textura::hoja_compuerta(), albedo: Color::WHITE,
            ka: 0.44, kd: 0.54, ks: 0.10, shininess: 24.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Mando de esclusa",
            textura: textura::panel_esclusa(), albedo: Color::WHITE,
            ka: 0.45, kd: 0.35, ks: 0.08, shininess: 20.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.22,
        },
        Material {
            nombre: "Lampara calida de laboratorio",
            textura: textura::solida(Color::new(255,221,152,255)), albedo: Color::WHITE,
            ka: 0.0, kd: 0.0, ks: 0.0, shininess: 1.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 1.0,
        },
        Material {
            nombre: "Boton rojo iluminado",
            textura: textura::solida(Color::new(230,56,38,255)), albedo: Color::WHITE,
            ka: 0.0, kd: 0.0, ks: 0.0, shininess: 1.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.9,
        },
        Material {
            nombre: "Casco de TARS", textura: crate::tars::textura_casco(), albedo: Color::WHITE,
            ka: 0.48, kd: 0.65, ks: 0.40, shininess: 48.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Juntas de TARS", textura: crate::tars::textura_negro(), albedo: Color::WHITE,
            ka: 0.48, kd: 0.65, ks: 0.10, shininess: 48.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Nombre de TARS", textura: crate::tars::textura_nombre(), albedo: Color::WHITE,
            ka: 0.48, kd: 0.65, ks: 0.30, shininess: 48.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Braille de TARS", textura: crate::tars::textura_braille(), albedo: Color::WHITE,
            ka: 0.48, kd: 0.65, ks: 0.30, shininess: 48.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 0.0,
        },
        Material {
            nombre: "Vidrio de seguridad del laboratorio",
            textura: textura_vidrio_laboratorio(),
            albedo: Color::new(250, 254, 255, 255),
            ka: 0.12, kd: 0.08, ks: 0.55, shininess: 120.0,
            reflectividad: 0.043, transparencia: 0.98, ior: 1.52, emision: 0.0,
        },
    ]
}

// Textura propia, casi transparente: borde pulido ligeramente cian.
// Se genera una sola vez y se consulta con las UV del bloque de vidrio.
fn textura_vidrio_laboratorio() -> Textura {
    let mut textura = Textura::new(128, 128, Color::WHITE);
    for y in 0i32..128 {
        for x in 0i32..128 {
            let borde = x.min(127 - x).min(y.min(127 - y));
            if borde < 2 {
                textura.set(x, y, Color::new(222, 246, 251, 255));
            }
        }
    }
    textura
}
