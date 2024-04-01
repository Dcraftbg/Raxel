use std::io;
use gl33::{global_loader::*, *};
#[derive(Debug)]
pub enum ShaderError {
    Compile { msg: String },
    Link { msg: String },
    Io (io::Error),
    Create // Error during creation of shader (glCreateShader or glCreateProgram returned 0)
}
impl From<io::Error> for ShaderError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)    
    }
}
type ShaderResult<T> = Result<T, ShaderError>;
fn compile_shader_gl_bytes(bytes: &[u8], typ: GLenum) -> ShaderResult<u32> {
  unsafe {
     let id = glCreateShader(typ);
     glShaderSource(
         id,
         1,
         &(bytes.as_ptr().cast()),
         &(bytes.len().try_into().unwrap()));
     glCompileShader(id);
     if id == 0 {return Err(ShaderError::Create); }

     let mut success: i32 = 0;
     glGetShaderiv(id, GL_COMPILE_STATUS, &mut success);
     if success == 0 {
         let mut v: Vec<u8> = Vec::with_capacity(4086); // Should be enough for everybody
         let mut log_len: i32 = 0_i32;
         glGetShaderInfoLog(id, v.capacity() as i32, &mut log_len, v.as_mut_ptr().cast());
         v.set_len(log_len as usize);
         glDeleteShader(id);
         Err(ShaderError::Compile { msg: String::from_utf8_lossy(v.as_slice()).to_string() })
     } else {
         Ok(id)
     }
  }
}
fn compile_shader_gl(path: &str, typ: GLenum) -> ShaderResult<u32> {
    let bytes = std::fs::read(path)?;
    Ok(compile_shader_gl_bytes(&bytes, typ)?)
}
#[derive(Debug)]
pub struct Shader {
    pub id: u32
}
impl Shader {
    pub unsafe fn from_raw_parts(vert: u32, frag: u32) -> ShaderResult<Self> {
        let id = glCreateProgram();
        if id == 0 { return Err(ShaderError::Create); }
        glAttachShader(id, vert);
        glAttachShader(id, frag);
        glLinkProgram(id);
        let mut success: i32 = 0;
        glGetProgramiv(id, GL_LINK_STATUS, &mut success);
        if success == 0 {
          
          let mut v: Vec<u8> = Vec::with_capacity(4086);
          let mut log_len = 0_i32;
          glGetProgramInfoLog(
            id,
            v.capacity() as i32,
            &mut log_len,
            v.as_mut_ptr().cast(),
          );
          v.set_len(log_len.try_into().unwrap());
          glDeleteShader(vert);
          glDeleteShader(frag);
          glDeleteProgram(id);
          return Err(ShaderError::Link { msg: String::from_utf8_lossy(v.as_slice()).to_string() })
        }
        glDeleteShader(vert);
        glDeleteShader(frag);
        Ok(Self {id})
    }
    pub fn from_bytes(vertex: &[u8], fragment: &[u8]) -> ShaderResult<Self> {
        unsafe {
            let vert = compile_shader_gl_bytes(vertex, GL_VERTEX_SHADER)?;
            let frag = match compile_shader_gl_bytes(fragment, GL_FRAGMENT_SHADER) {
                Err(e) => {
                    glDeleteShader(vert);
                    return Err(e)
                }
                Ok(v) => v
            };
            Self::from_raw_parts(vert, frag)
        }
    }
    #[allow(dead_code)]
    pub fn from_file(vertex: &str, fragment: &str) -> ShaderResult<Self> {
        unsafe {
            let vert = compile_shader_gl(vertex, GL_VERTEX_SHADER)?;
            let frag = match compile_shader_gl(fragment, GL_FRAGMENT_SHADER) {
                Err(e) => {
                    glDeleteShader(vert);
                    return Err(e)
                }
                Ok(v) => v
            };
            Self::from_raw_parts(vert, frag)
        }
    }
    pub fn bind(&self) {
        glUseProgram(self.id);
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        glDeleteProgram(self.id)
    }
}
