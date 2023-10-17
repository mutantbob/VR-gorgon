use bob_shaders::flat_color_shader::FlatColorShader;
use gl::types::GLfloat;
use gl_thin::gl_fancy::{BoundBuffersMut, GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLBufferType, GLErrorWrapper, Program};
use gl_thin::linear::XrMatrix4x4f;

//

pub struct RainbowTriangle<'a> {
    pub program: FlatColorShader,
    pub buffers: VertexBufferBundle<'a, GLfloat, u8>,
    pub indices_len: usize,
}

impl<'a> RainbowTriangle<'a> {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let program = FlatColorShader::new()?;

        program.program.use_()?;

        let mut buffers = VertexBufferBundle::<'static, GLfloat, u8>::incomplete()?;
        let indices_len = {
            let mut bindings = buffers.bind_mut(gpu_state)?;
            Self::configure_vertex_attributes(&bindings, &program.program, 2)?;

            const COLOR_TRIANGLE: [GLfloat; 3 * 5] = [
                -0.5, -0.5, 0.0, 1.0, 0.0, //
                0.0, 0.5, 0.0, 0.0, 1.0, //
                0.5, -0.5, 1.0, 0.0, 0.0,
            ];
            bindings.vertex_buffer.load(&COLOR_TRIANGLE)?;

            static INDICES: [u8; 3] = [0, 1, 2];
            let indices = &INDICES;
            bindings.index_buffer.load(indices)?;
            indices.len()
        };

        let rval = RainbowTriangle {
            buffers,
            indices_len,
            program,
        };

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

        binding.draw_elements(gl::TRIANGLES, self.indices_len as i32, 0)?;

        drop(binding);

        Ok(())
    }

    fn configure_vertex_attributes<AT: GLBufferType, IT>(
        buffers: &BoundBuffersMut<AT, IT>,
        program: &Program,
        xyz_width: i32,
    ) -> Result<(), GLErrorWrapper> {
        let stride = xyz_width + 3;
        buffers.rig_one_attribute_by_name(program, "position", xyz_width, stride, 0)?;
        buffers.rig_one_attribute_by_name(program, "color", 3, stride, xyz_width)?;
        Ok(())
    }
}

//
