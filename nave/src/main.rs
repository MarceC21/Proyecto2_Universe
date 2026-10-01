use raylib::prelude::*;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

mod framebuffer;
mod ray_intersect;
mod esfera;
mod luz;
mod planeta;
mod camara;
mod skybox;
mod cubo;
mod bvh;
mod material;
mod textura;
mod mesa;
mod escena;
mod colisiones;
mod cabina;
mod laboratorio;
mod techo;
mod tars;
mod compuerta;

use framebuffer::Framebuffer;
use ray_intersect::{Intersect, RayIntersect};
use esfera::Esfera;
use luz::{Luz, Iluminacion};
use planeta::{Planet, OrbitalParameters};
use camara::{Camera, CamaraPersona};
use skybox::sky_color;
use bvh::Bvh;
use material::{Material, crear_materiales};
use mesa::{pantalla_texture_size, consola_texture_size};
use escena::{
    crear_habitacion, obstaculos, planetario_origen, EYE_HEIGHT, PLAYER_RADIUS, PLAYER_START_X,
    PLAYER_START_Z, contorno_nave, WALL_THICKNESS,
};

// ---------------------------------------------------------------------
// Trazado de rayos y sombreado
// ---------------------------------------------------------------------

// Lanza un rayo desde ray_origin en la dirección ray_direction: busca
// con qué planeta choca primero y le pide a shade() el color final. Si
// no choca con nada, el color sale del fondo estelar (skybox.rs).
fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    planets: &[Planet],
    cubos: &Bvh,
    materials: &[Material],
    light: &Iluminacion,
    sun_center: Vector3,
    orbit_radii: &[f32],
    depth: u32,
) -> Color {

    let mut closest_distance = f32::INFINITY;
    let mut closest_intersect = Intersect::empty();

    for planet in planets {

        let intersect = planet.sphere.ray_intersect(ray_origin, ray_direction);

        if intersect.is_intersecting && intersect.distance < closest_distance {
            closest_distance = intersect.distance;
            closest_intersect = intersect;
        }

    }

    // La BVH descarta grupos enteros y visita primero los más cercanos.
    let intersect = cubos.intersectar(ray_origin, ray_direction, closest_distance);
    if intersect.is_intersecting && intersect.distance < closest_distance {
        closest_distance = intersect.distance;
        closest_intersect = intersect;
    }

    // Los anillos ya no solo aparecen cuando el rayo no choca con nada:
    // ahora hay paredes y una mesa, así que el anillo solo se dibuja si
    // está MÁS CERCA que el impacto más cercano (resuelve su profundidad).
    if let Some((ring_t, ring_color)) = orbit_ring_hit(ray_origin, ray_direction, sun_center, orbit_radii, closest_distance) {
        if ring_t < closest_distance {
            return filtrar_vidrios(ring_color, ray_origin, ray_direction, ring_t);
        }
    }

    if !closest_intersect.is_intersecting {
        return filtrar_vidrios(sky_color(ray_direction), ray_origin, ray_direction, f32::INFINITY);
    }

    let color = shade(
        &closest_intersect, ray_direction, materials, light,
        planets, cubos, sun_center, orbit_radii, depth,
    );
    filtrar_vidrios(color, ray_origin, ray_direction, closest_distance)
}

// Se conserva el visor del laboratorio y se añade el ventanal superior.
// Ambos usan el límite del impacto opaco y no lanzan rayos secundarios.
fn filtrar_vidrios(color: Color, origen: &Vector3, direccion: &Vector3, limite: f32) -> Color {
    let color = techo::filtrar_ventana(color, origen, direccion, limite);
    laboratorio::filtrar_visor(color, origen, direccion, limite)
}

// Profundidad máxima de rayos reflejados: cada rebote es un cast_ray()
// completo (consulta la BVH y los planetas), así que se mantiene bajo
// para no disparar el costo. 2 alcanza para que el marco metálico y el
// tablero se reflejen entre sí sin verse "cortada" la reflexión.
const MAX_REFLECTION_DEPTH: u32 = 2;
const REFLECTION_BIAS: f32 = 0.001;

