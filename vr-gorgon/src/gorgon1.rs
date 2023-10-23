use crate::control_panel::{CPCursor, GorgonAxis, GorgonShape};
use gl::types::{GLfloat, GLint, GLuint};
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLErrorWrapper, Program};
use gl_thin::linear::XrMatrix4x4f;
use std::cell::RefCell;
use std::collections::BTreeSet;

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

#[derive(Copy, Clone)]
pub enum GorgonSelector {
    Spiral,
    SphereAxes,
    TwoCircles,
}

impl GorgonSelector {
    pub fn next(&mut self) {
        *self = match self {
            GorgonSelector::Spiral => GorgonSelector::SphereAxes,
            GorgonSelector::SphereAxes => GorgonSelector::TwoCircles,
            GorgonSelector::TwoCircles => GorgonSelector::Spiral,
        }
    }
}

//

pub struct Gorgon1 {
    pub program: GorgonShader1,
    pub buffers: VertexBufferBundle<'static, GLfloat, u8>,
    pub indices_len: usize,

    selector: GorgonSelector,
}

impl Gorgon1 {
    pub(crate) fn next_gorgon(&mut self) -> Result<(), GLErrorWrapper> {
        self.selector.next();
        self.program = GorgonShader1::new(self.selector)?;
        Ok(())
    }
}

impl Gorgon1 {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let selector = GorgonSelector::Spiral;
        let program = GorgonShader1::new(selector)?;

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
            selector,
        };

        Ok(rval)
    }

    /// # parameters
    /// `phase` - should be a floating point number from \[0..1.0)
    pub fn paint(
        &mut self,
        matrix: &XrMatrix4x4f,
        phase: GLfloat,
        settings: &MultiGorgonSettings,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        if *settings.dirty.borrow() {
            self.program.rebuild(settings)?;
            settings.dirty.replace(false);
        }

        let program = &self.program.program;
        program.use_()?;

        self.program.set_params(matrix, phase)?;

        let binding = self.buffers.bind(gpu_state)?;

        binding.draw_elements(gl::TRIANGLES, self.indices_len as i32, 0)?;

        drop(binding);

        Ok(())
    }
}

//

const VERTEX_SHADER: &str = "
uniform mat4 matrix;

attribute vec3 position;

varying vec3 ray;

void main() {
    gl_Position = matrix * vec4(position, 1.0);
    ray = position;
}
";

pub struct GorgonShader1 {
    program: Program,
    sul_matrix: GLuint,
    sul_phase: GLuint,
    sal_position: GLuint,
}

impl GorgonShader1 {
    pub fn new(selector: GorgonSelector) -> Result<Self, GLErrorWrapper> {
        let fragment_shader = match selector {
            GorgonSelector::SphereAxes => gorgon_sphere_axes(),
            GorgonSelector::Spiral => gorgon_spiral(),
            GorgonSelector::TwoCircles => gorgon_two_circles(),
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

    pub fn rebuild(&mut self, settings: &MultiGorgonSettings) -> Result<(), GLErrorWrapper> {
        let program = Program::compile(VERTEX_SHADER, settings.fragment_shader())?;
        self.sul_matrix = program.get_uniform_location("matrix")?;
        self.sul_phase = program.get_uniform_location("phase")?;
        self.sal_position = program.get_attribute_location("position")?;
        self.program = program;
        Ok(())
    }
}

pub fn gorgon_two_circles() -> &'static str {
    "
varying vec3 ray;
uniform float phase;

void main() {
float d1 = distance(ray.xy, vec2(1,0));
float a = floor( mod(d1*4.0 + phase*4.0, 2.0));

float d2 = distance(ray.xy, vec2(-1,0));
float b = floor( mod(d2*5.0 + phase*6.0, 2.0));

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

pub fn gorgon_spiral() -> &'static str {
    include_str!("gorgon-spiral.glsl")
}

//

pub struct GorgonSettings {
    pub enabled: bool,
    pub frequency: u8,
    pub speed: f32,
    pub amplitude: f32,
    pub curl: f32,
}

impl Default for GorgonSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 4,
            speed: 1.0,
            amplitude: 0.0,
            curl: 0.0,
        }
    }
}

impl GorgonSettings {
    pub fn shader_header() -> String {
        "
precision highp float;

varying vec3 ray;
uniform float phase;

#define PI 3.1415926538

vec3 c2s(vec3 rayn)
{
    float r = length(rayn.xy);

    float theta = atan(rayn.z, r);
    float phi = atan(rayn.y, rayn.x);
    return vec3(theta, phi, r);
}
"
        .into()
    }

    pub fn spiral_shader(&self, index: usize) -> String {
        format!(
            "
bool checker{index}(vec3 sc)
{{
    float theta = sc.x;
    float phi = sc.y;
    return 0.5 > mod(phi*{frequency}.0/(2.0*PI) + {curl:.6}*theta/PI + phase*{speed:.6}, 1.0);
}}
",
            index = index,
            frequency = self.frequency,
            speed = self.speed,
            curl = self.curl,
        )
    }

    pub fn latitude_shader(&self, index: usize) -> String {
        format!(
            "
bool checker{index}(vec3 sc)
{{
    float theta = sc.x;
    float phi = sc.y;
    float offset = {amplitude:.6}*sin(mod(phase*2.0*{speed:.6}, 2.0)*PI);
    return 0.5 > mod( theta * {frequency}.0 / (2.0*PI) + offset, 1.0);
}}",
            index = index,
            frequency = self.frequency,
            speed = self.speed,
            amplitude = self.amplitude,
        )
    }
}

