use crate::shaders::{BoxOutline, ConcentricRings, Latitude, Latitwod, SpriteRect};
use crate::text_painting::render_glyphs_to_image;
use crate::thumbstick_smoother::ThumbstickSmoother;
use gl::types::{GLfloat, GLsizei, GLuint};
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLErrorWrapper, Texture};
use gl_thin::linear::{
    xr_matrix4x4f_create_scale, xr_matrix4x4f_create_translation, xr_matrix4x4f_uniform_scale,
    XrMatrix4x4f,
};
use image::{ImageBuffer, Rgb, RgbImage};
use openxr_sys::Vector2f;
use rusttype::{point, Font, Point, Scale};
use std::cmp::Ordering;

pub fn fab_uv_square(
    gpu_state: &mut GPUState,
    attributes: &[(GLuint, i32, i32); 2],
) -> Result<VertexBufferBundle<'static, GLfloat, u8>, GLErrorWrapper> {
    #[rustfmt::skip]
    const XYUV: &[GLfloat] = &[
        -1.0, -1.0, 0.0, 0.0,
        1.0, -1.0, 1.0, 0.0,
        -1.0, 1.0, -0.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
    ];
    VertexBufferBundle::new(
        gpu_state,
        XYUV.into(),
        (&[0, 1, 2, 3]).into(),
        4,
        attributes,
    )
}

pub struct ControlPanel {
    c_rings: ConcentricRings,
    sprite: SpriteRect,
    latitude: Latitude,
    latitwod: Latitwod,
    square: VertexBufferBundle<'static, GLfloat, u8>,
    sprites: SpriteSheet,
    ring: BoxOutline,
    cursor: CPCursor,

    thumbstick_x_smoother: ThumbstickSmoother,
    thumbstick_y_smoother: ThumbstickSmoother,
}

impl ControlPanel {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let c_rings = ConcentricRings::new()?;
        let square = fab_uv_square(gpu_state, &c_rings.attributes_tuples(2))?;
        let sprite = SpriteRect::new()?;
        let latitude = Latitude::new()?;

        Ok(Self {
            c_rings,
            latitude,
            latitwod: Latitwod::new()?,
            square,
            sprites: SpriteSheet::new()?,
            sprite,
            ring: BoxOutline::new()?,
            cursor: CPCursor::default(),
            thumbstick_x_smoother: Default::default(),
            thumbstick_y_smoother: Default::default(),
        })
    }

    pub fn draw(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        if false {
            self.sprite.draw(
                matrix,
                &[1.0; 2],
                &[0.0; 2],
                &self.sprites.texture,
                &self.square,
                gpu_state,
            )?;
        } else {
            self.header_1(matrix, gpu_state, -0.75)?;
            self.header_2(matrix, gpu_state, -0.25)?;
            self.header_3(matrix, gpu_state, 0.75)?;

            {
                let dx = match self.cursor.axis {
                    GorgonAxis::Y => 0.25,
                    GorgonAxis::Z => 0.75,
                    GorgonAxis::X => -0.25,
                };
                let dy = match self.cursor.row {
                    // -0.25
                    GorgonShape::Spiral => -0.75,
                    GorgonShape::Latitude => -0.25,
                    GorgonShape::Cartesian => 0.75,
                };
                let m2 = matrix
                    * xr_matrix4x4f_create_translation(dx, dy, -0.02)
                    * xr_matrix4x4f_uniform_scale(0.25);
                self.ring.draw(&m2, &self.square, gpu_state)?;
            }
        }
        Ok(())
    }

    fn header_1(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        dy: f32,
    ) -> Result<(), GLErrorWrapper> {
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, dy, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.c_rings.draw(&m2, &self.square, gpu_state)?;
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.25, dy, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite
                .draw2(&m2, self.sprites.x(), &self.square, gpu_state)?;
        }
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, dy, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite
                .draw2(&m2, self.sprites.y(), &self.square, gpu_state)?;
        }
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.75, dy, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite
                .draw2(&m2, self.sprites.z(), &self.square, gpu_state)?;
        };
        Ok(())
    }

    fn header_2(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        dy: f32,
    ) -> Result<(), GLErrorWrapper> {
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, dy, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitude.draw(&m2, &self.square, gpu_state)?;
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, dy, -0.01)
                * xr_matrix4x4f_create_scale(0.75, 0.25, 1.0);
            self.sprite.draw(
                &m2,
                &[0.75, 0.25],
                &[-0.1, 0.0],
                &self.sprites.texture,
                &self.square,
                gpu_state,
            )?;
        }
        Ok(())
    }

    fn header_3(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        dy: f32,
    ) -> Result<(), GLErrorWrapper> {
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, dy, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitwod.draw(&m2, &self.square, gpu_state)?;
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, dy, -0.01)
                * xr_matrix4x4f_create_scale(0.75, 0.25, 1.0);
            self.sprite.draw(
                &m2,
                &[0.75, 0.25],
                &[-0.1, 0.0],
                &self.sprites.texture,
                &self.square,
                gpu_state,
            )?;
        }
        Ok(())
    }

    pub(crate) fn handle_thumbstick(&mut self, delta: Vector2f) {
        let dx = delta.x;
        log::debug!("thumbstick {}", dx);
        match self.thumbstick_x_smoother.smooth_input(dx) {
            Ordering::Less => self.cursor.decr_x(),
            Ordering::Equal => {}
            Ordering::Greater => self.cursor.incr_x(),
        }
        match self.thumbstick_y_smoother.smooth_input(delta.y) {
            // yeah, this is a little backwards
            Ordering::Less => self.cursor.incr_y(),
            Ordering::Equal => {}
            Ordering::Greater => self.cursor.decr_y(),
        }
    }
}

