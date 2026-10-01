// Colisiones de primera persona en el plano XZ, independientes de la cámara.
// El contorno de la nave es convexo y sus vértices están en orden antihorario.

pub struct Obstaculo {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

// Caja girada en XZ. Se usa la misma orientación que Cubo::rotado_y.
pub struct CajaOrientada {
    pub centro_x: f32,
    pub centro_z: f32,
    pub mitad_x: f32,
    pub mitad_z: f32,
    pub angulo_y: f32,
}

impl CajaOrientada {
    fn bloqueado(&self, x: f32, z: f32, radio: f32) -> bool {
        let (c, s) = (self.angulo_y.cos(), self.angulo_y.sin());
        let (dx, dz) = (x - self.centro_x, z - self.centro_z);
        let (lx, lz) = (c * dx - s * dz, s * dx + c * dz);
        let ex = lx - lx.clamp(-self.mitad_x, self.mitad_x);
        let ez = lz - lz.clamp(-self.mitad_z, self.mitad_z);
        ex * ex + ez * ez < radio * radio
    }
}

struct Limite {
    nx: f32,
    nz: f32,
    minimo: f32,
}

pub struct Colisiones {
    limites: Vec<Limite>,
    cajas: Vec<CajaOrientada>,
}

impl Colisiones {
    // Se calcula una sola vez. Cada límite se desplaza hacia el interior
    // media pared, porque los vértices describen el centro de sus bloques.
    pub fn new(contorno: &[(f32, f32)], grosor_pared: f32) -> Self {
        assert!(contorno.len() >= 3);
        assert!(grosor_pared >= 0.0);
        let mut limites = Vec::with_capacity(contorno.len());
        for i in 0..contorno.len() {
            let (ax, az) = contorno[i];
            let (bx, bz) = contorno[(i + 1) % contorno.len()];
            let (dx, dz) = (bx - ax, bz - az);
            let longitud = (dx * dx + dz * dz).sqrt();
            assert!(longitud > 0.0);
            let (nx, nz) = (-dz / longitud, dx / longitud);
            limites.push(Limite {
                nx,
                nz,
                minimo: nx * ax + nz * az + grosor_pared * 0.5,
            });
        }
        Self { limites, cajas: Vec::new() }
    }

    pub fn con_cajas(mut self, cajas: Vec<CajaOrientada>) -> Self {
        self.cajas = cajas;
        self
    }

    pub fn bloqueado(&self, x: f32, z: f32, radio: f32, obstaculos: &[Obstaculo]) -> bool {
        // El círculo del jugador debe quedar dentro de TODOS los lados.
        // También se cierra el paso por ventanas, aunque aún no tengan vidrio.
        if self.limites.iter().any(|p| {
            p.nx * x + p.nz * z < p.minimo + radio
        }) {
            return true;
        }
        if self.cajas.iter().any(|caja| caja.bloqueado(x, z, radio)) {
            return true;
        }
        // Círculo contra rectángulo: permite rodear las esquinas de la mesa.
        obstaculos.iter().any(|o| {
            let cercano_x = x.clamp(o.min_x, o.max_x);
            let cercano_z = z.clamp(o.min_z, o.max_z);
            let dx = x - cercano_x;
            let dz = z - cercano_z;
            dx * dx + dz * dz < radio * radio
        })
    }

    pub fn mover(
        &self, x: f32, z: f32, dx: f32, dz: f32,
        radio: f32, obstaculos: &[Obstaculo],
    ) -> (f32, f32) {
        assert!(radio > 0.0);
        // Subpasos para no saltar obstáculos en un cuadro lento.
        let distancia = (dx * dx + dz * dz).sqrt();
        let pasos = (distancia / (radio * 0.5)).ceil().max(1.0) as usize;
        let (sx, sz) = (dx / pasos as f32, dz / pasos as f32);
        let (mut x, mut z) = (x, z);
        for _ in 0..pasos {
            // Resolver por ejes permite deslizarse junto a las paredes.
            if !self.bloqueado(x + sx, z, radio, obstaculos) {
                x += sx;
            }
            if !self.bloqueado(x, z + sz, radio, obstaculos) {
                z += sz;
            }
        }
        (x, z)
    }
}