//

#[derive(Default)]
pub struct GorgonFragmentShaderBuilder {
    pieces: Vec<(String, String)>,
}

impl GorgonFragmentShaderBuilder {
    pub fn add_spiral(&mut self, settings: &GorgonSettings, swizzle: &str) {
        let index = self.pieces.len();
        let glsl = settings.spiral_shader(index);
        self.pieces.push((swizzle.into(), glsl))
    }

    pub fn add_latitude(&mut self, settings: &GorgonSettings, swizzle: &str) {
        let index = self.pieces.len();
        let glsl = settings.latitude_shader(index);
        self.pieces.push((swizzle.into(), glsl))
    }

    pub fn build(&self) -> String {
        use std::fmt::Write;
        let mut rval = GorgonSettings::shader_header();

        let mut swizzles = BTreeSet::new();
        let mut swizzle_glsl = String::new();
        let mut xor_glsl = "true\n".to_string();
        for (index, (swizzle, function)) in self.pieces.iter().enumerate() {
            rval.push_str(function);
            if swizzles.insert(swizzle) {
                let _ = writeln!(
                    &mut swizzle_glsl,
                    "    vec3 sc_{swizzle} = c2s(rayn.{swizzle});\n",
                    swizzle = swizzle
                );
            }
            let _ = writeln!(&mut xor_glsl, "        ^^ checker{}(sc_{})", index, swizzle);
        }

        let _ = writeln!(
            &mut rval,
            "
void main()
{{
    vec3 rayn = normalize(ray);

{swizzle_glsl}
    float g = ({xor_glsl}        ) ? 1.0 : 0.0;
    gl_FragColor = vec4(g,g,g, 1.0);
}}",
            swizzle_glsl = swizzle_glsl,
            xor_glsl = xor_glsl
        );

        rval
    }
}

//

#[derive(Default)]
pub struct MultiGorgonSettings {
    dirty: RefCell<bool>,
    pub spirals: [GorgonSettings; 3],
    pub latitudes: [GorgonSettings; 3],
    pub cartesians: [GorgonSettings; 3],
}

impl MultiGorgonSettings {
    pub fn lookup_mut(&mut self, shape: GorgonShape, axis: GorgonAxis) -> &mut GorgonSettings {
        let shape = self.shape_for_mut(shape);
        &mut shape[axis.index() as usize]
    }

    pub fn lookup(&self, shape: GorgonShape, axis: GorgonAxis) -> &GorgonSettings {
        let shape = self.shape_for(shape);
        &shape[axis.index() as usize]
    }

    pub fn toggle_enabled(&mut self, cursor: CPCursor) {
        let gorgon = self.lookup_mut(cursor.row, cursor.axis);
        gorgon.enabled = !gorgon.enabled;
        self.dirty.replace(true);
    }

    pub fn adjust_frequency(&mut self, delta: i32, cursor: CPCursor) {
        if delta == 0 {
            return;
        }

        let gorgon = self.lookup_mut(cursor.row, cursor.axis);
        let freq = gorgon.frequency as i32 + delta;
        gorgon.frequency = freq.max(1).min(255) as u8;
        self.dirty.replace(true);
    }

    pub fn adjust_speed(&mut self, delta: f32, cursor: CPCursor) {
        if delta == 0.0 {
            return;
        }

        let gorgon = self.lookup_mut(cursor.row, cursor.axis);
        gorgon.speed += 0.1 * delta;
        self.dirty.replace(true);
    }

    pub fn adjust_amplitude(&mut self, delta: f32, cursor: CPCursor) {
        if delta == 0.0 {
            return;
        }

        let gorgon = self.lookup_mut(cursor.row, cursor.axis);
        gorgon.amplitude += 0.1 * delta;
        self.dirty.replace(true);
    }

    pub fn adjust_curl(&mut self, delta: f32, cursor: CPCursor) {
        if delta == 0.0 {
            return;
        }

        let gorgon = self.lookup_mut(cursor.row, cursor.axis);
        gorgon.curl += 0.1 * delta;
        self.dirty.replace(true);
    }

    pub fn shape_for(&self, shape: GorgonShape) -> &[GorgonSettings; 3] {
        match shape {
            GorgonShape::Spiral => &self.spirals,
            GorgonShape::Latitude => &self.latitudes,
            GorgonShape::Cartesian => &self.cartesians,
        }
    }
    pub fn shape_for_mut(&mut self, shape: GorgonShape) -> &mut [GorgonSettings; 3] {
        match shape {
            GorgonShape::Spiral => &mut self.spirals,
            GorgonShape::Latitude => &mut self.latitudes,
            GorgonShape::Cartesian => &mut self.cartesians,
        }
    }

    pub(crate) fn fragment_shader(&self) -> impl AsRef<str> + Sized {
        let mut builder = GorgonFragmentShaderBuilder::default();
        let swizzles = ["yzx", "zxy", "xyz"];
        for (swizzle, settings) in swizzles.iter().zip(self.spirals.iter()) {
            if settings.enabled {
                builder.add_spiral(settings, swizzle);
            }
        }
        for (swizzle, settings) in swizzles.iter().zip(self.latitudes.iter()) {
            if settings.enabled {
                builder.add_latitude(settings, swizzle);
            }
        }

        let rval = builder.build();
        log::debug!("new shader\n{}", &rval);
        rval
    }
}
