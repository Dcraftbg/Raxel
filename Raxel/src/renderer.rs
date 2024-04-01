use std::time::{Duration, SystemTime};

use crate::batch::Vertex;
use crate::time::Time;
use crate::{font, Batch, Color, Font, Vector2f, Vector3f};
use crate::Texture;
use beryllium::video::GlWindow;
use gl33::global_loader::*;
use gl33::*;
use beryllium::*;
#[derive(Clone, Debug)]
pub struct Boundary {
    pub pos: Vector2f,
    pub size: Vector2f
}
impl Boundary {
    pub fn top_left(&self) -> Vector2f {
        Vector2f(self.pos.0, self.pos.1 + self.size.1)
    }
}
pub struct Renderer {
    pub sdl: Sdl,
    pub win: GlWindow, 
    pub batches: Vec<Batch>,
    pub time: Time,
    pub texshape: Texture,
    pub targetFps: usize,
    desiredTime: f32
    //TODO:
    //pub default_shader: Shader
}
impl Renderer {
    pub fn new(sdl: Sdl, win: GlWindow) -> Self {
        Self { sdl, win, batches: Vec::new(), texshape: Texture::null(), time: Default::default(), targetFps: 0, desiredTime: 0.0}
    }
    pub fn targetfps(&mut self, fps: usize) {
        self.targetFps = fps;
        self.desiredTime = 1.0 / fps as f32;
    }
    pub fn batch_top<'a>(&'a self) -> Option<&'a Batch> {
        let len = self.batches.len();
        if len > 0 {
            Some(&self.batches[len-1])
        } 
        else {
            None
        }
    }

    pub fn batch_top_mut<'a>(&'a mut self) -> Option<&'a mut Batch> {
        let len = self.batches.len();
        if len > 0 {
            Some(&mut self.batches[len-1])
        }
        else {
            None
        }
    }
    pub fn batch_mut<'a>(&'a mut self, texid: u32) -> &'a mut Batch {
        let len = self.batches.len();
        for i in 0..len {
            if self.batches[i].texid == texid { return &mut self.batches[i]; }
        }
        self.batches.push(Batch::new(texid));
        &mut self.batches[len]
    }
    // TODO: Handle this a bit better. Don't think putting it in new is correct but I don't this is
    // correct either.
    //
    // TODO: Consider renaming to init_gl
    pub fn create_gl(&mut self) {
        unsafe {
        self.texshape = {
            let tex = Texture::new_gl();
            let pixel: [u8;4] = [0xff, 0xff, 0xff, 0xff];
            tex.buffer_raw_gl(0, GL_RGBA.0 as i32, 1, 1, GL_RGBA, GL_UNSIGNED_BYTE, pixel.as_ptr().cast());
            tex
        };
        }
    }
    pub fn gl_position2d(&self, pos: &Vector2f) -> Vector2f {
        Vector2f(
            (pos.0 / self.win.get_window_size().0 as f32) * 2.0 - 1.0,
            (pos.1 / self.win.get_window_size().1 as f32) * 2.0 - 1.0,
        )
    }
    pub fn update(&self) {}
    pub fn begin(&mut self) {
        self.time.now = SystemTime::now();
        self.time.update = self.time.now.duration_since(self.time.then).expect("TIme has gone backwards").as_secs_f32();
        self.time.then = self.time.now;
    }
    pub fn end(&mut self) {
        for batch in self.batches.iter_mut() {
            batch.update();
        }
        self.win.swap_window();
        self.time.now = SystemTime::now();
        self.time.draw = self.time.now.duration_since(self.time.then).expect("TIme has gone backwards").as_secs_f32();
        self.time.then = self.time.now;
        self.time.dt = (self.time.update + self.time.draw) as f32;
        if self.targetFps > 0 {
            let diff = self.desiredTime - self.time.dt;
            if diff > 0.0 {
                std::thread::sleep(Duration::from_secs_f32(diff));
            }
            self.time.now = SystemTime::now();
            self.time.wait = self.time.now.duration_since(self.time.then).expect("Time has gone backwards").as_secs_f32();
            self.time.then = self.time.now;
        } else {
            self.time.wait = 0.0;
        }
        self.time.dt += (self.time.wait) as f32;
    }
    pub fn scisorsBegin(&self, bound: &Boundary) {
        let x = bound.pos.0.round() as i32;
        let y = bound.pos.1.round() as i32;
        let w = bound.size.0.round() as i32;
        let h = bound.size.1.round() as i32;
        unsafe {
            glEnable(GL_SCISSOR_TEST);
            glScissor(x,y,w,h);
        }
    }
    pub fn scisorsEnd(&self) {
        unsafe {
        glDisable(GL_SCISSOR_TEST);
        }
    }
    pub fn vertex_2d(&self, v: Vector2f, tex: Vector2f, color: Color) -> Vertex {
        let p = self.point_to_gl(v);
        Vertex { pos: Vector3f(p.0, p.1, 0.0), color, tex }
    }
    pub fn draw_texture_rect_ex(&mut self, color: Color, pos: Vector2f, size: Vector2f, tex: &Texture, viewPos: Vector2f, viewSize: Vector2f) {
        let tex_bottom  = Vector2f(viewPos.0 / tex.width as f32, viewPos.1 / tex.height as f32);
        let tex_top     = Vector2f((viewPos.0+viewSize.0) / tex.width as f32, (viewPos.1+viewSize.1) / tex.height as f32);
        let bottom_left = self.vertex_2d(Vector2f(pos.0       , pos.1)       , tex_bottom, color);
        let bottom_right= self.vertex_2d(Vector2f(pos.0+size.0, pos.1)       , Vector2f(tex_top.0, tex_bottom.1), color);
        let top_right   = self.vertex_2d(Vector2f(pos.0+size.0, pos.1+size.1), tex_top, color);
        let top_left    = self.vertex_2d(Vector2f(pos.0       , pos.1+size.1), Vector2f(tex_bottom.0, tex_top.1), color);
        let batch = self.batch_mut(tex.id);
        let ps = [bottom_left, bottom_right, top_right, top_left];
        batch.quad(
            &ps
        );
    }
    pub fn draw_texture_rect(&mut self, color: Color, pos: Vector2f, size: Vector2f, tex: &Texture) {
        let bottom_left = self.vertex_2d(Vector2f(pos.0       , pos.1)       , Vector2f(0.0, 0.0), color);
        let bottom_right= self.vertex_2d(Vector2f(pos.0+size.0, pos.1)       , Vector2f(1.0, 0.0), color);
        let top_right   = self.vertex_2d(Vector2f(pos.0+size.0, pos.1+size.1), Vector2f(1.0, 1.0), color);
        let top_left    = self.vertex_2d(Vector2f(pos.0       , pos.1+size.1), Vector2f(0.0, 1.0), color);
        let batch = self.batch_mut(tex.id);
        let ps = [bottom_left, bottom_right, top_right, top_left];
        batch.quad(
            &ps
        );
    }
    pub fn draw_rect(&mut self, color: Color, pos: Vector2f, size: Vector2f) {
        let bottom_left = self.vertex_2d(Vector2f(pos.0       , pos.1)       , Vector2f(0.0, 0.0), color);
        let bottom_right= self.vertex_2d(Vector2f(pos.0+size.0, pos.1)       , Vector2f(1.0, 0.0), color);
        let top_right   = self.vertex_2d(Vector2f(pos.0+size.0, pos.1+size.1), Vector2f(1.0, 1.0), color);
        let top_left    = self.vertex_2d(Vector2f(pos.0       , pos.1+size.1), Vector2f(0.0, 1.0), color);
        let batch = self.batch_mut(self.texshape.id);
        let ps = [bottom_left, bottom_right, top_right, top_left];
        batch.quad(
            &ps
        );
    }
    pub fn draw_triangle(&mut self, color: Color, p1: Vector2f, p2: Vector2f, p3: Vector2f) {
        let p1 = self.vertex_2d(p1, Vector2f(0.0, 0.0), color);
        let p2 = self.vertex_2d(p2, Vector2f(1.0, 0.0), color);
        let p3 = self.vertex_2d(p3, Vector2f(1.0, 1.0), color);
        let batch = self.batch_mut(self.texshape.id);
        // TODO: Figure out something better
        batch.tria(
            &[p1, p2, p3]
        );
    }
    pub fn window_size(&self) -> Vector2f {
        let s = self.win.get_window_size();
        Vector2f(s.0 as f32, s.1 as f32)
    }
    pub fn point_to_gl(&self, p1: Vector2f) -> Vector2f {
        let Vector2f(w,h) = self.window_size();
        Vector2f(
            (p1.0 / w) * 2.0 - 1.0,
            (p1.1 / h) * 2.0 - 1.0,
        )
    }
    pub fn clear(&self, color: Color) {
        unsafe {
        glClearColor(
            color.r,
            color.g,
            color.b,
            color.a,
        );
        glClear(GL_COLOR_BUFFER_BIT);
        }
    }
    pub fn texture_slot(&self, id: u32) {
        assert!(id == 0, "TODO: Multiple texture_slots");
        unsafe {
        glActiveTexture(GLenum(GL_TEXTURE0.0 + id))
        }
    }
    pub fn draw_char_scale(&mut self, font: &Font, c: char, mut pos: Vector2f, color: Color, glythScale: f32) {
        let g = font.get_char(c);
        pos.1 += (g.bitmap_top - g.height) as f32 * glythScale;
        self.draw_texture_rect_ex(color, pos, Vector2f(g.width as f32 * glythScale, g.height as f32 * glythScale), &font.texture, g.bound.pos, g.bound.size);
    }
    pub fn draw_str_scale(&mut self, font: &Font, s: &str, mut pos: Vector2f, color: Color, glythScale: f32) {
        let posOrg = pos;
        for chr in s.chars() {
            match chr {
                ' ' => {
                    let g = font.get_spacing_char().expect("No spacing character :(");
                    pos.0 += (g.advance_x - g.bitmap_left) as f32 * glythScale;
                }
                '\t' => {
                    let g = font.get_spacing_char().expect("No spacing character :(");
                    pos.0 += (g.advance_x - g.bitmap_left) as f32 * glythScale * 4.0;
                }
                '\n' => {
                    pos.1 += glythScale * 32.0;
                    pos.0 = posOrg.0;
                }
                '\r' => {}
                _ => {
                    let g = font.get_char(chr);
                    self.draw_char_scale(font, chr, pos, color, glythScale);
                    //println!("Character: {}. advance_x: {}. bitmap_top: {}. {}x{}", chr, g.advance_x, g.bitmap_top, g.width, g.height);
                    pos.0 += (g.advance_x - g.bitmap_left) as f32 * glythScale;
                    pos.1 += g.advance_y as f32 * glythScale;
                }
            }
        }
    }

    pub fn draw_char_ex(&mut self, font: &Font, c: char, pos: Vector2f, color: Color, fontSize: f32) {
        self.draw_char_scale(font, c, pos, color, fontSize / font.fontSize as f32)
    }
    pub fn draw_char(&mut self, font: &Font, c: char, pos: Vector2f, color: Color) {
        self.draw_char_scale(font, c, pos, color, 1.0);
    }
    pub fn draw_str_ex(&mut self, font: &Font, s: &str, pos: Vector2f, color: Color, fontSize: f32) {
        self.draw_str_scale(font, s, pos, color, fontSize / font.fontSize as f32);
    }
    pub fn draw_str(&mut self, font: &Font, s: &str, pos: Vector2f, color: Color) {
        self.draw_str_scale(font, s, pos, color, 1.0);
    }
}
