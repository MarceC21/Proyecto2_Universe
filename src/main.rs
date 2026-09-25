use raylib::prelude::*;

mod config;
mod framebuffer;
mod ray_intersect;
mod esfera;
mod luz;
mod planeta;
mod camara;
mod skybox;

use framebuffer::Framebuffer;
use ray_intersect::{Intersect, RayIntersect};
use esfera::Esfera;
use luz::Luz;
use planeta::{Planet, OrbitalParameters};
use camara::Camera;
use skybox::sky_color;

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
    light: &Luz,
    sun_center: Vector3,
    orbit_radii: &[f32],
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

    if !closest_intersect.is_intersecting {
        if let Some(ring) = orbit_ring_color(ray_origin, ray_direction, sun_center, orbit_radii) {
            return ring;
        }
        return sky_color(ray_direction);
    }

    shade(&closest_intersect, light)
}

// Sombreado: ambiente + difusa (Ley de Lambert). El Sol es la única
// luz de la escena y es puntual: le pega a cada planeta desde su
// posición, sin importar desde qué lado lo esté viendo la cámara (la
// difusa NO depende del observador, solo del ángulo normal-luz), así
// que cada planeta queda con su "lado día" iluminado y su "lado noche"
// a oscuras, tal cual pasa en la realidad.
fn shade(intersect: &Intersect, light: &Luz) -> Color {

    // El Sol no recibe la luz: ES la luz. Se dibuja siempre a su color
    // pleno (si se sombreara como los demás, se vería como una bola
    // negra, porque nada más lo ilumina a él).
    if intersect.emissive {
        return intersect.color;
    }

    let light_dir = (light.position - intersect.point).normalize();

    // Se recorta en 0 cuando la cara mira para el otro lado de la luz
    // (esa cara queda sin difusa: es la noche de ese planeta).
    let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0);

    let ambient = mul_color(intersect.color, intersect.ka);

    let diffuse = mul_color(
        mul_colors(intersect.color, light.color),
        intersect.kd * diffuse_intensity * light.intensity,
    );

    add_colors(ambient, diffuse)
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
fn orbit_ring_color(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    sun_center: Vector3,
    orbit_radii: &[f32],
) -> Option<Color> {

    // Rayo casi paralelo al plano orbital: no vale la pena intersectar
    // (o no hay solución útil, o el "anillo" saldría estirado hasta el
    // infinito).
    if ray_direction.y.abs() < 1e-5 {
        return None;
    }

    let t = (sun_center.y - ray_origin.y) / ray_direction.y;

    // El plano queda detrás de la cámara: no se ve.
    if t <= 0.0001 {
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
            return Some(Color::new(70, 90, 120, 255)); // línea tenue, gris azulado
        }
    }

    None
}

/// El raytracer no es más que un for de dos dimensiones que recorre la
/// pantalla: por cada pixel se lanza un rayo y se pinta el color que
/// ese rayo trae de vuelta.
pub fn render(
    framebuffer: &mut Framebuffer,
    planets: &[Planet],
    camera: &Camera,
    light: &Luz,
    sun_center: Vector3,
    orbit_radii: &[f32],
) {

    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {

            // Se mapea la coordenada del pixel a espacio de pantalla [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Se ajusta por el aspect ratio
            let screen_x = screen_x * aspect_ratio;

            // Dirección del rayo en espacio de CÁMARA...
            let ray_direction_camera = Vector3::new(screen_x, screen_y, -1.0).normalize();

            // ...rotada al espacio del MUNDO según hacia dónde esté
            // orientada la cámara orbital ahora mismo.
            let ray_direction = camera.basis_change(&ray_direction_camera);

            let pixel_color = cast_ray(&camera.eye, &ray_direction, planets, light, sun_center, orbit_radii);

            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
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

const RADIUS_SCALE: f32 = 0.4; // 1.0 radio terrestre = 0.4 unidades
const DIST_SCALE: f32 = 5.0;   // 1.0 AU = 5.0 unidades
const EARTH_ORBIT_SECONDS: f32 = 20.0;

// Radios reales en "radios terrestres" (Tierra = 1.0)
const SUN_RADIUS_REAL: f32 = 3.0; // el Sol real mide ~109; se recorta
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

fn main() {

    let window_width = config::WINDOW_WIDTH;
    let window_height = config::WINDOW_HEIGHT;

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer - Sistema Solar")
        .build();

    framebuffer.set_background_color(Color::BLACK);
    framebuffer.clear();

    let mut planets = crear_sistema_solar();

    let sun_center = Vector3::zero();

    // Posiciona todo antes del primer frame. orbital_angle arranca en
    // 0.0, así que llamar update() con delta_time = 0.0 solo aplica la
    // fórmula una vez (coloca cada planeta a su distancia, sobre el
    // eje X) sin hacer avanzar nada todavía.
    for planet in planets.iter_mut() {
        planet.update(0.0, sun_center);
    }

    // La luz vive en el mismo punto que el Sol: el Sol de la escena es
    // tanto la esfera que se ve como la fuente que ilumina a los
    // demás. Un poco más intensa que antes, y con más ambiente, para
    // que la escena se sienta más luminosa sin agregar ningún cálculo
    // costoso (nada de sombras entre planetas ni rebotes todavía).
    let light = Luz::new(
        sun_center,
        Color::new(255, 244, 214, 255), // luz solar: blanco cálido
        1.7,
    );

    // Cámara oblicua desde el arranque: con algo de pitch se ve el
    // sistema "desde arriba y de costado", como en las referencias, en
    // vez de mirarlo perfectamente de canto.
    let mut camera = Camera::new(
        sun_center,
        22.0,
        25.0_f32.to_radians(),
        30.0_f32.to_radians(),
    );

    // Radios de las órbitas a dibujar (se excluye al Sol, que orbita a
    // distancia 0). Se calcula una sola vez porque las distancias no
    // cambian con el tiempo, solo el ángulo.
    let orbit_radii: Vec<f32> = planets
        .iter()
        .filter(|p| p.orbital.distance > 0.0)
        .map(|p| p.orbital.distance)
        .collect();

    render(&mut framebuffer, &planets, &camera, &light, sun_center, &orbit_radii);

    // Main Loop
    while !window.window_should_close() {

        // Tiempo real transcurrido desde el frame anterior. Todo lo
        // que se mueve en este loop (cámara y planetas) se mueve en
        // función de esto, no de "un paso por frame": es lo que hace
        // que la simulación se vea igual de rápida sin importar el
        // framerate de la máquina. Se recorta a 0.1s por si el primer
        // frame (o cualquier otro) tarda un instante inusualmente
        // largo, para no hacer "saltar" a los planetas de golpe.
        let delta_time = window.get_frame_time().min(0.1);

        camera.handle_input(&window, delta_time);
        camera.update(delta_time);

        for planet in planets.iter_mut() {
            planet.update(delta_time, sun_center);
        }

        // Los planetas se mueven solos en cada frame (traslación
        // orbital), así que ya no tiene sentido re-renderizar solo "si
        // la cámara se movió": la escena cambia siempre. Por eso ahora
        // se vuelve a trazar en cada vuelta del loop.
        render(&mut framebuffer, &planets, &camera, &light, sun_center, &orbit_radii);

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}