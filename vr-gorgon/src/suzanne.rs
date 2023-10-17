use bob_shaders::sun_phong_shader::SunPhongShader;
use bob_shaders::GeometryBuffer;
use gl::types::{GLfloat, GLsizei, GLushort};
use gl_thin::gl_fancy::{BoundBuffers, GPUState, VertexBufferBundle};
use gl_thin::gl_helper::GLErrorWrapper;
use gl_thin::linear::XrMatrix4x4f;

pub struct Suzanne {
    phong: SunPhongShader,
    buffers: VertexBufferBundle<'static, GLfloat, GLushort>,
    index_count: GLsizei,
}

impl Suzanne {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let phong = SunPhongShader::new()?;

        let indices = &crate::suzanne_geometry::TRIANGLE_INDICES;
        let buffers = VertexBufferBundle::new(
            gpu_state,
            (&crate::suzanne_geometry::XYZABC).into(),
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
        m_matrix: &XrMatrix4x4f,
        pv_matrix: &XrMatrix4x4f,
        sun_direction: &[f32; 3],
        color: &[f32; 3],
        n_indices: GLsizei,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.phong.draw(
            m_matrix,
            pv_matrix,
            sun_direction,
            color,
            self,
            n_indices,
            gpu_state,
        )
    }
}

impl GeometryBuffer<GLfloat, GLushort> for Suzanne {
    fn activate<'a>(&'a self, gpu_state: &'a mut GPUState) -> BoundBuffers<'a, GLfloat, GLushort> {
        self.buffers.bind(gpu_state).unwrap()
    }

    fn deactivate(&self, _droppable: BoundBuffers<GLfloat, GLushort>) {}
}
