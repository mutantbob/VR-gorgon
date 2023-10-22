use crate::sprites::SpriteLocation;
use gl::types::{GLfloat, GLsizei, GLuint};
use gl_thin::gl_fancy::{ActiveTextureUnit, GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLBufferType, GLErrorWrapper, Program, Texture};
use gl_thin::linear::XrMatrix4x4f;

pub fn vertex_shader() -> &'static str {
    "
precision mediump float;
uniform mat4 matrix;

attribute vec3 position;
attribute vec2 uv;

varying vec2 tex_coord;

void main() {
    gl_Position = matrix * vec4(position, 1.0) ;
    tex_coord = uv;
}
"
}

/// draw a set of concentric rings into a unit UV square
pub struct ConcentricRings {
    program: Program,
    pub sul_matrix: GLuint,
    pub sal_position: GLuint,
    pub sal_uv: GLuint,
}

impl ConcentricRings {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let program = Program::compile(vertex_shader(), Self::fragment_shader())?;
        let sul_matrix = program.get_uniform_location("matrix")?;
        let sal_position = program.get_attribute_location("position")?;
        let sal_uv = program.get_attribute_location("uv")?;
        Ok(Self {
            program,
            sul_matrix,
            sal_position,
            sal_uv,
        })
    }

    pub fn fragment_shader() -> &'static str {
        "
precision mediump float;
varying vec2 tex_coord;
void main()
{
    float d1 = distance(vec2(0.5,0.5), tex_coord) ;
    float a = d1>= 0.5 ? 0.0 : 1.0;
    bool b = 0.5 > mod(d1*6.0,1.0);
    float g = b ? 1.0 : 0.0;
    gl_FragColor = vec4(g,g,g,a);
}
"
    }

    /// use this with [VertexBufferBundle::new]
    pub fn attributes_tuples(&self, position_len: i32) -> [(GLuint, i32, i32); 2] {
        [
            (self.sal_position, position_len, 0),
            (self.sal_uv, 2, position_len),
        ]
    }

    pub fn set_parameters(&self, matrix: &XrMatrix4x4f) -> Result<(), GLErrorWrapper> {
        self.program.set_mat4u(self.sul_matrix as _, &matrix.m)
    }

    pub fn draw<IT: GLBufferType>(
        &self,
        matrix: &XrMatrix4x4f,
        buffers: &VertexBufferBundle<GLfloat, IT>,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.program.use_()?;
        self.set_parameters(matrix)?;

        let n_indices = buffers.index_count;
        let bound = buffers.bind(gpu_state)?;
        bound.draw_elements(gl::TRIANGLE_STRIP, n_indices as GLsizei, 0)?;
        Ok(())
    }
}

//

/// draw a hemisphere with latitude-based pie slices
pub struct Latitude {
    program: Program,
    pub sul_matrix: GLuint,
    pub sal_position: GLuint,
    pub sal_uv: GLuint,
}

impl Latitude {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let program = Program::compile(vertex_shader(), Self::fragment_shader())?;
        let sul_matrix = program.get_uniform_location("matrix")?;
        let sal_position = program.get_attribute_location("position")?;
        let sal_uv = program.get_attribute_location("uv")?;
        Ok(Self {
            program,
            sul_matrix,
            sal_position,
            sal_uv,
        })
    }

    /// use this with [VertexBufferBundle::new]
    pub fn attributes_tuples(&self, position_len: i32) -> [(GLuint, i32, i32); 2] {
        [
            (self.sal_position, position_len, 0),
            (self.sal_uv, 2, position_len),
        ]
    }

    pub fn set_parameters(&self, matrix: &XrMatrix4x4f) -> Result<(), GLErrorWrapper> {
        self.program.set_mat4u(self.sul_matrix as _, &matrix.m)
    }

    pub fn draw<IT: GLBufferType>(
        &self,
        matrix: &XrMatrix4x4f,
        buffers: &VertexBufferBundle<GLfloat, IT>,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.program.use_()?;
        self.set_parameters(matrix)?;

        let n_indices = buffers.index_count;
        let bound = buffers.bind(gpu_state)?;
        bound.draw_elements(gl::TRIANGLE_STRIP, n_indices as GLsizei, 0)?;
        Ok(())
    }

    pub fn fragment_shader() -> &'static str {
        "
