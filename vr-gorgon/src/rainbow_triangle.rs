use bob_shaders::flat_color_shader::FlatColorShader;
use bob_shaders::sun_phong_shader::SunPhongShader;
use bob_shaders::GeometryBuffer;
use gl::types::{GLfloat, GLsizei, GLushort};
use gl_thin::gl_fancy::{BoundBuffers, BoundBuffersMut, GPUState, VertexBufferBundle};
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

pub struct Suzanne {
    phong: SunPhongShader,
    buffers: VertexBufferBundle<'static, GLfloat, GLushort>,
    index_count: GLsizei,
}

impl Suzanne {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let phong = SunPhongShader::new()?;

        let indices = &crate::suzanne::TRIANGLE_INDICES;
        let buffers = VertexBufferBundle::new(
            gpu_state,
            (&crate::suzanne::XYZABC).into(),
            (indices).into(),
            6,
            &[(phong.sal_position, 3, 0), (phong.sal_normal, 3, 3)],
        )?;

        Ok(Self {
            phong,
            buffers,
            index_count: indices.len() as GLsizei,
        })
    }

    pub fn index_count(&self) -> GLsizei {
        self.index_count
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        matrix: &XrMatrix4x4f,
        sun_direction: &[f32; 3],
        color: &[f32; 3],
        n_indices: GLsizei,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.phong
            .draw(matrix, sun_direction, color, self, n_indices, gpu_state)
    }
}

impl GeometryBuffer<GLfloat, GLushort> for Suzanne {
    fn activate<'a>(&'a self, gpu_state: &'a mut GPUState) -> BoundBuffers<'a, GLfloat, GLushort> {
        self.buffers.bind(gpu_state).unwrap()
    }

    fn deactivate(&self, _droppable: BoundBuffers<GLfloat, GLushort>) {}
}

//
