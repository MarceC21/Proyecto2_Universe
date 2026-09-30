// Materiales: textura + parámetros de respuesta a la luz.
//
// Modelo de color de una superficie (Blinn-Phong + reflexión + emisión):
//
//   base   = textura(u, v) * albedo                (color propio)
//   local  = ka*base                               (ambiente)
//          + kd*base*luz*max(N.L, 0)               (difusa de Lambert)
//          + ks*luz*max(N.H, 0)^shininess          (especular; H = normalizado(L+V))
//   color  = local*(1 - reflectividad) + reflejo*reflectividad + base*emision
//
// `reflejo` es el color que devuelve OTRO rayo lanzado en la dirección
// reflejada R = D - 2(D.N)N (esto es reflexión trazada, distinta del
// brillo especular, que solo es un punto de luz).
//
// `transparencia` e `ior` ya forman parte del material pero todavía no
// se usan: se activarán con el vidrio (refracción), que está fuera de
// esta etapa.
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
    pub transparencia: f32,  // 0..1 (sin uso todavía)
    pub ior: f32,            // índice de refracción (sin uso todavía)
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
        // Luces indicadoras: solo emisión (no se sombrean).
        Material {
            nombre: "Indicador cian",
            textura: textura::solida(Color::new(70, 230, 255, 255)),
            albedo: Color::new(255, 255, 255, 255),
            ka: 0.0, kd: 0.0, ks: 0.0, shininess: 1.0,
            reflectividad: 0.0, transparencia: 0.0, ior: 1.0, emision: 1.0,
        },
    ]
}