precision mediump float;
#define PI 3.1415926538
varying vec2 tex_coord;
void main() {
    vec2 dxy = tex_coord - vec2(0.5, 1.0); 
    float d1 = length(dxy);
    float theta = atan(dxy.x, dxy.y);
    bool a = 0.5 > mod(theta * 6.0 / PI, 1.0);
    float g = a ? 1.0:0.0; 
    gl_FragColor = d1 > 0.5 ? vec4(0.0, 0.0, 0.0, 0.0) : vec4(g,g, g, 1.0) ; 
}
        "
    }
}

//

/// Lati-two-d : draw a hemisphere with U coordinate based stripes.
pub struct Latitwod {
    program: Program,
    pub sul_matrix: GLuint,
    pub sal_position: GLuint,
    pub sal_uv: GLuint,
}

impl Latitwod {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let program = Program::compile(vertex_shader(), Self::fragment_shader())?;
        let sul_matrix = program.get_uniform_location("matrix")?;
        let sal_position = program.get_attribute_location("position")?;
        let sal_uv = program.get_attribute_location("uv")?;
        Ok(Self {
            program,
            sul_matrix,
            sal_position,
            sal_uv,
        })
    }

    /// use this with [VertexBufferBundle::new]
    pub fn attributes_tuples(&self, position_len: i32) -> [(GLuint, i32, i32); 2] {
        [
            (self.sal_position, position_len, 0),
            (self.sal_uv, 2, position_len),
        ]
    }

    pub fn set_parameters(&self, matrix: &XrMatrix4x4f) -> Result<(), GLErrorWrapper> {
        self.program.set_mat4u(self.sul_matrix as _, &matrix.m)
    }

    pub fn draw<IT: GLBufferType>(
        &self,
        matrix: &XrMatrix4x4f,
        buffers: &VertexBufferBundle<GLfloat, IT>,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.program.use_()?;
        self.set_parameters(matrix)?;

        let n_indices = buffers.index_count;
        let bound = buffers.bind(gpu_state)?;
        bound.draw_elements(gl::TRIANGLE_STRIP, n_indices as GLsizei, 0)?;
        Ok(())
    }

    pub fn fragment_shader() -> &'static str {
        "
precision mediump float;
#define PI 3.1415926538
varying vec2 tex_coord;
void main() {
    vec2 dxy = tex_coord - vec2(0.5, 1.0); 
    float d1 = length(dxy);
    bool a = 0.5 > mod(tex_coord.x * 4.0, 1.0);
    float g = a ? 1.0:0.0; 
    gl_FragColor = d1 > 0.5 ? vec4(0.0, 0.0, 0.0, 0.0) : vec4(g,g, g, 1.0) ; 
}
        "
    }
}

//

/// Shader to draw a subset (based on `scale` and `offset`) of a sprite sheet into a unit UV ([0..1]) square.
pub struct SpriteRect {
    program: Program,
    pub sal_position: GLuint,
    pub sal_uv: GLuint,
    pub sul_matrix: GLuint,
    pub sul_scale: GLuint,
    pub sul_offset: GLuint,
    pub sul_texture: GLuint,
}

impl SpriteRect {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let program = Program::compile(Self::vertex_shader(), Self::fragment_shader())?;
        let sal_position = program.get_attribute_location("position")?;
        let sal_uv = program.get_attribute_location("uv")?;
        let sul_matrix = program.get_uniform_location("matrix")?;
        let sul_scale = program.get_uniform_location("scale")?;
        let sul_offset = program.get_uniform_location("offset")?;
        let sul_texture = program.get_uniform_location("tex")?;
        Ok(Self {
            program,
            sal_position,
            sal_uv,
            sul_matrix,
            sul_scale,
            sul_offset,
            sul_texture,
        })
    }

    pub fn vertex_shader() -> &'static str {
        "
precision mediump float;
uniform mat4 matrix;
uniform vec2 scale;
uniform vec2 offset;

attribute vec3 position;
attribute vec2 uv;

varying vec2 tex_coord;

void main() {
    gl_Position = matrix * vec4(position, 1.0) ;
    tex_coord = uv*scale + offset;
}
    "
    }

    pub fn fragment_shader() -> &'static str {
        "
precision mediump float;

uniform sampler2D tex;
varying vec2 tex_coord;

