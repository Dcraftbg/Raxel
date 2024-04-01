use crate::renderer::Boundary;
use crate::Vector2f;
use crate::texture::Texture;
#[derive(Clone, Debug)]
pub struct Glyth {
    pub width: i32,
    pub height: i32,
    pub atlas_off: i32,
    pub bitmap_left: i32,
    pub bitmap_top: i32,
    pub advance_x: i32,
    pub advance_y: i32,
    pub bound: Boundary
}
pub struct Font {
    pub glyths: Vec<Option<Glyth>>,
    pub texture: Texture,
    pub fontSize: u32
}
impl Font {
    pub fn null_char(&self) -> Glyth {
        self.glyths.get('?' as usize).expect("No null character").as_ref().expect("No null character").clone()
    }
    pub fn get_char(&self, c: char) -> Glyth {
        match self.glyths.get(c as usize) {
            Some(g) => {
                match g.clone() {
                    Some(v) => v,
                    None => self.null_char()
                }
            }
            None => self.null_char()
        }
    }
    pub fn measure_char(&self, c: char, fontSize: f32) -> Option<Vector2f> {
        let glythScale = fontSize / self.fontSize as f32;
        let glyth = self.glyths.get(c as usize)?.as_ref()?;
        Some(Vector2f(glyth.width as f32, glyth.height as f32)*glythScale)
    }
    pub fn measure_text(&self, s: &str, fontSize: f32) -> Option<Vector2f> {
        let mut res = Vector2f::ZERO();    
        let glythScale: f32 = fontSize / self.fontSize as f32;
        for chr in s.chars() {
            match chr {
                '\t' => {
                    let g = self.get_spacing_char()?;
                    res.0 += (g.advance_x - g.bitmap_left) as f32 * glythScale * 4.0;
                    if res.1 < g.height as f32 {
                        res.1 = g.height as f32
                    }
                }
                ' ' => {
                    let g = self.get_spacing_char()?;
                    res.0 += (g.advance_x - g.bitmap_left) as f32 * glythScale;
                    if res.1 < g.height as f32 {
                        res.1 = g.height as f32
                    }
                }
                '\n' => {
                    res.1 += glythScale * 32.0;
                }
                '\r' => {}
                _ => {
                    let g = self.get_char(chr);
                    res.0 += (g.advance_x - g.bitmap_left) as f32 * glythScale;
                    if res.1 < g.height as f32 {
                        res.1 = g.height as f32
                    }
                }
            }
            // res.1 += g.advance_y as f32 * glythScale;
        }
        Some(res)
    }
    pub fn get_spacing_char(&self) -> Option<Glyth> {
        Some(self.glyths.get(' ' as usize)?.as_ref()?.clone())
    }
}
