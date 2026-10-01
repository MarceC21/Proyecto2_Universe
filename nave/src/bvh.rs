// Jerarquía de cajas envolventes para geometría estática. Sin dependencias nuevas.
// Construir una vez; reconstruir si se mueve alguno de los cubos contenidos.
use raylib::prelude::*;
use crate::cubo::Cubo;
use crate::ray_intersect::{Intersect, RayIntersect};

struct Nodo {
    min: [f32; 3],
    max: [f32; 3],
    inicio: usize,
    fin: usize,
    hijos: Option<(usize, usize)>,
}

pub struct Bvh {
    cubos: Vec<Cubo>,
    indices: Vec<usize>,
    nodos: Vec<Nodo>,
}

fn componentes(v: &Vector3) -> [f32; 3] { [v.x, v.y, v.z] }

impl Bvh {
    pub fn new(cubos: Vec<Cubo>) -> Self {
        let n = cubos.len();
        let mut bvh = Self { cubos, indices: (0..n).collect(), nodos: Vec::with_capacity(n * 2) };
        if n > 0 { bvh.construir(0, n); }
        bvh
    }

    fn construir(&mut self, inicio: usize, fin: usize) -> usize {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        let mut cmin = [f32::INFINITY; 3];
        let mut cmax = [f32::NEG_INFINITY; 3];
        for &i in &self.indices[inicio..fin] {
            let a = componentes(&self.cubos[i].min);
            let b = componentes(&self.cubos[i].max);
            for eje in 0..3 {
                // Margen conservador para cajas giradas y errores de redondeo.
                min[eje] = min[eje].min(a[eje] - 0.0001);
                max[eje] = max[eje].max(b[eje] + 0.0001);
                let c = (a[eje] + b[eje]) * 0.5;
                cmin[eje] = cmin[eje].min(c);
                cmax[eje] = cmax[eje].max(c);
            }
        }
        let id = self.nodos.len();
        self.nodos.push(Nodo { min, max, inicio, fin, hijos: None });
        if fin - inicio > 4 {
            let mut eje = 0;
            for k in 1..3 {
                if cmax[k] - cmin[k] > cmax[eje] - cmin[eje] { eje = k; }
            }
            let cubos = &self.cubos;
            self.indices[inicio..fin].sort_unstable_by(|&a, &b| {
                let ca = componentes(&cubos[a].min)[eje] + componentes(&cubos[a].max)[eje];
                let cb = componentes(&cubos[b].min)[eje] + componentes(&cubos[b].max)[eje];
                ca.total_cmp(&cb).then(a.cmp(&b))
            });
            let mitad = inicio + (fin - inicio) / 2;
            let izquierda = self.construir(inicio, mitad);
            let derecha = self.construir(mitad, fin);
            self.nodos[id].hijos = Some((izquierda, derecha));
        }
        id
    }

    fn entrada(&self, id: usize, o: &[f32; 3], d: &[f32; 3], limite: f32) -> Option<f32> {
        let nodo = &self.nodos[id];
        let (mut cerca, mut lejos) = (0.0f32, limite);
        for k in 0..3 {
            if d[k] == 0.0 {
                if o[k] < nodo.min[k] || o[k] > nodo.max[k] { return None; }
            } else {
                let a = (nodo.min[k] - o[k]) / d[k];
                let b = (nodo.max[k] - o[k]) / d[k];
                cerca = cerca.max(a.min(b));
                lejos = lejos.min(a.max(b));
                if cerca > lejos { return None; }
            }
        }
        Some(cerca)
    }

    pub fn intersectar(&self, origen: &Vector3, direccion: &Vector3, limite: f32) -> Intersect {
        let mut hit = Intersect::empty();
        if self.nodos.is_empty() { return hit; }
        let mut mejor = limite;
        let mut indice = usize::MAX;
        self.visitar(0, origen, direccion, &componentes(origen), &componentes(direccion),
            &mut mejor, &mut indice, &mut hit);
        hit
    }

    // Cualquier obstáculo basta para la sombra; no buscamos el más cercano.
    // omitir identifica el cubo emisor en el orden original de construcción.
    pub fn ocluido(&self, origen: &Vector3, direccion: &Vector3,
                  limite: f32, omitir: Option<usize>) -> bool {
        if self.nodos.is_empty() || limite <= 0.0 { return false; }
        self.visibilidad(0,origen,direccion,&componentes(origen),&componentes(direccion),limite,omitir)
    }
    fn visibilidad(&self,id:usize,origen:&Vector3,direccion:&Vector3,
                   o:&[f32;3],d:&[f32;3],limite:f32,omitir:Option<usize>) -> bool {
        if self.entrada(id,o,d,limite).is_none() { return false; }
        let nodo=&self.nodos[id];
        if let Some((a,b))=nodo.hijos {
            let orden=match (self.entrada(a,o,d,limite),self.entrada(b,o,d,limite)) {
                (Some(x),Some(y)) if y<x => [b,a], _ => [a,b],
            };
            self.visibilidad(orden[0],origen,direccion,o,d,limite,omitir) ||
            self.visibilidad(orden[1],origen,direccion,o,d,limite,omitir)
        } else {
            self.indices[nodo.inicio..nodo.fin].iter().any(|&i|
                Some(i)!=omitir && self.cubos[i].ocluye(origen,direccion,limite))
        }
    }

