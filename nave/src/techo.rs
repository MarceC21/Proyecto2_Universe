// Techo anular y vidrio panorámico. Todo el contorno se prepara una vez.
// El vidrio se compone sobre el color trazado; no añade refracción ni rebotes.
use std::sync::OnceLock;
use raylib::prelude::*;
use crate::cubo::Cubo;
use crate::material::{PISO_NAVE, REFUERZO_NAVE};

pub const ANCHO_BORDE: f32 = 1.10;
const GROSOR: f32 = 0.24;
const JUNTA: f32 = 0.08;

// Material óptico del ventanal: independiente de las superficies opacas.
// Cambiar estos valores no requiere una textura ni una biblioteca nueva.
pub struct MaterialVentana {
    pub tinte: [u8; 3],
    pub opacidad_frontal: f32,
    pub opacidad_oblicua: f32,
    pub opacidad_borde: f32,
}
pub const VIDRIO_TECHO: MaterialVentana = MaterialVentana {
    tinte: [115, 183, 203],
    opacidad_frontal: 0.055,
    opacidad_oblicua: 0.14,
    opacidad_borde: 0.26,
};

#[derive(Clone, Copy, Default)]
struct Limite { nx: f32, nz: f32, distancia: f32 }
static LIMITES: OnceLock<[Limite; 8]> = OnceLock::new();
fn limites() -> &'static [Limite; 8] {
    LIMITES.get_or_init(|| {
        let v = crate::escena::contorno_nave();
        let mut resultado = [Limite::default(); 8];
        for i in 0..8 {
            let (ax, az) = v[i];
            let (bx, bz) = v[(i+1)%8];
            let (dx, dz) = (bx-ax,bz-az);
            let largo = (dx*dx+dz*dz).sqrt();
            let (nx,nz) = (-dz/largo,dx/largo);
            resultado[i] = Limite { nx, nz,
                distancia: nx*ax+nz*az+ANCHO_BORDE+JUNTA*0.5 };
        }
        resultado
    })
}

pub fn construir(cubos: &mut Vec<Cubo>) {
    let v = crate::escena::contorno_nave();
    let h = crate::escena::ROOM_HEIGHT;
    let exterior = crate::escena::WALL_THICKNESS * 0.5;
    let _ = limites();
    for i in 0..8 {
        let (ax,az) = v[i];
        let (bx,bz) = v[(i+1)%8];
        let (dx,dz) = (bx-ax,bz-az);
        let largo = (dx*dx+dz*dz).sqrt();
        let (nx,nz) = (-dz/largo,dx/largo);
        let angulo = (-dz).atan2(dx);
        // El solape sella las esquinas. El borde sigue cada pared de la nave.
        let offset = (ANCHO_BORDE-exterior)*0.5;
        cubos.push(Cubo::new(
            Vector3::new((ax+bx)*0.5+nx*offset,h+GROSOR*0.5,(az+bz)*0.5+nz*offset),
            Vector3::new(largo+0.60,GROSOR,ANCHO_BORDE+exterior),Color::WHITE,0.0,0.0,
        ).rotado_y(angulo).con_material(PISO_NAVE,Some(1.2)));
        // Junta fina al ras del hueco, a la misma altura que el vidrio.
        // Acortar a la intersección de los límites evita puntas sobre el cristal.
        let prev = (i+7)%8;
        let next = (i+1)%8;
        let inicio = esquina_interior(prev,i);
        let fin = esquina_interior(i,next);
        let longitud = ((fin.0-inicio.0).powi(2)+(fin.1-inicio.1).powi(2)).sqrt();
        cubos.push(Cubo::new(
            Vector3::new((inicio.0+fin.0)*0.5,h+0.10,(inicio.1+fin.1)*0.5),
            Vector3::new(longitud+JUNTA,0.12,JUNTA),Color::WHITE,0.0,0.0,
        ).rotado_y(angulo).con_material(REFUERZO_NAVE,None));
    }
}

fn esquina_interior(a: usize,b: usize) -> (f32,f32) {
    let [p,q] = [limites()[a],limites()[b]];
    let determinante = p.nx*q.nz-q.nx*p.nz;
    ((p.distancia*q.nz-q.distancia*p.nz)/determinante,
     (p.nx*q.distancia-q.nx*p.distancia)/determinante)
}

pub fn filtrar_ventana(color: Color,origen: &Vector3,direccion: &Vector3,limite: f32) -> Color {
    if direccion.y.abs()<0.000001 { return color; }
    let t = (crate::escena::ROOM_HEIGHT+0.10-origen.y)/direccion.y;
    // Una pared, marco o planeta delante del vidrio debe conservar su color.
    if t<=0.001 || t>=limite { return color; }
    let x=origen.x+direccion.x*t;
    let z=origen.z+direccion.z*t;
    let mut margen=f32::INFINITY;
    for borde in limites() {
        let d=borde.nx*x+borde.nz*z-borde.distancia;
        if d<0.0 { return color; }
        margen=margen.min(d);
    }
    let vidrio=&VIDRIO_TECHO;
    // Mayor presencia a ras del cristal y en los bordes; el centro es claro.
    // Los rayos del renderer están normalizados.
    let rasante=1.0-direccion.y.abs().min(1.0);
    let mut alpha=vidrio.opacidad_frontal+
        (vidrio.opacidad_oblicua-vidrio.opacidad_frontal)*rasante*rasante;
    if margen<0.06 {
        alpha += (vidrio.opacidad_borde-alpha)*(1.0-margen/0.06);
    }
    let mezcla=|v:u8,t:u8| (v as f32*(1.0-alpha)+t as f32*alpha) as u8;
    Color::new(mezcla(color.r,vidrio.tinte[0]),mezcla(color.g,vidrio.tinte[1]),
        mezcla(color.b,vidrio.tinte[2]),255)
}