void main() {
    vec4 rgba = texture2D(tex, tex_coord);
    float alpha = rgba.r;
    vec4 fg = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 bg = vec4(0.0, 1.0, 0.0, 0.5);
    //float alpha = mix(0.5, 1.0, rgba.r); 
    gl_FragColor = //vec4(0.0, 0.0, 0.0, alpha);
    mix(bg, fg, alpha); 
}
"
    }

    /// use this with [VertexBufferBundle::new]
    pub fn attributes_tuples(&self, position_len: i32) -> [(GLuint, i32, i32); 2] {
        [
            (self.sal_position, position_len, 0),
            (self.sal_uv, 2, position_len),
        ]
    }

    pub fn set_parameters(
        &self,
        matrix: &XrMatrix4x4f,
        scale: &[f32; 2],
        offset: &[f32; 2],
        tex_unit: &ActiveTextureUnit,
    ) -> Result<(), GLErrorWrapper> {
        self.program.set_mat4u(self.sul_matrix as _, &matrix.m)?;
        self.program.set_uniform_2fv(self.sul_scale as _, scale)?;
        self.program.set_uniform_2fv(self.sul_offset as _, offset)?;
        self.program
            .set_uniform_1i(self.sul_texture as _, tex_unit.0 as _)
    }

    pub fn draw<IT: GLBufferType>(
        &self,
        matrix: &XrMatrix4x4f,
        scale: &[f32; 2],
        offset: &[f32; 2],
        texture: &Texture,
        buffers: &VertexBufferBundle<GLfloat, IT>,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.program.use_()?;
        let texture_unit = ActiveTextureUnit(0);
        self.set_parameters(matrix, scale, offset, &texture_unit)?;

        gpu_state.set_active_texture(texture_unit)?;
        texture.bind(gl::TEXTURE_2D)?;

        let n_indices = buffers.index_count;
        let bound = buffers.bind(gpu_state)?;
        bound.draw_elements(gl::TRIANGLE_STRIP, n_indices as GLsizei, 0)?;
        Ok(())
    }

    pub fn draw2(
        &self,
        matrix: &XrMatrix4x4f,
        sprite: &SpriteLocation,
        buffers: &VertexBufferBundle<GLfloat, u8>,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.draw(
            matrix,
            sprite.scale(),
            sprite.offset(),
            sprite.texture,
            buffers,
            gpu_state,
        )
    }
}

//

/// draw a square "ring" (used to highlight entries in the control panel)
pub struct BoxOutline {
    program: Program,
    pub sul_matrix: GLuint,
    pub sal_position: GLuint,
    pub sal_uv: GLuint,
}

impl BoxOutline {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let program = Program::compile(vertex_shader(), Self::fragment_shader())?;
        let sul_matrix = program.get_uniform_location("matrix")?;
        let sal_position = program.get_attribute_location("position")?;
        let sal_uv = program.get_attribute_location("uv")?;
        Ok(Self {
            program,
            sul_matrix,
            sal_position,
            sal_uv,
        })
    }

    pub fn fragment_shader() -> &'static str {
        "
precision mediump float;
varying vec2 tex_coord;
void main()
{
    float thick = 0.06;
    bool b = tex_coord.x >thick && tex_coord.x + thick < 1.0 && tex_coord.y > thick && tex_coord.y+thick < 1.0;
    float g = b ? 0.0 : 1.0;
    gl_FragColor = vec4(g,g,g,g);
}
"
    }

    /// use this with [VertexBufferBundle::new]
    pub fn attributes_tuples(&self, position_len: i32) -> [(GLuint, i32, i32); 2] {
        [
            (self.sal_position, position_len, 0),
            (self.sal_uv, 2, position_len),
        ]
    }

    pub fn set_parameters(&self, matrix: &XrMatrix4x4f) -> Result<(), GLErrorWrapper> {
        self.program.set_mat4u(self.sul_matrix as _, &matrix.m)
    }

    pub fn draw<IT: GLBufferType>(
        &self,
        matrix: &XrMatrix4x4f,
        buffers: &VertexBufferBundle<GLfloat, IT>,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.program.use_()?;
        self.set_parameters(matrix)?;

        let n_indices = buffers.index_count;
        let bound = buffers.bind(gpu_state)?;
        bound.draw_elements(gl::TRIANGLE_STRIP, n_indices as GLsizei, 0)?;
        Ok(())
    }
}
