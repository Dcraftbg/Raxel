use bytemuck::offset_of;
use gl33::{global_loader::*, *};

use crate::{Color, Vector2f, Vector3f};

#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub pos: Vector3f,
    pub color: Color,
    pub tex: Vector2f
}
#[derive(Debug)]
pub struct Batch {
    pub verts: Vec<Vertex>,
    pub indxs: Vec<u32>,
    pub vbo: u32,
    pub vao: u32,
    pub vio: u32,
    pub texid: u32,
}
impl Batch {
    pub fn new(texid: u32) -> Self {
        // TODO: define buffers[u8; 2]
        // And:
        //  const BUFFER_VBO: usize = 0;
        //  const BUFFER_VIO: usize = 1;
        let vao = unsafe {
            let mut vao = 0;
            glGenVertexArrays(1, &mut vao);
            // TODO: return error on this
            assert_ne!(vao, 0);
            vao
        };
        let vbo = unsafe {
            let mut vbo = 0;
            glGenBuffers(1, &mut vbo);
            // TODO: return error on this
            assert_ne!(vbo, 0);
            vbo
        };
        let vio = unsafe {
            let mut vio = 0;
            glGenBuffers(1, &mut vio);
            // TODO: return error on this
            assert_ne!(vio, 0);
            vio
        };
        unsafe { 
        glBindVertexArray(vao);
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        glEnableVertexAttribArray(0);
        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            0,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            offset_of!(Vertex, pos) as *const _,
        );
        glEnableVertexAttribArray(1);
        glVertexAttribPointer(
            1,
            4,
            GL_FLOAT,
            0,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            offset_of!(Vertex, color) as *const _,
        );

        glEnableVertexAttribArray(2);
        glVertexAttribPointer(
            2,
            2,
            GL_FLOAT,
            0,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            offset_of!(Vertex, tex) as *const _,
        );
        };
        Self { verts: Vec::new(), indxs: Vec::new(), vbo, vao, vio, texid }
    }
    pub fn draw(&self) {
        unsafe {
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, self.texid);
        glBindVertexArray(self.vao);
        glBindBuffer(GL_ARRAY_BUFFER, self.vbo);
        glBufferData(GL_ARRAY_BUFFER,
                 (self.verts.len() * std::mem::size_of::<Vertex>()) as isize,
                 self.verts.as_ptr().cast(),
                 GL_DYNAMIC_DRAW);

        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, self.vio);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, 
                     (self.indxs.len() * std::mem::size_of::<u32>()) as isize,
                     self.indxs.as_ptr().cast(),
                     GL_DYNAMIC_DRAW);
        glDrawElements(GL_TRIANGLES, self.indxs.len() as i32, GL_UNSIGNED_INT, 0 as *const _);
        }
    }
    pub fn update(&mut self) {
       self.draw();
       self.verts.clear();
       self.indxs.clear();
    }
}


impl Batch {
    pub fn tria(&mut self, ps: &[Vertex;3]) {

        let a = self.verts.len() as u32;
        self.verts.extend(ps.iter().map(|v| *v));
        self.indxs.extend(
            [
                a, a+1, a+2,
            ]
        )
    }
    pub fn quad(&mut self, ps: &[Vertex;4]) {
        let a = self.verts.len() as u32;
        self.verts.extend(ps.iter().map(|v| *v));
        self.indxs.extend(
            [
                a, a+1, a+2,
                a, a+3, a+2
            ]
        )
    }
}
impl Drop for Batch {
    fn drop(&mut self) {
        unsafe {
        glDeleteVertexArrays(1, &self.vao);
        glDeleteBuffers(1, &self.vbo);
        glDeleteBuffers(1, &self.vio);
        }
    }    
}
