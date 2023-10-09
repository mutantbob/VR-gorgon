use gl::types::{GLfloat, GLint, GLuint};
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLErrorWrapper, Program};
use gl_thin::linear::XrMatrix4x4f;

#[rustfmt::skip]
static CUBE_VERTICES: &[GLfloat] = &[
    -1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    -1.0, 1.0, -1.0,
    1.0, 1.0, -1.0,
    -1.0, -1.0, 1.0,
    1.0, -1.0, 1.0,
    -1.0, 1.0, 1.0,
    1.0, 1.0, 1.0,
];

#[rustfmt::skip]
static INDICES: &[u8] = &[
    0, 1, 2, 1, 3, 2, // front
    4, 5, 6, 6, 7, 5, // back
    1, 5, 3, 3, 5, 7, // right
    0, 4, 1, 1, 4, 5, // floor
    0, 6, 4, 0, 2, 6, // left
    2, 3, 7, 2, 7, 6, // ceiling
];

pub struct Gorgon1 {
    pub program: GorgonShader1,
    pub buffers: VertexBufferBundle<'static, GLfloat, u8>,
    pub indices_len: usize,
}

impl Gorgon1 {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let program = GorgonShader1::new()?;

        program.program.use_()?;

        let buffers = VertexBufferBundle::<'static, GLfloat, u8>::new(
            gpu_state,
            (CUBE_VERTICES).into(),
            (INDICES).into(),
            3,
            &[(program.sal_position, 3, 0)],
        )?;
        let indices_len = INDICES.len();

        let rval = Self {
            buffers,
            indices_len,
            program,
        };

        Ok(rval)
    }

    /// # parameters
    /// `phase` - should be a floating point number from \[0..64)
    pub fn paint(
        &self,
        matrix: &XrMatrix4x4f,
        phase: GLfloat,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        let program = &self.program.program;
        program.use_()?;

        self.program.set_params(matrix, phase)?;

        let binding = self.buffers.bind(gpu_state)?;

        binding.draw_elements(gl::TRIANGLES, self.indices_len as i32, 0)?;

        drop(binding);

        Ok(())
    }
}

pub struct GorgonShader1 {
    program: Program,
    sul_matrix: GLuint,
    sul_phase: GLuint,
    sal_position: GLuint,
}

impl GorgonShader1 {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        const VERTEX_SHADER: &str = "
uniform mat4 matrix;

attribute vec3 position;

varying vec3 ray;

void main() {
    gl_Position = matrix * vec4(position, 1.0) ;
    ray = position;
}
            ";
        let fragment_shader = match 1 {
            1 => gorgon_sphere_axes(),
            _ => gorgon_two_circles(),
        };
        let program = Program::compile(VERTEX_SHADER, fragment_shader)?;
        let sul_matrix = program.get_uniform_location("matrix")?;
        let sul_phase = program.get_uniform_location("phase")?;
        let sal_position = program.get_attribute_location("position")?;
        Ok(Self {
            program,
            sul_matrix,
            sul_phase,
            sal_position,
        })
    }

    pub fn set_params(&self, matrix: &XrMatrix4x4f, phase: GLfloat) -> Result<(), GLErrorWrapper> {
        self.program
            .set_mat4u(self.sul_matrix as GLint, &matrix.m)?;
        self.program
            .set_uniform_1f(self.sul_phase as GLint, phase)?;
        Ok(())
    }
}

pub fn gorgon_two_circles() -> &'static str {
    "
varying vec3 ray;
uniform float phase;

void main() {
float d1 = distance(ray.xy, vec2(1,0));
float a = floor( mod(d1*4.0 + phase/16.0, 2.0));

float d2 = distance(ray.xy, vec2(-1,0));
float b = floor( mod(d2*5.0 + phase*6.0/64.0, 2.0));

float g;
if (a!=b) {
    g = 1.0;
} else {
    g = 0.0;
} 

gl_FragColor = vec4(g,g,g, 1.0);
}
        "
}

pub fn gorgon_sphere_axes() -> &'static str {
    include_str!("gorgon-sphere-axes.glsl")
}
