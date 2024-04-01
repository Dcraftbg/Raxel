#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(dead_code)]
mod image;
mod texture;
mod shader;
mod renderer;
mod batch;
mod font;
mod time;
use font::*;
use std::{io::Read, process::exit, time::{Duration, SystemTime}};

use beryllium::{events::{self, SDL_Keycode, SDLK_BACKSPACE, SDLK_DOWN, SDLK_END, SDLK_HOME, SDLK_LEFT, SDLK_LSHIFT, SDLK_RIGHT, SDLK_RSHIFT, SDLK_SPACE, SDLK_UP}, video, Sdl};
use batch::Batch;
use freetype::face::LoadFlag;
use image::Image;
use shader::Shader;
use texture::Texture;
use renderer::{Boundary, Renderer};
use gl33::{global_loader::{self, *}, *};

const VERT_SHADER: &str = r#"#version 330 core
in vec3 pos;
in vec4 color;
in vec2 texCoords;

out vec4 f_Color;
out vec2 f_TexCoords;

void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    f_Color = color;
    f_TexCoords = texCoords;
}
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  in vec4 f_Color;
  in vec2 f_TexCoords;
  out vec4 color;
  uniform sampler2D slotZero;
  void main() {
    vec4 tex_color = texture(slotZero, f_TexCoords);
    color = f_Color * tex_color;
  }