// Ambiente tenue + luces locales con alcance + Sol con caída de intensidad.
// La emisión hace visible la fuente; las sombras controlan la luz que recibe
// cada superficie. Acumulamos RGB flotante antes de convertir a Color.
fn shade(
    intersect: &Intersect,
    ray_direction: &Vector3,
    materials: &[Material],
    light: &Iluminacion,
    planets: &[Planet],
    cubos: &Bvh,
    sun_center: Vector3,
    orbit_radii: &[f32],
    depth: u32,
) -> Color {

    // El Sol no recibe la luz: ES la luz.
    if intersect.emissive {
        return intersect.color;
    }

    let (base, ka, kd, ks, shininess, reflectivity, emission) = match intersect.material {
        Some(id) => {
            let m = &materials[id];
            let tex = m.textura.sample(intersect.u, intersect.v);
            (mul_colors(tex, m.albedo), m.ka, m.kd, m.ks, m.shininess, m.reflectividad, m.emision)
        }
        None => (intersect.color, intersect.ka, intersect.kd, 0.0, 32.0, 0.0, 0.0),
    };

    let emision_visible = match intersect.material {
        // Solo botones/pantallitas coloreados brillan; el plástico gris no.
        Some(material::POLIMERO | material::BOTONERA_CABINA |
             material::CONTROLES_COMANDO | material::PANEL_ESCLUSA) => {
            let maximo=base.r.max(base.g).max(base.b) as f32;
            let minimo=base.r.min(base.g).min(base.b) as f32;
            emission*((maximo-minimo)/65.0).clamp(0.0,1.0)
        }
        _ => emission,
    };
    // Tiras, lámpara y botones emisivos puros no necesitan rayos de sombra.
    if kd==0.0 && ks==0.0 && reflectivity==0.0 {
        return mul_color(base,ka*luz::AMBIENTE+emision_visible);
    }
    let base_rgb=[base.r as f32,base.g as f32,base.b as f32];
    let mut rgb=base_rgb.map(|c|c*ka*luz::AMBIENTE);
    let view_dir=*ray_direction * -1.0;
    let delta=light.sol.position-intersect.point;
    let d2=delta.dot(delta);
    if d2>0.000001 {
        let energia=light.sol.intensity/(1.0+0.10*d2);
        let color=light.sol.color;
        sumar_luz(&mut rgb,base_rgb,kd,ks,shininess,intersect,view_dir,
            light.sol.position,[color.r as f32/255.0*energia,
                color.g as f32/255.0*energia,color.b as f32/255.0*energia],
            None,cubos,planets);
    }
    for muestra in light.cercanas(intersect.point,intersect.normal).into_iter().flatten() {
        sumar_luz(&mut rgb,base_rgb,kd,ks,shininess,intersect,view_dir,
            muestra.posicion,muestra.energia,Some(muestra.cubo),cubos,planets);
    }
    let local=Color::new(rgb[0].clamp(0.0,255.0) as u8,rgb[1].clamp(0.0,255.0) as u8,
        rgb[2].clamp(0.0,255.0) as u8,255);

    // Reflexión trazada: se relanza un rayo completo en la dirección
    // reflejada y se mezcla con el sombreado local según `reflectivity`.
    // Si ya se llegó a la profundidad máxima, se trata como si no
    // reflejara nada (en vez de oscurecer con un color reflejado
    // vacío): mejor un material algo menos brillante que un borde
    // negro en los rebotes más profundos.
    let final_local = if reflectivity > 0.001 && depth < MAX_REFLECTION_DEPTH {
        let reflected_dir = *ray_direction
            - intersect.normal * (2.0 * ray_direction.dot(intersect.normal));
        let reflected_origin = intersect.point + intersect.normal * REFLECTION_BIAS;
        let reflected_color = cast_ray(
            &reflected_origin, &reflected_dir, planets, cubos, materials,
            light, sun_center, orbit_radii, depth + 1,
        );
        add_colors(
            mul_color(local, 1.0 - reflectivity),
            mul_color(reflected_color, reflectivity),
        )
    } else {
        local
    };

    // Emisión: se suma color propio (líneas cian, indicadores) sin
    // depender de la luz. `emission` puede pasar de 1.0 a propósito
    // (para que se lea incluso con la ambiental baja de la nave).
    add_colors(final_local, mul_color(base, emision_visible))
}


