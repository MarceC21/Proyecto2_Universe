use raylib::prelude::*;

// Buffer RGBA persistente: se escribe directamente desde los trabajadores.
// La textura GPU solo se crea al iniciar o cuando cambia la resolución.
pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pixels: Vec<u8>,
    texture: Option<Texture2D>,
    background: Color,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);
        let mut fb = Self {
            width, height, pixels: vec![0; width as usize * height as usize * 4],
            texture: None, background: Color::BLACK,
        };
        fb.clear();
        fb
    }

    pub fn set_background_color(&mut self, color: Color) { self.background = color; }

    pub fn clear(&mut self) {
        let c = self.background;
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[c.r, c.g, c.b, c.a]);
        }
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] { &mut self.pixels }

    pub fn resize(&mut self, width: i32, height: i32) {
        if width == self.width && height == self.height { return; }
        assert!(width > 0 && height > 0);
        self.width = width;
        self.height = height;
        self.pixels.resize(width as usize * height as usize * 4, 0);
        self.texture = None;
        // El siguiente render escribe todos los bytes, incluido el canal alfa.
    }

    pub fn swap_buffers(&mut self, window: &mut RaylibHandle, thread: &RaylibThread, hud: &str) {
        if self.texture.is_none() {
            let image = Image::gen_image_color(self.width, self.height, Color::BLACK);
            let texture = window.load_texture_from_image(thread, &image)
                .expect("No se pudo crear la textura del framebuffer");
            texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            self.texture = Some(texture);
        }
        let texture = self.texture.as_mut().unwrap();
        texture.update_texture(&self.pixels).expect("Buffer RGBA incompatible");

        let sw = window.get_screen_width() as f32;
        let sh = window.get_screen_height() as f32;
        let scale = (sw / self.width as f32).min(sh / self.height as f32);
        let dw = self.width as f32 * scale;
        let dh = self.height as f32 * scale;
        let mut d = window.begin_drawing(thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(&*texture,
            Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
            Rectangle::new((sw - dw) * 0.5, (sh - dh) * 0.5, dw, dh),
            Vector2::zero(), 0.0, Color::WHITE);
        if !hud.is_empty() {
            d.draw_rectangle(8, 8, 875, 48, Color::new(0, 0, 0, 190));
            d.draw_text(hud, 16, 16, 16, Color::WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rgba_y_cambio_de_tamano() {
        let mut fb = Framebuffer::new(3, 2);
        fb.set_background_color(Color::new(7, 23, 81, 255));
        fb.clear();
        assert_eq!(fb.pixels.len(), 24);
        for p in fb.pixels.chunks_exact(4) { assert_eq!(p, &[7, 23, 81, 255]); }
        fb.resize(7, 5);
        assert_eq!(fb.pixels.len(), 140);
        fb.clear();
        assert_eq!(&fb.pixels[136..], &[7, 23, 81, 255]);
    }
}
