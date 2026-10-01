// Fondo estelar procedural.
//
// Tres formas típicas de resolver esto: un skybox (textura), una
// esfera invertida (geometría), o una lista de estrellas generada una
// sola vez. Acá se usa una cuarta, más barata que las tres: el color
// del cielo se calcula con una función DETERMINÍSTICA de la dirección
// del rayo. Se cuantiza esa dirección en una rejilla angular fija
// sobre la esfera celeste (una especie de "cuadrícula de casillas del
// cielo") y se hashea cada celda a un número; si ese número supera un
// umbral, esa celda "tiene estrella".
//
// Como la función siempre da el mismo resultado para la misma
// dirección del mundo, el cielo nunca se reconstruye ni se recalcula
// de forma distinta entre frames — es, en efecto, un skybox fijo —
// solo que no hace falta generarlo antes de arrancar ni guardar nada
// en memoria (ni una lista de estrellas, ni una textura). Es sencillo
// de sostener sin castigar el rendimiento: por cada pixel de fondo son
// un par de senos/cosenos y un hash, nada de recorrer una lista.
use raylib::prelude::*;

const STAR_DENSITY: f32 = 0.0025; // fracción de celdas del cielo con estrella
const CELLS_PER_RADIAN: f32 = 140.0; // qué tan fina es la rejilla (más = estrellas más chicas y numerosas)

// Hash 2D barato: no es criptográfico ni necesita serlo, solo tiene
// que "verse" pseudo-aleatorio y ser siempre igual para la misma
// entrada.
fn hash2(x: f32, y: f32) -> f32 {
    let dot = x * 12.9898 + y * 78.233;
    let s = dot.sin() * 43758.5453;
    s - s.floor()
}

// Color de fondo (espacio + estrellas, si toca) para un rayo que no
// chocó con ningún planeta ni con el Sol.
pub fn sky_color(direction: &Vector3) -> Color {
    let space = Color::new(4, 4, 12, 255);

    // Coordenadas esféricas de la dirección del rayo EN EL MUNDO (ya
    // pasó por camera.basis_change en main.rs), no en espacio de
    // cámara: por eso las estrellas quedan fijas en el cielo cuando la
    // cámara orbita, en vez de girar pegadas a la pantalla.
    let azimuth = direction.z.atan2(direction.x);
    let elevation = direction.y.clamp(-1.0, 1.0).asin();

    let cell_x = (azimuth * CELLS_PER_RADIAN).floor();
    let cell_y = (elevation * CELLS_PER_RADIAN).floor();

    let star_chance = hash2(cell_x, cell_y);

    if star_chance > 1.0 - STAR_DENSITY {
        // Brillo variable por estrella, también determinístico (otro
        // hash de la misma celda, con un offset para no correlacionar
        // con star_chance).
        let brightness = 140.0 + hash2(cell_x + 0.5, cell_y + 0.5) * 115.0;
        Color::new(
            brightness as u8,
            brightness as u8,
            (brightness * 0.9 + 15.0).min(255.0) as u8,
            255,
        )
    } else {
        space
    }
}