// Visibilidad hasta la luz: las esferas en movimiento se prueban aparte.
// El Sol emisor no se bloquea a sí mismo. Los vidrios analíticos transmiten
// estas luces sin refracción; no forman parte de los cubos opacos de la BVH.
fn bloqueado(origen:&Vector3,direccion:&Vector3,limite:f32,
             omitir:Option<usize>,cubos:&Bvh,planets:&[Planet])->bool {
    for planeta in planets {
        let e=&planeta.sphere;
        if e.emissive { continue; }
        let oc=*origen-e.center;
        let b=oc.dot(*direccion);
        let c=oc.dot(oc)-e.radius*e.radius;
        if c<0.0 { return true; }
        let discriminante=b*b-c;
        if discriminante>=0.0 {
            let t=-b-discriminante.sqrt();
            if t>0.0001 && t<limite { return true; }
        }
    }
    cubos.ocluido(origen,direccion,limite,omitir)
}

#[allow(clippy::too_many_arguments)]
fn sumar_luz(rgb:&mut [f32;3],base:[f32;3],kd:f32,ks:f32,brillo:f32,
             hit:&Intersect,vista:Vector3,posicion:Vector3,energia:[f32;3],
             omitir:Option<usize>,cubos:&Bvh,planets:&[Planet]) {
    let origen=hit.point+hit.normal*0.003;
    let delta=posicion-origen;
    let distancia2=delta.dot(delta);
    if distancia2<0.000001 { return; }
    let distancia = distancia2.sqrt();
    let direccion = delta * (1.0 / distancia);
    let difusa=hit.normal.dot(direccion).max(0.0);
    if difusa<=0.0 || (kd==0.0 && ks==0.0) { return; }
    if bloqueado(&origen,&direccion,distancia-0.002,omitir,cubos,planets) { return; }
    let mitad=direccion+vista;
    let mitad2=mitad.dot(mitad);
    let especular=if ks>0.0 && mitad2>0.000001 {
        ks * hit.normal.dot(mitad * (1.0 / mitad2.sqrt())).max(0.0).powf(brillo)
    } else { 0.0 };
    for k in 0..3 { rgb[k]+=energia[k]*(base[k]*kd*difusa+255.0*especular); }
}

// Escala un color por un factor (recorta en 0..255 por si se pasa)
fn mul_color(color: Color, factor: f32) -> Color {
    Color::new(
        (color.r as f32 * factor).clamp(0.0, 255.0) as u8,
        (color.g as f32 * factor).clamp(0.0, 255.0) as u8,
        (color.b as f32 * factor).clamp(0.0, 255.0) as u8,
        255,
    )
}

// Multiplica dos colores canal por canal (normalizando 0..255 a 0..1),
// para que el color de la luz "tiña" el albedo del material.
fn mul_colors(a: Color, b: Color) -> Color {
    Color::new(
        ((a.r as f32 / 255.0) * (b.r as f32 / 255.0) * 255.0) as u8,
        ((a.g as f32 / 255.0) * (b.g as f32 / 255.0) * 255.0) as u8,
        ((a.b as f32 / 255.0) * (b.b as f32 / 255.0) * 255.0) as u8,
        255,
    )
}

// Suma dos colores canal por canal, recortando en 255 (satura, no
// desborda).
fn add_colors(a: Color, b: Color) -> Color {
    Color::new(
        (a.r as u16 + b.r as u16).min(255) as u8,
        (a.g as u16 + b.g as u16).min(255) as u8,
        (a.b as u16 + b.b as u16).min(255) as u8,
        255,
    )
}

