// Fondo estelar procedural.
//
// Este se calcula para cada rayo que no choca con la nave

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