//

#[derive(Default)]
pub enum GorgonShape {
    #[default]
    Spiral,
    Latitude,
    Cartesian,
}

#[derive(Default)]
pub enum GorgonAxis {
    #[default]
    X,
    Y,
    Z,
}

#[derive(Default)]
pub enum GorgonParam {
    #[default]
    Enable,
    Frequency,
    Speed,
    Amplitude,
    Curl,
}

#[derive(Default)]
pub struct CPCursor {
    row: GorgonShape,
    axis: GorgonAxis,
    subrow: GorgonParam,
}

impl CPCursor {
    pub fn incr_x(&mut self) {
        self.axis = match self.axis {
            GorgonAxis::X => GorgonAxis::Y,
            GorgonAxis::Y => GorgonAxis::Z,
            GorgonAxis::Z => GorgonAxis::X,
        };
    }

    pub fn decr_x(&mut self) {
        self.axis = match self.axis {
            GorgonAxis::X => GorgonAxis::Z,
            GorgonAxis::Y => GorgonAxis::X,
            GorgonAxis::Z => GorgonAxis::Y,
        };
    }

    pub fn incr_y(&mut self) {
        self.row = match self.row {
            GorgonShape::Spiral => GorgonShape::Latitude,
            GorgonShape::Latitude => GorgonShape::Cartesian,
            GorgonShape::Cartesian => GorgonShape::Spiral,
        }
    }
    pub fn decr_y(&mut self) {
        self.row = match self.row {
            GorgonShape::Spiral => GorgonShape::Cartesian,
            GorgonShape::Latitude => GorgonShape::Spiral,
            GorgonShape::Cartesian => GorgonShape::Latitude,
        }
    }
}

//

fn paint_text_in_image(
    font: &Font,
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    scale: f32,
    offset: Point<f32>,
    msg: &str,
) {
    let glyphs: Vec<_> = font.layout(msg, Scale::uniform(scale), offset).collect();

    render_glyphs_to_image(&glyphs, img);
}

//

pub struct SpriteLocation<'a> {
    pub scale: [f32; 2],
    pub offset: [f32; 2],
    pub texture: &'a Texture,
}

impl<'a> SpriteLocation<'a> {
    fn new(scale: [f32; 2], offset: [f32; 2], texture: &'a Texture) -> Self {
        Self {
            scale,
            offset,
            texture,
        }
    }

    pub fn scale(&self) -> &[f32; 2] {
        &self.scale
    }

    pub fn offset(&self) -> &[f32; 2] {
        &self.offset
    }
}

//

pub struct SpriteSheet {
    pub texture: Texture,
}

impl SpriteSheet {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let font = Font::try_from_bytes(include_bytes!("AlbertText-Bold.ttf")).unwrap();

        let width: GLsizei = 256;
        let height = 256;
        let mut img = RgbImage::new(width as _, height as _);
        let font_size = 60.0;

        let m1 = font.v_metrics(Scale::uniform(font_size));
        let ascent = m1.ascent;

        for (i, msg) in ["x", "y", "z"].iter().enumerate() {
            paint_text_in_image(
                &font,
                &mut img,
                font_size,
                point((width * i as GLsizei) as f32 / 4.0, ascent),
                msg,
            );
        }

        let small_font = 30.0;
        let m2 = font.v_metrics(Scale::uniform(small_font));

        for (i, msg) in ["freq", "speed", "amplitude", "curl"].iter().enumerate() {
            let y2 =
                height as f32 / 4.0 + 1.0 + m2.ascent + i as f32 * 1.5 * (m2.ascent + m2.descent);
            paint_text_in_image(&font, &mut img, small_font, point(0.0, y2), msg);
        }

        // let (width, height) = target.get_dimensions()?;
        let mut target = Texture::new()?;

        if true {
            log::debug!(
                "text pixels {:?} .. {:?}",
                img.as_raw().iter().min(),
                img.as_raw().iter().max()
            );
        }

        target.write_pixels_and_generate_mipmap(
            gl::TEXTURE_2D,
            0,
            gl::RGB as _,
            width,
            height,
            gl::RGB,
            img.as_raw(),
        )?;
        Ok(Self { texture: target })
    }

    pub fn x(&self) -> SpriteLocation<'_> {
        SpriteLocation::new([0.25; 2], [-0.1, 0.0], &self.texture)
    }

    pub fn y(&self) -> SpriteLocation {
        SpriteLocation::new([0.25; 2], [0.15, 0.0], &self.texture)
    }

    pub fn z(&self) -> SpriteLocation {
        SpriteLocation::new([0.25; 2], [0.40, 0.0], &self.texture)
    }
}