// Anillos de órbita, como referencia visual (no es algo que se vería
// en el espacio real: es una ayuda, igual que en cualquier diagrama
// del sistema solar). Se calculan SOLO cuando el rayo no chocó con
// ningún planeta: se intersecta el rayo contra el plano orbital
// (y = altura del Sol) y se compara qué tan lejos quedó ese punto del
// Sol contra el radio de cada órbita. Nada de geometría ni listas
// precalculadas: es una división y una resta por cada órbita.
fn orbit_ring_hit(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    sun_center: Vector3,
    orbit_radii: &[f32],
    limite: f32,
) -> Option<(f32, Color)> {

    // Rayo casi paralelo al plano orbital: no vale la pena intersectar
    // (o no hay solución útil, o el "anillo" saldría estirado hasta el
    // infinito).
    if ray_direction.y.abs() < 1e-5 {
        return None;
    }

    let t = (sun_center.y - ray_origin.y) / ray_direction.y;

    // El plano queda detrás de la cámara: no se ve.
    if t <= 0.0001 || t >= limite {
        return None;
    }

    let hit = *ray_origin + *ray_direction * t;
    let radial_distance = ((hit.x - sun_center.x).powi(2) + (hit.z - sun_center.z).powi(2)).sqrt();

    for &radius in orbit_radii {
        // Grosor de la línea proporcional al radio, para que no se vea
        // desproporcionadamente gruesa la órbita de Mercurio ni
        // desproporcionadamente fina la de Marte.
        let thickness = (radius * 0.004).max(0.01);

        if (radial_distance - radius).abs() < thickness {
            return Some((t, Color::new(70, 90, 120, 255))); // línea tenue, gris azulado
        }
    }

    None
}

/// El raytracer no es más que un for de dos dimensiones que recorre la
/// pantalla: por cada pixel se lanza un rayo y se pinta el color que
/// ese rayo trae de vuelta.
///
/// Dos mejoras de velocidad respecto a la versión anterior:
///   - La base de la cámara (forward/right/up) se calcula UNA vez por
///     frame y se recibe ya hecha, en vez de recalcularla por pixel.
///   - Los hilos toman grupos pequeños de filas de una cola compartida,
///     evitando esperar a un único bloque con muchos reflejos.
pub fn render(
    framebuffer: &mut Framebuffer,
    planets: &[Planet],
    cubos: &Bvh,
    materials: &[Material],
    eye: Vector3,
    basis: (Vector3, Vector3, Vector3),
    light: &Iluminacion,
    sun_center: Vector3,
    orbit_radii: &[f32],
) {

    let w = framebuffer.width as usize;
    let h = framebuffer.height as usize;
    let width = w as f32;
    let height = h as f32;
    let aspect_ratio = width / height;
    let (forward, right, up) = basis;

    static HILOS: OnceLock<usize> = OnceLock::new();
    let threads = *HILOS.get_or_init(|| std::thread::available_parallelism()
        .map(|n| n.get()).unwrap_or(1));
    const FILAS_POR_TRABAJO: usize = 4;
    // Cada tarea obtiene una porción mutable exclusiva del buffer. El candado
    // solo protege la entrega de tareas, nunca el trazado ni la escritura.
    // No hay copias al framebuffer, asignaciones por píxel ni unsafe.
    let trabajos = Mutex::new(framebuffer.pixels_mut()
        .chunks_mut(w * 4 * FILAS_POR_TRABAJO).enumerate());
    std::thread::scope(|scope| {
        for _ in 0..threads.min(h.div_ceil(FILAS_POR_TRABAJO)) {
            scope.spawn(|| loop {
                let siguiente = { trabajos.lock().expect("Cola de render bloqueada").next() };
                let Some((indice, bytes)) = siguiente else { break; };
                let fila_inicial = indice * FILAS_POR_TRABAJO;
                for (i, pixel) in bytes.chunks_exact_mut(4).enumerate() {
                    let x = i % w;
                    let y = fila_inicial + i / w;
                    // Muestreo en el centro del píxel.
                    let screen_x = (2.0 * (x as f32 + 0.5) / width - 1.0) * aspect_ratio;
                    let screen_y = 1.0 - 2.0 * (y as f32 + 0.5) / height;
                    let direction = (right * screen_x + up * screen_y + forward).normalize();
                    let color = cast_ray(&eye, &direction, planets, cubos, materials,
                        light, sun_center, orbit_radii, 0);
                    pixel.copy_from_slice(&[color.r, color.g, color.b, 255]);
                }
            });
        }
    });
}