"#;
#[derive(Debug, Clone, Copy, Default)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}
impl Color {
    // If you wanna know why they aren't const
    // its due to rust being dumb and their
    // stupid constant system being unable to
    // evaluate floats at compile time LOL
    const fn WHITE() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0
        }
    }
    fn from_hex(color: u32) -> Self {
        Self {
            r: ((color>>24) & 0xff) as f32 / 0xff as f32,
            g: ((color>>16) & 0xff) as f32 / 0xff as f32,
            b: ((color>>8) & 0xff) as f32 / 0xff as f32,
            a: ((color>>0) & 0xff) as f32 / 0xff as f32,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Vector3f(f32, f32, f32);
#[derive(Debug, Default, Clone, Copy)]
struct Vector2f(f32, f32);
impl std::ops::Add<Vector2f> for Vector2f {
    type Output = Vector2f;
    fn add(self, rhs: Vector2f) -> Self::Output {
        Vector2f(self.0 + rhs.0, self.1 + rhs.1) 
    }
}

impl std::ops::Mul<Vector2f> for Vector2f {
    type Output = Vector2f;
    fn mul(self, rhs: Vector2f) -> Self::Output {
        Vector2f(self.0 * rhs.0, self.1 * rhs.1) 
    }
}
impl std::ops::Mul<f32> for Vector2f {
    type Output = Vector2f;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector2f(self.0 * rhs, self.1 * rhs)
    }
}
impl std::ops::AddAssign<Vector2f> for Vector2f {
   fn add_assign(&mut self, rhs: Vector2f) {
       self.0 += rhs.0;
       self.1 += rhs.1;
   } 
}
impl Vector2f {
    const fn ZERO() -> Self {
        Self(0.0, 0.0)
    }
}
const SCALAR: i32 = 80;
const W_RATIO: i32 = 16;
const H_RATIO: i32 = 9;
const WIDTH: i32 = W_RATIO * SCALAR;
const HEIGHT: i32 = H_RATIO * SCALAR;

#[derive(Debug)]
struct Line {
    at: usize, // NOTE: In bytes
    len: usize // NOTE: In bytes
}
#[derive(Debug)]
struct Lines {
    inner: Vec<Line> 
}
impl Lines {
    fn parse_bytes(bytes: &[u8]) -> Self {
        let mut inner: Vec<Line> = Vec::new();
        inner.push(Line { at: 0, len: 0 });
        let mut last: usize = 0;
        for (mut at, _) in bytes.iter().enumerate().filter(|(_, b)| **b == '\n' as u8) {
           at += 1;
           if inner.len() > 0 {
             let len = inner.len();
             inner[len-1].len = at-last-1;
           }
           inner.push(Line { at, len: 0});
           last = at
        }
        if inner.len() > 0 {
            let len = inner.len();
            inner[len-1].len = bytes.len()-last;
        }
        Self { inner }
    }
}
#[derive(Clone, Copy)]
struct Cursor {
    line: usize,
    chr: usize,
}
impl Cursor {
    const fn new() -> Self {
        Self { line: 0, chr: 0 }
    }
}
struct Editor {
    view: Vector2f, // Coordinates from top left of text
    cursor: Cursor,
    lines: Lines,
    bytes: Vec<u8>,
}
impl Editor {
    fn delete_char(&mut self, at: Cursor) -> Option<usize> { // None means we deleted the line
        if at.line >= self.lines.inner.len() { return Some(0); }
        if at.chr == 0 && self.lines.inner[at.chr].len == 1 {
            self.lines.inner.remove(at.line);
            return None;
        }
        if at.chr == 0 {
            todo!("unite lines");
        }
        let extra = {
            let l = &mut self.lines.inner[at.line];
            assert!(at.chr < l.len);
            let len = || -> usize {
                let s = std::str::from_utf8(&self.bytes[l.at .. l.at+l.len]).expect("Could not delete char from utf8");
                let mut off: usize = 0;
                for chr in s.chars() {
                    let cl = chr.len_utf8();
                    if off == at.chr { return cl; }
                    off += cl 
                }
                unreachable!("This should be unreachable")
            }();
            let i = l.at + at.chr;
            self.bytes.drain(i-len..i);
            l.len -= len;
            len
        };
        let len = self.lines.inner.len();
        for line in &mut self.lines.inner[at.line+1..len] {
            line.at -= extra;
        }
        Some(extra)
    }
    fn insert_char(&mut self, c: char, at: Cursor) {
        if at.line >= self.lines.inner.len() { return; }
        let extra = {
            let l = &mut self.lines.inner[at.line];
            assert!(at.chr < l.len);
            let i = l.at + at.chr;
            let mut buf: [u8; 4] = [0; 4];
            let len = c.encode_utf8(&mut buf).len();
            self.bytes.splice(i..i, buf[..len].iter().copied());
            l.len += len;
            len
        };
        let len = self.lines.inner.len();
        for line in &mut self.lines.inner[at.line+1..len] {
            line.at += extra;
        }
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { cursor: Cursor::new(), lines: Lines::parse_bytes(&bytes), bytes, view: Vector2f::ZERO()}
    }
    fn display(&self, r: &mut Renderer, bound: Boundary, font: &Font) {
        r.scisorsBegin(&bound);
        //println!("bound: {:?}",bound);
        assert!(self.view.0 == 0.0, "X coordinate not considered yet.");
        // Gets you the glyth step
        let glythH = font.fontSize + 1;
        let advanceY = glythH + 4;
        let lineBegin = self.view.1.floor().max(0.0) as usize;
        let mut pos = bound.top_left();
        for i in lineBegin..self.lines.inner.len() {
            let line = &self.lines.inner[i];
            let bytes = &self.bytes[line.at .. line.at+line.len];
            let s = std::str::from_utf8(bytes).expect("TODO: Implement parsing of non-utf8 text. It should be simple. Check notes");
            let m = font.measure_text(s, font.fontSize as f32).unwrap_or(Vector2f(0.0, font.fontSize as f32));
            if pos.1 > bound.pos.1 + bound.size.1 + m.1 || pos.1 < bound.pos.1 - m.1 {
                break;
            }
            pos.1 -= advanceY as f32;
            {
                let mut pos = pos;
                for (ic, chr) in s.chars().enumerate() {
                    if chr != '\t' && chr != '\r' {
                        r.draw_char(font, chr, pos, Color::WHITE());
                    }
                    if i == self.cursor.line && ic == self.cursor.chr {
                        let yoff = 2.0;
                        let h = font.fontSize as f32;        
                        let w = 3.0;
                        let x = pos.0;
                        let y = pos.1 - yoff;
                        r.draw_rect(Color::WHITE(), Vector2f(x, y), Vector2f(w, h))
                    }
                    pos.0 += font.get_char(chr).advance_x as f32;
                }
            }
            //println!("{}> fp: {:?}. string {:?}",i, fp,s);
            // r.draw_str(font, s, fp, Color::WHITE());
        }
        r.scisorsEnd();
    }
}
fn main() {
    let sdl = Sdl::init(beryllium::init::InitFlags::EVERYTHING);
    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_context_minor_version(3).unwrap();
    let win_args = video::CreateWinArgs {
        title: "Hello World",
        width: WIDTH,
        height: HEIGHT,
        allow_high_dpi: true,
        borderless: false,
        resizable: true 
    };
    let win = sdl.create_gl_window(win_args).expect("Couldn't create window");
    unsafe {
        global_loader::load_global_gl(&|c_char_ptr| win.get_proc_address(c_char_ptr));
    };
    let mut renderer = Renderer::new(sdl, win);
    renderer.create_gl();
    renderer.win.set_swap_interval(video::GlSwapInterval::Vsync).unwrap();
    let shader = Shader::from_bytes(VERT_SHADER.as_bytes(), FRAG_SHADER.as_bytes()).expect("Default shaders should work");
    shader.bind();

    let slotZero = unsafe {
      glGetUniformLocation(shader.id, b"slotZero\0".as_ptr())
    };
    if slotZero < 0 {
        eprintln!("[WARN] Failed to find slotZero!");
    }
    unsafe {
        if slotZero >= 0 {
            glUniform1i(slotZero, 0);
        }
    }
    
    unsafe {
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        glEnable(GL_BLEND);
    }
    let mut font = unsafe {
    Font { glyths: vec![None;128], texture: Texture::new_gl(), fontSize: 18}
    }; 
    font.texture.bind();

    unsafe {
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE.0 as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE.0 as i32);
    }
    
    let lib = freetype::Library::init().expect("Initializing freetype should work");
    let face = lib.new_face("Iosevka-Regular.ttc", 0) .expect("Loading Iosevka should work");
    //let face = lib.new_face("Arial.ttf", 0) .expect("Loading Arial should work");
    //face.set_char_size(40*64, 0, 50, 0).expect("Setting char size should work");
    face.set_pixel_sizes(0, font.fontSize).expect("set_pixel_sizes should not fail");
    
    for i in 32..128 {
        if let Err(err) = face.load_char(i, LoadFlag::RENDER ) {
           println!("Failed to load glyth<{}>: {}",i, err);
           continue;
        }
        let glyth = face.glyph();
        let bitmap = glyth.bitmap();
        let width = bitmap.width() as u32;
        let height = bitmap.rows() as u32;
        font.texture.width += width as i32;
        font.texture.height = if font.texture.height < height as i32 { height as i32} else { font.texture.height};
    }
    unsafe {
    font.texture.buffer_raw_gl(0, GL_RGBA8.0 as i32, font.texture.width, font.texture.height, GL_RGBA, GL_UNSIGNED_BYTE, 0 as *const _);
    }
    // Load all ascii characters
    let mut off: u32 = 0;
    for i in 32..128 {
       if let Err(err) = face.load_char(i, LoadFlag::RENDER) {
           println!("Failed to load glyth<{}>: {}",i, err);
           continue;
       }
       let glyth = face.glyph();
       let bitmap = glyth.bitmap();
       let width = bitmap.width();
       let height = bitmap.rows();
       let bitmap_left = glyth.bitmap_left();
       let bitmap_top = glyth.bitmap_top();
       let atlas_off: i32 = off as i32;
       let advance = glyth.advance();
       let advance_x = advance.x >> 6;
       let advance_y = advance.y >> 6;
       let bound = Boundary { pos: Vector2f(atlas_off as f32, height as f32), size: Vector2f(width as f32, -(height as f32)) };
       let glyth_res = 
           Glyth { width, height, atlas_off, bitmap_left, bitmap_top, bound, advance_x, advance_y};
       unsafe {
           let pixels: Vec<u32> = bitmap.buffer().iter().map(|p| 0x00ffffffu32 | (*p as u32) << 24).collect();
           font.texture.buffer_sub_gl(
               0,
               off as i32,
               0,
               width as i32,
               height as i32,
               GL_RGBA,
               GL_UNSIGNED_BYTE,
               pixels.as_ptr().cast()
            );
       }
       off += width as u32;
       if let Some(v) = font.glyths.get_mut(i) {
          *v = Some(glyth_res); 
       }
    }
    unsafe {
    glGenerateMipmap(GL_TEXTURE_2D);
    }
    let mut mpos = Vector2f::ZERO();
    let mut args = std::env::args();
    let _program = args.next().expect("program");
    let path = args.next().expect("path");
    let f = std::fs::read(&path).expect("main.rs");
    // let msg = f.as_str();
    println!("font.texture: {:?}",font.texture);
    // #[allow(unused_mut)]
    // let mut view = Vector2f::ZERO();

    let mut editor = Editor::from_bytes(f);
    const FPS: u32 = 60;
    const DESIRED_TIME: f64 = 1.0 / FPS as f64;
    'game_loop: loop {
        let mut scroll: f32 = 0.0;
        // TODO: move this into update
        while let Some((event, _)) = renderer.sdl.poll_events() {
            match event {
                events::Event::Quit => break 'game_loop,
                events::Event::WindowResized { win_id: _, width, height }  => {
                    unsafe {
                    glViewport(0, 0, width, height);
                    }
                }
                #[allow(unused_variables)]
                events::Event::Key { win_id, pressed, repeat, scancode, keycode, modifiers } => {
                    match keycode {
                        SDLK_RIGHT => {
                            if pressed {
                                if editor.cursor.chr < editor.lines.inner.get(editor.cursor.line).unwrap().len-1 {
                                    editor.cursor.chr += 1;
                                }
                            }
                        }
                        SDLK_LEFT => {
                            if pressed {
                                if editor.cursor.chr > 0 {
                                    editor.cursor.chr -= 1;
                                }
                            }
                        }
                        SDLK_UP => {
                            if pressed {
                                if editor.cursor.line > 0 {
                                    editor.cursor.line -= 1;
                                    editor.cursor.chr = editor.cursor.chr.min(editor.lines.inner.get(editor.cursor.line).unwrap().len-1)
                                }
                            }
                        }
                        SDLK_DOWN => {
                            if pressed {
                                if editor.cursor.line < editor.lines.inner.len() {
                                    editor.cursor.line += 1;
                                    editor.cursor.chr = editor.cursor.chr.min(editor.lines.inner.get(editor.cursor.line).unwrap().len-1)
                                }
                            }
                        }
                        SDLK_HOME => {
                            if pressed {
                                editor.cursor.chr = 0;
                            }
                        }
                        SDLK_END => {
                            if pressed {
                                editor.cursor.chr = editor.lines.inner.get(editor.cursor.line).unwrap().len-1;
                            }
                        }
                        SDLK_SPACE => {
                            if pressed {
                                editor.insert_char(' ', editor.cursor);
                            }
                        }
                        SDLK_BACKSPACE => {
                            if pressed {
                                let res = editor.delete_char(editor.cursor);
                                match res {
                                    Some(size) => editor.cursor.chr -= size,
                                    None => {
                                        editor.cursor.chr = editor.cursor.chr.clamp(0, editor.lines.inner.get(editor.cursor.line).unwrap().len);
                                    }
                                }
                            }
                        }
                        k => {
                            if pressed {
                                if let Some(chr) = std::char::from_u32(unsafe { std::mem::transmute(k.0) } ) {
                                   editor.insert_char(chr, editor.cursor);
                                }
                            }
                        }
                    }
                }
                #[allow(unused_variables)]
                events::Event::MouseMotion { win_id, mouse_id, button_state, x_win, y_win, x_delta, y_delta } => {
                    let wsize = renderer.window_size();
                    mpos.0 = x_win as f32;
                    mpos.1 = wsize.1 - y_win as f32;
                }
                #[allow(unused_variables)]
                events::Event::MouseWheel { win_id, mouse_id, x, mut y } => {
                    y = y.clamp(-1, 1);
                    scroll = y as f32;
                }
                _ => ()
            }
        }
        renderer.update();
    
        editor.view.1 += -scroll * 10.0;
        //println!("{}> editor.view: {:?}. lines: {}",scroll, editor.view, editor.lines.inner.len() as f32);
        //editor.view.1 = view.1.clamp(0.0, editor.lines.inner.len() as f32);
        renderer.begin();
            renderer.clear(Color::from_hex(0x212121ff));
            shader.bind();
            let ws = renderer.window_size();
            let bound = Boundary { pos: Vector2f::ZERO(), size: ws };
            editor.display(&mut renderer, bound, &font);
            //let parts = msg.split('\n');
            //let ws = renderer.window_size();
            //let mut pos = ws+view;
            //pos.0 = 0.0;
            //for part in parts {
            //    if pos.1 < -16.0 { break }
            //    let m = font.measure_text(part, font.fontSize as f32).unwrap_or(Vector2f(0.0, font.fontSize as f32)).1 + 4.0;
            //    pos.1 -= m;
            //    if pos.1 > ws.1 {
            //        continue 
            //    }
            //    renderer.draw_str(&font, part, pos, Color::WHITE());
            //}
        renderer.end();
    }
}
