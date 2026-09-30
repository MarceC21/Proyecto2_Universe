// Cámara orbital: controles + suavizado, separados del resto del
// motor (main.rs ya no sabe nada de qué tecla hace qué).
//
// Se separan tres responsabilidades que antes vivían mezcladas en el
// loop principal:
//   - handle_input(): qué tecla/mouse mueve qué. Es lo único que sabe
//     de raylib::RaylibHandle.
//   - update(): el suavizado. A qué velocidad se alcanza el objetivo.
//   - basis()/basis_change(): la geometría de la cámara (adónde
//     apunta cada rayo).
//
// El input NUNCA toca yaw/pitch/distance/center directamente: solo
// mueve los valores "target_*". update() es quien, cada frame, acerca
// los valores actuales a esos objetivos. Esto es lo que resuelve los
// "problemas de fluidez" que había: antes cada frame sumaba un paso
// fijo (por ejemplo ORBIT_SPEED tal cual), así que a 30 FPS la cámara
// giraba la mitad de rápido que a 60 FPS, y cualquier variación de
// framerate se sentía como un tirón. Acá el input avanza el objetivo
// ya escalado por delta_time, y el suavizado exponencial de update()
// también depende de delta_time, así que el resultado es el mismo
// recorrido (en segundos reales) sin importar el framerate, y además
// queda con una desaceleración suave en vez de parar en seco.
use raylib::prelude::*;

use crate::escena::Obstaculo;

const ORBIT_SPEED: f32 = 1.2; // radianes/segundo
const ZOOM_KEY_SPEED: f32 = 10.0; // unidades/segundo (teclas Q/E)
const ZOOM_WHEEL_SPEED: f32 = 2.5; // unidades por "muesca" de la rueda del mouse

// Límites de la cámara orbital: la nave es cerrada, así que el ojo
// debe quedar dentro de la habitación (5 de mitad de fondo, 5 de alto).
const MIN_DISTANCE: f32 = 1.0;
const MAX_DISTANCE: f32 = 4.5;
const MIN_PITCH_DEG: f32 = -5.0; // no bajar del tablero de la mesa
const MAX_PITCH_DEG: f32 = 45.0; // no atravesar el techo

// Qué tan rápido "alcanza" la cámara a su objetivo (suavizado
// exponencial, no es una unidad física exacta). Más alto = más
// inmediato; más bajo = más lento y cinematográfico.
const SMOOTHING_RATE: f32 = 9.0;

pub struct Camera {
    pub eye: Vector3,
    pub up: Vector3,

    // Valores actuales: lo que se usa para renderizar este frame.
    pub center: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,

    // Valores objetivo: hacia dónde se dirige la cámara. Solo
    // handle_input() los toca.
    target_center: Vector3,
    target_yaw: f32,
    target_pitch: f32,
    target_distance: f32,
}

impl Camera {
    pub fn new(center: Vector3, distance: f32, yaw: f32, pitch: f32) -> Self {
        let mut camera = Self {
            eye: Vector3::zero(),
            up: Vector3::new(0.0, 1.0, 0.0),
            center,
            yaw,
            pitch,
            distance,
            target_center: center,
            target_yaw: yaw,
            target_pitch: pitch,
            target_distance: distance,
        };
        camera.update_eye();
        camera
    }

    // Recalcula "eye" a partir de center/yaw/pitch/distance
    // (coordenadas esféricas alrededor del centro).
    fn update_eye(&mut self) {
        self.eye = Vector3::new(
            self.center.x + self.distance * self.pitch.cos() * self.yaw.sin(),
            self.center.y + self.distance * self.pitch.sin(),
            self.center.z + self.distance * self.pitch.cos() * self.yaw.cos(),
        );
    }

    // Vectores forward/right/up de la cámara, a partir de eye y
    // center. Se usan tanto para mover "center" (travel) como para
    // transformar cada rayo de espacio de cámara a espacio del mundo.
    pub fn basis(&self) -> (Vector3, Vector3, Vector3) {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        (forward, right, up)
    }


    // --- Controles: solo tocan los valores "target_*" ---

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.target_yaw += delta_yaw;

