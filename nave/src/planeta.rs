// Un planeta 

// Geometría/material: la esfera + los parámetros que describen cómo se mueve con el tiempo: 
// a qué distancia orbita, a qué velocidad, y a qué velocidad gira sobre su propio eje.

// Separarlo así es lo que permite que "mover el planeta"

use raylib::prelude::*;

use crate::esfera::Esfera;

pub struct OrbitalParameters {
    pub distance: f32, // radio de la órbita alrededor del Sol
    pub orbital_speed: f32, // velocidad angular orbital, en radianes/segundo
    pub orbital_angle: f32, // ángulo actual sobre la órbita

    pub rotation_speed: f32, // velocidad de rotación propia, en radianes/segundo
    pub rotation_angle: f32, // ángulo actual de rotación propia
}

impl OrbitalParameters {
    pub fn new(distance: f32, orbital_speed: f32, rotation_speed: f32) -> Self {
        Self {
            distance,
            orbital_speed,
            orbital_angle: 0.0,
            rotation_speed,
            rotation_angle: 0.0,
        }
    }
}

pub struct Planet {
    pub name: &'static str,
    pub sphere: Esfera,
    pub orbital: OrbitalParameters,
}

impl Planet {
    pub fn new(name: &'static str, sphere: Esfera, orbital: OrbitalParameters) -> Self {
        Self {
            name,
            sphere,
            orbital,
        }
    }

    // Avanza la órbita y la rotación propia según cuánto TIEMPO REAL
    // pasó desde el frame anterior (delta_time)
    pub fn update(&mut self, delta_time: f32, sun_center: Vector3) {
        self.orbital.orbital_angle += self.orbital.orbital_speed * delta_time;
        self.orbital.rotation_angle += self.orbital.rotation_speed * delta_time;

        // Se mantienen los ángulos acotados a una vuelta completa
        let two_pi = std::f32::consts::TAU;
        self.orbital.orbital_angle %= two_pi;
        self.orbital.rotation_angle %= two_pi;

        // Órbita circular sobre el plano XZ ("visto desde arriba",
        // como en las referencias): el Sol queda fijo en sun_center y
        // cada planeta se mueve en círculo a su propia distancia. El
        // Sol mismo se modela como un Planet con distance = 0, así que
        // esta misma fórmula también lo deja quieto en su sitio.
        self.sphere.center = sun_center
            + Vector3::new(
                self.orbital.distance * self.orbital.orbital_angle.cos(),
                0.0,
                self.orbital.distance * self.orbital.orbital_angle.sin(),
            );

    }
}
