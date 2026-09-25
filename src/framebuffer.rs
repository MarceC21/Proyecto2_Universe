// Para crear un framebuffer y dibujar en él

use raylib::prelude::*;

// Framebuffer
pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub image: Image,

    background_color: Color,
    current_color: Color,
}

// Implementación de Framebuffer
impl Framebuffer {

    // Crear framebuffer
    pub fn new(width: i32, height: i32) -> Self {

        let background_color = Color::BLACK;

        Self {
            width,
            height,
            image: Image::gen_image_color(width, height, background_color),

            background_color,
            current_color: Color::WHITE,
        }
    }

    // Cambiar el color de fondo
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    // Cambiar el color actual (con el que se dibuja con point())
    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    // Limpiar el framebuffer (lo deja todo del color de fondo)
    pub fn clear(&mut self) {

        self.image = Image::gen_image_color(
            self.width,
            self.height,
            self.background_color,
        );

    }

    // Dibujar un punto usando el color actual
    pub fn point(&mut self, x: i32, y: i32) {

        if x >= 0 &&
           x < self.width &&
           y >= 0 &&
           y < self.height {

            self.image.draw_pixel(x, y, self.current_color);

        }

    }

    // Dibujar un punto usando un color específico (set color al pixel)
    pub fn set_pixel_color(&mut self, x: i32, y: i32, color: Color) {

        if x >= 0 &&
           x < self.width &&
           y >= 0 &&
           y < self.height {

            self.image.draw_pixel(x, y, color);

        }

    }

    pub fn point_color(&mut self, x: i32, y: i32, color: Color) {
        self.set_pixel_color(x, y, color);
    }

    // Obtener el color de un píxel
    pub fn get_color(&self, x: i32, y: i32) -> Color {

        if x >= 0 &&
           x < self.width &&
           y >= 0 &&
           y < self.height {

            self.image.get_color(x, y)

        } else {

            // Si está fuera del framebuffer, se usa el color de fondo
            self.background_color

        }

    }

    // Intercambiar buffers y mostrar el framebuffer en la ventana..
    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.image) {

            texture.set_texture_filter(raylib_thread, TextureFilter::TEXTURE_FILTER_POINT);

            let screen_width = window.get_screen_width();
            let screen_height = window.get_screen_height();

            // Ratio entero: cuántas veces completas cabe el framebuffer en la ventana, en X y en Y.
            let ratio_x = screen_width / self.width;
            let ratio_y = screen_height / self.height;

            // Se toma el menor de los dos para que el framebuffer entre
            // completo sin deformarse, y al menos 1 para no desaparecer
            // si la ventana es más chica que el framebuffer.
            let scale = ratio_x.min(ratio_y).max(1) as f32;

            let dest_width = self.width as f32 * scale;
            let dest_height = self.height as f32 * scale;

            // Se centra el rectángulo de destino en la ventana
            let dest_x = (screen_width as f32 - dest_width) * 0.5;
            let dest_y = (screen_height as f32 - dest_height) * 0.5;

            let mut renderer = window.begin_drawing(raylib_thread);

            // Lo que quede fuera del framebuffer escalado se pinta de negro en vez de dejarlo con basura de otro frame.
            renderer.clear_background(Color::BLACK);

            renderer.draw_texture_pro(
                &texture,
                Rectangle::new(
                    0.0,
                    0.0,
                    self.width as f32,
                    self.height as f32,
                ),
                Rectangle::new(
                    dest_x,
                    dest_y,
                    dest_width,
                    dest_height,
                ),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
    }

    // Guardar la imagen
    pub fn render_to_file(&self, filename: &str) {

        self.image.export_image(filename);

    }

}