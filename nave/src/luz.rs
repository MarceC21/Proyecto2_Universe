// La luz de la escena
// Por ahora solo se modela una luz puntual
use raylib::prelude::*;

pub struct Luz {
    pub position: Vector3,
    pub color: Color,
    pub intensity: f32,
}

impl Luz {
    pub fn new(position: Vector3, color: Color, intensity: f32) -> Self {
        Self {position,color,intensity,}
    }
}