// ---------------------------------------------------------------------
// Sistema solar
// ---------------------------------------------------------------------
//
// No se usa la escala real del sistema solar (a escala real el Sol
// mide ~109 radios terrestres y Neptuno está a 30 AU: o el Sol no cabe
// en la escena, o los planetas quedan invisibles de tan lejos/
// pequeños). En vez de eso se usan escalas independientes que sí
// respetan las PROPORCIONES reales:
//
//   - RADIUS_SCALE: "radios terrestres" -> unidades de escena.
//   - DIST_SCALE:   "AU" -> unidades de escena (radio de la órbita).
//   - EARTH_ORBIT_SECONDS: cuántos segundos de reloj dura una órbita
//     completa de la Tierra en la simulación. Los demás planetas
//     calculan su velocidad angular a partir de su periodo orbital
//     REAL (en años terrestres) contra esta escala, así que las
//     proporciones de velocidad entre planetas también son reales:
//     Mercurio (0.24 años) sigue siendo el más rápido y Marte (1.88
//     años) el más lento, nada más que el tiempo entero está acelerado
//     para que se note en pantalla.

const RADIUS_SCALE: f32 = 0.12; // 1.0 radio terrestre = 0.12 unidades (cabe sobre la mesa)
const DIST_SCALE: f32 = 0.85;  // 1.0 AU = 0.85 unidades (Marte a 1.29, cabe dentro de la
                                // pantalla de la mesa: SCREEN_HALF_Z = 1.55 en mesa.rs)
const EARTH_ORBIT_SECONDS: f32 = 20.0;

// Radios reales en "radios terrestres" (Tierra = 1.0)
const SUN_RADIUS_REAL: f32 = 2.5; // el Sol real mide ~109; se recorta
                                   // a un tamaño que no tape todo el
                                   // sistema, pero sigue siendo, por
                                   // mucho, el cuerpo más grande.
const MERCURY_RADIUS_REAL: f32 = 0.383;
const VENUS_RADIUS_REAL: f32 = 0.949;
const EARTH_RADIUS_REAL: f32 = 1.0;
const MARS_RADIUS_REAL: f32 = 0.532;

// Distancia real al Sol en AU (radio de la órbita)
const MERCURY_DIST_AU: f32 = 0.39;
const VENUS_DIST_AU: f32 = 0.72;
const EARTH_DIST_AU: f32 = 1.00;
const MARS_DIST_AU: f32 = 1.52;

// Periodo orbital real en años terrestres
const MERCURY_PERIOD_YEARS: f32 = 0.2408;
const VENUS_PERIOD_YEARS: f32 = 0.6152;
const EARTH_PERIOD_YEARS: f32 = 1.0;
const MARS_PERIOD_YEARS: f32 = 1.8809;

// Coeficientes de material para los planetas (rocosos, superficie
// mate: nada de brillo especular todavía, eso es un siguiente paso).
const PLANET_KA: f32 = 0.12; // ambiente: qué tan visible queda el lado de noche
const PLANET_KD: f32 = 0.95; // difuso: qué tanto responde a la luz directa del Sol

// Convierte un periodo orbital real (en años terrestres) a velocidad
// angular de la simulación (radianes/segundo), según EARTH_ORBIT_SECONDS.
fn orbital_speed_from_period(period_years: f32) -> f32 {
    std::f32::consts::TAU / (period_years * EARTH_ORBIT_SECONDS)
}

