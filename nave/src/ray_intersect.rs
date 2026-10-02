// Lógica del rayo para el raytracing
// Un rayo es una semirrecta: P(t) = O + t*D

use raylib::prelude::*;

// Representa un rayo en 3D
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    // El origen es el punto O de donde sale el rayo (en un rayo primario
    // es la posición de la cámara). La dirección se normaliza siempre,
    // así que ||D|| = 1.
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    // Cualquier punto sobre el rayo: P(t) = O + t*D
    pub fn at(&self, t: f32) -> Vector3 {
        self.origin + self.direction * t
    }
}

// Intersección de un rayo con un objeto: punto, normal, color y parámetros de iluminación.
#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub distance: f32,
    pub point: Vector3,
    pub normal: Vector3,
    pub color: Color,   // albedo: el color propio de la superficie
    pub ka: f32,        // coeficiente ambiental (luz indirecta, siempre presente)
    pub kd: f32,        // coeficiente difuso (cuánto rebota la luz directa)
    pub emissive: bool, // true si el objeto emite su propia luz (el Sol)
    pub is_intersecting: bool,

    // Objetos con material completo (ver material.rs): índice en el
    // catálogo y coordenadas de textura. Con `None` se usan color/ka/kd.
    pub material: Option<usize>,
    pub u: f32,
    pub v: f32,
}

impl Intersect {
    pub fn new(
        point: Vector3,
        normal: Vector3,
        distance: f32,
        color: Color,
        ka: f32,
        kd: f32,
        emissive: bool,
    ) -> Self {
        Self {
            point,
            normal,
            distance,
            color,
            ka,
            kd,
            emissive,
            is_intersecting: true,
            material: None,
            u: 0.0,
            v: 0.0,
        }
    }

    // Asocia el impacto a un material del catálogo y a un punto UV.
    pub fn with_material(mut self, material: usize, u: f32, v: f32) -> Self {
        self.material = Some(material);
        self.u = u;
        self.v = v;
        self
    }

    // "No hay solución real": el rayo pasa de largo sin tocar el objeto
    pub fn empty() -> Self {
        Self {
            point: Vector3::zero(),
            normal: Vector3::zero(),
            distance: f32::INFINITY,
            color: Color::BLACK,
            ka: 0.0,
            kd: 0.0,
            emissive: false,
            is_intersecting: false,
            material: None,
            u: 0.0,
            v: 0.0,
        }
    }
}

// Cualquier objeto de la escena que se pueda intersectar con un rayo
// implementa este trait (esferas, planos, cajas, etc.)
pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect;
}
