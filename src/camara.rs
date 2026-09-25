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

use crate::config::{
    CAMERA_ORBIT_SPEED as ORBIT_SPEED,
    CAMERA_ZOOM_KEY_SPEED as ZOOM_KEY_SPEED,
    CAMERA_ZOOM_WHEEL_SPEED as ZOOM_WHEEL_SPEED,
    CAMERA_TRAVEL_SPEED as TRAVEL_SPEED,
    CAMERA_SMOOTHING_RATE as SMOOTHING_RATE,
    CAMERA_PITCH_LIMIT_DEGREES,
    CAMERA_ZOOM_MIN,
    CAMERA_ZOOM_MAX,
};

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

    // Transforma una dirección en espacio de cámara (donde "adelante"
    // es -Z) al espacio del mundo, según hacia dónde esté orientada la
    // cámara ahora mismo.
    pub fn basis_change(&self, direction: &Vector3) -> Vector3 {
        let (forward, right, up) = self.basis();
        (right * direction.x + up * direction.y - forward * direction.z).normalize()
    }

    // --- Controles: solo tocan los valores "target_*" ---

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.target_yaw += delta_yaw;

        // Se limita el pitch para no "voltear" la cámara de cabeza al
        // pasar por los polos.
        let limit = CAMERA_PITCH_LIMIT_DEGREES.to_radians();
        self.target_pitch = (self.target_pitch + delta_pitch).clamp(-limit, limit);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.target_distance = (self.target_distance + delta).clamp(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX);
    }

    // Mueve el punto que la cámara orbita ("center"), usando la
    // orientación actual como referencia. Así uno "viaja" de un
    // planeta a otro sin dejar de poder orbitarlo.
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

        if window.is_key_down(KeyboardKey::KEY_W) {
            self.travel(TRAVEL_SPEED * delta_time, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            self.travel(-TRAVEL_SPEED * delta_time, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_A) {
            self.travel(0.0, -TRAVEL_SPEED * delta_time);
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            self.travel(0.0, TRAVEL_SPEED * delta_time);
        }
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