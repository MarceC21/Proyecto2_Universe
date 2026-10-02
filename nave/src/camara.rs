// Cámara orbital: controles + suavizado
use raylib::prelude::*;

use crate::colisiones::{Colisiones, Obstaculo};

const ORBIT_SPEED: f32 = 0.65; // radianes/segundo
const TRAVEL_SPEED: f32 = 3.5; // unidades/segundo (teclas W/S)
const ZOOM_KEY_SPEED: f32 = 5.0; // unidades/segundo (teclas Q/E)
const ZOOM_WHEEL_SPEED: f32 = 1.2; // unidades por "muesca" de la rueda del mouse

// El zoom permite alternar entre el detalle de la mesa y una vista completa
// de la nave. El pitch evita que la órbita se invierta en los polos.
const MIN_DISTANCE: f32 = 1.0;
const MAX_DISTANCE: f32 = 35.0;
const MIN_PITCH_DEG: f32 = -80.0;
const MAX_PITCH_DEG: f32 = 80.0;

// Qué tan rápido "alcanza" la cámara a su objetivo (suavizado  exponencial, no es una unidad física exacta). 
// Más alto = más inmediato , más bajo = más lento y cinematográfico.
const SMOOTHING_RATE: f32 = 5.0;

pub struct Camera {
    pub eye: Vector3,
    pub up: Vector3,

    // Valores actuales: lo que se usa para renderizar este frame
    pub center: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,

    // Valores objetivo: hacia dónde se dirige la cámara. Solo handle_input() los toca.
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

    // Recalcula "eye" 
    fn update_eye(&mut self) {
        self.eye = Vector3::new(
            self.center.x + self.distance * self.pitch.cos() * self.yaw.sin(),
            self.center.y + self.distance * self.pitch.sin(),
            self.center.z + self.distance * self.pitch.cos() * self.yaw.cos(),
        );
    }

    // Vectores forward/right/up de la cámara, a partir de eye y
    // center. 
    // Se usan tanto para mover "center" (travel) como para transformar cada rayo de espacio de cámara a espacio del mundo.
    pub fn basis(&self) -> (Vector3, Vector3, Vector3) {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        (forward, right, up)
    }


    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.target_yaw += delta_yaw;

        // Se limita el pitch para evitar singularidades en los polos.
        self.target_pitch = (self.target_pitch + delta_pitch)
            .clamp(MIN_PITCH_DEG.to_radians(), MAX_PITCH_DEG.to_radians());
    }

    pub fn zoom(&mut self, delta: f32) {
        self.target_distance = (self.target_distance + delta).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    // Mueve el punto que la cámara orbita ("center"), usando la orientación actual como referencia.
    pub fn travel(&mut self, forward_amount: f32, right_amount: f32) {
        let (forward, right, _up) = self.basis();
        self.target_center = self.target_center + forward * forward_amount + right * right_amount;
    }


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

        let mut forward = 0.0;
        if window.is_key_down(KeyboardKey::KEY_W) {
            forward += TRAVEL_SPEED * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            forward -= TRAVEL_SPEED * delta_time;
        }
        if forward != 0.0 {
            self.travel(forward, 0.0);
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let t = 1.0 - (-SMOOTHING_RATE * delta_time).exp();

        self.yaw += (self.target_yaw - self.yaw) * t;
        self.pitch += (self.target_pitch - self.pitch) * t;
        self.distance += (self.target_distance - self.distance) * t;
        self.center = self.center + (self.target_center - self.center) * t;

        self.update_eye();
    }
}


// Cámara en primera persona (estilo Minecraft)
const MOUSE_SENSITIVITY: f32 = 0.0025; // radianes por píxel
const LOOK_KEY_SPEED: f32 = 1.8; // radianes/segundo con las flechas
const WALK_SPEED: f32 = 3.5; // unidades/segundo
const ACCELERATION: f32 = 14.0; // qué tan rápido se alcanza la velocidad (suavizado)
const MAX_LOOK_PITCH_DEG: f32 = 85.0;

pub struct CamaraPersona {
    pub eye: Vector3,
    afuera: bool,
    en_espacio: bool,
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
            afuera: false,
            en_espacio: false,
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

    pub fn update(
        &mut self,
        window: &RaylibHandle,
        delta_time: f32,
        mouse_look: bool,
        colisiones: &Colisiones,
        radius: f32,
        puerta_abierta: bool,
        puerta_lateral_abierta: bool,
        obstacles: &[Obstaculo],
    ) {
        // --- Mirar: ratón (si está capturado) y flechas ---
        if mouse_look {
            let delta = window.get_mouse_delta();
            // Se descartan saltos enormes
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
        // En diagonal (W+D) la suma mide 1.41: se normaliza para no ir más rápido en diagonal.
        let len = (wish_x * wish_x + wish_z * wish_z).sqrt();
        if len > 0.0 {
            wish_x /= len;
            wish_z /= len;
        }

        let t = 1.0 - (-ACCELERATION * delta_time).exp();
        self.velocity_x += (wish_x * WALK_SPEED - self.velocity_x) * t;
        self.velocity_z += (wish_z * WALK_SPEED - self.velocity_z) * t;

        // La cámara solicita el desplazamiento; colisiones resuelve el recorrido.
        let dx = self.velocity_x * delta_time;
        let dz = self.velocity_z * delta_time;
        let (x, z) = colisiones.mover_con_compuerta(
            self.eye.x, self.eye.z, dx, dz, radius, obstacles,
            puerta_abierta, puerta_lateral_abierta, &mut self.afuera, &mut self.en_espacio,
        );
        if (x - (self.eye.x + dx)).abs() > 0.00001 {
            self.velocity_x = 0.0;
        }
        if (z - (self.eye.z + dz)).abs() > 0.00001 {
            self.velocity_z = 0.0;
        }
        self.eye.x = x;
        self.eye.z = z;

        self.eye.y = self.eye_height;
    }
}
