// ---------------------------------------------------------------------
// Configuración centralizada del renderizador
// ---------------------------------------------------------------------
//
// Este módulo no agrega comportamiento nuevo: junta en un solo lugar
// las constantes que antes vivían sueltas en main.rs y camara.rs, para
// poder ajustarlas sin tener que rastrearlas por todo el proyecto. No
// se centralizan acá los datos del sistema solar (radios/distancias/
// periodos reales de los planetas en main.rs): esos son DATOS DE
// ESCENA, no configuración del renderizador, y se organizarán aparte
// cuando exista escena.rs.
//
// -----------------------------------------------------------------
// Convenciones del proyecto (para no tener que redescubrirlas leyendo
// el código cada vez):
//
// Coordenadas:
//   - Sistema de mano derecha. Y es "arriba", el plano XZ es el plano
//     orbital del sistema solar (se ve "desde arriba" con Y+ hacia la
//     cámara).
//   - En ESPACIO DE CÁMARA, "adelante" es -Z (ver render() en main.rs:
//     Vector3::new(screen_x, screen_y, -1.0)). Camera::basis_change
//     convierte esa dirección a espacio del mundo.
//
// Ángulos:
//   - Todos los cálculos internos (yaw, pitch, ángulos orbitales) se
//     guardan y operan en RADIANES. Los literales pensados para un
//     humano (p. ej. el pitch inicial de la cámara) se escriben en
//     grados y se convierten una sola vez con `.to_radians()`.
//
// Colores:
//   - `raylib::prelude::Color` con canales u8 en 0..255. El sombreado
//     actual (mul_color/mul_colors/add_colors en main.rs) mezcla en
//     ese mismo espacio 0..255 (normalizando a 0..1 solo para el
//     producto de colores), no en espacio lineal. Es una
//     simplificación consciente que puede revisarse más adelante si
//     la iluminación con varios materiales se ve mal; no se toca en
//     esta etapa.
//
// Rayos:
//   - Un rayo válido requiere t > INTERSECTION_EPSILON (no t > 0), para
//     no autointersectar la superficie de la que sale.
// -----------------------------------------------------------------

// --- Ventana / framebuffer ---
pub const WINDOW_WIDTH: i32 = 1300;
pub const WINDOW_HEIGHT: i32 = 1000;

// --- Trazado de rayos ---
// Distancia mínima para aceptar una intersección como válida: evita
// que un rayo se choque consigo mismo justo en su punto de origen por
// error de redondeo de punto flotante
pub const INTERSECTION_EPSILON: f32 = 0.0001;

// Profundidad máxima de recursión para rayos secundarios (reflexión /
// refracción). Todavía no se usa: cast_ray() aún no es recursivo.
#[allow(dead_code)]
pub const MAX_RECURSION_DEPTH: u32 = 3;

// --- Cámara orbital (camara.rs) ---
pub const CAMERA_ORBIT_SPEED: f32 = 2.2; // radianes/segundo
pub const CAMERA_ZOOM_KEY_SPEED: f32 = 10.0; // unidades/segundo (teclas Q/E)
pub const CAMERA_ZOOM_WHEEL_SPEED: f32 = 2.5; // unidades por "muesca" de rueda
pub const CAMERA_TRAVEL_SPEED: f32 = 6.0; // unidades/segundo
pub const CAMERA_SMOOTHING_RATE: f32 = 8.0; // suavizado exponencial, 1/segundo
pub const CAMERA_PITCH_LIMIT_DEGREES: f32 = 89.0;
pub const CAMERA_ZOOM_MIN: f32 = 1.0;
pub const CAMERA_ZOOM_MAX: f32 = 300.0;