    fn visitar(&self, id: usize, origen: &Vector3, direccion: &Vector3,
        o: &[f32; 3], d: &[f32; 3], mejor: &mut f32, indice: &mut usize, hit: &mut Intersect,
    ) {
        if self.entrada(id, o, d, *mejor).is_none() { return; }
        let nodo = &self.nodos[id];
        if let Some((a, b)) = nodo.hijos {
            let ta = self.entrada(a, o, d, *mejor);
            let tb = self.entrada(b, o, d, *mejor);
            let orden = match (ta, tb) {
                (Some(x), Some(y)) if y < x => [b, a],
                _ => [a, b],
            };
            for hijo in orden {
                self.visitar(hijo, origen, direccion, o, d, mejor, indice, hit);
            }
        } else {
            for &i in &self.indices[nodo.inicio..nodo.fin] {
                let candidato = self.cubos[i].ray_intersect(origen, direccion);
                if candidato.is_intersecting && (candidato.distance < *mejor
                    || (hit.is_intersecting && candidato.distance == *mejor && i < *indice)) {
                    *mejor = candidato.distance;
                    *indice = i;
                    *hit = candidato;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coincide_con_recorrido_completo() {
        let mut cubos = Vec::new();
        for i in 0..80 {
            cubos.push(Cubo::new(
                Vector3::new((i % 10) as f32 - 5.0, (i % 3) as f32 * 0.8, (i / 10) as f32 - 4.0),
                Vector3::new(0.7, 0.6, 0.5), Color::WHITE, 0.3, 0.7,
            ).rotado_y(i as f32 * 0.17).con_material(i, Some(1.2)));
        }
        let bvh = Bvh::new(cubos);
        for i in 0..1200 {
            let o = Vector3::new((i % 17) as f32 - 8.0, (i % 7) as f32 * 0.5, 8.0);
            let d = Vector3::new(((i * 7) % 23) as f32 * 0.07 - 0.8, -0.15, -1.0).normalize();
            comprobar(&bvh, o, d, f32::INFINITY);
            comprobar(&bvh, o, d, 3.0);
        }
        // Rayos paralelos a caras, desde dentro de cajas y fuera de la escena.
        for c in &bvh.cubos {
            let o = (c.min + c.max) * 0.5;
            for d in [Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, -1.0)] {
                comprobar(&bvh, o, d, f32::INFINITY);
            }
        }
        assert!(!Bvh::new(Vec::new()).intersectar(&Vector3::zero(), &Vector3::new(0.0, 0.0, 1.0), 10.0).is_intersecting);
    }

    #[test]
    fn sombra_finita_emisor_y_caja_girada() {
        let cajas=vec![
            Cubo::new(Vector3::new(0.0,0.0,3.0),Vector3::new(1.0,2.0,1.0),Color::WHITE,0.3,0.7).rotado_y(0.45),
            Cubo::new(Vector3::new(0.0,0.0,8.0),Vector3::new(1.0,1.0,1.0),Color::WHITE,0.3,0.7),
        ];
        let bvh=Bvh::new(cajas);
        let o=Vector3::zero(); let d=Vector3::new(0.0,0.0,1.0);
        assert!(!bvh.ocluido(&o,&d,1.0,None)); // obstáculo detrás de la luz
        assert!(bvh.ocluido(&o,&d,5.0,None));
        assert!(!bvh.ocluido(&o,&d,5.0,Some(0))); // no autoocluir emisor
        assert!(bvh.ocluido(&o,&d,10.0,Some(0))); // sí conservar otros obstáculos
        assert!(!bvh.ocluido(&Vector3::new(4.0,0.0,0.0),&d,10.0,None));
        assert!(bvh.ocluido(&Vector3::new(0.0,0.0,3.0),&d,0.1,None));
        for i in 0..400 {
            let o=Vector3::new((i%20) as f32*0.3-3.0, (i/20) as f32*0.12-1.2, -2.0);
            let d=Vector3::new(0.05,-0.02,1.0).normalize();
            for limite in [1.0,4.0,12.0] {
                // Referencia independiente: recorrido completo y cálculo de impacto.
                let esperado=bvh.cubos.iter().any(|c| {
                    let h=c.ray_intersect(&o,&d);
                    h.is_intersecting && h.distance<limite
                });
                assert_eq!(bvh.ocluido(&o,&d,limite,None),esperado);
            }
        }
    }

    fn comprobar(bvh: &Bvh, o: Vector3, d: Vector3, limite: f32) {
        let mut esperado = Intersect::empty();
        let mut mejor = limite;
        for c in &bvh.cubos {
            let hit = c.ray_intersect(&o, &d);
            if hit.is_intersecting && hit.distance < mejor {
                mejor = hit.distance;
                esperado = hit;
            }
        }
        let real = bvh.intersectar(&o, &d, limite);
        assert_eq!(real.is_intersecting, esperado.is_intersecting);
        if real.is_intersecting {
            assert_eq!(real.distance, esperado.distance);
            assert_eq!(real.material, esperado.material);
            assert_eq!((real.u, real.v), (esperado.u, esperado.v));
        }
    }
}
