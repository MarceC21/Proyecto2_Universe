// Iluminación local sin dependencias: emisores obtenidos de la geometría una vez.
// Rejilla espacial para consultar solo luces cercanas, hasta cuatro por impacto.

use raylib::prelude::*;
use crate::cubo::Cubo;
use crate::material::*;

// La luz que esta siempre presente
pub const AMBIENTE: f32 = 1.22;

// Máximo de luces locales que se pueden usar en un impacto
pub const MAX_LUCES_LOCALES: usize = 3;
const CELDA: f32 = 3.0;

pub struct Luz { pub position: Vector3, pub color: Color, pub intensity: f32 }
impl Luz {
    pub fn new(position:Vector3,color:Color,intensity:f32)->Self { Self {position,color,intensity} }
}
struct Emisor {
    centro: Vector3, ejes: [Vector3;3], mitad: [f32;3], cara: usize,
    radio: f32, intensidad: f32, color: [f32;3], cubo: usize, puntual: bool,
}
#[derive(Clone,Copy)]
pub struct MuestraLuz {
    pub posicion: Vector3,
    pub energia: [f32;3], pub cubo: usize,
}
pub struct Iluminacion {
    pub sol: Luz,
    emisores: Vec<Emisor>,
    celdas: Vec<Vec<usize>>,
    minimo: [i32;3], dimensiones: [usize;3],
}
impl Iluminacion {
    pub fn new(sol:Luz,cubos:&[Cubo])->Self {
        let mut emisores=Vec::new();
        for (id,cubo) in cubos.iter().enumerate() {
            let Some(material)=cubo.material else { continue; };
            let (centro,h,ejes)=cubo.marco_luz();
            let mitad=[h.x,h.y,h.z];
            let cara=(0..3).min_by(|&a,&b| mitad[a].total_cmp(&mitad[b])).unwrap();
            let area=4.0*mitad[(cara+1)%3]*mitad[(cara+2)%3];
            let (color,intensidad,radio,puntual)=match material {
                PANTALLA|RADAR_CABINA|SISTEMAS_CABINA|MONITOR_COMANDO =>
                    ([0.26,0.58,1.0],0.72,4.2,false),
                POLIMERO|BOTONERA_CABINA|CONTROLES_COMANDO|PANEL_ESCLUSA =>
                    ([0.24,0.58,1.0],0.32,2.0,false),
                INDICADOR if area>0.045 => ([0.22,0.72,1.0],0.65,2.7,false),
                INDICADOR => ([0.22,0.72,1.0],0.35,0.85,false),
                LUZ_LABORATORIO => ([1.0,0.82,0.48],2.4,4.0,true),
                BOTON_ROJO => ([1.0,0.12,0.06],0.40,0.85,false),
                _ => continue,
            };
            emisores.push(Emisor {centro,ejes,mitad,cara,radio,intensidad,color,cubo:id,puntual});
        }
        let mut minimo=[i32::MAX;3]; let mut maximo=[i32::MIN;3];
        let mut rangos=Vec::with_capacity(emisores.len());
        for e in &emisores {
            let c=&cubos[e.cubo];
            let mn=[c.min.x-e.radio,c.min.y-e.radio,c.min.z-e.radio];
            let mx=[c.max.x+e.radio,c.max.y+e.radio,c.max.z+e.radio];
            let lo=mn.map(|v|(v/CELDA).floor() as i32);
            let hi=mx.map(|v|(v/CELDA).floor() as i32);
            for k in 0..3 { minimo[k]=minimo[k].min(lo[k]);maximo[k]=maximo[k].max(hi[k]); }
            rangos.push((lo,hi));
        }
        if emisores.is_empty() { minimo=[0;3];maximo=[0;3]; }
        let dimensiones=std::array::from_fn(|k|(maximo[k]-minimo[k]+1) as usize);
        let mut celdas=vec![Vec::new();dimensiones.iter().product()];
        for (id,(lo,hi)) in rangos.into_iter().enumerate() {
            for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] {
                let idx=((z-minimo[2]) as usize*dimensiones[1]+(y-minimo[1]) as usize)*dimensiones[0]+(x-minimo[0]) as usize;
                celdas[idx].push(id);
            }}}
        }
        Self {sol,emisores,celdas,minimo,dimensiones}
    }

    pub fn cercanas(&self,p:Vector3,normal:Vector3)->[Option<MuestraLuz>;MAX_LUCES_LOCALES] {
        let mut resultado=[None;MAX_LUCES_LOCALES];
        let xyz=[p.x,p.y,p.z]; let mut celda=[0usize;3];
        for k in 0..3 {
            let n=(xyz[k]/CELDA).floor() as i32-self.minimo[k];
            if n<0 || n>=self.dimensiones[k] as i32 { return resultado; }
            celda[k]=n as usize;
        }
        let idx=(celda[2]*self.dimensiones[1]+celda[1])*self.dimensiones[0]+celda[0];
        let mut pesos=[0.0f32;MAX_LUCES_LOCALES];
        for &id in &self.celdas[idx] {
            let e=&self.emisores[id];
            let offset=p-e.centro;
            let mut posicion=e.centro;
            // Pantallas/tiras: aproximación de superficie con un punto cercano.
            // Lámpara del laboratorio: foco fijo para sombras estables de frascos.
            for k in 0..3 {
                let local=offset.dot(e.ejes[k]);
                let v=if k==e.cara {
                    if local>=0.0 { e.mitad[k]+0.035 } else { -e.mitad[k]-0.035 }
                } else if e.puntual { 0.0 } else { local.clamp(-e.mitad[k],e.mitad[k]) };
                posicion=posicion+e.ejes[k]*v;
            }
            let delta=posicion-p;
            let d2=delta.dot(delta);
            if d2>=e.radio*e.radio || d2<0.000001 { continue; }
            let distancia = d2.sqrt();
            let direccion = delta * (1.0 / distancia);
            let lambert=normal.dot(direccion).max(0.0);
            if lambert<=0.0 { continue; }
            let borde=1.0-d2/(e.radio*e.radio);
            let atenuacion=e.intensidad*borde*borde/(1.0+0.20*d2);
            let peso=atenuacion*lambert;
            let menor=(0..MAX_LUCES_LOCALES).min_by(|&a,&b|pesos[a].total_cmp(&pesos[b])).unwrap();
            if peso<=pesos[menor] { continue; }
            pesos[menor]=peso;
            resultado[menor]=Some(MuestraLuz {posicion,
                energia:e.color.map(|c|c*atenuacion),cubo:e.cubo});
        }
        resultado
    }
}