fn crear_sistema_solar() -> Vec<Planet> {

    let sun = Planet::new(
        "Sol",
        Esfera::new(
            Vector3::zero(),
            SUN_RADIUS_REAL * RADIUS_SCALE,
            Color::new(255, 225, 130, 255),
            0.0,
            0.0,
            true, // emissive: no recibe sombreado, se dibuja a color pleno
        ),
        // No orbita nada (distancia y velocidad orbital en 0). Se le
        // deja una rotación propia lenta, lista para cuando haya
        // textura (hoy no se nota: una esfera de color plano se ve
        // igual gire o no).
        OrbitalParameters::new(0.0, 0.0, 0.05),
    );

    let mercury = Planet::new(
        "Mercurio",
        Esfera::new(
            Vector3::zero(), // se posiciona en el primer update()
            MERCURY_RADIUS_REAL * RADIUS_SCALE,
            Color::new(169, 169, 169, 255), // gris rocoso
            PLANET_KA,
            PLANET_KD,
            false,
        ),
        OrbitalParameters::new(
            MERCURY_DIST_AU * DIST_SCALE,
            orbital_speed_from_period(MERCURY_PERIOD_YEARS),
            0.6,
        ),
    );

    let venus = Planet::new(
        "Venus",
        Esfera::new(
            Vector3::zero(),
            VENUS_RADIUS_REAL * RADIUS_SCALE,
            Color::new(230, 200, 150, 255), // nubes densas, amarillento
            PLANET_KA,
            PLANET_KD,
            false,
        ),
        OrbitalParameters::new(
            VENUS_DIST_AU * DIST_SCALE,
            orbital_speed_from_period(VENUS_PERIOD_YEARS),
            // Venus rota al revés que casi todos los demás planetas
            // (rotación retrógrada), igual que el real: por eso la
            // velocidad es negativa.
            -0.25,
        ),
    );

    let earth = Planet::new(
        "Tierra",
        Esfera::new(
            Vector3::zero(),
            EARTH_RADIUS_REAL * RADIUS_SCALE,
            Color::new(70, 130, 180, 255), // azul por los océanos
            PLANET_KA,
            PLANET_KD,
            false,
        ),
        OrbitalParameters::new(
            EARTH_DIST_AU * DIST_SCALE,
            orbital_speed_from_period(EARTH_PERIOD_YEARS),
            1.4,
        ),
    );

    let mars = Planet::new(
        "Marte",
        Esfera::new(
            Vector3::zero(),
            MARS_RADIUS_REAL * RADIUS_SCALE,
            Color::new(193, 68, 14, 255), // el "planeta rojo"
            PLANET_KA,
            PLANET_KD,
            false,
        ),
        OrbitalParameters::new(
            MARS_DIST_AU * DIST_SCALE,
            orbital_speed_from_period(MARS_PERIOD_YEARS),
            1.3,
        ),
    );

    vec![sun, mercury, venus, earth, mars]
}

// F1: nativa; F2: 75%; F3: 50%. F4 activa resolución dinámica opcional.
// Se inicia en nativa para recuperar los detalles pequeños.
fn tamano_render(calidad: usize) -> (i32, i32) {
    match calidad { 1 => (900, 700), 2 => (675, 525), _ => (450, 350) }
}

enum Modo {
    Persona, // primera persona: caminar y mirar
    Orbital, // vista de diorama: rotar y acercar alrededor de la mesa
}

