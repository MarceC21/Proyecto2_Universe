// La esfera
// Una esfera de centro C y radio r es el conjunto de puntos que están
// exactamente a distancia r del centro: ||P - C||^2 = r^2
//
// Sustituyendo el rayo P(t) = O + t*D se obtiene una ecuación cuadrática
// en t:
//      a*t^2 + b*t + c = 0
// con
//      a = D . D
//      b = 2 * D . (O - C)
//      c = ||O - C||^2 - r^2

use raylib::prelude::*;

use crate::ray_intersect::{Intersect, RayIntersect};

pub struct Esfera {
    pub center: Vector3,
    pub radius: f32,

    // Material: lo que el sombreador (main.rs) consulta cuando un rayo
    // golpea esta esfera.
    pub albedo: Color, // color propio de la superficie
    pub ka: f32,       // coeficiente ambiental
    pub kd: f32,       // coeficiente difuso
    pub emissive: bool, // true solo para el Sol: no recibe luz, la ES
}

impl Esfera {
    pub fn new(
        center: Vector3,
        radius: f32,
        albedo: Color,
        ka: f32,
        kd: f32,
        emissive: bool,
    ) -> Self {
        Self {
            center,
            radius,
            albedo,
            ka,
            kd,
            emissive,
        }
    }
}

impl RayIntersect for Esfera {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        // O - C
        let oc = *ray_origin - self.center;

        let a = ray_direction.dot(*ray_direction);
        let b = 2.0 * ray_direction.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;

        // El discriminante dice, por sí solo, la relación geométrica
        // entre el rayo y la esfera.
        let discriminant = b * b - 4.0 * a * c;

        // Delta < 0: no hay solución real, el rayo pasa de largo.
        if discriminant < 0.0 {
            return Intersect::empty();
        }

        let sqrt_discriminant = discriminant.sqrt();

        let t1 = (-b - sqrt_discriminant) / (2.0 * a);
        let t2 = (-b + sqrt_discriminant) / (2.0 * a);

        // De las dos raíces se toma la menor que sea positiva: es la
        // cara de la esfera que mira hacia la cámara.
        let t = if t1 > 0.0001 {
            t1
        } else if t2 > 0.0001 {
            t2
        } else {
            return Intersect::empty();
        };

        let point = *ray_origin + *ray_direction * t;
        let normal = (point - self.center).normalize();

        Intersect::new(point, normal, t, self.albedo, self.ka, self.kd, self.emissive)
    }
}