        // Se limita el pitch para no atravesar techo ni mesa.
        self.target_pitch = (self.target_pitch + delta_pitch)
            .clamp(MIN_PITCH_DEG.to_radians(), MAX_PITCH_DEG.to_radians());
    }

    pub fn zoom(&mut self, delta: f32) {
        self.target_distance = (self.target_distance + delta).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    // Mueve el punto que la cámara orbita ("center"), usando la
    // orientación actual como referencia. Así uno "viaja" de un
    // planeta a otro sin dejar de poder orbitarlo.
    #[allow(dead_code)]
    pub fn travel(&mut self, forward_amount: f32, right_amount: f32) {
        let (forward, right, _up) = self.basis();
        self.target_center = self.target_center + forward * forward_amount + right * right_amount;
    }

    // Lee teclado/mouse y traduce el input a movimientos del OBJETIVO
    // de la cámara. Todo escalado por delta_time, salvo la rueda del
    // mouse (que ya es un evento discreto: cada "muesca" es un solo
    // input, no algo que dependa de cuánto duró el frame).
    pub fn handle_input(&mut self, window: &RaylibHandle, delta_time: f32) {
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.orbit(-ORBIT_SPEED * delta_time, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.orbit(ORBIT_SPEED * delta_time, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.orbit(0.0, ORBIT_SPEED * delta_time);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.orbit(0.0, -ORBIT_SPEED * delta_time);
        }

        let wheel = window.get_mouse_wheel_move();
        if wheel != 0.0 {
            self.zoom(-wheel * ZOOM_WHEEL_SPEED);
        }
        if window.is_key_down(KeyboardKey::KEY_Q) {
            self.zoom(-ZOOM_KEY_SPEED * delta_time);
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            self.zoom(ZOOM_KEY_SPEED * delta_time);
        }

        // (W/A/S/D ya no desplazan el centro orbital: en la nave
        // cerrada ese movimiento es de la cámara en primera persona.)
    }

    // Acerca los valores actuales a los "target_*", a una velocidad
    // independiente del framerate (suavizado exponencial: cada frame
    // se recorre una FRACCIÓN de lo que falta, no un paso fijo). Se
    // llama una vez por frame, después de handle_input().
    pub fn update(&mut self, delta_time: f32) {
        let t = 1.0 - (-SMOOTHING_RATE * delta_time).exp();

        self.yaw += (self.target_yaw - self.yaw) * t;
        self.pitch += (self.target_pitch - self.pitch) * t;
        self.distance += (self.target_distance - self.distance) * t;
        self.center = self.center + (self.target_center - self.center) * t;

        self.update_eye();
    }
}


// ---------------------------------------------------------------------
// Cámara en primera persona (estilo Minecraft)
// ---------------------------------------------------------------------
//
// El jugador tiene una posición (x, z) sobre el suelo y los ojos a una
// altura fija. Dos ángulos definen hacia dónde mira:
//   yaw   = giro horizontal (0 = mirando hacia -Z, crece hacia la derecha)
//   pitch = inclinación vertical (arriba/abajo)
//
//   forward = ( sin(yaw)*cos(pitch),  sin(pitch), -cos(yaw)*cos(pitch) )
//   right   = ( cos(yaw), 0, sin(yaw) )        (siempre horizontal)
//   up      = right x forward
//
// El movimiento usa SOLO el yaw (dirección horizontal):
//   adelante = ( sin(yaw), 0, -cos(yaw) )
// así mirar hacia arriba o abajo no hace que uno vuele ni se hunda, y
// W siempre avanza "hacia donde miras" en el plano del suelo, sin
// importar por dónde se haya empezado.
const MOUSE_SENSITIVITY: f32 = 0.0025; // radianes por píxel
const LOOK_KEY_SPEED: f32 = 1.8; // radianes/segundo con las flechas
const WALK_SPEED: f32 = 3.5; // unidades/segundo
const ACCELERATION: f32 = 14.0; // qué tan rápido se alcanza la velocidad (suavizado)
const MAX_LOOK_PITCH_DEG: f32 = 85.0;

pub struct CamaraPersona {
    pub eye: Vector3,
    yaw: f32,
    pitch: f32,
    velocity_x: f32,
    velocity_z: f32,
    eye_height: f32,
}

impl CamaraPersona {
    pub fn new(x: f32, z: f32, eye_height: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            eye: Vector3::new(x, eye_height, z),
            yaw,
            pitch,
            velocity_x: 0.0,
            velocity_z: 0.0,
            eye_height,
        }
    }

    pub fn basis(&self) -> (Vector3, Vector3, Vector3) {
        let forward = Vector3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        );
        let right = Vector3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        let up = right.cross(forward).normalize();
        (forward, right, up)
    }

    // ¿Está (x, z) dentro de una pared o de un obstáculo? Se trata al
    // jugador como un círculo de radio `radius`, aproximado con un
    // cuadrado (basta para cajas alineadas con los ejes).
    fn is_blocked(x: f32, z: f32, half_x: f32, half_z: f32, radius: f32, obstacles: &[Obstaculo]) -> bool {
        if x.abs() > half_x - radius || z.abs() > half_z - radius {
            return true;
        }
        obstacles.iter().any(|o| {
            x > o.min_x - radius && x < o.max_x + radius && z > o.min_z - radius && z < o.max_z + radius
        })
    }

    pub fn update(
        &mut self,
        window: &RaylibHandle,
        delta_time: f32,
        mouse_look: bool,
        half_x: f32,
        half_z: f32,
        radius: f32,
        obstacles: &[Obstaculo],
    ) {
        // --- Mirar: ratón (si está capturado) y flechas ---
        if mouse_look {
            let delta = window.get_mouse_delta();
            // Se descartan saltos enormes (el primer frame tras capturar
            // el cursor puede traer un delta espurio).
            if delta.x.abs() < 200.0 && delta.y.abs() < 200.0 {
                self.yaw += delta.x * MOUSE_SENSITIVITY;
                self.pitch -= delta.y * MOUSE_SENSITIVITY;
            }
        }
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.yaw -= LOOK_KEY_SPEED * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.yaw += LOOK_KEY_SPEED * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.pitch += LOOK_KEY_SPEED * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.pitch -= LOOK_KEY_SPEED * delta_time;
        }
        let limit = MAX_LOOK_PITCH_DEG.to_radians();
        self.pitch = self.pitch.clamp(-limit, limit);
        self.yaw %= std::f32::consts::TAU;

        // --- Caminar: W/S adelante/atrás, A/D izquierda/derecha ---
        let mut move_forward = 0.0;
        let mut move_right = 0.0;
        if window.is_key_down(KeyboardKey::KEY_W) { move_forward += 1.0; }
        if window.is_key_down(KeyboardKey::KEY_S) { move_forward -= 1.0; }
        if window.is_key_down(KeyboardKey::KEY_D) { move_right += 1.0; }
        if window.is_key_down(KeyboardKey::KEY_A) { move_right -= 1.0; }

        // Direcciones horizontales según el yaw.
        let (fx, fz) = (self.yaw.sin(), -self.yaw.cos());
        let (rx, rz) = (self.yaw.cos(), self.yaw.sin());

        let mut wish_x = fx * move_forward + rx * move_right;
        let mut wish_z = fz * move_forward + rz * move_right;
        // En diagonal (W+D) la suma mide 1.41: se normaliza para no
        // ir más rápido en diagonal.
        let len = (wish_x * wish_x + wish_z * wish_z).sqrt();
        if len > 0.0 {
            wish_x /= len;
            wish_z /= len;
        }

        // Suavizado exponencial hacia la velocidad deseada (mismo
        // criterio que la cámara orbital: depende de delta_time).
        let t = 1.0 - (-ACCELERATION * delta_time).exp();
        self.velocity_x += (wish_x * WALK_SPEED - self.velocity_x) * t;
        self.velocity_z += (wish_z * WALK_SPEED - self.velocity_z) * t;

        // Se mueve cada eje por separado: si un eje choca, el otro sigue
        // libre y el jugador "resbala" a lo largo de la pared o la mesa.
        let new_x = self.eye.x + self.velocity_x * delta_time;
        if Self::is_blocked(new_x, self.eye.z, half_x, half_z, radius, obstacles) {
            self.velocity_x = 0.0;
        } else {
            self.eye.x = new_x;
        }
        let new_z = self.eye.z + self.velocity_z * delta_time;
        if Self::is_blocked(self.eye.x, new_z, half_x, half_z, radius, obstacles) {
            self.velocity_z = 0.0;
        } else {
            self.eye.z = new_z;
        }

        self.eye.y = self.eye_height;
    }
}