fn main() {

    let window_width = 1000;
    let window_height = 800;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Diorama - Interior de la nave")
        .build();

    // Se declara después de la ventana: libera la textura antes de cerrar OpenGL.
    let (rw, rh) = tamano_render(1);
    let mut framebuffer = Framebuffer::new(rw, rh);

    // El cursor queda capturado para mirar con el ratón (TAB lo libera).
    window.disable_cursor();

    framebuffer.set_background_color(Color::BLACK);
    framebuffer.clear();

    let mut planets = crear_sistema_solar();

    // Origen común del planetario: sobre la mesa. Las órbitas se
    // recalculan cada frame como origen + offset (nada se acumula).
    let sun_center = planetario_origen();

    let geometria = crear_habitacion();
    let mut tars = tars::Tars::reposo();
    let mut puerta = compuerta::Compuerta::new();
    let mut geometria_dinamica = Vec::with_capacity(14);
    puerta.construir(&mut geometria_dinamica);
    tars.construir(&mut geometria_dinamica);
    let cantidad_cubos = geometria.len() + geometria_dinamica.len();
    // Registrar emisores antes de mover los cubos a la BVH (mismos índices).
    let light = Iluminacion::new(Luz::new(sun_center,
        Color::new(255,244,214,255),1.7), &geometria);
    let mut cubos = Bvh::new(geometria);
    cubos.actualizar_dinamicos(geometria_dinamica);
    let mut obstaculos = obstaculos();
    let indice_puerta = obstaculos.len();
    obstaculos.extend(puerta.obstaculos());
    let indice_tars = obstaculos.len();
    obstaculos.push(tars.obstaculo());
    let colisiones = colisiones::Colisiones::new(&contorno_nave(), WALL_THICKNESS)
        .con_cajas(cabina::obstaculos_cabina(&contorno_nave(), WALL_THICKNESS))
        .con_cajas(laboratorio::obstaculos());

    // Catálogo de materiales de la mesa (metal claro/oscuro, pantalla,
    // polímero, indicador cian): genera sus texturas una sola vez, al
    // arrancar. Los tamaños de textura de la pantalla y de la placa de
    // controles se piden a mesa.rs para que la proporción coincida con
    // la de la superficie real (celdas cuadradas, sin ovalar anillos).
    let materials = crear_materiales(pantalla_texture_size(), consola_texture_size());

    // Posiciona todo antes del primer frame (delta_time = 0.0 solo
    // aplica la fórmula una vez).
    for planet in planets.iter_mut() {
        planet.update(0.0, sun_center);
    }

    // Primera persona: frente a la mesa (lado +Z), mirando hacia -Z y
    // un poco hacia abajo para ver el planetario.
    let mut persona = CamaraPersona::new(
        PLAYER_START_X,
        PLAYER_START_Z,
        EYE_HEIGHT,
        0.0,
        -12.0_f32.to_radians(),
    );

    // Vista de diorama (tecla C): orbita alrededor del planetario, con
    // distancia y pitch limitados para quedarse dentro de la nave.
    let mut orbital = Camera::new(
        sun_center,
        4.0,
        0.0,
        25.0_f32.to_radians(),
    );

    let mut calidad = 1usize;
    let mut dinamica = false;
    let mut mostrar_datos = true;
    let mut ojo_anterior: Option<Vector3> = None;
    let mut direccion_anterior: Option<Vector3> = None;
    let mut ultimo_movimiento = Instant::now();
    let mut reloj_estadisticas = Instant::now();
    let mut suma_render = 0.0f64;
    let mut suma_presentacion = 0.0f64;
    let mut cuadros = 0u32;
    let mut hud = format!("BVH: {} cubos | midiendo...\nF1 nativa  F2 75%  F3 50%  F4 dinamica  F5 datos\nESCLUSA: acercate al panel trasero; vuelve por el umbral", cantidad_cubos);
    if cfg!(debug_assertions) {
        eprintln!("AVISO: compilacion debug. Para medir rendimiento: cargo run --release");
    }
    let mut modo = Modo::Persona;
    let mut mouse_capturado = true;

    // Radios de las órbitas a dibujar (se excluye al Sol).
    let orbit_radii: Vec<f32> = planets
        .iter()
        .filter(|p| p.orbital.distance > 0.0)
        .map(|p| p.orbital.distance)
        .collect();

    // Main Loop
    while !window.window_should_close() {

        let delta_time = window.get_frame_time().min(0.25);

        if window.is_key_pressed(KeyboardKey::KEY_F1) { calidad = 1; }
        if window.is_key_pressed(KeyboardKey::KEY_F2) { calidad = 2; }
        if window.is_key_pressed(KeyboardKey::KEY_F3) { calidad = 3; }
        if window.is_key_pressed(KeyboardKey::KEY_F4) { dinamica = !dinamica; }
        if window.is_key_pressed(KeyboardKey::KEY_F5) { mostrar_datos = !mostrar_datos; }

        // C: cambiar entre primera persona y vista de diorama.
        if window.is_key_pressed(KeyboardKey::KEY_C) {
            modo = match modo {
                Modo::Persona => {
                    window.enable_cursor();
                    Modo::Orbital
                }
                Modo::Orbital => {
                    if mouse_capturado {
                        window.disable_cursor();
                    }
                    Modo::Persona
                }
            };
        }

        // TAB: soltar / recapturar el cursor (solo en primera persona).
        if window.is_key_pressed(KeyboardKey::KEY_TAB) && matches!(modo, Modo::Persona) {
            mouse_capturado = !mouse_capturado;
            if mouse_capturado {
                window.disable_cursor();
            } else {
                window.enable_cursor();
            }
        }

        let (eye, basis) = match modo {
            Modo::Persona => {
                persona.update(
                    &window,
                    delta_time,
                    mouse_capturado,
                    &colisiones,
                    PLAYER_RADIUS,
                    puerta.abierta(),
                    &obstaculos,
                );
                (persona.eye, persona.basis())
            }
            Modo::Orbital => {
                orbital.handle_input(&window, delta_time);
                orbital.update(delta_time);
                (orbital.eye, orbital.basis())
            }
        };

        let cambio_puerta = puerta.actualizar(delta_time,
            if matches!(modo, Modo::Persona) { Some(persona.eye) } else { None });
        let obstaculos_puerta = puerta.obstaculos();
        obstaculos[indice_puerta] = obstaculos_puerta[0];
        obstaculos[indice_puerta + 1] = obstaculos_puerta[1];

        // La cámara ya colisionó con la posición anterior. Ahora TARS evita al
        // jugador y publica su nueva huella para el siguiente cuadro.
        let jugador = if matches!(modo, Modo::Persona) { Some(persona.eye) } else { None };
        let cambio_tars = tars.actualizar(delta_time, &colisiones, &obstaculos[..indice_tars], jugador);
        obstaculos[indice_tars] = tars.obstaculo();
        if cambio_puerta || cambio_tars {
            let mut geometria_dinamica = Vec::with_capacity(14);
            puerta.construir(&mut geometria_dinamica);
            tars.construir(&mut geometria_dinamica);
            cubos.actualizar_dinamicos(geometria_dinamica);
        }

        for planet in planets.iter_mut() {
            planet.update(delta_time, sun_center);
        }

        // La reducción automática es opcional. Solo considera la cámara;
        // los planetas siguen animados y no impiden recuperar el detalle.
        let cambio = |a: Vector3, b: Vector3| { let d = a - b; d.dot(d) };
        let se_mueve = ojo_anterior.map_or(true, |v| cambio(v, eye) > 0.000001)
            || direccion_anterior.map_or(true, |v| cambio(v, basis.0) > 0.000001);
        if se_mueve { ultimo_movimiento = Instant::now(); }
        ojo_anterior = Some(eye);
        direccion_anterior = Some(basis.0);
        let calidad_efectiva = if dinamica && ultimo_movimiento.elapsed().as_secs_f32() < 0.6 {
            3
        } else { calidad };
        let (rw, rh) = tamano_render(calidad_efectiva);
        framebuffer.resize(rw, rh);

        let inicio_render = Instant::now();
        render(&mut framebuffer, &planets, &cubos, &materials, eye, basis, &light, sun_center, &orbit_radii);
        suma_render += inicio_render.elapsed().as_secs_f64();
        let inicio_presentacion = Instant::now();
        framebuffer.swap_buffers(&mut window, &raylib_thread, if mostrar_datos { &hud } else { "" });
        suma_presentacion += inicio_presentacion.elapsed().as_secs_f64();
        cuadros += 1;
        let transcurrido = reloj_estadisticas.elapsed().as_secs_f64();
        if transcurrido >= 0.75 {
            hud = format!("{:.1} FPS | traza {:.1} ms | presenta {:.1} ms | {}x{} | {} cubos | {}\nF1 nativa  F2 75%  F3 50%  F4 auto:{}  F5 ocultar\nESCLUSA: acercate al panel trasero; vuelve por el umbral",
                cuadros as f64 / transcurrido,
                suma_render * 1000.0 / cuadros as f64,
                suma_presentacion * 1000.0 / cuadros as f64, rw, rh, cantidad_cubos,
                if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" },
                if dinamica { "si" } else { "no" });
            suma_render = 0.0;
            suma_presentacion = 0.0;
            cuadros = 0;
            reloj_estadisticas = Instant::now();
        }
    }
}
