// Colisiones de primera persona en el plano XZ, independientes de la cámara.
// El contorno de la nave es convexo y sus vértices están en orden antihorario.

#[derive(Clone, Copy)]
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
        self.cajas.extend(cajas);
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
            if !self.bloqueado(x + sx, z, radio, obstaculos) {
                x += sx;
            }
            if !self.bloqueado(x, z + sz, radio, obstaculos) {
                z += sz;
            }
        }
        (x, z)
    }

    pub fn mover_con_compuerta(
        &self, x: f32, z: f32, dx: f32, dz: f32,
        radio: f32, obstaculos: &[Obstaculo], puerta_abierta: bool,
        puerta_lateral_abierta: bool, afuera: &mut bool, en_espacio: &mut bool,
    ) -> (f32, f32) {
        assert!(radio > 0.0);
        let distancia = (dx * dx + dz * dz).sqrt();
        let pasos = (distancia / (radio * 0.5)).ceil().max(1.0) as usize;
        let (sx, sz) = (dx / pasos as f32, dz / pasos as f32);
        let (mut x, mut z) = (x, z);
        for _ in 0..pasos {
            let anterior_x = x;
            if !self.bloqueado_compuerta(x + sx, z, x, z, radio,
                obstaculos, puerta_abierta, puerta_lateral_abierta, *afuera, *en_espacio) {
                x += sx;
                self.actualizar_lado_puerta(x, z, anterior_x, radio,
                    puerta_lateral_abierta, afuera, en_espacio);
            }
            let anterior_x = x;
            if !self.bloqueado_compuerta(x, z + sz, x, z, radio,
                obstaculos, puerta_abierta, puerta_lateral_abierta, *afuera, *en_espacio) {
                z += sz;
                self.actualizar_lado_puerta(x, z, anterior_x, radio,
                    puerta_lateral_abierta, afuera, en_espacio);
            }
        }
        (x, z)
    }

    fn dentro_nave(&self, x: f32, z: f32, radio: f32) -> bool {
        self.limites.iter().all(|p| p.nx * x + p.nz * z >= p.minimo + radio)
    }

    fn actualizar_lado_puerta(
        &self, x: f32, z: f32, anterior_x: f32, radio: f32,
        puerta_lateral_abierta: bool, afuera: &mut bool, en_espacio: &mut bool,
    ) {
        if *afuera && self.dentro_nave(x, z, radio) {
            *afuera = false;
            *en_espacio = false;
        } else if !*afuera && z > 8.15 && x.abs() < 2.2 {
            *afuera = true;
            *en_espacio = false;
        } else if *afuera && !*en_espacio && puerta_lateral_abierta
            && (z - crate::pasillo::PUERTA_Z).abs() < crate::pasillo::PUERTA_ANCHO * 0.5 - radio
            && anterior_x < crate::pasillo::ANCHO_INTERIOR * 0.5
                + crate::escena::WALL_THICKNESS + radio
            && x >= crate::pasillo::ANCHO_INTERIOR * 0.5
                + crate::escena::WALL_THICKNESS + radio {
            *en_espacio = true;
        } else if *afuera && *en_espacio && puerta_lateral_abierta
            && (z - crate::pasillo::PUERTA_Z).abs() < crate::pasillo::PUERTA_ANCHO * 0.5 - radio
            && anterior_x > crate::pasillo::ANCHO_INTERIOR * 0.5 - radio
            && x <= crate::pasillo::ANCHO_INTERIOR * 0.5 - radio {
            *en_espacio = false;
        }
    }

    fn bloqueado_compuerta(
        &self, x: f32, z: f32, anterior_x: f32, anterior_z: f32, radio: f32,
        obstaculos: &[Obstaculo], puerta_abierta: bool, puerta_lateral_abierta: bool,
        afuera: bool, en_espacio: bool,
    ) -> bool {
        // Las puertas solo cambian los limites del recinto. Las cajas de
        // cabina, sillones y laboratorio siguen siendo solidas en ambos modos.
        if self.cajas.iter().any(|caja| caja.bloqueado(x, z, radio)) {
            return true;
        }
        if afuera {
            let cruce_retorno = puerta_abierta && anterior_z > 7.5 && z <= 7.5
                && x.abs() < 1.90 - radio;
            if self.dentro_nave(x, z, radio) && !cruce_retorno { return true; }
            if en_espacio {
                if x.abs() > 40.0 || z.abs() > 40.0 { return true; }
            } else {
                let limite_x = crate::pasillo::ANCHO_INTERIOR * 0.5 - radio;
                let vano_lateral = puerta_lateral_abierta
                    && (z - crate::pasillo::PUERTA_Z).abs()
                        < crate::pasillo::PUERTA_ANCHO * 0.5 - radio
                    && x > limite_x
                    && x <= crate::pasillo::ANCHO_INTERIOR * 0.5
                        + crate::escena::WALL_THICKNESS + radio * 1.5;
                if (x < -limite_x || x > limite_x) && !vano_lateral
                    || z > crate::pasillo::FIN_Z - radio {
                    return true;
                }
            }
        } else if self.limites.iter().enumerate().any(|(i, p)| {
            let fuera = p.nx * x + p.nz * z < p.minimo + radio;
            let vano = i == 4 && puerta_abierta && x.abs() < 1.90 - radio
                && anterior_x.abs() < 1.90 - radio;
            fuera && !vano
        }) {
            return true;
        }
        obstaculos.iter().any(|o| {
            let cercano_x = x.clamp(o.min_x, o.max_x);
            let cercano_z = z.clamp(o.min_z, o.max_z);
            let dx = x - cercano_x;
            let dz = z - cercano_z;
            dx * dx + dz * dz < radio * radio
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::Vector3;

    #[test]
    fn movimiento_con_puertas_respeta_cajas_de_muebles() {
        let contorno = crate::escena::contorno_nave();
        let cajas = crate::cabina::obstaculos_cabina(&contorno, crate::escena::WALL_THICKNESS)
            .into_iter().chain(crate::laboratorio::obstaculos());
        let colisiones = Colisiones::new(&contorno, crate::escena::WALL_THICKNESS)
            .con_cajas(cajas.collect());
        // Cada centro estaba protegido en el camino original y debe estarlo
        for caja in &colisiones.cajas {
            for abierta in [false, true] {
                assert!(colisiones.bloqueado_compuerta(
                    caja.centro_x, caja.centro_z, caja.centro_x, caja.centro_z,
                    crate::escena::PLAYER_RADIUS, &[], abierta, abierta, false, false));
            }
        }
    }

    #[test]
    fn restaura_muebles_sin_cerrar_el_paso_trasero() {
        let contorno = crate::escena::contorno_nave();
        let colisiones = Colisiones::new(&contorno, crate::escena::WALL_THICKNESS)
            .con_cajas(crate::cabina::obstaculos_cabina(&contorno, crate::escena::WALL_THICKNESS))
            .con_cajas(crate::laboratorio::obstaculos());
        let mut puerta = crate::compuerta::Compuerta::new();
        puerta.actualizar(1.3, Some(Vector3::new(0.0, 1.65, crate::escena::ROOM_BACK_Z)));
        assert!(puerta.abierta());
        let mut obstaculos = crate::escena::obstaculos();
        obstaculos.extend(puerta.obstaculos());
        let (mut afuera, mut espacio) = (false, false);
        let (x, z) = colisiones.mover_con_compuerta(0.0, 6.0, 0.0, 4.0,
            crate::escena::PLAYER_RADIUS, &obstaculos, true, false, &mut afuera, &mut espacio);
        assert!(afuera && z > 9.5);
        let (_, z) = colisiones.mover_con_compuerta(x, z, 0.0, -4.0,
            crate::escena::PLAYER_RADIUS, &obstaculos, true, false, &mut afuera, &mut espacio);
        assert!(!afuera && z < 6.5);
    }

    #[test]
    fn cruza_y_regresa_por_compuerta_lateral_abierta() {
        let contorno = crate::escena::contorno_nave();
        let colisiones = Colisiones::new(&contorno, crate::escena::WALL_THICKNESS);
        let mut puerta = crate::pasillo::CompuertaEspacio::new();
        let jugador = Vector3::new(
            crate::pasillo::ANCHO_INTERIOR * 0.5 + crate::escena::WALL_THICKNESS * 0.5,
            crate::escena::EYE_HEIGHT,
            crate::pasillo::PUERTA_Z,
        );
        for _ in 0..100 { puerta.actualizar(1.0 / 60.0, Some(jugador)); }
        assert!(puerta.abierta());

        let mut obstaculos = crate::pasillo::obstaculos();
        obstaculos.push(puerta.obstaculo());
        let mut afuera = true;
        let mut en_espacio = false;
        let (x, z) = colisiones.mover_con_compuerta(
            0.0, crate::pasillo::PUERTA_Z, 5.0, 0.0,
            crate::escena::PLAYER_RADIUS, &obstaculos, true, puerta.abierta(),
            &mut afuera, &mut en_espacio,
        );
        assert!(x > crate::pasillo::ANCHO_INTERIOR * 0.5);
        assert!(en_espacio);

        let (x, _) = colisiones.mover_con_compuerta(
            x, z, -5.0, 0.0, crate::escena::PLAYER_RADIUS,
            &obstaculos, true, puerta.abierta(), &mut afuera, &mut en_espacio,
        );
        assert!(x < crate::pasillo::ANCHO_INTERIOR * 0.5);
        assert!(!en_espacio);
    }
}
