// El cubo (en realidad una caja alineada con los ejes, AABB)
//
// Una caja alineada con los ejes es la intersección de tres "losas"
// (slabs): una por eje, cada una limitada por dos planos paralelos.
//      min.x <= x <= max.x,   min.y <= y <= max.y,   min.z <= z <= max.z
//
// Para el rayo P(t) = O + t*D, en cada eje i el rayo entra a la losa en
//      t0 = (min_i - O_i) / D_i      t1 = (max_i - O_i) / D_i
// (se intercambian si D_i < 0, para que t0 <= t1).
//
// El rayo está dentro de la caja mientras esté dentro de LAS TRES losas
// a la vez, es decir, en el intervalo
//      t_near = max(t0_x, t0_y, t0_z)     t_far = min(t1_x, t1_y, t1_z)
// Hay impacto si t_near <= t_far (y t_far > 0: la caja no está detrás).
//
// La normal del impacto es la del plano que dio el t_near (la cara por
// la que el rayo ENTRA). Rayo paralelo a un eje (D_i ~ 0): no hay
// división; solo se comprueba que el origen esté dentro de esa losa.

use raylib::prelude::*;

use crate::ray_intersect::{Intersect, RayIntersect};

const EPSILON: f32 = 0.0001;

pub struct Cubo {
    pub min: Vector3,
    pub max: Vector3,
    center: Vector3,
    half_size: Vector3,
    cos_y: f32,
    sin_y: f32,

    pub albedo: Color,
    pub ka: f32,
    pub kd: f32,
    pub material: Option<usize>,
    pub uv_tile: Option<f32>,
}

impl Cubo {
    pub fn new(center: Vector3, size: Vector3, albedo: Color, ka: f32, kd: f32) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
            center,
            half_size: half,
            cos_y: 1.0,
            sin_y: 0.0,
            albedo,
            ka,
            kd,
            material: None,
            uv_tile: None,
        }
    }

    // La caja conserva dimensiones locales; min/max se actualizan como
    // límites globales para que también describan el volumen rotado.
    pub fn rotado_y(mut self, angle: f32) -> Self {
        let (sin_y, cos_y) = angle.sin_cos();
        self.cos_y = cos_y;
        self.sin_y = sin_y;
        let c = cos_y.abs();
        let s = sin_y.abs();
        let extent_x = c * self.half_size.x + s * self.half_size.z;
        let extent_z = s * self.half_size.x + c * self.half_size.z;
        let extent = Vector3::new(extent_x, self.half_size.y, extent_z);
        self.min = self.center - extent;
        self.max = self.center + extent;
        self
    }

    pub fn con_material(mut self, material: usize, uv_tile: Option<f32>) -> Self {
        self.material = Some(material);
        self.uv_tile = uv_tile;
        self
    }
}


impl RayIntersect for Cubo {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let (c, s) = (self.cos_y, self.sin_y);

        // Rotación inversa: rayo de mundo a espacio local de la caja.
        let offset = *ray_origin - self.center;
        // La mayoría de las cajas no están giradas: evita las dos rotaciones.
        let (origin, direction) = if s == 0.0 && c == 1.0 {
            (offset, *ray_direction)
        } else {
            (Vector3::new(c * offset.x - s * offset.z, offset.y, s * offset.x + c * offset.z),
             Vector3::new(c * ray_direction.x - s * ray_direction.z, ray_direction.y,
                 s * ray_direction.x + c * ray_direction.z))
        };

        let o = [origin.x, origin.y, origin.z];
        let d = [direction.x, direction.y, direction.z];
        let mn = [-self.half_size.x, -self.half_size.y, -self.half_size.z];
        let mx = [self.half_size.x, self.half_size.y, self.half_size.z];

        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;
        let (mut near_axis, mut near_sign) = (0usize, -1.0f32);
        let (mut far_axis, mut far_sign) = (0usize, 1.0f32);

        for i in 0..3 {
            if d[i].abs() < 1e-8 {
                if o[i] < mn[i] || o[i] > mx[i] {
                    return Intersect::empty();
                }
                continue;
            }

            let inv = 1.0 / d[i];
            let mut t0 = (mn[i] - o[i]) * inv;
            let mut t1 = (mx[i] - o[i]) * inv;
            let (mut s0, mut s1) = (-1.0f32, 1.0f32);

            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
                std::mem::swap(&mut s0, &mut s1);
            }
            if t0 > t_near {
                t_near = t0;
                near_axis = i;
                near_sign = s0;
            }
            if t1 < t_far {
                t_far = t1;
                far_axis = i;
                far_sign = s1;
            }
            if t_near > t_far {
                return Intersect::empty();
            }
        }

        if t_far < EPSILON {
            return Intersect::empty();
        }

        let (t, axis, sign) = if t_near > EPSILON {
            (t_near, near_axis, near_sign)
        } else {
            (t_far, far_axis, -far_sign)
        };

        let mut n = [0.0f32; 3];
        n[axis] = sign;
        let local_normal = Vector3::new(n[0], n[1], n[2]);
        let world_normal = Vector3::new(
            c * local_normal.x + s * local_normal.z,
            local_normal.y,
            -s * local_normal.x + c * local_normal.z,
        );
        let local_point = origin + direction * t;
        let world_point = *ray_origin + *ray_direction * t;

        let hit = Intersect::new(
            world_point, world_normal, t, self.albedo, self.ka, self.kd, false,
        );

        match self.material {
            None => hit,
            Some(id) => {
                let ((pu, pv), (eu, ev)) = match axis {
                    1 => (
                        (local_point.x - mn[0], local_point.z - mn[2]),
                        (mx[0] - mn[0], mx[2] - mn[2]),
                    ),
                    0 => (
                        (local_point.z - mn[2], mx[1] - local_point.y),
                        (mx[2] - mn[2], mx[1] - mn[1]),
                    ),
                    _ => (
                        (local_point.x - mn[0], mx[1] - local_point.y),
                        (mx[0] - mn[0], mx[1] - mn[1]),
                    ),
                };
                let (u, v) = match self.uv_tile {
                    Some(tile) => (pu / tile, pv / tile),
                    None => (pu / eu, pv / ev),
                };
                hit.with_material(id, u, v)
            }
        }
    }
}
