use bob_shaders::flat_color_shader::FlatColorShader;
use gl::types::GLfloat;
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle};
use gl_thin::gl_helper::GLErrorWrapper;
use gl_thin::linear::XrMatrix4x4f;

//

pub struct RainbowTriangle<'a> {
    pub program: FlatColorShader,
    pub buffers: VertexBufferBundle<'a, GLfloat, u8>,
}

impl<'a> RainbowTriangle<'a> {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let program = FlatColorShader::new()?;

        program.program.use_()?;

        /*  let mut buffers = VertexBufferBundle::<'static, GLfloat, u8>::incomplete()?;
        let indices_len = {
            let mut bindings = buffers.bind_mut(gpu_state)?;
            Self::configure_vertex_attributes(&bindings, &program.program, 2)?;

            bindings.vertex_buffer.load(&COLOR_TRIANGLE)?;

            let indices = &INDICES;
            bindings.index_buffer.load(indices)?;
            indices.len()
        };*/

        let buffers = {
            const COLOR_TRIANGLE: [GLfloat; 3 * 5] = [
                -0.5, -0.5, 0.0, 1.0, 0.0, //
                0.0, 0.5, 0.0, 0.0, 1.0, //
                0.5, -0.5, 1.0, 0.0, 0.0,
            ];

            static INDICES: [u8; 3] = [0, 1, 2];
            VertexBufferBundle::<'static, GLfloat, u8>::new(
                gpu_state,
                (&COLOR_TRIANGLE).into(),
                (&INDICES).into(),
                5,
                &[(program.sal_position, 2, 0), (program.sal_color, 3, 2)],
            )?
        };

        let rval = RainbowTriangle { buffers, program };

        Ok(rval)
    }

    pub fn paint_color_triangle(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        let program = &self.program.program;
        program.use_().unwrap();

        self.program.set_params(matrix);

        let binding = self.buffers.bind(gpu_state)?;

        binding.draw_elements(gl::TRIANGLES, self.buffers.index_count as i32, 0)?;

        drop(binding);

        Ok(())
    }
}

//
